use std::io;


use vulkanalia::vk::DeviceV1_0;
use vulkanalia::{vk, Device};
use zune_png::error::PngDecodeErrors;

/// TODO: abstract away from Vulkan handles
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Texture {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub size: [u32; 2],
}

impl Texture {
    pub const FALLBACK: &'static str = "<fallback>";
    pub const BLANK: &'static str = "<blank>";
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

impl Texture {
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);
        }
    }
}
