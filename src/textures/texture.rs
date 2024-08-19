use std::io;

use crate::math::Vec2;
use vulkanalia::vk;
use zune_png::error::PngDecodeErrors;

/// TODO: abstract away from Vulkan handles
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Texture {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub size: [u32; 2],
}

#[derive(Debug)]
pub struct TextureError(String);

impl From<&str> for TextureError {
    fn from(error: &str) -> Self {
        TextureError(error.to_string())
    }
}

impl From<io::Error> for TextureError {
    fn from(error: io::Error) -> Self {
        TextureError(error.to_string())
    }
}

impl From<PngDecodeErrors> for TextureError {
    fn from(error: PngDecodeErrors) -> Self {
        TextureError(error.to_string())
    }
}
