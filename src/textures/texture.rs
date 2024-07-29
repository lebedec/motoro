use std::io;

use vulkanalia::vk;
use zune_png::error::PngDecodeErrors;

/// TODO: abstract away from Vulkan handles
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Texture {
    pub id: usize,
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
}

impl PartialOrd<Texture> for Texture {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Texture {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.id > other.id {
            std::cmp::Ordering::Greater
        } else if self.id < other.id {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
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
