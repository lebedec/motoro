use crate::math::{
    mat4_from_scale, mat4_from_translation, mat4_identity, mat4_look_at_rh, mat4_mul,
    mat4_orthographic, Mat4, Vec2, Vec2u, Vec3, VecArith, VecComponents, VecMagnitude, VecNeg,
};
use crate::{Graphics, UserInput};
use sdl2::keyboard::Keycode;
use sdl2::sys::KeyCode;

pub struct Camera {
    pub eye: Vec3,
    pub eye_target: Vec3,
    pub resolution_scale: f32,
    pub zoom: f32,
    screen: Vec2,
    resolution_reference: Option<[u32; 2]>,
    pub enabled: bool,
    pub control_speed: f32,
    pub speed: f32,
}

impl Camera {
    pub fn create(graphics: &Graphics) -> Self {
        Self {
            eye: [0.0; 3],
            eye_target: [0.0; 3],
            resolution_scale: 1.0,
            zoom: 1.0,
            screen: graphics.vulkan.swapchain_image_size(),
            resolution_reference: None,
            enabled: false,
            control_speed: 100.0,
            speed: 100.0,
        }
    }

    pub fn update(&mut self, graphics: &Graphics) {
        self.screen = graphics.vulkan.swapchain_image_size();
        if let Some(reference) = self.resolution_reference {
            self.resolution_scale = self.screen.y() / reference.y() as f32;
        }
        if self.enabled {
            self.control(&graphics.input)
        }
    }

    fn control(&mut self, input: &UserInput) {
        if input.mouse.wheel.y() > 0.0 {
            self.zoom -= 0.05;
        }
        if input.mouse.wheel.y() < 0.0 {
            self.zoom += 0.05;
        }
        let mut delta = [0.0, 0.0, 0.0];
        if input.keys.down.contains(&Keycode::W) {
            delta[1] -= 1.0;
        }
        if input.keys.down.contains(&Keycode::A) {
            delta[0] -= 1.0;
        }
        if input.keys.down.contains(&Keycode::S) {
            delta[1] += 1.0;
        }
        if input.keys.down.contains(&Keycode::D) {
            delta[0] += 1.0;
        }
        let time = input.time.as_secs_f32();
        let delta = delta.normal().mul(time * self.control_speed);

        self.eye_target = self.eye_target.add(delta);
        let direction = self.eye_target.sub(self.eye);

        let distance = direction.magnitude();
        let step = self.speed * time;
        if distance < step {
            self.eye = self.eye_target;
        } else {
            let direction = direction.normal();
            self.eye = self.eye.add(direction.mul(step))
        }
    }

    pub fn reset_transform(&mut self) {
        self.eye = [0.0; 3];
        self.zoom = 1.0;
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
