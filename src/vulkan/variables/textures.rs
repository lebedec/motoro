use crate::{Texture, Variable};
use log::info;
use vulkanalia::vk::{
    DescriptorPoolCreateFlags, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutCreateFlags,
    DescriptorType, DeviceV1_0, HasBuilder, Sampler, ShaderStageFlags,
};
use vulkanalia::{vk, Device};

/// Represents GLSL sampler2d non-uniform qualifier.
/// ```glsl
/// #extension GL_EXT_nonuniform_qualifier: require
/// layout (set = 1, binding = 0) uniform sampler2D textures[];
/// ```
pub struct Textures {
    pub(crate) slot: u32,
    pub(crate) binding: u32,
    max_descriptors: u32,
    layout: DescriptorSetLayout,
    set: DescriptorSet,
    textures: Vec<Texture>,
    device: Device,
}

impl Textures {
    pub fn layout(&self) -> DescriptorSetLayout {
        self.layout
    }

    pub fn descriptor(&self) -> DescriptorSet {
        self.set
    }

    pub fn create(slot: u32, binding: u32, device: &Device) -> Self {
        info!("Creates bindless texture, layout(set = {slot}, binding = {binding})");
        let max_descriptors = 256;
        // layout
        let bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(binding)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(max_descriptors)
            .stage_flags(ShaderStageFlags::ALL)
            .build()];
        let binding_flags = [
            //vk::DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT
            vk::DescriptorBindingFlags::PARTIALLY_BOUND
                | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND, // | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING
        ];
        let mut binding_flags = vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
            .binding_flags(&binding_flags)
            .build();
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings)
            .flags(DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .push_next(&mut binding_flags);
        let layout = unsafe {
            device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("descriptor set layout must be created")
        };
        // pool
        let pool_sizes = [vk::DescriptorPoolSize::builder()
            .type_(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(max_descriptors) // max_descriptors ?
            .build()];
        let pool = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(1)
            .flags(DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .build();
        let pool = unsafe {
            device
                .create_descriptor_pool(&pool, None)
                .expect("descriptor pool must be created")
        };
        let layouts = [layout];
        let descriptors = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(&layouts)
            .build();
        let descriptors = unsafe {
            device
                .allocate_descriptor_sets(&descriptors)
                .expect("descriptor sets must be created")
        };
        Self {
            slot,
            binding,
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
                let image = [vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(texture.view)
                    .sampler(sampler)
                    .build()];
                let write = vk::WriteDescriptorSet::builder()
                    .dst_set(self.set)
                    .dst_binding(self.binding)
                    .dst_array_element(index)
                    .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image)
                    .build();
                let writes = [write];
                unsafe {
                    self.device
                        .update_descriptor_sets(&writes, &[] as &[vk::CopyDescriptorSet]);
                }
                self.textures.push(texture);
                index
            }
            Some(index) => index as u32,
        }
    }
}
