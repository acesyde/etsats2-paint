//! A DDS reader for the formats SCS templates use: BC1, BC2 and BC3
//! (DXT1/3/5 or DX10 headers) and uncompressed 24 or 32-bit RGB(A).
//! Only the top mip level is read.

use image::RgbaImage;

const MAGIC: &[u8; 4] = b"DDS ";
const HEADER: usize = 4 + 124;
const DX10_HEADER: usize = 20;

const DDPF_ALPHAPIXELS: u32 = 0x1;
const DDPF_FOURCC: u32 = 0x4;
const DDPF_RGB: u32 = 0x40;
const DDSCAPS2_CUBEMAP: u32 = 0x200;
const DDSCAPS2_VOLUME: u32 = 0x20_0000;

/// Pixel layout of the data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layout {
    Bc(texpresso::Format),
    /// Uncompressed, `bytes` per pixel, with channel masks (R, G, B, A).
    Masks {
        bytes: usize,
        masks: [u32; 4],
    },
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 bytes"))
}

/// Decodes the top mip level of a DDS file. The error says why it is not
/// supported, in English.
pub fn decode(bytes: &[u8]) -> Result<RgbaImage, String> {
    if bytes.len() < HEADER || &bytes[..4] != MAGIC {
        return Err("not a DDS file".into());
    }
    let header = &bytes[4..HEADER];
    let height = u32_at(header, 8);
    let width = u32_at(header, 12);
    let pf_flags = u32_at(header, 76);
    let four_cc = &header[80..84];
    let bit_count = u32_at(header, 84);
    let masks = [
        u32_at(header, 88),
        u32_at(header, 92),
        u32_at(header, 96),
        u32_at(header, 100),
    ];
    let caps2 = u32_at(header, 108);
    if width == 0 || height == 0 {
        return Err("empty image".into());
    }
    if caps2 & (DDSCAPS2_CUBEMAP | DDSCAPS2_VOLUME) != 0 {
        return Err("cube maps and volume textures are not supported".into());
    }
    let mut data_start = HEADER;
    let layout = if pf_flags & DDPF_FOURCC != 0 {
        match four_cc {
            b"DXT1" => Layout::Bc(texpresso::Format::Bc1),
            b"DXT2" | b"DXT3" => Layout::Bc(texpresso::Format::Bc2),
            b"DXT4" | b"DXT5" => Layout::Bc(texpresso::Format::Bc3),
            b"DX10" => {
                if bytes.len() < HEADER + DX10_HEADER {
                    return Err("truncated DX10 header".into());
                }
                data_start += DX10_HEADER;
                let dx10 = &bytes[HEADER..HEADER + DX10_HEADER];
                let array_size = u32_at(dx10, 12);
                if array_size > 1 {
                    return Err("texture arrays are not supported".into());
                }
                dxgi_layout(u32_at(dx10, 0))?
            }
            other => {
                return Err(format!(
                    "compression {} is not supported",
                    String::from_utf8_lossy(other).trim_end_matches('\0')
                ));
            }
        }
    } else if pf_flags & DDPF_RGB != 0 && (bit_count == 24 || bit_count == 32) {
        let mut masks = masks;
        if pf_flags & DDPF_ALPHAPIXELS == 0 {
            masks[3] = 0;
        }
        Layout::Masks {
            bytes: bit_count as usize / 8,
            masks,
        }
    } else {
        return Err(format!(
            "pixel format with {bit_count} bits per pixel is not supported"
        ));
    };

    let (w, h) = (width as usize, height as usize);
    let data = &bytes[data_start..];
    let mut rgba = vec![0u8; w * h * 4];
    match layout {
        Layout::Bc(format) => {
            if data.len() < format.compressed_size(w, h) {
                return Err("truncated image data".into());
            }
            format.decompress(data, w, h, &mut rgba);
        }
        Layout::Masks { bytes: bpp, masks } => {
            if data.len() < w * h * bpp {
                return Err("truncated image data".into());
            }
            for (pixel, out) in data.chunks_exact(bpp).zip(rgba.as_chunks_mut::<4>().0) {
                let mut raw = [0u8; 4];
                raw[..bpp].copy_from_slice(pixel);
                let value = u32::from_le_bytes(raw);
                for (channel, mask) in masks.iter().enumerate() {
                    out[channel] = if *mask == 0 {
                        if channel == 3 { 255 } else { 0 }
                    } else {
                        channel_value(value, *mask)
                    };
                }
            }
        }
    }
    RgbaImage::from_raw(width, height, rgba).ok_or_else(|| "invalid size".into())
}

