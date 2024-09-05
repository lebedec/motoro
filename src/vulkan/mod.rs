use log::{debug, error, info, trace, warn};
use sdl2::video::Window;
use std::collections::{HashMap, HashSet};
use std::convert::Into;
use std::env::var;
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use std::{env, fmt, fs, io, thread};
use vulkanalia::bytecode::Bytecode;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::vk::{
    Buffer, DescriptorImageInfo, DescriptorPoolCreateFlags, DescriptorSet,
    DescriptorSetLayoutCreateFlags, DescriptorType, DeviceV1_0, EntryV1_0, InstanceV1_0,
    InstanceV1_1, KhrSwapchainExtension, PhysicalDeviceDescriptorIndexingProperties,
    PhysicalDeviceProperties2, PipelineVertexInputStateCreateInfo, Sampler, ShaderStageFlags,
    WriteDescriptorSet,
};
use vulkanalia::vk::{ExtDebugUtilsExtension, Handle, HasBuilder};
use vulkanalia::vk::{KhrSurfaceExtension, PhysicalDevice};
use vulkanalia::{vk, Device, Entry, Instance, Version};
use zune_png::error::PngDecodeErrors;
use zune_png::PngDecoder;

use crate::camera::Camera;
use crate::math::{
    mat4_from_scale, mat4_from_translation, mat4_identity, mat4_look_at_rh, mat4_mul,
    mat4_orthographic, mat4_prepend_scale, Mat4, Vec2, Vec3,
};
use crate::textures::{Texture, TextureLoader};
use crate::vulkan::device::create_logical_device;
use crate::vulkan::textures::VulkanTextureLoaderDevice;
use crate::{Mesh, Program, Shader, Storage, Uniform};
use mesura::{Counter, Gauge, GaugeValue};
use sdl2::sys::Atom;

mod device;
pub mod program;
pub mod shaders;
pub mod textures;
pub mod variables;

pub struct Vulkan {
    _entry: Entry,
    _messenger: vk::DebugUtilsMessengerEXT,
    pub(crate) instance: Instance,
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) device: Device,
    queues: QueueFamilyIndex,
    queue: vk::Queue,
    present_queue: vk::Queue,
    surface: vk::SurfaceKHR,
    pub(crate) swapchain: Swapchain,
    pub(crate) render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    sync: Sync,
    pub(crate) chain: usize,
    need_resize: bool,
    programs: Vec<AtomicPtr<Program>>,
    cameras: Vec<AtomicPtr<Camera>>,
    start: Instant,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    command_pools: Vec<vk::CommandPool>,
    present_mode: vk::PresentModeKHR,
}

#[derive(Debug)]
pub enum FrameError {
    Vulkan(vk::ErrorCode),
}

impl From<vk::ErrorCode> for FrameError {
    fn from(value: vk::ErrorCode) -> Self {
        FrameError::Vulkan(value)
    }
}

impl Vulkan {
    pub(crate) fn device(&self) -> &Device {
        &self.device
    }

