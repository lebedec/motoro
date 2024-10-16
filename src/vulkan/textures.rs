use crate::textures::{read_texture_from_data, Texture, TextureError, TextureLoaderDevice};
use crate::vulkan::{
    command_once, create_buffer, create_image_view, get_memory_type_index, submit_commands,
    MemoryBuffer,
};
use log::debug;
use std::time::Instant;

use vulkanalia::vk::{CommandPool, DeviceV1_0, HasBuilder, InstanceV1_0, PhysicalDevice, Queue};
use vulkanalia::{vk, Device, Instance};

#[derive(Clone)]
pub struct VulkanTextureLoaderDevice {
    pub(crate) instance: Instance,
    pub(crate) device: Device,
    pub(crate) physical_device: PhysicalDevice,
    pub(crate) command_pool: CommandPool,
    pub(crate) queue: Queue,
}

impl VulkanTextureLoaderDevice {
    pub fn update_texture_data(&self, texture: Texture, data: &[u8]) {
        unsafe {
            let format = vk::Format::R8G8B8A8_UNORM;
            update_image(
                &self.instance,
                &self.device,
                self.physical_device,
                self.queue,
                self.command_pool,
                texture,
                format,
                data,
            )
        }
    }

    pub fn create_texture_handle(&self, width: usize, height: usize) -> Texture {
        unsafe {
            let format = vk::Format::R8G8B8A8_UNORM;
            create_image(
                &self.instance,
                &self.device,
                self.physical_device,
                width as u32,
                height as u32,
                format,
                vk::ImageTiling::LINEAR,
                vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
        }
    }

    pub fn create_texture(&self, width: u32, height: u32, data: &[u8]) -> Texture {
        let texture = unsafe {
            create_texture(
                &self.instance,
                &self.device,
                self.physical_device,
                self.queue,
                self.command_pool,
                width,
                height,
                data,
            )
        };
        texture
    }
}

impl TextureLoaderDevice for VulkanTextureLoaderDevice {
    fn load_texture_from(&self, data: &[u8]) -> Result<Texture, TextureError> {
        read_texture_from_data(data).and_then(|(image, data)| {
            let texture = unsafe {
                create_texture(
                    &self.instance,
                    &self.device,
                    self.physical_device,
                    self.queue,
                    self.command_pool,
                    image.width as u32,
                    image.height as u32,
                    &data,
                )
            };
            Ok(texture)
        })
    }
}

unsafe fn update_image(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    texture: Texture,
    format: vk::Format,
    data: &[u8],
) {
    let t = Instant::now();
    let [width, height] = texture.size;
    let size = data.len() as u64;
    let physical_device_memory = instance.get_physical_device_memory_properties(physical_device);
    let staging = create_buffer(
        device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        physical_device_memory,
    );
    let t0 = t.elapsed();
    let t = Instant::now();
    staging.update(device, data);
    let t1 = t.elapsed();
    let t = Instant::now();
    transition_image_layout(
        device,
        queue,
        command_pool,
        texture.image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    let t2 = t.elapsed();
    let t = Instant::now();
    copy_buffer_to_image(
        device,
        queue,
        command_pool,
        staging.handle,
        texture.image,
        width,
        height,
    );
    let t3 = t.elapsed();
    let t = Instant::now();
    transition_image_layout(
        device,
        queue,
        command_pool,
        texture.image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    let t4 = t.elapsed();
    device.destroy_buffer(staging.handle, None);
    device.free_memory(staging.memory, None);
    // println!(
    //     "create_buffer {t0:?}, update {t1:?}, trans1 {t2:?}, copy_buffer {t3:?}, trans2 {t4:?} {texture:?}"
    // );
}

unsafe fn create_texture(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    width: u32,
    height: u32,
    data: &[u8],
) -> Texture {
    let size = data.len() as u64;
    let physical_device_memory = instance.get_physical_device_memory_properties(physical_device);
    let staging = create_buffer(
        device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        physical_device_memory,
    );
    let memory = device
        .map_memory(staging.memory, 0, size, vk::MemoryMapFlags::empty())
        .expect("memory must be mapped");
    std::ptr::copy_nonoverlapping(data.as_ptr(), memory.cast(), data.len());
    device.unmap_memory(staging.memory);
    let format = vk::Format::R8G8B8A8_UNORM;
    let texture = create_image(
        instance,
        device,
        physical_device,
        width,
        height,
        format,
        vk::ImageTiling::LINEAR,
        vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );
    debug!("Performs layout transition {texture:?}");
    transition_image_layout(
        device,
        queue,
        command_pool,
        texture.image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    copy_buffer_to_image(
        device,
        queue,
        command_pool,
        staging.handle,
        texture.image,
        width,
        height,
    );
    transition_image_layout(
        device,
        queue,
        command_pool,
        texture.image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    device.destroy_buffer(staging.handle, None);
    device.free_memory(staging.memory, None);
    texture
}

unsafe fn create_image(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Texture {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::_1);
    let image = device
        .create_image(&info, None)
        .expect("image must be created");
    let requirements = device.get_image_memory_requirements(image);
    let physical_device_memory = instance.get_physical_device_memory_properties(physical_device);
    let memory_type_index = get_memory_type_index(properties, requirements, physical_device_memory);
    let info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);
    let memory = device
        .allocate_memory(&info, None)
        .expect("image memory must be allocated");
    device
        .bind_image_memory(image, memory, 0)
        .expect("image memory must bound");
    let view = create_image_view(device, image, vk::Format::R8G8B8A8_UNORM);
    Texture {
        image,
        memory,
        view,
        size: [width, height],
    }
}

unsafe fn transition_image_layout(
    device: &Device,
    queue: vk::Queue,
    pool: vk::CommandPool,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) {
    let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
        match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            _ => {
                panic!("unsupported image layout transition from {old_layout:?} to {new_layout:?}")
            }
        };
    let commands = command_once(device, pool);
    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);
    let barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);
    device.cmd_pipeline_barrier(
        commands,
        src_stage_mask,
        dst_stage_mask,
        vk::DependencyFlags::empty(),
        &[] as &[vk::MemoryBarrier],
        &[] as &[vk::BufferMemoryBarrier],
        &[barrier],
    );
    submit_commands(device, queue, pool, commands);
}

unsafe fn copy_buffer_to_image(
    device: &Device,
    queue: vk::Queue,
    pool: vk::CommandPool,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) {
    let commands = command_once(device, pool);
    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);
    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        });
    device.cmd_copy_buffer_to_image(
        commands,
        buffer,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[region],
    );
    submit_commands(device, queue, pool, commands);
}
