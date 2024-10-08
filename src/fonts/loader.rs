use crate::fonts::rasterize_font_to_image_file;
use crate::math::{Vec3, VecArith, VecMagnitude};
use crate::{Font, FontError, MISSING_CHAR};
use log::info;
use std::fs;
use std::sync::{Arc, RwLock};

struct Record {
    family: String,
    weight: u16,
    style: String,
    size: f32,
    font: Font,
}

impl Record {
    fn diff(&self, weigth: u16, style: &str, size: f32) -> f32 {
        let search = Self::embed(weigth, style, size);
        let target = Self::embed(self.weight, &self.style, self.size);
        target.sub(search).magnitude()
    }

    #[inline(always)]
    fn embed(weight: u16, style: &str, size: f32) -> Vec3 {
        let style = match style {
            "normal" => 0.0,
            "italic" => 1.0,
            "oblique" => 2.0,
            _ => 9.0,
        };
        [size * 1000.0, weight as f32, style]
    }
}

pub type FontLoaderHandle = Arc<RwLock<FontLoader>>;

pub struct FontLoader {
    resolution_scale: f32,
    registry: Vec<Record>,
    cache: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FontIndex(usize);

impl FontLoader {
    pub fn new(cache: &str, resolution_scale: f32) -> FontLoaderHandle {
        info!("Creates font loader");
        let default = include_bytes!("builtin/Roboto/Roboto-Regular.ttf");
        let mut loader = Self {
            resolution_scale,
            registry: vec![],
            cache: cache.to_string(),
        };
        loader
            .load_font(
                "system-ui",
                400,
                "normal",
                16.0,
                &(ascii() + &cyrillic()),
                default,
            )
            .expect("default font must be loaded");
        Arc::new(RwLock::new(loader))
    }

    pub fn load_font_file(
        &mut self,
        family: &str,
        weight: u16,
        style: &str,
        size: f32,
        alphabet: &str,
        path: &str,
    ) -> Result<&Font, FontError> {
        let data = fs::read(path).map_err(|error| FontError(error.to_string()))?;
        self.load_font(family, weight, style, size, alphabet, &data)
    }

    pub fn load_font(
        &mut self,
        family: &str,
        weight: u16,
        style: &str,
        size: f32,
        alphabet: &str,
        data: &[u8],
    ) -> Result<&Font, FontError> {
        let font = rasterize_font_to_image_file(
            data,
            &self.cache,
            &format!("{family}-{weight}-{style}"),
            alphabet,
            size,
            self.resolution_scale,
        )?;
        self.registry.push(Record {
            family: family.to_string(),
            weight,
            style: style.to_string(),
            size,
            font,
        });
        Ok(&self.registry[self.registry.len() - 1].font)
    }

    pub fn match_font(&self, family: &str, weight: u16, style: &str, size: f32) -> FontIndex {
        let mut best = 0;
        let mut best_diff = f32::INFINITY;
        for (index, record) in self.registry.iter().enumerate() {
            if record.family == family {
                let diff = record.diff(weight, style, size);
                if diff < best_diff {
                    best_diff = diff;
                    best = index;
                }
                if diff == 0.0 {
                    break;
                }
            }
        }
        FontIndex(best)
    }

    #[inline(always)]
    pub fn get_font(&self, index: FontIndex) -> &Font {
        &self.registry[index.0].font
    }
}

pub fn ascii() -> String {
    let mut string = String::from(MISSING_CHAR);
    for code in 0x20..=0x7e {
        string.push(code as u8 as char);
    }
    string
}

pub fn cyrillic() -> String {
    let mut string = String::from(MISSING_CHAR);
    for code in 0x0400..=0x04FF {
        string.push(unsafe { char::from_u32_unchecked(code) });
    }
    string
}