    pub unsafe fn create(window: &Window, present_mode: vk::PresentModeKHR) -> Self {
        info!("Loads Vulkan library");
        let loader = LibloadingLoader::new(LIBRARY).expect("Vulkan loader must be created");
        let entry = Entry::new(loader).expect("Vulkan entry point must be loaded");
        let version = entry.version().expect("entry version must be got");
        info!("Uses Vulkan {version}");
        let available_layers = entry
            .enumerate_instance_layer_properties()
            .expect("entry layers must be got")
            .iter()
            .map(|layer| layer.layer_name)
            .collect::<HashSet<_>>();
        for layer in available_layers {
            debug!("Vulkan layer {layer}")
        }
        let application_info = vk::ApplicationInfo::builder()
            .application_name(b"Vulkan Tutorial\0")
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"No Engine\0")
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(vk::make_version(1, 0, 0));
        let mut extensions: Vec<_> = window
            .vulkan_instance_extensions()
            .expect("SDL2 vulkan extensions must be got")
            .iter()
            .map(|name| name.as_ptr() as *const _)
            .collect();
        let mut flags = vk::InstanceCreateFlags::empty();
        if cfg!(target_os = "macos") && version >= Version::new(1, 3, 216) {
            info!("Enables extensions for macOS portability");
            extensions.push(
                vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION
                    .name
                    .as_ptr(),
            );
            extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
            flags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        }
        let mut layers = Vec::new();
        let is_vulkan_debug = env::var("VULKAN_DEBUG").is_ok();
        if is_vulkan_debug {
            info!("Enables validation layer");
            layers.push(VALIDATION_LAYER.as_ptr());
            extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
        }
        let mut info = vk::InstanceCreateInfo::builder()
            .flags(flags)
            .application_info(&application_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions);
        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .user_callback(Some(debug_callback));
        if is_vulkan_debug {
            info = info.push_next(&mut debug_info);
        }
        info!("Creates Vulkan instance");
        let instance = entry
            .create_instance(&info, None)
            .expect("Vulkan instance must be created");
        let mut messenger = Default::default();
        if is_vulkan_debug {
            messenger = instance
                .create_debug_utils_messenger_ext(&debug_info, None)
                .expect("Vulkan debug messenger must be created");
        }
        debug!("Creates Vulkan surface");
        let surface_handle = window
            .vulkan_create_surface(instance.handle().as_raw())
            .expect("SDL2 Vulkan surface must be created");
        let surface = vk::SurfaceKHR::from_raw(surface_handle);
        let (queues, physical_device) = find_physical_device(&instance, surface);
        let device = create_logical_device(&instance, physical_device, queues);
        let queue = device.get_device_queue(queues.graphics.family, queues.graphics.queue);
        let present_queue = device.get_device_queue(queues.present.family, queues.present.queue);
        //
        let swapchain = Swapchain::create(
            window,
            &instance,
            &device,
            physical_device,
            queues,
            surface,
            present_mode,
        );
        let render_pass = create_render_pass(&device, &swapchain);
        let framebuffers = create_framebuffers(&device, render_pass, &swapchain);
        let command_pool = create_command_pool(&device, queues.graphics);
        let command_pools = create_command_pools(&device, queues.graphics, &swapchain);
        let command_buffers = create_command_buffers(&device, &command_pools);
        let sync = Sync::create(&device, &swapchain);
        Vulkan {
            _entry: entry,
            instance,
            _messenger: messenger,
            physical_device,
            device,
            queues,
            queue,
            present_queue,
            surface,
            swapchain,
            render_pass,
            framebuffers,
            sync,
            need_resize: false,
            programs: vec![],
            cameras: vec![],
            start: Instant::now(),
            command_pool,
            command_buffers,
            command_pools,
            chain: 0,
            present_mode,
        }
    }

    pub fn create_texture_loader_device(&self) -> VulkanTextureLoaderDevice {
        unsafe {
            let queues = &self.queues;
            let queue = self
                .device
                .get_device_queue(queues.loading.family, queues.loading.queue);
            let command_pool = create_command_pool(&self.device, queues.loading);
            VulkanTextureLoaderDevice {
                instance: self.instance.clone(),
                device: self.device.clone(),
                physical_device: self.physical_device.clone(),
                command_pool,
                queue,
            }
        }
    }

    pub fn register(&mut self, program: &mut Box<Program>) {
        let ptr = AtomicPtr::new(program.as_mut());
        self.programs.push(ptr);
    }

    pub fn register_camera(&mut self, camera: &mut Box<Camera>) {
        let ptr = AtomicPtr::new(camera.as_mut());
        self.cameras.push(ptr);
    }

    pub fn update(&mut self) {
        #[cfg(debug_assertions)]
        {
            for (_index, program) in self.programs().into_iter().enumerate() {
                if program.frag.changed() || program.vert.changed() {
                    unsafe {
                        self.device.device_wait_idle().expect("device must be idle");
                        program.recreate(&self.swapchain, self.render_pass);
                        info!("Recreate done");
                    }
                }
            }
        }
    }

    pub fn programs(&self) -> Vec<&mut Program> {
        unsafe {
            let mut values = vec![];
            for ptr in &self.programs {
                let ptr = ptr.load(Ordering::Relaxed);
                let value = &mut *ptr;
                values.push(value);
            }
            values
        }
    }

    pub fn cameras(&self) -> Vec<&mut Camera> {
        unsafe {
            let mut values = vec![];
            for ptr in &self.cameras {
                let ptr = ptr.load(Ordering::Relaxed);
                let value = &mut *ptr;
                values.push(value);
            }
            values
        }
    }

    pub fn prepare(&mut self, window: &Window, clear_color: [f32; 4]) {
        loop {
            unsafe {
                if let Some(chain) = self.acquire_next_image(window) {
                    self.chain = chain;
                    self.begin_render_pass(clear_color);
                    for program in self.programs() {
                        program.set_command_buffer(self.command_buffers[self.chain]);
                        program.set_chain(self.chain);
                    }
                    break;
                }
            }
        }
    }

    unsafe fn acquire_next_image(&mut self, window: &Window) -> Option<usize> {
        let fence = self.sync.fences[self.sync.frame];
        self.device
            .wait_for_fences(&[fence], true, u64::MAX)
            .expect("fence must be acquired");

        if self.need_resize {
            self.resize(window);
            self.need_resize = false;
            return None;
        }

        let result = self.device.acquire_next_image_khr(
            self.swapchain.handle,
            u64::MAX,
            self.sync.image_available[self.sync.frame],
            vk::Fence::null(),
        );

        let chain = match result {
            Ok((next_image, _)) => next_image as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                self.resize(window);
                return None;
            }
            Err(error) => panic!("unable to acquire next image {error}"),
        };

        let image = self.sync.images[chain];
        if !image.is_null() {
            self.device
                .wait_for_fences(&[image], true, u64::MAX)
                .expect("image must be acquired");
        }
        self.sync.images[chain] = fence;
        Some(chain)
    }

    pub fn present(&mut self) {
        unsafe {
            self.end_render_pass();
        }

        let fence = self.sync.images[self.chain];
        let wait_semaphores = &[self.sync.image_available[self.sync.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.command_buffers[self.chain]];
        let signal_semaphores = &[self.sync.render_finished[self.sync.frame]];
        let info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);
        unsafe {
            self.device
                .reset_fences(&[fence])
                .expect("fence must be reset");
            self.device
                .queue_submit(self.queue, &[info], fence)
                .expect("queue must be submit");
        }

        let swapchains = &[self.swapchain.handle];
        let image_indices = &[self.chain as u32];
        let info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);
        let result = unsafe { self.device.queue_present_khr(self.present_queue, &info) };
        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
        if changed {
            self.need_resize = true;
        } else if let Err(error) = result {
            panic!("unable to present {}", error);
        }
        self.sync.frame = (self.sync.frame + 1) % FRAMES_PROCESSING_CONCURRENCY;
    }

    unsafe fn begin_render_pass(&self, clear_color: [f32; 4]) {
        let command_pool = self.command_pools[self.chain];
        self.device
            .reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())
            .expect("command pool must be reset");
        let buf = self.command_buffers[self.chain];
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        self.device
            .begin_command_buffer(buf, &info)
            .expect("command buffer must begin");
        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.swapchain.extent);
        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: clear_color,
            },
        };
        let clear_values = &[color_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[self.chain])
            .render_area(render_area)
            .clear_values(clear_values);
        self.device
            .cmd_begin_render_pass(buf, &info, vk::SubpassContents::INLINE);
    }

    unsafe fn end_render_pass(&self) {
        let buf = self.command_buffers[self.chain];
        self.device.cmd_end_render_pass(buf);
        self.device
            .end_command_buffer(buf)
            .expect("command buffer must end");
    }

    pub fn swapchain_image_size(&self) -> [f32; 2] {
        [
            self.swapchain.extent.width as f32,
            self.swapchain.extent.height as f32,
        ]
    }

    pub unsafe fn resize(&mut self, window: &Window) {
        info!(
            "Handles window resize from {:?} to {:?}",
            self.swapchain.extent,
            window.size()
        );
        self.device.device_wait_idle().expect("device must be idle");
        self.framebuffers
            .iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));
        self.device.destroy_render_pass(self.render_pass, None);
        self.swapchain
            .views
            .iter()
            .for_each(|image| self.device.destroy_image_view(*image, None));
        self.device
            .destroy_swapchain_khr(self.swapchain.handle, None);
        self.swapchain = Swapchain::create(
            window,
            &self.instance,
            &self.device,
            self.physical_device,
            self.queues,
            self.surface,
            self.present_mode,
        );
        self.render_pass = create_render_pass(&self.device, &self.swapchain);
        self.framebuffers = create_framebuffers(&self.device, self.render_pass, &self.swapchain);
        // recreate programs
        self.device.device_wait_idle().expect("device must be idle");
        for program in self.programs() {
            program.recreate(&self.swapchain, self.render_pass);
        }
        for camera in self.cameras() {
            camera.update(self);
        }
        self.sync
            .images
            .resize(self.swapchain.images.len(), vk::Fence::null());
    }

    // pub unsafe fn destroy(&mut self) {
    //     self.sync.destroy(&self.device);
    //     self.command_pools
    //         .iter()
    //         .for_each(|pool| self.device.destroy_command_pool(*pool, None));
    //     self.device.destroy_command_pool(self.command_pool, None);
    //     self.framebuffers
    //         .iter()
    //         .for_each(|buffer| self.device.destroy_framebuffer(*buffer, None));
    //     self.device.destroy_render_pass(self.render_pass, None);
    //     self.swapchain.destroy(&self.device);
    //     self.device.destroy_device(None);
    //     self.instance.destroy_surface_khr(self.surface, None);
    //     if !self.messenger.is_null() {
    //         self.instance
    //             .destroy_debug_utils_messenger_ext(self.messenger, None);
    //     }
    //     self.instance.destroy_instance(None);
    // }
}

