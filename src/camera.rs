use crate::math::{
    mat4_from_scale, mat4_from_translation, mat4_identity, mat4_look_at_rh, mat4_mul,
    mat4_orthographic, Mat4, Vec2, Vec2u, Vec3, VecArith, VecComponents, VecNeg,
};
use crate::Graphics;

pub struct Camera {
    pub eye: Vec3,
    pub resolution_scale: f32,
    pub zoom: f32,
    screen: Vec2,
    resolution_reference: Option<[u32; 2]>,
}

impl Camera {
    pub fn create(graphics: &Graphics) -> Self {
        Self {
            eye: Default::default(),
            resolution_scale: 1.0,
            zoom: 1.0,
            screen: graphics.vulkan.swapchain_image_size(),
            resolution_reference: None,
        }
    }

    pub fn update(&mut self, graphics: &Graphics) {
        self.screen = graphics.vulkan.swapchain_image_size();
        if let Some(reference) = self.resolution_reference {
            self.resolution_scale = self.screen.y() / reference.y() as f32;
        }
    }

    pub fn viewport(&self) -> Vec2 {
        self.screen.div(self.resolution_scale)
    }

    pub fn reference(&mut self, resolution: Vec2u) {
        self.resolution_reference = Some(resolution);
    }

    pub fn offset(&self) -> Vec3 {
        // floor makes camera coordinates int
        // it eliminates artifacts of pixel perfect for now
        // [-self.eye.x.floor(), -self.eye.y.floor(), 0.0]
        self.eye.neg()
    }

    pub fn scaling(&self) -> Vec3 {
        [self.resolution_scale, self.resolution_scale, 1.0].mul(self.zoom)
    }

    pub fn get_transform(&self) -> Transform {
        let [width, height] = self.screen;
        let proj = mat4_orthographic(0.0, width, 0.0, height, 0.0, 1.0);
        let view = mat4_look_at_rh([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let model = mat4_mul(
            mat4_from_scale(self.scaling()),
            mat4_from_translation(self.offset()),
        );
        Transform { model, view, proj }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Transform {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            model: mat4_identity(),
            view: mat4_identity(),
            proj: mat4_identity(),
        }
    }
}
