use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::{fs, thread};

use log::{debug, error, info};
use mesura::GaugeValue;

use crate::textures::{Texture, TextureError, TextureLoaderMetrics, TexturePrefabMetrics};

pub trait TextureLoaderDevice: Clone + Send {
    fn load_texture_from(&self, id: usize, data: &[u8]) -> Result<Texture, TextureError>;
}

pub struct TextureLoader {
    requests: Receiver<String>,
    requests_queue: Sender<String>,
    loading: Receiver<(String, Texture)>,
    loading_queue: Arc<RwLock<Vec<String>>>,
    records: HashMap<String, Option<Texture>>,
    pub fallback: Texture,
    pub blank: Texture,
    metrics: TexturePrefabMetrics,
}

impl TextureLoader {
    pub fn new(device: impl TextureLoaderDevice + 'static) -> Self {
        info!("Creates texture loader");
        let default = include_bytes!("builtin/default.png");
        let default = device
            .load_texture_from(0, default)
            .expect("default texture must be loaded");
        let rect = include_bytes!("builtin/rect.png");
        let rect = device
            .load_texture_from(1, rect)
            .expect("rect texture must be loaded");

        let loading_queue = Arc::new(RwLock::new(Vec::<String>::new()));
        let (requests_queue, requests) = channel();
        let (responses_queue, responses) = channel();

        for id in 0..1 {
            // NOTE: to implement multiple loaders need to synchronize queue
            let name = format!("texture-loader-{id}");
            let thread_loading_queue = loading_queue.clone();
            let thread_responses_queue = responses_queue.clone();
            // let thread_instance = thread_instance.clone();
            // let thread_device = thread_device.clone();
            let thread_device = device.clone();
            let loading = move || {
                let mut texture_id = 2;
                let mut metrics = TextureLoaderMetrics::new(id);
                info!("Starts texture loader thread");
                loop {
                    let path = {
                        // NOTE: minimize lock time
                        thread_loading_queue
                            .write()
                            .expect("loading queue must be available")
                            .pop()
                    };
                    if let Some(path) = path {
                        debug!("Starts texture '{path}' loading");
                        let time = Instant::now();
                        let texture = fs::read(&path)
                            .map_err(TextureError::from)
                            .and_then(|data| thread_device.load_texture_from(texture_id, &data));
                        metrics.loading_time.add(time);
                        match texture {
                            Ok(texture) => {
                                debug!("Loads texture '{path}'");
                                texture_id += 1;
                                thread_responses_queue
                                    .send((path, texture))
                                    .expect("response must be sent");
                                metrics.loads.inc();
                            }
                            Err(error) => {
                                error!("Unable to load texture '{path}', {error:?}");
                                metrics.errors.inc();
                            }
                        }
                    } else {
                        thread::sleep(Duration::from_millis(250));
                    }
                }
            };
            thread::Builder::new()
                .name(name.clone())
                .spawn(loading)
                .expect(&format!("{name} thread must be spawned"));
        }
        Self {
            requests_queue,
            requests,
            loading: responses,
            loading_queue,
            records: HashMap::default(),
            fallback: default,
            blank: rect,
            metrics: TexturePrefabMetrics::new(),
        }
    }

    pub fn get_texture(&mut self, path: &str) -> Texture {
        match self.records.get(path) {
            None => {
                self.metrics.requests.inc();
                self.metrics.loadings.inc();
                self.requests_queue
                    .send(path.to_string())
                    .expect("request must be sent");
                self.fallback
            }
            Some(record) => match record {
                None => {
                    self.metrics.loadings.inc();
                    self.fallback
                }
                Some(texture) => {
                    self.metrics.uses.inc();
                    *texture
                }
            },
        }
    }

    pub fn update(&mut self) {
        for request in self.requests.try_iter() {
            if !self.records.contains_key(&request) {
                self.records.insert(request.clone(), None);
                self.loading_queue
                    .write()
                    .expect("loading queue must be available")
                    .push(request);
            }
        }
        for (path, texture) in self.loading.try_iter() {
            self.records.insert(path, Some(texture));
        }
    }
}