unsafe fn create_buffer(
    device: &Device,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
    physical_device_memory: vk::PhysicalDeviceMemoryProperties,
) -> MemoryBuffer {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let handle = device
        .create_buffer(&buffer_info, None)
        .expect("buffer must be created");
    let requirements = device.get_buffer_memory_requirements(handle);
    let memory_type_index = get_memory_type_index(properties, requirements, physical_device_memory);
    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);
    let memory = device
        .allocate_memory(&memory_info, None)
        .expect("buffer memory must be allocated");
    device
        .bind_buffer_memory(handle, memory, 0)
        .expect("buffer memory must be bound");
    MemoryBuffer { handle, memory }
}

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();
    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        trace!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }
    vk::FALSE
}

fn log_indexing(instance: &Instance, physical_device: PhysicalDevice) {
    let mut indexing = PhysicalDeviceDescriptorIndexingProperties::default();
    let mut props = PhysicalDeviceProperties2::builder().push_next(&mut indexing);
    unsafe {
        instance.get_physical_device_properties2(physical_device, &mut props);
    }
    #[rustfmt::skip]
    info!("Max indexing storages {}", indexing.max_descriptor_set_update_after_bind_storage_buffers);
    #[rustfmt::skip]
    info!("Max indexing uniforms {}", indexing.max_descriptor_set_update_after_bind_uniform_buffers);
    #[rustfmt::skip]
    info!("Max indexing textures {}", indexing.max_descriptor_set_update_after_bind_sampled_images);
    #[rustfmt::skip]
    info!("Max indexing samplers {}", indexing.max_descriptor_set_update_after_bind_samplers);
}

