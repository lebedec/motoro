use crate::textures::TextureError;
use crate::{Texture, TextureLoaderRequest};
use log::{error, info};
use std::fs;
use std::sync::mpsc::{Receiver, Sender};
use zune_png::{PngDecoder, PngInfo};

pub fn read_texture_info(data: &[u8]) -> Result<PngInfo, TextureError> {
    let mut decoder = PngDecoder::new(data);
    decoder.decode_headers()?;
    let image = decoder.get_info().ok_or("png has no header")?;
    Ok(image.clone())
}

pub fn read_texture_from_data(data: &[u8]) -> Result<(PngInfo, Vec<u8>), TextureError> {
    let mut decoder = PngDecoder::new(data);
    decoder.decode_headers()?;
    let image = decoder.get_info().ok_or("png has no header")?.clone();
    let data = decoder.decode()?.u8().ok_or("png has non 8-bit channels")?;
    Ok((image, data))
}

pub fn handle_reader_thread(
    id: usize,
    files: Receiver<(String, Texture)>,
    loader: Sender<TextureLoaderRequest>,
) {
    info!("Starts texture reader id={id}");
    for (path, handle) in files.iter() {
        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(error) => {
                error!("unable to read texture file, {error:?}");
                continue;
            }
        };
        let (info, data) = match read_texture_from_data(&data) {
            Ok(data) => data,
            Err(error) => {
                error!("unable to read texture, {error:?}");
                continue;
            }
        };
        let request = TextureLoaderRequest::Load(path, handle, info.width, info.height, data);
        if let Err(error) = loader.send(request) {
            error!("unable to send loader request, {error:?}");
            break;
        }
    }
    info!("Stops texture reader id={id}");
}
