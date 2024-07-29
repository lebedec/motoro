#[derive(Clone, Debug, serde::Deserialize)]
pub struct GraphicsConfig {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default)]
    pub mode: GraphicsMode,
    #[serde(default = "default_resolution")]
    pub resolution: [u32; 2],
    #[serde(default)]
    pub position: Option<[i32; 2]>,
    #[serde(default = "default_vsync")]
    pub vsync: bool,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            title: default_title(),
            mode: GraphicsMode::default(),
            resolution: default_resolution(),
            position: None,
            vsync: default_vsync(),
        }
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

fn default_title() -> String { "motoro".to_string() }

fn default_vsync() -> bool {
    true
}

fn default_resolution() -> [u32; 2] {
    [1920, 1080]
}