unsafe fn find_physical_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
) -> (QueueFamilyIndex, vk::PhysicalDevice) {
    let physical_devices = instance
        .enumerate_physical_devices()
        .expect("physical devices must be got");
    for physical_device in physical_devices {
        let properties = instance.get_physical_device_properties(physical_device);
        if let Some(queues) = QueueFamilyIndex::find(instance, physical_device, surface) {
            let support = SwapchainSupport::get(instance, surface, physical_device);
            if support.formats.is_empty() || support.present_modes.is_empty() {
                info!(
                    "Skips physical device {} because swap chain not supported",
                    properties.device_name
                );
                continue;
            }
            info!("Uses physical device {}", properties.device_name);
            info!("Uses queues {queues:?}");
            log_indexing(instance, physical_device);
            return (queues, physical_device);
        } else {
            info!("Skips physical device {}", properties.device_name);
        }
    }
    panic!("unable to find physical device");
}

#[derive(Copy, Clone, Default)]
struct QueueIndex {
    family: u32,
    queue: u32,
}

impl fmt::Debug for QueueIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}:{}]", self.family, self.queue)
    }
}

impl QueueIndex {
    pub fn new(family: u32, queue: u32) -> Self {
        Self { family, queue }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct QueueFamilyIndex {
    graphics: QueueIndex,
    present: QueueIndex,
    loading: QueueIndex,
}

impl QueueFamilyIndex {
    fn indices(&self) -> Vec<QueueIndex> {
        vec![self.graphics, self.present, self.loading]
    }

    unsafe fn find(
        instance: &Instance,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Option<Self> {
        // NOTE: typically the graphics queue should be first,
        // but for better device support we can make the search more generic
        let families = instance.get_physical_device_queue_family_properties(device);
        for family in &families {
            info!(
                "Queue family {:?} {}",
                family.queue_flags, family.queue_count
            );
        }
        let mut families = families.into_iter();
        let family = families.next().expect("first queue family must exist");
        if family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            && instance.get_physical_device_surface_support_khr(device, 0, surface) == Ok(true)
        {
            if family.queue_count > 1 {
                return Some(QueueFamilyIndex {
                    graphics: QueueIndex::new(0, 0),
                    present: QueueIndex::new(0, 0),
                    loading: QueueIndex::new(0, 1),
                });
            } else {
                let family = families.next().expect("second queue family must exist");
                if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    return Some(QueueFamilyIndex {
                        graphics: QueueIndex::new(0, 0),
                        present: QueueIndex::new(0, 0),
                        loading: QueueIndex::new(1, 0),
                    });
                }
            }
        }
        None
    }
}

const FRAMES_PROCESSING_CONCURRENCY: usize = 2;

struct Sync {
    image_available: Vec<vk::Semaphore>,
    render_finished: Vec<vk::Semaphore>,
    fences: Vec<vk::Fence>,
    images: Vec<vk::Fence>,
    frame: usize,
}

impl Sync {
    unsafe fn create(device: &Device, swapchain: &Swapchain) -> Self {
        info!("Creates Vulkan sync objects");
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let mut image_available = vec![];
        let mut render_finished = vec![];
        let mut fences = vec![];
        for _ in 0..FRAMES_PROCESSING_CONCURRENCY {
            let semaphore = device
                .create_semaphore(&semaphore_info, None)
                .expect("semaphore must be created");
            image_available.push(semaphore);
            let semaphore = device
                .create_semaphore(&semaphore_info, None)
                .expect("semaphore must be created");
            render_finished.push(semaphore);
            let fence = device
                .create_fence(&fence_info, None)
                .expect("fence must be created");
            fences.push(fence);
        }
        let images = swapchain.images.iter().map(|_| vk::Fence::null()).collect();
        Self {
            image_available,
            render_finished,
            fences,
            images,
            frame: 0,
        }
    }

    // unsafe fn destroy(&mut self, device: &Device) {
    //     self.fences
    //         .iter()
    //         .for_each(|fence| device.destroy_fence(*fence, None));
    //     self.render_finished
    //         .iter()
    //         .for_each(|semaphore| device.destroy_semaphore(*semaphore, None));
    //     self.image_available
    //         .iter()
    //         .for_each(|semaphore| device.destroy_semaphore(*semaphore, None));
    // }
}

const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[
    vk::KHR_SWAPCHAIN_EXTENSION.name,
    vk::EXT_DESCRIPTOR_INDEXING_EXTENSION.name,
];

pub struct Swapchain {
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub handle: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub views: Vec<vk::ImageView>,
}

impl Swapchain {
    unsafe fn create(
        window: &Window,
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        index: QueueFamilyIndex,
        surface: vk::SurfaceKHR,
        present_mode: vk::PresentModeKHR,
    ) -> Self {
        let support = SwapchainSupport::get(instance, surface, physical_device);
        let surface_format = support.get_swapchain_surface_format();
        let present_mode = support.get_swapchain_present_mode(present_mode);
        let extent = support.get_swapchain_extent(window);
        let format = surface_format.format;
        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count != 0
            && image_count > support.capabilities.max_image_count
        {
            image_count = support.capabilities.max_image_count;
        }
        let mut queue_family_indices = vec![];
        let image_sharing_mode = if index.graphics.family != index.present.family {
            queue_family_indices.push(index.graphics.family);
            queue_family_indices.push(index.present.family);
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };
        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());
        let handle = device
            .create_swapchain_khr(&info, None)
            .expect("swap chain must be created");
        let images = device
            .get_swapchain_images_khr(handle)
            .expect("swap chain images must be got");
        let views = images
            .iter()
            .map(|image| create_image_view(device, *image, format))
            .collect();
        info!("Creates swap chain mode={present_mode:?} format={format:?} extent={extent:?} images={} handle={handle:?}", images.len());
        Swapchain {
            format,
            extent,
            handle,
            images,
            views,
        }
    }

    // unsafe fn destroy(&mut self, device: &Device) {
    //     self.views
    //         .iter()
    //         .for_each(|view| device.destroy_image_view(*view, None));
    //     device.destroy_swapchain_khr(self.handle, None);
    // }
}

#[derive(Clone, Debug)]
struct SwapchainSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    unsafe fn get(
        instance: &Instance,
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
    ) -> Self {
        Self {
            capabilities: instance
                .get_physical_device_surface_capabilities_khr(physical_device, surface)
                .expect("swap chain capabilities must be got"),
            formats: instance
                .get_physical_device_surface_formats_khr(physical_device, surface)
                .expect("swap chain formats must be got"),
            present_modes: instance
                .get_physical_device_surface_present_modes_khr(physical_device, surface)
                .expect("swap chain present modes must be got"),
        }
    }

