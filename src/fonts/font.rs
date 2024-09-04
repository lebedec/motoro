use crate::math::{Vec2, Vec4};
pub use fontdue::layout::LayoutSettings;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use std::collections::HashMap;
use std::io;

pub struct Font {
    pub texture: String,
    pub charset: HashMap<char, Char>,
    pub font: fontdue::Font,
    pub size: f32,
    pub missing_char: Char,
    pub resolution_scale: f32,
    pub line_height: f32,
    pub baseline: f32,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Char {
    pub position: Vec2,
    pub image: Vec2,
    pub src: Vec2,
    pub uv: Vec2,
    pub size: Vec2,
    pub glyph_offset: f32,
    pub glyph_width: f32,
}

impl Font {
    /// NOTE: Resolution scale must be applied to layout coordinates for better kerning and spacing
    /// calculations in font engine. Result glyph x and y coordinates different depends on
    /// TextStyle size and layout settings. You can't just scale atlas texture with font letters!
    pub fn layout(&self, text: &str, max_width: f32, line_height: f32) -> Vec<Char> {
        let scale = self.resolution_scale;
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        let settings = LayoutSettings {
            max_width: Some(max_width * scale),
            line_height,
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
            draw.position = [glyph.x / scale, (glyph.y - draw.glyph_offset) / scale].into();
            // let char = glyph.parent;
            // if char == '$' || char == '&' || char == ',' || char == '+' || char == 'j' {
            //     println!(
            //         "GLYPH {char} pos{:?} gy{} goffset{}",
            //         draw.position, glyph.y, draw.glyph_offset
            //     );
            // }
            draws.push(draw);
        }
        draws
    }
}

pub const MISSING_CHAR: char = '□';

#[derive(Debug)]
pub struct FontError(pub String);

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
