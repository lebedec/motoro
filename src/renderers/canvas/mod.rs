#[cfg(feature = "canvas-bumaga")]
pub mod bumaga;

use crate::math::{Vec2, Vec4};
use crate::{
    Font, Graphics, Program, Sampler2D, Shader, Storage, Texture, Transform, Uniform, Vertex2D,
};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Elem {
    pub position: Vec2,
    pub image: Vec2,
    pub src: Vec2,
    pub uv: Vec2,
    pub size: Vec2,
    // pub kind: u32,
    // pub texture: u32,
    // pub brush: u32,
    pub _unused: [f32; 2],
    pub attrs: [u32; 4],
}

impl Elem {
    pub const IMAGE: u32 = 0;
    pub const RECTANGLE: u32 = 1;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Brush {
    pub fg: Vec4,
    pub bg: Vec4,
    pub radius: Vec4,
    pub border: f32,
    pub _unused: [f32; 3],
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            fg: [1.0; 4],
            bg: [1.0; 4],
            radius: [0.0; 4],
            border: 0.0,
            _unused: Default::default(),
        }
    }
}

const MAX_ELEMENTS: usize = 4096;
const MAX_BRUSHES: usize = 4096;

pub struct CanvasRenderer {
    program: Box<Program>,
    transform: Uniform<Transform>,
    textures: Sampler2D,
    elements: Storage<Elem>,
    brushes: Storage<Brush>,
}

impl CanvasRenderer {
    pub fn new(graphics: &mut Graphics) -> Self {
        unsafe {
            let elements = graphics.storage(0, 4, MAX_ELEMENTS);
            let textures = graphics.sampler(1, 0);
            let transform = graphics.uniform(2, 0);
            let brushes = graphics.storage(3, 4, MAX_ELEMENTS);

            let program = graphics.create_program(
                "canvas",
                Shader::new("./assets/shaders/canvas.vert.spv"),
                Shader::new("./assets/shaders/canvas.frag.spv"),
                vec![],
                graphics.create_pixel_perfect_sampler(),
                vec![
                    elements.layout(),
                    textures.layout,
                    transform.layout(),
                    brushes.layout(),
                ],
                Vertex2D::no_input(),
            );

            Self {
                program,
                transform,
                textures,
                elements,
                brushes,
            }
        }
    }

    pub fn bind(&mut self, transform: Transform) {
        self.program.bind_pipeline();
        self.transform.update(self.program.frame(), &transform);
        self.program.bind_uniform(&self.transform);
    }

    pub fn render_text(
        &mut self,
        text: &str,
        color: Vec4,
        position: Vec2,
        max_width: f32,
        font: &Font,
        texture: Texture,
    ) {
        let chars = font.layout(position, max_width, &text);
        for char in chars {
            let element = Elem {
                position: char.position,
                image: char.image,
                src: char.src,
                uv: char.uv,
                size: char.size,
                // kind: Elem::IMAGE,
                // texture: self.textures.store(texture, self.program.sampler),
                // brush: 0,
                _unused: Default::default(),
                attrs: [
                    Elem::IMAGE,
                    self.textures.store(texture, self.program.sampler),
                    0,
                    0,
                ],
            };
            self.render(element, Brush::default(), texture);
        }
    }

    pub fn render(&mut self, mut element: Elem, brush: Brush, texture: Texture) {
        element.attrs[1] = self.textures.store(texture, self.program.sampler);
        element.attrs[2] = self.brushes.push(brush);
        self.elements.push(element);
    }

    pub fn draw(&mut self) {
        unsafe {
            if self.elements.is_empty() {
                return;
            }

            let count = self.elements.take_and_update(self.program.frame());
            self.brushes.take_and_update(self.program.frame());
            self.program.bind_storage(&self.elements);
            self.program.bind_storage(&self.brushes);
            self.program.bind_descriptor(1, self.textures.set);
            self.program.draw(6, count);
        }
    }
}