    fn get_swapchain_surface_format(&self) -> vk::SurfaceFormatKHR {
        self.formats
            .iter()
            .cloned()
            .find(|surface| {
                surface.format == vk::Format::R8G8B8A8_UNORM
                    && surface.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or_else(|| self.formats[0])
    }

    fn get_swapchain_present_mode(&self, preferred: vk::PresentModeKHR) -> vk::PresentModeKHR {
        self.present_modes
            .iter()
            .cloned()
            .find(|mode| *mode == preferred)
            .unwrap_or(vk::PresentModeKHR::IMMEDIATE)
    }

    fn get_swapchain_extent(&self, window: &Window) -> vk::Extent2D {
        if self.capabilities.current_extent.width != u32::MAX {
            self.capabilities.current_extent
        } else {
            let (width, height) = window.vulkan_drawable_size();
            let clamp = |min: u32, max: u32, v: u32| min.max(max.min(v));
            let width = clamp(
                self.capabilities.min_image_extent.width,
                self.capabilities.max_image_extent.width,
                width,
            );
            let height = clamp(
                self.capabilities.min_image_extent.height,
                self.capabilities.max_image_extent.height,
                height,
            );
            vk::Extent2D::builder().width(width).height(height).build()
        }
    }
}

unsafe fn create_shader_module(device: &Device, bytecode: &[u8]) -> vk::ShaderModule {
    let bytecode = Bytecode::new(bytecode).unwrap();
    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());
    device
        .create_shader_module(&info, None)
        .expect("shader module must be created")
}

unsafe fn create_render_pass(device: &Device, swapchain: &Swapchain) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription::builder()
        .format(swapchain.format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let color_attachments = &[color_attachment_ref];
    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments);
    let attachments = &[color_attachment];
    let subpasses = &[subpass];
    let info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses);
    info!("Creates render pass");
    device
        .create_render_pass(&info, None)
        .expect("render pass must be created")
}