/// A channel extracted with `mask` and scaled to 8 bits.
fn channel_value(value: u32, mask: u32) -> u8 {
    let shift = mask.trailing_zeros();
    let bits = mask.count_ones();
    let raw = (value & mask) >> shift;
    let max = (1u64 << bits) - 1;
    ((u64::from(raw) * 255 + max / 2) / max) as u8
}

fn dxgi_layout(format: u32) -> Result<Layout, String> {
    const RGBA: [u32; 4] = [0xff, 0xff00, 0xff_0000, 0xff00_0000];
    const BGRA: [u32; 4] = [0xff_0000, 0xff00, 0xff, 0xff00_0000];
    Ok(match format {
        71 | 72 => Layout::Bc(texpresso::Format::Bc1),
        74 | 75 => Layout::Bc(texpresso::Format::Bc2),
        77 | 78 => Layout::Bc(texpresso::Format::Bc3),
        28 | 29 => Layout::Masks {
            bytes: 4,
            masks: RGBA,
        },
        87 | 91 => Layout::Masks {
            bytes: 4,
            masks: BGRA,
        },
        other => return Err(format!("DXGI format {other} is not supported")),
    })
}

/// Test helpers: DDS files built in memory.
#[cfg(test)]
pub(crate) mod build {
    use super::*;

    /// A header for `width`×`height` with the given pixel format fields.
    pub fn header(
        width: u32,
        height: u32,
        pf_flags: u32,
        four_cc: &[u8; 4],
        bit_count: u32,
        masks: [u32; 4],
    ) -> Vec<u8> {
        let mut h = vec![0u8; HEADER];
        h[..4].copy_from_slice(MAGIC);
        let mut put = |offset: usize, v: u32| {
            h[4 + offset..4 + offset + 4].copy_from_slice(&v.to_le_bytes());
        };
        put(0, 124);
        put(4, 0x1007);
        put(8, height);
        put(12, width);
        put(72, 32);
        put(76, pf_flags);
        put(84, bit_count);
        for (i, m) in masks.iter().enumerate() {
            put(88 + i * 4, *m);
        }
        put(104, 0x1000);
        h[4 + 80..4 + 84].copy_from_slice(four_cc);
        h
    }

    /// A BC-compressed DDS of `image` with a FourCC header.
    pub fn bc(image: &RgbaImage, format: texpresso::Format, four_cc: &[u8; 4]) -> Vec<u8> {
        let mut out = header(
            image.width(),
            image.height(),
            DDPF_FOURCC,
            four_cc,
            0,
            [0; 4],
        );
        out.extend(compress(image, format));
        out
    }

    /// A BC-compressed DDS of `image` with a DX10 header.
    pub fn dx10(image: &RgbaImage, format: texpresso::Format, dxgi: u32) -> Vec<u8> {
        let mut out = header(
            image.width(),
            image.height(),
            DDPF_FOURCC,
            b"DX10",
            0,
            [0; 4],
        );
        let mut dx10 = vec![0u8; DX10_HEADER];
        dx10[..4].copy_from_slice(&dxgi.to_le_bytes());
        dx10[4..8].copy_from_slice(&3u32.to_le_bytes());
        dx10[12..16].copy_from_slice(&1u32.to_le_bytes());
        out.extend(dx10);
        out.extend(compress(image, format));
        out
    }

