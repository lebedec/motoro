use crate::fonts::MISSING_CHAR;
use crate::{Char, Font, FontError};
use fontdue::FontSettings;
use log::info;
use std::collections::HashMap;
use std::fs;
use zune_png::zune_core::bit_depth::BitDepth;
use zune_png::zune_core::colorspace::ColorSpace;
use zune_png::zune_core::options::EncoderOptions;
use zune_png::PngEncoder;

/// NOTE: Resolution scale here improves pixel perfect rendering of font. It can't improve
/// letters spacing in result text rendering. See FontPrefab::layout for details.
pub(crate) fn rasterize_font_to_image_file(
    input: &[u8],
    cache: &str,
    name: &str,
    alphabet: &str,
    size: f32,
    resolution_scale: f32,
) -> Result<Font, FontError> {
    let key = format!("{name}-{}-{}.png", (size) as u32, (resolution_scale) as u32);
    let texture = format!("{cache}/{key}");

    let size = size * resolution_scale;
    info!("Starts font {texture} loading");
    let font_settings = FontSettings {
        collection_index: 0,
        scale: size,
        load_substitutions: true,
    };
    let font = fontdue::Font::from_bytes(input, font_settings)?;

    let w = (512.0 * resolution_scale) as usize;
    let h = (512.0 * resolution_scale) as usize;
    let mut data = vec![0; w * h * 4];
    let mut offset_x = 0usize;
    let mut offset_y = 0usize;
    // rounding up is need to produce coordinates without loss of accuracy
    let line_height = round_up_pow_2(size as usize);
    let mut charset = HashMap::new();
    let mut missing_char = Char::default();
    for char in alphabet.chars() {
        let (glyph, bitmap) = font.rasterize(char, size);
        let step_x = round_up_pow_2(glyph.width);
        if offset_x + step_x >= w {
            offset_x = 0;
            offset_y += line_height;
        }
        for (index, alpha) in bitmap.iter().enumerate() {
            let y = offset_y + index / glyph.width;
            let x = offset_x + index % glyph.width;
            let offset = (y * w * 4) + x * 4;
            data[offset + 0] = 255;
            data[offset + 1] = 255;
            data[offset + 2] = 255;
            data[offset + 3] = *alpha;
        }
        // if char == '.' || char == 'y' || char == 'e' {
        //     println!(
        //         "GLYPH {char} h{} ah{} ymin{} bymin{} bh{} lh{}",
        //         glyph.height,
        //         glyph.advance_height,
        //         glyph.ymin,
        //         glyph.bounds.ymin,
        //         glyph.bounds.height,
        //         line_height
        //     );
        // }
        let constants = Char {
            position: [0.0; 2],
            image: [w as f32, h as f32],
            src: [offset_x as f32 / w as f32, offset_y as f32 / h as f32],
            uv: [step_x as f32 / w as f32, line_height as f32 / h as f32],
            size: [
                step_x as f32 / resolution_scale,
                line_height as f32 / resolution_scale,
            ],
            height: glyph.height as f32,
        };
        charset.insert(char, constants);
        if char == MISSING_CHAR {
            missing_char = constants;
        }
        offset_x += step_x;
    }

    let options = EncoderOptions::new(w, h, ColorSpace::RGBA, BitDepth::Eight);
    let mut encoder = PngEncoder::new(&data, options);
    fs::write(&texture, encoder.encode())?;

    info!("Creates font prefab {texture} charset={}", charset.len());
    Ok(Font {
        texture,
        charset,
        font,
        size,
        missing_char,
        resolution_scale,
    })
}

fn round_up_pow_2(value: usize) -> usize {
    if value == 0 {
        return 1;
    }
    let mut value = value - 1;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    let v = value + 1;
    v
}
