use mesura::{Counter, Gauge};

pub struct TexturePrefabMetrics {
    pub requests: Counter,
    pub loadings: Counter,
    pub uses: Counter,
}

impl TexturePrefabMetrics {
    pub fn new() -> Self {
        TexturePrefabMetrics {
            requests: Counter::with_labels("get_texture", ["result"], ["request"]),
            loadings: Counter::with_labels("get_texture", ["result"], ["loading"]),
            uses: Counter::with_labels("get_texture", ["result"], ["use"]),
        }
    }
}

pub struct TextureLoaderMetrics {
    pub loads: Counter,
    pub errors: Counter,
    pub loading_time: Gauge,
}

impl TextureLoaderMetrics {
    pub fn new(id: usize) -> Self {
        let id = id.to_string();
        let id = id.as_str();
        Self {
            loads: Counter::with_labels("texture_loads", ["loader", "status"], [id, "ok"]),
            errors: Counter::with_labels("texture_loads", ["loader", "status"], [id, "error"]),
            loading_time: Gauge::with_labels("texture_loading_time", ["loader"], [id]),
        }
    }
}