unsafe fn create_pipeline(
    device: &Device,
    swapchain: &Swapchain,
    render_pass: vk::RenderPass,
    descriptor_layouts: Vec<vk::DescriptorSetLayout>,
    vert: &[u8],
    frag: &[u8],
    push_constants: Vec<vk::PushConstantRange>,
    vertex_input: PipelineVertexInputStateCreateInfo,
) -> (vk::PipelineLayout, vk::Pipeline) {
    debug!("Compiles vert shader");
    let vert_shader_module = create_shader_module(device, vert);
    debug!("Compiles frag shader");
    let frag_shader_module = create_shader_module(device, frag);
    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");
    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);
    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain.extent.width as f32)
        .height(swapchain.extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);
    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(swapchain.extent);
    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);
    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        // .cull_mode(vk::CullModeFlags::BACK)
        // .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false);
    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::_1);
    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);
    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);
    let push_constant_ranges = push_constants.as_slice();
    let mut layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&descriptor_layouts);
    if push_constant_ranges.len() > 0 {
        layout_info = layout_info.push_constant_ranges(push_constant_ranges);
    }
    debug!("Creates pipeline layout");
    let pipeline_layout = device
        .create_pipeline_layout(&layout_info, None)
        .expect("pipeline layout must be created");
    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);
    debug!("Creates graphics pipeline");
    let pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)
        .expect("graphics pipeline must be created")
        .0[0];
    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);
    (pipeline_layout, pipeline)
}

