pub use crate::colors::*;
use crate::math::{Vec2, VecArith};
pub use crate::textures::*;
pub use crate::vulkan::program::*;
pub use crate::vulkan::shaders::*;
pub use crate::vulkan::variables::*;
use crate::{Camera, Graphics};
use vulkanalia::vk;
use vulkanalia::vk::{DeviceV1_0, HasBuilder, PipelineVertexInputStateCreateInfo};

impl Graphics {
    pub fn create_sampler(&self) -> ImageSampler {
        ImageSampler::create(&self.vulkan.device, 100)
    }

    pub fn camera(&self) -> Camera {
        Camera::create(self)
    }

    pub fn sampler(&self, slot: u32, binding: u32) -> Textures {
        Textures::create(slot, binding, &self.vulkan.device)
    }

    pub fn uniform<T>(&self, slot: u32, binding: u32) -> Uniform<T> {
        unsafe { Uniform::create(slot, binding, &self.vulkan) }
    }

    pub fn storage<T>(&self, slot: u32, binding: u32, n: usize) -> Storage<T>
    where
        T: Default + Clone + Copy,
    {
        unsafe { Storage::create_many(slot, binding, &self.vulkan, n) }
    }

    pub fn mesh(&self, n: usize) -> Mesh {
        unsafe { Mesh::create(&self.vulkan, n) }
    }

    pub fn texture_from(&self, width: u32, height: u32, data: &[u8]) -> Texture {
        self.textures.create_texture(width, height, data)
    }

    pub fn create_pixel_perfect_sampler(&self) -> vk::Sampler {
        let info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .address_mode_v(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .address_mode_w(vk::SamplerAddressMode::MIRRORED_REPEAT)
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
        vertex_input: Option<PipelineVertexInputStateCreateInfo>,
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
                vertex_input,
            )
        };
        let mut program = Box::new(program);
        self.vulkan.register(&mut program);
        program
    }

    pub fn chain(&self) -> usize {
        self.vulkan.chain
    }

    pub fn destroy_texture(&self, texture: Texture) {
        texture.destroy(&self.vulkan.device);
    }

    pub fn destroy_mesh(&self, mesh: &Mesh) {
        mesh.destroy();
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
