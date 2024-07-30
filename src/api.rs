use crate::math::Vec2;
pub use crate::textures::*;
pub use crate::vulkan::program::*;
pub use crate::vulkan::shaders::*;
pub use crate::vulkan::variables::*;
use crate::Graphics;
use vulkanalia::vk;
use vulkanalia::vk::{DeviceV1_0, HasBuilder, PipelineVertexInputStateCreateInfo};

impl Graphics {
    pub fn create_sampler(&self) -> ImageSampler {
        ImageSampler::create(&self.vulkan.device, 100)
    }

    pub fn sampler(&self) -> Sampler2D {
        Sampler2D::create(&self.vulkan.device)
    }

    pub fn uniform<T>(&self, slot: u32, binding: u32) -> Uniform<T> {
        unsafe { Uniform::create(slot, binding, &self.vulkan) }
    }

    pub fn storage<T>(&self, slot: u32, binding: u32, n: usize) -> Storage<T> {
        unsafe { Storage::create_many(slot, binding, &self.vulkan, n) }
    }

    pub fn mesh(&self, vertices: &[Vertex2D]) -> Mesh {
        unsafe { Mesh::create(vertices, &self.vulkan) }
    }

    pub fn create_pixel_perfect_sampler(&self) -> vk::Sampler {
        let info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .anisotropy_enable(false)
            .max_anisotropy(16.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .min_lod(0.0)
            .max_lod(0.0)
            .mip_lod_bias(0.0);
        unsafe {
            self.vulkan
                .device
                .create_sampler(&info, None)
                .expect("sampler must be created")
        }
    }

    pub fn create_program(
        &mut self,
        name: &str,
        vert: Shader,
        frag: Shader,
        push_constants: Vec<vk::PushConstantRange>,
        sampler: vk::Sampler,
        layouts: Vec<vk::DescriptorSetLayout>,
        vertex_input_state: PipelineVertexInputStateCreateInfo,
    ) -> Box<Program> {
        let program = unsafe {
            Program::create(
                name,
                &self.vulkan.device,
                &self.vulkan.swapchain,
                self.vulkan.render_pass,
                vert,
                frag,
                push_constants,
                sampler,
                layouts,
                vertex_input_state,
            )
        };
        let mut program = Box::new(program);
        self.vulkan.register(&mut program);
        program
    }

    pub fn chain(&self) -> usize {
        self.vulkan.chain
    }

    pub fn screen(&self) -> Vec2 {
        self.vulkan.screen()
    }
}

/*
let textures = ImageSampler::create(&vulkan.device, 100);
            let storages = StorageBufferOld::create(&vulkan);
            let transform = UniformBuffer::create(&vulkan);
            let vertex_buffer = create_vertex_buffer(
                &vulkan.device,
                &vulkan.instance,
                vulkan.physical_device,
                &SHAPE_VERTICES,
            );
            let program = Program::create(
                "shapes",
                &vulkan.device,
                &vulkan.swapchain,
                vulkan.render_pass,
                ShaderFile::new(fs.get_asset_path("./shaders/shapes.vert.spv")),
                ShaderFile::new(fs.get_asset_path("./shaders/shapes.frag.spv")),
                vec![ShapePushConstants::range()],
                create_pixel_perfect_sampler(&vulkan.device),
                vec![transform.layout, textures.layout, storages.layout],
                Vertex::input_state(),
            );
 */
