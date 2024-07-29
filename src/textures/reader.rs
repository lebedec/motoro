use zune_png::{PngDecoder, PngInfo};

use crate::textures::TextureError;

pub fn read_texture_from_data(data: &[u8]) -> Result<(PngInfo, Vec<u8>), TextureError> {
    let mut decoder = PngDecoder::new(data);
    decoder.decode_headers()?;
    let image = decoder.get_info().ok_or("png has no header")?.clone();
    let data = decoder.decode()?.u8().ok_or("png has non 8-bit channels")?;
    Ok((image, data))
}