unsafe fn create_framebuffers(
    device: &Device,
    render_pass: vk::RenderPass,
    swapchain: &Swapchain,
) -> Vec<vk::Framebuffer> {
    info!("Creates {} frame buffers", swapchain.views.len());
    swapchain
        .views
        .iter()
        .map(|image| {
            let attachments = &[*image];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);
            device
                .create_framebuffer(&create_info, None)
                .expect("frame buffer must be created")
        })
        .collect()
}

unsafe fn create_command_buffers(
    device: &Device,
    command_pools: &Vec<vk::CommandPool>,
) -> Vec<vk::CommandBuffer> {
    let mut buffers = vec![];
    for pool in command_pools {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = device
            .allocate_command_buffers(&allocate_info)
            .expect("command buffers must be allocated")[0];
        buffers.push(command_buffer);
    }
    buffers
}

#[derive(Debug, Clone)]
pub struct MemoryBuffer {
    pub handle: vk::Buffer,
    memory: vk::DeviceMemory,
}

impl MemoryBuffer {
    pub fn update<T: Sized>(&self, device: &Device, data: &[T]) {
        let size = (data.len() * std::mem::size_of::<T>()) as u64;
        let flags = vk::MemoryMapFlags::empty();
        unsafe {
            let memory = device
                .map_memory(self.memory, 0, size, flags)
                .expect("memory must be mapped");
            std::ptr::copy_nonoverlapping(data.as_ptr(), memory.cast(), data.len());
            device.unmap_memory(self.memory);
        }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_buffer(self.handle, None);
            device.free_memory(self.memory, None);
        }
    }
}

pub unsafe fn create_buffers(
    usage: vk::BufferUsageFlags,
    device: &Device,
    swapchain: usize,
    physical_device_memory: vk::PhysicalDeviceMemoryProperties,
    size: usize,
) -> Vec<MemoryBuffer> {
    let mut buffers = vec![];
    for _ in 0..swapchain {
        let buffer = create_buffer(
            device,
            size as u64,
            usage,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            physical_device_memory,
        );
        buffers.push(buffer);
    }
    buffers
}

type DescriptorSetBinding = (u32, vk::DescriptorType, vk::ShaderStageFlags, usize);

unsafe fn create_descriptor_pool(
    device: &Device,
    bindings: &Vec<DescriptorSetBinding>,
    sets: usize,
) -> vk::DescriptorPool {
    let pool_sizes: Vec<vk::DescriptorPoolSize> = bindings
        .iter()
        .map(|(_, t, _, count)| {
            vk::DescriptorPoolSize::builder()
                .type_(*t)
                .descriptor_count((*count * sets) as u32)
                .build()
        })
        .collect();
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(sets as u32);
    let pool = device
        .create_descriptor_pool(&info, None)
        .expect("descriptor pool must be created");
    debug!("Creates descriptor pool {pool:?} max sets={sets}");
    pool
}

unsafe fn create_descriptor_set_layout(
    device: &Device,
    bindings: Vec<DescriptorSetBinding>,
) -> vk::DescriptorSetLayout {
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings
        .into_iter()
        .map(|(binding, types, stages, count)| {
            vk::DescriptorSetLayoutBinding::builder()
                .binding(binding)
                .descriptor_type(types)
                .descriptor_count(count as u32)
                .stage_flags(stages)
                .build()
        })
        .collect();
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings.as_slice());
    device
        .create_descriptor_set_layout(&info, None)
        .expect("descriptor set layout must be created")
}

unsafe fn create_descriptors(
    device: &Device,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    count: usize,
) -> Vec<vk::DescriptorSet> {
    let layouts = vec![descriptor_set_layout; count];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);
    let descriptor_sets = device
        .allocate_descriptor_sets(&info)
        .expect("descriptor sets must be created");
    debug!("Creates variables {:?}", descriptor_sets);
    descriptor_sets
}
//
// pub fn radians(degrees: f32) -> f32 {
//     degrees * std::f32::consts::PI / 180.0
// }
//
// pub fn degrees(radians: f32) -> f32 {
//     radians * std::f32::consts::PI / 180.0
// }

