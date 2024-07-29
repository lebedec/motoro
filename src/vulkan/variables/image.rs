use crate::vulkan::{create_descriptor_pool, create_descriptor_set_layout, create_descriptors};
use crate::Texture;
use std::collections::HashMap;
use vulkanalia::vk::{DescriptorImageInfo, DeviceV1_0, HasBuilder, WriteDescriptorSet};
use vulkanalia::{vk, Device};

pub struct ImageSampler {
    pool: vk::DescriptorPool,
    pub layout: vk::DescriptorSetLayout,
    sets: HashMap<u64, vk::DescriptorSet>,
}

impl ImageSampler {
    pub fn create(device: &Device, sets: usize) -> ImageSampler {
        Self::create_array(device, sets, 1)
    }

    pub fn create_array(device: &Device, sets: usize, count: usize) -> ImageSampler {
        let bindings = vec![(
            0,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            vk::ShaderStageFlags::FRAGMENT,
            count,
        )];
        unsafe {
            let pool = create_descriptor_pool(device, &bindings, sets);
            let layout = create_descriptor_set_layout(device, bindings);
            ImageSampler {
                pool,
                layout,
                sets: Default::default(),
            }
        }
    }

    pub fn describe(
        &mut self,
        texture: Texture,
        sampler: vk::Sampler,
        device: &Device,
    ) -> vk::DescriptorSet {
        let factory = || unsafe {
            let set = create_descriptors(device, self.pool, self.layout, 1)[0];
            Self::write_many(set, sampler, vec![texture], device);
            set
        };
        *self.sets.entry(texture.id as u64).or_insert_with(factory)
    }

    pub fn create_descriptors(&self, device: &Device, count: usize) -> Vec<vk::DescriptorSet> {
        unsafe { create_descriptors(device, self.pool, self.layout, count) }
    }

    pub fn create_descriptor(&self, device: &Device) -> vk::DescriptorSet {
        unsafe { create_descriptors(device, self.pool, self.layout, 1)[0] }
    }

    pub fn write_many_bindless(
        set: vk::DescriptorSet,
        sampler: vk::Sampler,
        texture: Vec<Texture>,
        device: &Device,
    ) {
        let mut writes = [WriteDescriptorSet::default(); 3];
        let mut images = [DescriptorImageInfo::default(); 3];

        unsafe {
            for (index, texture) in texture.into_iter().enumerate() {
                images[index] = vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(texture.view)
                    .sampler(sampler)
                    .build();
                writes[index] = vk::WriteDescriptorSet::builder()
                    .dst_set(set)
                    .dst_binding(0)
                    .dst_array_element(index as u32)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&[images[index]])
                    .build();
            }
            device.update_descriptor_sets(&writes, &[] as &[vk::CopyDescriptorSet]);
        }
    }

    pub fn write_many(
        set: vk::DescriptorSet,
        sampler: vk::Sampler,
        texture: Vec<Texture>,
        device: &Device,
    ) {
        unsafe {
            let images: Vec<_> = texture
                .iter()
                .map(|texture| {
                    vk::DescriptorImageInfo::builder()
                        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(texture.view)
                        .sampler(sampler)
                        .build()
                })
                .collect();
            let write = vk::WriteDescriptorSet::builder()
                .dst_set(set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(images.as_slice());
            device.update_descriptor_sets(&[write], &[] as &[vk::CopyDescriptorSet]);
        }
    }
}
