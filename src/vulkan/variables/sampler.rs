use crate::Texture;
use log::info;
use vulkanalia::vk::{
    DescriptorPoolCreateFlags, DescriptorSet, DescriptorSetLayoutCreateFlags, DescriptorType,
    DeviceV1_0, HasBuilder, Sampler, ShaderStageFlags,
};
use vulkanalia::{vk, Device};

pub struct Sampler2D {
    max_descriptors: u32,
    pub layout: vk::DescriptorSetLayout,
    pub set: DescriptorSet,
    textures: Vec<Texture>,
    device: Device,
}

impl Sampler2D {
    pub fn create(device: &Device) -> Self {
        let max_descriptors = 256;
        // layout
        let binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(max_descriptors)
            .stage_flags(ShaderStageFlags::ALL);
        let binding_flags = [vk::DescriptorBindingFlags::PARTIALLY_BOUND
            | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND];
        let mut binding_flags =
            vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder().binding_flags(&binding_flags);
        let layout = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&[binding])
            .flags(DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .push_next(&mut binding_flags)
            .build();
        let layout = unsafe {
            device
                .create_descriptor_set_layout(&layout, None)
                .expect("descriptor set layout must be created")
        };
        // pool
        let pool_size = vk::DescriptorPoolSize::builder()
            .type_(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(max_descriptors) // max_descriptors ?
            .build();
        let pool = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&[pool_size])
            .max_sets(1)
            .flags(DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .build();
        let pool = unsafe {
            device
                .create_descriptor_pool(&pool, None)
                .expect("descriptor pool must be created")
        };
        // create
        let descriptors = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(&[layout])
            .build();
        let descriptors = unsafe {
            device
                .allocate_descriptor_sets(&descriptors)
                .expect("descriptor sets must be created")
        };
        info!("Creates {} bindless variables", descriptors.len());
        Self {
            max_descriptors,
            layout,
            set: descriptors[0],
            textures: vec![],
            device: device.clone(),
        }
    }

    pub fn store(&mut self, texture: Texture, sampler: Sampler) -> u32 {
        match self
            .textures
            .iter()
            .position(|record| record.image == texture.image)
        {
            None => {
                let index = self.textures.len() as u32;
                if index == self.max_descriptors {
                    panic!("unable to store texture, all variables are used up")
                }
                let image = vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(texture.view)
                    .sampler(sampler)
                    .build();
                let write = vk::WriteDescriptorSet::builder()
                    .dst_set(self.set)
                    .dst_binding(0)
                    .dst_array_element(index)
                    .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&[image])
                    .build();
                unsafe {
                    self.device
                        .update_descriptor_sets(&[write], &[] as &[vk::CopyDescriptorSet]);
                }
                self.textures.push(texture);
                index
            }
            Some(index) => index as u32,
        }
    }
}