unsafe fn get_memory_type_index(
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
    memory: vk::PhysicalDeviceMemoryProperties,
) -> u32 {
    let criteria = |index: &u32| {
        let suitable = (requirements.memory_type_bits & (1 << index)) != 0;
        let memory_type = memory.memory_types[*index as usize];
        suitable && memory_type.property_flags.contains(properties)
    };
    let mut types = 0..memory.memory_type_count;
    types
        .find(criteria)
        .expect("suitable memory type must be found")
}

unsafe fn create_command_pools(
    device: &Device,
    queue: QueueIndex,
    swapchain: &Swapchain,
) -> Vec<vk::CommandPool> {
    let mut command_pools = vec![];
    for _ in 0..swapchain.images.len() {
        let command_pool = create_command_pool(device, queue);
        command_pools.push(command_pool);
    }
    command_pools
}

unsafe fn create_command_pool(device: &Device, queue: QueueIndex) -> vk::CommandPool {
    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(queue.family);
    device
        .create_command_pool(&info, None)
        .expect("command pool must be created")
}

unsafe fn command_once(device: &Device, pool: vk::CommandPool) -> vk::CommandBuffer {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(pool)
        .command_buffer_count(1);
    let buffer = device
        .allocate_command_buffers(&info)
        .expect("command buffer must be allocated")[0];
    let flags = vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT;
    let info = vk::CommandBufferBeginInfo::builder().flags(flags);
    device
        .begin_command_buffer(buffer, &info)
        .expect("command buffer must begin");
    buffer
}

unsafe fn submit_commands(
    device: &Device,
    queue: vk::Queue,
    pool: vk::CommandPool,
    buffer: vk::CommandBuffer,
) {
    device
        .end_command_buffer(buffer)
        .expect("command buffer must end");
    let command_buffers = &[buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);
    device
        .queue_submit(queue, &[info], vk::Fence::null())
        .expect("queue must be submitted");
    device.queue_wait_idle(queue).expect("queue must be idle");
    device.free_command_buffers(pool, &[buffer]);
}

// unsafe fn create_pixel_perfect_sampler(device: &Device) -> vk::Sampler {
//     let info = vk::SamplerCreateInfo::builder()
//         .mag_filter(vk::Filter::NEAREST)
//         .min_filter(vk::Filter::NEAREST)
//         .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
//         .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
//         .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
//         .anisotropy_enable(false)
//         .max_anisotropy(16.0)
//         .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
//         .unnormalized_coordinates(false)
//         .compare_enable(false)
//         .compare_op(vk::CompareOp::ALWAYS)
//         .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
//         .min_lod(0.0)
//         .max_lod(0.0)
//         .mip_lod_bias(0.0);
//     device
//         .create_sampler(&info, None)
//         .expect("sampler must be created")
// }

// unsafe fn create_smooth_sampler(device: &Device) -> vk::Sampler {
//     let info = vk::SamplerCreateInfo::builder()
//         .mag_filter(vk::Filter::LINEAR)
//         .min_filter(vk::Filter::LINEAR)
//         .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
//         .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
//         .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
//         .anisotropy_enable(true)
//         .max_anisotropy(16.0)
//         .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
//         .unnormalized_coordinates(false)
//         .compare_enable(false)
//         .compare_op(vk::CompareOp::ALWAYS)
//         .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
//         .min_lod(0.0)
//         .max_lod(2.0)
//         .mip_lod_bias(0.0);
//     device
//         .create_sampler(&info, None)
//         .expect("sampler must be created")
// }

// unsafe fn create_sampler(device: &Device) -> vk::Sampler {
//     let info = vk::SamplerCreateInfo::builder()
//         .mag_filter(vk::Filter::LINEAR)
//         .min_filter(vk::Filter::LINEAR)
//         .address_mode_u(vk::SamplerAddressMode::REPEAT)
//         .address_mode_v(vk::SamplerAddressMode::REPEAT)
//         .address_mode_w(vk::SamplerAddressMode::REPEAT)
//         .anisotropy_enable(true)
//         .max_anisotropy(16.0)
//         .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
//         .unnormalized_coordinates(false)
//         .compare_enable(false)
//         .compare_op(vk::CompareOp::ALWAYS)
//         .mipmap_mode(vk::SamplerMipmapMode::LINEAR);
//     device
//         .create_sampler(&info, None)
//         .expect("sampler must be created")
// }

unsafe fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
) -> vk::ImageView {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);
    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);
    device
        .create_image_view(&info, None)
        .expect("image view must be created")
}