    /// An uncompressed 32-bit BGRA DDS of `image`.
    pub fn bgra(image: &RgbaImage) -> Vec<u8> {
        let mut out = header(
            image.width(),
            image.height(),
            DDPF_RGB | DDPF_ALPHAPIXELS,
            &[0; 4],
            32,
            [0xff_0000, 0xff00, 0xff, 0xff00_0000],
        );
        for p in image.pixels() {
            out.extend([p[2], p[1], p[0], p[3]]);
        }
        out
    }

    fn compress(image: &RgbaImage, format: texpresso::Format) -> Vec<u8> {
        let (w, h) = (image.width() as usize, image.height() as usize);
        let mut out = vec![0u8; format.compressed_size(w, h)];
        format.compress(image.as_raw(), w, h, texpresso::Params::default(), &mut out);
        out
    }

    /// A 16×16 image with flat colour quadrants (exact in BC formats).
    pub fn quadrants() -> RgbaImage {
        RgbaImage::from_fn(16, 16, |x, y| match (x < 8, y < 8) {
            (true, true) => image::Rgba([255, 0, 0, 255]),
            (false, true) => image::Rgba([0, 255, 0, 255]),
            (true, false) => image::Rgba([0, 0, 255, 255]),
            (false, false) => image::Rgba([255, 255, 255, 255]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::build::*;
    use super::*;

    fn close(a: &RgbaImage, b: &RgbaImage, tolerance: u8) -> bool {
        a.dimensions() == b.dimensions()
            && a.pixels()
                .zip(b.pixels())
                .all(|(p, q)| p.0.iter().zip(q.0).all(|(x, y)| x.abs_diff(y) <= tolerance))
    }

    #[test]
    fn bc_formats_decode() {
        let src = quadrants();
        for (format, cc) in [
            (texpresso::Format::Bc1, b"DXT1"),
            (texpresso::Format::Bc2, b"DXT3"),
            (texpresso::Format::Bc3, b"DXT5"),
        ] {
            let out = decode(&bc(&src, format, cc)).unwrap();
            assert!(close(&out, &src, 8), "{format:?}");
        }
        let out = decode(&dx10(&src, texpresso::Format::Bc3, 77)).unwrap();
        assert!(close(&out, &src, 8), "DX10 BC3");
    }

    #[test]
    fn uncompressed_decodes_exactly() {
        let src = RgbaImage::from_fn(5, 3, |x, y| {
            image::Rgba([x as u8 * 40, y as u8 * 80, 7, 128])
        });
        assert_eq!(decode(&bgra(&src)).unwrap(), src);
        // 24-bit without alpha is opaque.
        let mut rgb = header(2, 1, DDPF_RGB, &[0; 4], 24, [0xff_0000, 0xff00, 0xff, 0]);
        rgb.extend([1, 2, 3, 4, 5, 6]);
        let out = decode(&rgb).unwrap();
        assert_eq!(out.get_pixel(0, 0).0, [3, 2, 1, 255]);
        assert_eq!(out.get_pixel(1, 0).0, [6, 5, 4, 255]);
    }

    #[test]
    fn unsupported_files_are_refused() {
        let src = quadrants();
        // BC7 (DXGI 98).
        let err = decode(&dx10(&src, texpresso::Format::Bc3, 98)).unwrap_err();
        assert!(err.contains("98"), "{err}");
        // Truncated data.
        let mut bytes = bc(&src, texpresso::Format::Bc1, b"DXT1");
        bytes.truncate(bytes.len() - 1);
        assert!(decode(&bytes).unwrap_err().contains("truncated"));
        assert!(decode(b"not a dds").is_err());
        assert!(decode(&bc(&src, texpresso::Format::Bc1, b"ATI2")).is_err());
    }
}
