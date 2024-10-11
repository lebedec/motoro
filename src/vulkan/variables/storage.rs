use crate::vulkan::{
    create_buffers, create_descriptor_pool, create_descriptor_set_layout, create_descriptors,
    MemoryBuffer, Vulkan,
};
use crate::Variable;
use log::{error, info};
use std::any::type_name;
use std::marker::PhantomData;
use vulkanalia::vk::{
    Buffer, BufferUsageFlags, CopyDescriptorSet, DescriptorBufferInfo, DescriptorSet,
    DescriptorSetLayout, DescriptorType, DeviceV1_0, HasBuilder, InstanceV1_0, MemoryMapFlags,
    ShaderStageFlags, WriteDescriptorSet,
};
use vulkanalia::{vk, Device};

/// Represents GLSL variable declared as storage buffer.
///
/// ```glsl
/// layout (std140, set = 0, binding = 4) readonly buffer Canvas {
///     Element elements[];
/// } canvas;
/// ```
pub struct Storage<T> {
    pub(crate) buffers: Vec<MemoryBuffer>,
    pub(crate) range: u64,
    device: Device,
    collection: Vec<T>,
    cursor: usize,
}

impl<T: Default + Clone + Copy> Storage<T> {
    pub unsafe fn create(vulkan: &Vulkan, n: usize) -> Self {
        let device = &vulkan.device;
        let frames = vulkan.swapchain.images.len();
        let physical_device_memory = vulkan
            .instance
            .get_physical_device_memory_properties(vulkan.physical_device);
        let size = size_of::<T>();
        let range = size * n;
        info!("Creates storage buffers n={n} size={size} range={range}");
        let buffers = create_buffers(
            BufferUsageFlags::STORAGE_BUFFER,
            device,
            frames,
            physical_device_memory,
            range,
        );
        Self {
            buffers,
            device: device.clone(),
            collection: vec![T::default(); n],
            cursor: 0,
            range: range as u64,
        }
    }

    pub fn push(&mut self, value: T) -> u32 {
        if self.cursor >= self.collection.len() {
            error!(
                "enable to push, storage limit {} exceeded",
                self.collection.len()
            );
            return 0;
        }
        self.collection[self.cursor] = value;
        self.cursor += 1;
        (self.cursor - 1) as u32
    }

    pub fn extend(&mut self, values: &[T]) -> u32 {
        let count = values.len();
        if self.cursor + count >= self.collection.len() {
            error!(
                "unable to extend, storage limit {} exceeded",
                self.collection.len()
            );
            return 0;
        }
        self.collection[self.cursor..self.cursor + count].copy_from_slice(values);
        self.cursor += count;
        (self.cursor - count) as u32
    }

    pub fn is_empty(&self) -> bool {
        self.cursor == 0
    }

    pub fn take_and_update(&mut self, frame: usize) -> usize {
        let value = self.collection.as_slice();
        let count = self.cursor;
        self.cursor = 0;
        self.update_from(frame, value);
        count
    }

    pub fn update_from(&self, frame: usize, value: &[T]) {
        unsafe {
            let memory = self
                .device
                .map_memory(
                    self.buffers[frame].memory,
                    0,
                    (value.len() * size_of::<T>()) as u64,
                    MemoryMapFlags::empty(),
                )
                .expect("memory must be mapped");
            std::ptr::copy_nonoverlapping(value.as_ptr(), memory.cast(), value.len());
            self.device.unmap_memory(self.buffers[frame].memory);
        }
    }

    pub fn layout(&self, set: u32, binding: u32) -> Variable {
        let device = &self.device;
        let frames = self.buffers.len();
        unsafe {
            let bindings = vec![(
                binding,
                DescriptorType::STORAGE_BUFFER,
                ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX,
                1,
            )];
            let pool = create_descriptor_pool(device, &bindings, frames);
            let layout = create_descriptor_set_layout(device, bindings);
            let descriptors = create_descriptors(device, pool, layout, frames);
            let variable = Variable {
                set,
                binding,
                layout,
                descriptors,
            };
            for frame in 0..frames {
                self.write(device, frame, &variable);
            }
            variable
        }
    }

    fn write(&self, device: &Device, frame: usize, variable: &Variable) {
        let info = DescriptorBufferInfo::builder()
            .buffer(self.buffers[frame].handle)
            .offset(0)
            .range(self.range);
        let buffer_info = &[info];
        let buffer_write = WriteDescriptorSet::builder()
            .dst_set(variable.descriptors[frame])
            .dst_binding(variable.binding)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::STORAGE_BUFFER)
            .buffer_info(buffer_info);
        unsafe {
            device.update_descriptor_sets(&[buffer_write], &[] as &[CopyDescriptorSet]);
        }
    }
}
