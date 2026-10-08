//! Texture encoders: mipmaps, PNG and DDS (BC3/DXT5 or uncompressed).

use resvg::tiny_skia::{IntSize, Pixmap};

use crate::render::{Cancelled, to_rgba};

/// DDS pixel encoding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DdsEncoding {
    /// BC3 (DXT5): compressed, with alpha.
    #[default]
    Bc3,
    /// 32-bit BGRA, uncompressed.
    Rgba,
}

/// Mipmap chain from `base` down to 1×1 (base included), averaging 2×2
/// premultiplied pixels.
pub fn mip_chain(base: &Pixmap) -> Vec<Pixmap> {
    let mut levels = vec![base.clone()];
    while let Some(last) = levels.last() {
        let (w, h) = (last.width(), last.height());
        if w == 1 && h == 1 {
            break;
        }
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let src = last.data();
        let mut data = vec![0u8; (nw * nh * 4) as usize];
        for y in 0..nh {
            for x in 0..nw {
                for c in 0..4 {
                    let mut sum = 0u32;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let sx = (x * 2 + dx).min(w - 1);
                        let sy = (y * 2 + dy).min(h - 1);
                        sum += u32::from(src[((sy * w + sx) * 4 + c) as usize]);
                    }
                    data[((y * nw + x) * 4 + c) as usize] = ((sum + 2) / 4) as u8;
                }
            }
        }
        let size = IntSize::from_wh(nw, nh).expect("non-zero");
        levels.push(Pixmap::from_vec(data, size).expect("valid premultiplied data"));
    }
    levels
}

/// PNG bytes (straight RGBA).
pub fn encode_png(pixmap: &Pixmap) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    to_rgba(pixmap)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

/// JPEG bytes at `quality` (1–100), drawn over white (JPEG has no alpha).
pub fn encode_jpeg(pixmap: &Pixmap, quality: u8) -> Result<Vec<u8>, String> {
    let rgba = to_rgba(pixmap);
    let rgb = image::RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let [r, g, b, a] = rgba.get_pixel(x, y).0;
        let over_white =
            |c: u8| ((u16::from(c) * u16::from(a) + 255 * (255 - u16::from(a)) + 127) / 255) as u8;
        image::Rgb([over_white(r), over_white(g), over_white(b)])
    });
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality)
        .encode_image(&rgb)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

fn bc3_level_size(w: u32, h: u32) -> usize {
    texpresso::Format::Bc3.compressed_size(w as usize, h as usize)
}

/// Exact DDS file size for a square texture of `size` pixels.
pub fn dds_size(size: u32, encoding: DdsEncoding) -> u64 {
    let mut total = 128u64;
    let mut s = size.max(1);
    loop {
        total += match encoding {
            DdsEncoding::Bc3 => bc3_level_size(s, s) as u64,
            DdsEncoding::Rgba => u64::from(s) * u64::from(s) * 4,
        };
        if s == 1 {
            break;
        }
        s = (s / 2).max(1);
    }
    total
}

fn header(width: u32, height: u32, levels: u32, encoding: DdsEncoding) -> Vec<u8> {
    const CAPS: u32 = 0x1;
    const HEIGHT: u32 = 0x2;
    const WIDTH: u32 = 0x4;
    const PITCH: u32 = 0x8;
    const PIXELFORMAT: u32 = 0x1000;
    const MIPMAPCOUNT: u32 = 0x2_0000;
    const LINEARSIZE: u32 = 0x8_0000;
    const PF_ALPHAPIXELS: u32 = 0x1;
    const PF_FOURCC: u32 = 0x4;
    const PF_RGB: u32 = 0x40;
    const CAPS_COMPLEX: u32 = 0x8;
    const CAPS_TEXTURE: u32 = 0x1000;
    const CAPS_MIPMAP: u32 = 0x40_0000;

    let mut h = Vec::with_capacity(128);
    let mut put = |v: u32| h.extend_from_slice(&v.to_le_bytes());
    put(u32::from_le_bytes(*b"DDS "));
    put(124);
    let (size_flag, pitch) = match encoding {
        DdsEncoding::Bc3 => (LINEARSIZE, bc3_level_size(width, height) as u32),
        DdsEncoding::Rgba => (PITCH, width * 4),
    };
    put(CAPS | HEIGHT | WIDTH | PIXELFORMAT | MIPMAPCOUNT | size_flag);
    put(height);
    put(width);
    put(pitch);
    put(0); // depth
    put(levels);
    for _ in 0..11 {
        put(0);
    }
    // Pixel format.
    put(32);
    match encoding {
        DdsEncoding::Bc3 => {
            put(PF_FOURCC);
            put(u32::from_le_bytes(*b"DXT5"));
            for _ in 0..5 {
                put(0);
            }
        }
        DdsEncoding::Rgba => {
            put(PF_RGB | PF_ALPHAPIXELS);
            put(0);
            put(32);
            put(0x00FF_0000);
            put(0x0000_FF00);
            put(0x0000_00FF);
            put(0xFF00_0000);
        }
    }
    put(CAPS_TEXTURE | CAPS_MIPMAP | CAPS_COMPLEX);
    for _ in 0..4 {
        put(0);
    }
    h
}

/// DDS bytes with a full mipmap chain. `progress(level, levels)` is called
/// after each level; returning false cancels.
pub fn encode_dds(
    pixmap: &Pixmap,
    encoding: DdsEncoding,
    progress: &mut dyn FnMut(usize, usize) -> bool,
) -> Result<Vec<u8>, Cancelled> {
    let levels = mip_chain(pixmap);
    let mut out = header(
        pixmap.width(),
        pixmap.height(),
        levels.len() as u32,
        encoding,
    );
    for (i, level) in levels.iter().enumerate() {
        let rgba = to_rgba(level);
        let (w, h) = (level.width(), level.height());
        match encoding {
            DdsEncoding::Bc3 => {
                let mut block = vec![0u8; bc3_level_size(w, h)];
                let algorithm = if w <= 2048 {
                    texpresso::Algorithm::ClusterFit
                } else {
                    texpresso::Algorithm::RangeFit
                };
                let params = texpresso::Params {
                    algorithm,
                    ..texpresso::Params::default()
                };
                texpresso::Format::Bc3.compress(
                    rgba.as_raw(),
                    w as usize,
                    h as usize,
                    params,
                    &mut block,
                );
                out.extend_from_slice(&block);
            }
            DdsEncoding::Rgba => {
                for px in rgba.pixels() {
                    let [r, g, b, a] = px.0;
                    out.extend_from_slice(&[b, g, r, a]);
                }
            }
        }
        if !progress(i + 1, levels.len()) {
            return Err(Cancelled);
        }
    }
    Ok(out)
}
