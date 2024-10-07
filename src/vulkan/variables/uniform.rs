use crate::vulkan::{
    create_buffers, create_descriptor_pool, create_descriptor_set_layout, create_descriptors,
    MemoryBuffer, Vulkan,
};
use log::info;
use std::any::type_name;
use std::marker::PhantomData;
use vulkanalia::vk::{
    Buffer, BufferUsageFlags, CopyDescriptorSet, DescriptorBufferInfo, DescriptorSet,
    DescriptorSetLayout, DescriptorType, DeviceV1_0, HasBuilder, InstanceV1_0, MemoryMapFlags,
    ShaderStageFlags, WriteDescriptorSet,
};
use vulkanalia::Device;

/// Represents GLSL variable declared with the "uniform" storage qualifier.
///
/// ```glsl
/// layout (set = 0, binding = 0) uniform Transform {
///     mat4 model;
///     mat4 view;
///     mat4 proj;
/// } transform;
/// ```
pub struct Uniform<T> {
    pub(crate) slot: u32,
    pub(crate) binding: u32,
    layout: DescriptorSetLayout,
    sets: Vec<DescriptorSet>,
    buffers: Vec<MemoryBuffer>,
    device: Device,
    _phantom: PhantomData<T>,
}

impl<T> Uniform<T> {
    pub fn layout(&self) -> DescriptorSetLayout {
        self.layout
    }

    pub fn descriptor(&self, frame: usize) -> DescriptorSet {
        self.sets[frame]
    }

    pub unsafe fn create(slot: u32, binding: u32, vulkan: &Vulkan) -> Uniform<T> {
        info!(
            "Creates uniform<{}>, layout(set = {slot}, binding = {binding})",
            type_name::<T>()
        );
        let device = &vulkan.device;
        let frames = vulkan.swapchain.images.len();
        let bindings = vec![(
            binding,
            DescriptorType::UNIFORM_BUFFER,
            ShaderStageFlags::VERTEX,
            1,
        )];
        let pool = create_descriptor_pool(device, &bindings, frames);
        let layout = create_descriptor_set_layout(device, bindings);
        let sets = create_descriptors(device, pool, layout, frames);
        let physical_device_memory = vulkan
            .instance
            .get_physical_device_memory_properties(vulkan.physical_device);
        let buffers = create_buffers(
            BufferUsageFlags::UNIFORM_BUFFER,
            device,
            frames,
            physical_device_memory,
            size_of::<T>(),
        );
        let uniform = Uniform {
            slot,
            binding,
            layout,
            sets,
            buffers,
            device: device.clone(),
            _phantom: Default::default(),
        };
        for i in 0..frames {
            uniform.write(device, i, uniform.buffers[i].handle);
        }
        uniform
    }

    pub fn update(&self, frame: usize, value: &T) {
        unsafe {
            let memory = self
                .device
                .map_memory(
                    self.buffers[frame].memory,
                    0,
                    size_of::<T>() as u64,
                    MemoryMapFlags::empty(),
                )
                .expect("memory must be mapped");
            std::ptr::copy_nonoverlapping(value, memory.cast(), 1);
            self.device.unmap_memory(self.buffers[frame].memory);
        }
    }

    fn write(&self, device: &Device, frame: usize, buffer: Buffer) {
        let info = DescriptorBufferInfo::builder()
            .buffer(buffer)
            .offset(0)
            .range(size_of::<T>() as u64);
        let buffer_info = &[info];
        let buffer_write = WriteDescriptorSet::builder()
            .dst_set(self.sets[frame])
            .dst_binding(self.binding)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);
        unsafe {
            device.update_descriptor_sets(&[buffer_write], &[] as &[CopyDescriptorSet]);
        }
    }
}
