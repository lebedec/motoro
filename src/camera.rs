use crate::math::{
    mat4_from_scale, mat4_from_translation, mat4_identity, mat4_look_at_rh, mat4_mul,
    mat4_orthographic, vec3_neg, vec3_scale, Mat4, Vec3,
};

pub struct Camera {
    pub eye: Vec3,
    pub resolution_scale: f32,
    pub input_scale: f32,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: Default::default(),
            resolution_scale: 1.0,
            input_scale: 1.0,
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub fn offset(&self) -> Vec3 {
        // floor makes camera coordinates int
        // it eliminates artifacts of pixel perfect for now
        // [-self.eye.x.floor(), -self.eye.y.floor(), 0.0]
        vec3_neg(self.eye)
    }

    pub fn scaling(&self) -> Vec3 {
        vec3_scale(
            [self.resolution_scale, self.resolution_scale, 1.0],
            self.zoom,
        )
    }

    pub fn get_transform(&self, screen: [f32; 2]) -> Transform {
        let [width, height] = screen;
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
