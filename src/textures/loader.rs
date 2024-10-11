use crate::handle_reader_thread;
use crate::textures::{Texture, TextureError, TextureLoaderMetrics, TexturePrefabMetrics};
use crate::vulkan::textures::VulkanTextureLoaderDevice;
use log::{debug, error, info};
use mesura::GaugeValue;
use std::collections::HashMap;
use std::mem::take;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::{fs, thread};

pub trait TextureLoaderDevice: Clone + Send {
    fn load_texture_from(&self, data: &[u8]) -> Result<Texture, TextureError>;
}

pub struct TextureRecord {
    pub current: Texture,
    pub loading: Option<Texture>,
}

pub struct TexturesManager {
    pub records: HashMap<String, TextureRecord>,
    pub responses: Receiver<TextureLoaderResponse>,
    pub readers: Vec<Sender<(String, Texture)>>,
    pub readers_index: usize,
    pub loader: Sender<TextureLoaderRequest>,
    pub fallback: Texture,
    pub blank: Texture,
    pub device: VulkanTextureLoaderDevice,
}

pub enum TextureLoaderRequest {
    Load(String, Texture, usize, usize, Vec<u8>),
}

pub enum TextureLoaderResponse {
    Loaded(String, Texture),
}

pub fn handle_loader_thread(
    device: VulkanTextureLoaderDevice,
    requests: Receiver<TextureLoaderRequest>,
    manager: Sender<TextureLoaderResponse>,
    null: Texture,
) {
    let mut metrics = TextureLoaderMetrics::new(0);
    for request in requests.iter() {
        match request {
            TextureLoaderRequest::Load(path, mut handle, width, height, data) => {
                debug!("Starts texture '{path}' loading");
                let time = Instant::now();
                if handle == null {
                    handle = device.create_texture_handle(width, height);
                    debug!("Creates texture '{path}' handle {handle:?}");
                }
                device.update_texture_data(handle, &data);
                metrics.loading_time.add(time);
                // println!("loading time: {:?}", time.elapsed());
                let response = TextureLoaderResponse::Loaded(path, handle);
                if let Err(error) = manager.send(response) {
                    error!("unable to send manager response, {error:?}");
                    break;
                }
            }
        }
    }
}

impl TexturesManager {
    pub fn new(device: VulkanTextureLoaderDevice) -> Self {
        info!("Creates textures manager");
        let fallback = include_bytes!("builtin/default.png");
        let fallback = device
            .load_texture_from(fallback)
            .expect("fallback texture must be loaded");
        let blank = include_bytes!("builtin/rect.png");
        let blank = device
            .load_texture_from(blank)
            .expect("blank texture must be loaded");
        // TODO: remove, use only loader thread instead
        let manager_device = device.clone();
        let (loader, requests) = channel();
        let (manager, responses) = channel();
        let mut readers = vec![];
        // multiple readers are efficient for parallel file loading
        for id in 0..2 {
            let loader = loader.clone();
            let (reader, files) = channel();
            readers.push(reader);
            thread::Builder::new()
                .name(format!("texture-reader-{id}"))
                .spawn(move || handle_reader_thread(id, files, loader))
                .expect("reader thread spawned");
        }
        let readers_index = readers.len() - 1;
        // one loader, one loading Vulkan queue
        thread::Builder::new()
            .name("texture-loader".to_string())
            .spawn(move || handle_loader_thread(device, requests, manager, fallback))
            .expect("loader thread spawned");
        Self {
            records: HashMap::new(),
            responses,
            readers,
            readers_index,
            loader,
            fallback,
            blank,
            device: manager_device,
        }
    }

    pub fn create_texture(&self, width: u32, height: u32, data: &[u8]) -> Texture {
        self.device.create_texture(width, height, data)
    }

    pub fn create_dynamic_texture(&mut self, width: usize, height: usize, data: Vec<u8>) -> String {
        let path = format!("memory:{}", self.records.len());
        let record = TextureRecord {
            current: self.fallback,
            loading: Some(self.fallback),
        };
        self.records.insert(path.clone(), record);
        self.update_dynamic_texture(&path, width, height, data);
        path
    }

    pub fn update_dynamic_texture(
        &mut self,
        path: &str,
        width: usize,
        height: usize,
        data: Vec<u8>,
    ) {
        let record = match self.records.get_mut(path) {
            Some(record) => record,
            None => {
                error!("unable to update texture {path}, record not found");
                return;
            }
        };
        let handle = match take(&mut record.loading) {
            Some(handle) => handle,
            None => {
                error!("unable to update texture {path}, loading in progress");
                return;
            }
        };
        let request = TextureLoaderRequest::Load(path.to_string(), handle, width, height, data);
        if let Err(error) = self.loader.send(request) {
            error!("unable to send loader request, {error:?}");
        }
    }

    pub fn get_texture(&mut self, path: &str) -> Texture {
        if path == Texture::FALLBACK {
            return self.fallback;
        }

        if path == Texture::BLANK {
            return self.blank;
        }

        let record = self
            .records
            .entry(path.to_string())
            .or_insert_with(|| TextureRecord {
                current: self.fallback,
                loading: Some(self.fallback),
            });

        if !path.starts_with("memory:") && record.current == self.fallback {
            if let Some(handle) = take(&mut record.loading) {
                self.readers_index = (self.readers_index + 1) % self.readers.len();
                let request = (path.to_string(), handle);
                if let Err(error) = self.readers[self.readers_index].send(request) {
                    error!("unable to send reader request, {error:?}");
                }
            } else {
                // loading in progress
            }
        }

        record.current
    }

    pub fn update(&mut self) {
        for response in self.responses.try_iter() {
            match response {
                TextureLoaderResponse::Loaded(path, handle) => {
                    let record = match self.records.get_mut(&path) {
                        Some(record) => record,
                        None => {
                            // TODO: destroy handle
                            error!("unable to update loaded texture {path}, record not found");
                            continue;
                        }
                    };
                    record.loading = Some(record.current);
                    record.current = handle;
                }
            }
        }
    }
}
