use crate::math::{Vec2, Vec4};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use std::collections::HashMap;
use std::io;

pub struct Font {
    pub texture: String,
    pub charset: HashMap<char, Char>,
    pub font: fontdue::Font,
    pub size: f32,
    pub missing_char: Char,
    pub resolution_scale: f32,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Char {
    pub position: Vec2,
    pub image: Vec2,
    pub src: Vec2,
    pub uv: Vec2,
    pub size: Vec2,
    pub height: f32,
}

impl Font {
    /// NOTE: Resolution scale must be applied to layout coordinates for better kerning and spacing
    /// calculations in font engine. Result glyph x and y coordinates different depends on
    /// TextStyle size and layout settings. You can't just scale atlas texture with font letters!
    pub fn layout(&self, start: Vec2, max_width: f32, text: &str) -> Vec<Char> {
        let scale = self.resolution_scale;
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        let settings = LayoutSettings {
            x: start[0] * scale,
            y: start[1] * scale,
            // HACK: font size added to fix layout recalculation in same max_width
            max_width: Some(max_width * scale + self.size),
            line_height: 1.2,
            ..LayoutSettings::default()
        };
        layout.reset(&settings);
        let text = TextStyle::new(text, self.size, 0);
        let fonts = [&self.font];
        layout.append(&fonts, &text);
        let mut draws = vec![];
        for glyph in layout.glyphs() {
            let mut draw = match self.charset.get(&glyph.parent) {
                Some(char) => *char,
                None => self.missing_char,
            };
            draw.position = [glyph.x / scale, glyph.y / scale].into();
            draws.push(draw);
        }
        draws
    }
}

pub const MISSING_CHAR: char = '□';

#[derive(Debug)]
pub struct FontError(String);

impl From<&str> for FontError {
    fn from(error: &str) -> Self {
        FontError(error.to_string())
    }
}

impl From<io::Error> for FontError {
    fn from(error: io::Error) -> Self {
        FontError(error.to_string())
    }
}
