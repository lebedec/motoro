use crate::fonts::rasterize_font_to_image_file;
use crate::{Font, MISSING_CHAR};
use log::info;
use std::sync::{Arc, RwLock};

pub type FontLoaderHandle = Arc<RwLock<FontLoader>>;

pub struct FontLoader {
    pub default: Font,
    resolution_scale: f32,
}

impl FontLoader {
    pub fn new(cache: &str, resolution_scale: f32) -> FontLoaderHandle {
        info!("Creates font loader");
        let default = include_bytes!("builtin/Roboto/Roboto-Regular.ttf");
        let default = rasterize_font_to_image_file(
            default,
            cache,
            "default",
            &(ascii() + &cyrillic()),
            16.0,
            resolution_scale,
        )
        .expect("default font must be created");
        let loader = Self {
            default,
            resolution_scale,
        };
        Arc::new(RwLock::new(loader))
    }

    pub fn get_font(&self, path: &str, size: f32) -> &Font {
        &self.default
    }
}

fn ascii() -> String {
    let mut string = String::from(MISSING_CHAR);
    for code in 0x20..=0x7e {
        string.push(code as u8 as char);
    }
    string
}

fn cyrillic() -> String {
    let mut string = String::from(MISSING_CHAR);
    for code in 0x0400..=0x04FF {
        string.push(unsafe { char::from_u32_unchecked(code) });
    }
    string
}
