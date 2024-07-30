use crate::input::poll_event;
use crate::textures::TextureLoader;
use crate::vulkan::Vulkan;
use crate::{dpi, FontLoader, GraphicsConfig, GraphicsMode};
use log::info;
use sdl2::video::{FullscreenType, Window, WindowPos};
use std::fs::create_dir_all;
use vulkanalia::vk;

/// Provides the context for the rendering graphics on screen.
pub struct Graphics {
    pub(crate) window: Window,
    pub(crate) vulkan: Vulkan,
    pub textures: TextureLoader,
    pub fonts: FontLoader,
}

impl Graphics {
    pub fn create(config: GraphicsConfig) -> Self {
        dpi::native::setup_process_dpi();
        let system = sdl2::init().expect("SDL2 must be initialized");
        let video = system.video().expect("SDL2 video must be initialized");
        let display = 0;
        let bounds = video
            .display_bounds(display)
            .expect("display bounds must be determined");
        let dpi = video
            .display_dpi(display)
            .expect("display dpi must be determined");
        info!("SDL first display bounds is {bounds:?} dpi is {dpi:?}");
        let [width, height] = config.resolution;
        let mut window = video
            .window(&config.title, width, height)
            .vulkan()
            //.allow_highdpi()
            .resizable()
            .build()
            .expect("SDL2 window must be created");
        match config.mode {
            GraphicsMode::Windowed => {}
            GraphicsMode::Fullscreen => {
                window
                    .set_fullscreen(FullscreenType::True)
                    .expect("fullscreen mode must be set");
            }
            GraphicsMode::Borderless => {
                window.set_bordered(false);
            }
        }
        if let Some([x, y]) = config.position {
            window.set_position(WindowPos::Positioned(x), WindowPos::Positioned(y));
        }
        let drawable = window.vulkan_drawable_size();
        let window_size = window.size();
        let dpi_scale = drawable.1 as f32 / window_size.1 as f32;
        info!("SDL window size is {window_size:?} drawable is {drawable:?} dpi scale={dpi_scale}");
        // let mut camera = Camera::default();
        // reference resolution is working resolution of game assets
        // pixel art style makes it possible to work in lower resolution
        // scaling in several times looks acceptable
        // let resolution_scale = drawable.1 as f32 / Camera::REFERENCE.y;
        // info!(
        //     "Reference resolution is {:?}, camera scale is {resolution_scale:.1}",
        //     Camera::REFERENCE
        // );
        // camera.resolution_scale = resolution_scale;
        // camera.input_scale = dpi_scale;
        let present_mode = if config.vsync {
            vk::PresentModeKHR::FIFO
        } else {
            vk::PresentModeKHR::IMMEDIATE
        };
        let mut vulkan = unsafe { Vulkan::create(&window, present_mode) };
        info!("Configures asset loaders");
        create_dir_all(&config.fonts.cache).expect("all cache sub directories must be created");
        let textures = vulkan.create_texture_loader_device();
        let textures = TextureLoader::new(textures);
        let fonts_resolution_scale = match config.fonts.resolution_reference {
            None => 1.0,
            Some([_, height]) => drawable.1 as f32 / height as f32,
        };
        let fonts = FontLoader::new(&config.fonts.cache, fonts_resolution_scale);
        Self {
            window,
            vulkan,
            textures,
            fonts,
        }
    }

    pub fn prepare(&mut self) {
        self.vulkan.update();
        self.vulkan.prepare(&self.window);
    }

    pub fn present(&mut self) {
        self.vulkan.present();
    }

    pub fn capture_user_input(&mut self) {
        while let Some(event) = poll_event() {}
    }
}
