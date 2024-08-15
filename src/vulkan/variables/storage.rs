use crate::vulkan::{
    create_buffers, create_descriptor_pool, create_descriptor_set_layout, create_descriptors,
    MemoryBuffer, Vulkan,
};
use log::{error, info};
use std::marker::PhantomData;
use vulkanalia::vk::{
    Buffer, BufferUsageFlags, CopyDescriptorSet, DescriptorBufferInfo, DescriptorSet,
    DescriptorSetLayout, DescriptorType, DeviceV1_0, HasBuilder, InstanceV1_0, MemoryMapFlags,
    ShaderStageFlags, WriteDescriptorSet,
};
use vulkanalia::Device;

/// Represents GLSL variable declared as storage buffer.
///
/// ```glsl
/// layout (std140, set = 0, binding = 4) readonly buffer Canvas {
///     Element elements[];
/// } canvas;
/// ```
pub struct Storage<T> {
    pub(crate) slot: u32,
    pub(crate) binding: u32,
    layout: DescriptorSetLayout,
    sets: Vec<DescriptorSet>,
    buffers: Vec<MemoryBuffer>,
    device: Device,
    collection: Vec<T>,
    cursor: usize,
    _phantom: PhantomData<T>,
}

impl<T: Default + Clone> Storage<T> {
    pub fn layout(&self) -> DescriptorSetLayout {
        self.layout
    }

    pub fn descriptor(&self, frame: usize) -> DescriptorSet {
        self.sets[frame]
    }

    pub unsafe fn create_many(slot: u32, binding: u32, vulkan: &Vulkan, n: usize) -> Self {
        let device = &vulkan.device;
        let frames = vulkan.swapchain.images.len();
        let bindings = vec![(
            binding,
            DescriptorType::STORAGE_BUFFER,
            ShaderStageFlags::FRAGMENT | ShaderStageFlags::VERTEX,
            1,
        )];
        let pool = create_descriptor_pool(device, &bindings, frames);
        let layout = create_descriptor_set_layout(device, bindings);
        let sets = create_descriptors(device, pool, layout, frames);
        let physical_device_memory = vulkan
            .instance
            .get_physical_device_memory_properties(vulkan.physical_device);
        info!(
            "Creates storage buffers n={} size={} mem={}",
            n,
            std::mem::size_of::<T>(),
            n * std::mem::size_of::<T>()
        );
        let buffers = create_buffers(
            BufferUsageFlags::STORAGE_BUFFER,
            device,
            frames,
            physical_device_memory,
            n * std::mem::size_of::<T>(),
        );
        let storage = Self {
            slot,
            binding,
            layout,
            sets,
            buffers,
            device: device.clone(),
            _phantom: Default::default(),
            collection: vec![T::default(); n],
            cursor: 0,
        };
        for i in 0..frames {
            storage.write(i, storage.buffers[i].handle, n);
        }
        storage
    }

    pub fn push(&mut self, value: T) -> u32 {
        if self.cursor >= self.collection.len() {
            error!("storage limit exceeded");
            return 0;
        }
        self.collection[self.cursor] = value;
        self.cursor += 1;
        (self.cursor - 1) as u32
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
                    (value.len() * std::mem::size_of::<T>()) as u64,
                    MemoryMapFlags::empty(),
                )
                .expect("memory must be mapped");
            std::ptr::copy_nonoverlapping(value.as_ptr(), memory.cast(), value.len());
            self.device.unmap_memory(self.buffers[frame].memory);
        }
    }

    fn write(&self, frame: usize, buffer: Buffer, n: usize) {
        let info = DescriptorBufferInfo::builder()
            .buffer(buffer)
            .offset(0)
            .range(n as u64 * std::mem::size_of::<T>() as u64);
        let buffer_info = &[info];
        let buffer_write = WriteDescriptorSet::builder()
            .dst_set(self.sets[frame])
            .dst_binding(self.binding)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::STORAGE_BUFFER)
            .buffer_info(buffer_info);
        unsafe {
            self.device
                .update_descriptor_sets(&[buffer_write], &[] as &[CopyDescriptorSet]);
        }
    }
}
