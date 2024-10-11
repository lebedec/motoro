use vulkanalia::vk::{DescriptorSet, DescriptorSetLayout};

pub struct Variable {
    pub set: u32,
    pub binding: u32,
    pub layout: DescriptorSetLayout,
    pub descriptors: Vec<DescriptorSet>,
}

impl Variable {
    pub fn descriptor(&self, frame: usize) -> DescriptorSet {
        self.descriptors[frame]
    }
}
