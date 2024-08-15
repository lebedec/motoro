#[derive(Clone, Debug, serde::Deserialize)]
pub struct GraphicsConfig {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default)]
    pub mode: GraphicsMode,
    #[serde(default = "default_resolution")]
    pub resolution: [u32; 2],
    #[serde(default)]
    pub resolution_reference: Option<[u32; 2]>,
    #[serde(default)]
    pub position: Option<[i32; 2]>,
    #[serde(default = "default_vsync")]
    pub vsync: bool,
    #[serde(default)]
    pub fonts: FontsConfig,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            title: default_title(),
            mode: GraphicsMode::default(),
            resolution: default_resolution(),
            resolution_reference: None,
            position: None,
            vsync: default_vsync(),
            fonts: FontsConfig::default(),
        }
    }
}

impl GraphicsConfig {
    pub fn fonts<F>(mut self, config: F) -> Self
    where
        F: FnOnce(FontsConfig) -> FontsConfig,
    {
        self.fonts = config(self.fonts);
        self
    }

    pub fn resolution(mut self, resolution: [u32; 2]) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn resolution_reference(mut self, resolution: [u32; 2]) -> Self {
        self.resolution_reference = Some(resolution);
        self
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize)]
pub enum GraphicsMode {
    Windowed,
    Fullscreen,
    Borderless,
}

impl Default for GraphicsMode {
    fn default() -> Self {
        Self::Windowed
    }
}

fn default_title() -> String {
    "motoro".to_string()
}

fn default_vsync() -> bool {
    true
}

fn default_resolution() -> [u32; 2] {
    [1920, 1080]
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct FontsConfig {
    #[serde(default = "default_fonts_cache")]
    pub cache: String,
    #[serde(default)]
    pub resolution_reference: Option<[u32; 2]>,
}

impl Default for FontsConfig {
    fn default() -> Self {
        Self {
            cache: default_fonts_cache(),
            resolution_reference: None,
        }
    }
}

impl FontsConfig {
    pub fn cache(mut self, cache: &str) -> Self {
        self.cache = cache.to_string();
        self
    }

    pub fn resolution_reference(mut self, resolution: [u32; 2]) -> Self {
        self.resolution_reference = Some(resolution);
        self
    }
}

fn default_fonts_cache() -> String {
    "./assets/cache/fonts".to_string()
}
