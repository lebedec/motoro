use crate::vulkan::{create_pipeline, Swapchain};
use crate::{Mesh, Shader, Storage, Uniform};
use log::info;
use vulkanalia::vk::{DeviceV1_0, Handle, HasBuilder, PipelineVertexInputStateCreateInfo};
use vulkanalia::{vk, Device};

pub struct Program {
    name: String,
    pub device: Device,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    pub(crate) vert: Shader,
    pub(crate) frag: Shader,
    pub sampler: vk::Sampler,
    push_constants: Vec<vk::PushConstantRange>,
    layouts: Vec<vk::DescriptorSetLayout>,
    current_commands: vk::CommandBuffer,
    current_frame: usize,
    vertex_input_state: PipelineVertexInputStateCreateInfo,
}

pub fn range<T>() -> vk::PushConstantRange {
    vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(std::mem::size_of::<T>() as u32)
        .build()
}

impl Program {
    pub fn frame(&self) -> usize {
        self.current_frame
    }

    pub unsafe fn create(
        name: &str,
        // instance: &Instance,
        device: &Device,
        // physical_device: vk::PhysicalDevice,
        swapchain: &Swapchain,
        render_pass: vk::RenderPass,
        mut vert: Shader,
        mut frag: Shader,
        push_constants: Vec<vk::PushConstantRange>,
        sampler: vk::Sampler,
        layouts: Vec<vk::DescriptorSetLayout>,
        vertex_input_state: PipelineVertexInputStateCreateInfo,
    ) -> Self {
        let (pipeline_layout, pipeline) = create_pipeline(
            &device,
            &swapchain,
            render_pass,
            layouts.clone(),
            &vert.read(),
            &frag.read(),
            push_constants.clone(),
            vertex_input_state,
        );
        info!("Creates {name} {:?}", pipeline);
        Self {
            name: name.to_string(),
            device: device.clone(),
            pipeline_layout,
            pipeline,
            vert,
            frag,
            sampler,
            push_constants,
            current_commands: vk::CommandBuffer::null(),
            current_frame: 0,
            layouts,
            vertex_input_state,
        }
    }

    pub fn commands(&self) -> vk::CommandBuffer {
        if self.current_commands == vk::CommandBuffer::null() {
            panic!("program command buffer must be configured")
        }
        self.current_commands
    }

    pub fn set_command_buffer(&mut self, commands: vk::CommandBuffer) {
        self.current_commands = commands
    }

    pub fn set_chain(&mut self, chain: usize) {
        self.current_frame = chain;
    }

    pub fn bind_pipeline(&mut self) {
        unsafe {
            self.device.cmd_bind_pipeline(
                self.commands(),
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
        }
    }

    pub fn bind_uniform<T>(&self, variable: &Uniform<T>) {
        self.bind_descriptor(variable.slot, variable.descriptor(self.current_frame));
    }

    pub fn bind_storage<T>(&self, variable: &Storage<T>) {
        self.bind_descriptor(variable.slot, variable.descriptor(self.current_frame));
    }

    pub fn bind_mesh(&self, mesh: &Mesh) {
        unsafe {
            let buf = self.current_commands;
            self.device
                .cmd_bind_vertex_buffers(buf, 0, &[mesh.buffer.handle], &[0]);
        }
    }

    pub fn bind_descriptor(&self, index: u32, set: vk::DescriptorSet) {
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                self.commands(),
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                index,
                &[set],
                &[],
            );
        }
    }

    pub fn push_constants<T>(&self, value: &T) {
        let buf = self.current_commands;
        unsafe {
            let size = std::mem::size_of::<T>();
            let constants = std::slice::from_raw_parts(value as *const T as *const u8, size);
            self.device.cmd_push_constants(
                buf,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                constants,
            );
        }
    }

    pub unsafe fn destroy(&mut self) {
        info!("Destroy program: {} {:?}", self.name, self.pipeline);
        let device = &self.device;
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
    }

    pub unsafe fn recreate(&mut self, swapchain: &Swapchain, render_pass: vk::RenderPass) {
        self.destroy();
        info!("Renew program: {} {:?}", self.name, self.pipeline);
        self.vert = self.vert.renew();
        self.frag = self.frag.renew();
        let (pipeline_layout, pipeline) = create_pipeline(
            &self.device,
            &swapchain,
            render_pass,
            self.layouts.clone(),
            &self.vert.read(),
            &self.frag.read(),
            self.push_constants.clone(),
            self.vertex_input_state.clone(),
        );
        self.pipeline = pipeline;
        self.pipeline_layout = pipeline_layout;
    }

    pub fn draw(&self, vertex_count: usize, elements: usize) {
        unsafe {
            let buf = self.current_commands;
            self.device
                .cmd_draw(buf, vertex_count as u32, elements as u32, 0, 0);
        }
    }
}
