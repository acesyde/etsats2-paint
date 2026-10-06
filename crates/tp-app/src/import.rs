//! Reading image files to import: format sniffing, validation and size.

use std::path::Path;
use std::sync::{Arc, OnceLock};

use resvg::usvg;
use tp_core::AssetKind;
use tp_core::kurbo::Size;

/// A file ready to become a project asset.
#[derive(Clone, Debug)]
pub struct ImportedFile {
    /// File name without extension.
    pub name: String,
    pub kind: AssetKind,
    pub bytes: Arc<[u8]>,
    /// Natural size: pixels, or SVG user units.
    pub size: Size,
}

/// Why a file was not imported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportError {
    /// File name shown to the user.
    pub file: String,
    pub reason: String,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.file, self.reason)
    }
}

/// Fonts available to texts inside SVG files (the bundled fonts).
pub fn svg_options() -> &'static usvg::Options<'static> {
    static OPTIONS: OnceLock<usvg::Options<'static>> = OnceLock::new();
    OPTIONS.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        for data in tp_text::bundled_font_data() {
            db.load_font_data(data.to_vec());
        }
        db.set_sans_serif_family(tp_text::FALLBACK_FAMILY);
        usvg::Options {
            fontdb: Arc::new(db),
            ..usvg::Options::default()
        }
    })
}

fn sniff(bytes: &[u8]) -> Option<AssetKind> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") || bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(AssetKind::Raster);
    }
    let head = &bytes[..bytes.len().min(4096)];
    let text = String::from_utf8_lossy(head);
    let trimmed = text.trim_start_matches('\u{feff}').trim_start();
    (trimmed.starts_with('<') && text.contains("<svg")).then_some(AssetKind::Svg)
}

/// Validates bytes named `file_name` and reads their natural size.
pub fn read_bytes(file_name: &str, bytes: Vec<u8>) -> Result<ImportedFile, ImportError> {
    let error = |reason: &str| ImportError {
        file: file_name.to_owned(),
        reason: reason.to_owned(),
    };
    let name = Path::new(file_name).file_stem().map_or_else(
        || file_name.to_owned(),
        |s| s.to_string_lossy().into_owned(),
    );
    let kind = sniff(&bytes).ok_or_else(|| error("unsupported format"))?;
    let size = match kind {
        AssetKind::Raster => {
            let reader = image::ImageReader::new(std::io::Cursor::new(&bytes))
                .with_guessed_format()
                .map_err(|_| error("unsupported format"))?;
            let image = reader.decode().map_err(|_| error("the image is damaged"))?;
            Size::new(f64::from(image.width()), f64::from(image.height()))
        }
        AssetKind::Svg => {
            let tree = usvg::Tree::from_data(&bytes, svg_options())
                .map_err(|_| error("the SVG file is invalid"))?;
            let s = tree.size();
            Size::new(f64::from(s.width()), f64::from(s.height()))
        }
    };
    if size.width < 1.0 || size.height < 1.0 {
        return Err(error("the image is empty"));
    }
    Ok(ImportedFile {
        name,
        kind,
        bytes: bytes.into(),
        size,
    })
}

/// Reads files from disk.
pub fn read_files(paths: &[std::path::PathBuf]) -> Vec<Result<ImportedFile, ImportError>> {
    paths
        .iter()
        .map(|path| {
            let file_name = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            match std::fs::read(path) {
                Ok(bytes) => read_bytes(&file_name, bytes),
                Err(err) => Err(ImportError {
                    file: file_name,
                    reason: format!("cannot be read ({err})"),
                }),
            }
        })
        .collect()
}

/// Encodes a solid-color PNG (tests and examples).
pub fn solid_png(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(width, height, image::Rgba(rgba));
    let mut out = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_size_and_name() {
        let file = read_bytes("logo.png", solid_png(80, 40, [0, 0, 255, 255])).unwrap();
        assert_eq!((file.name.as_str(), file.kind), ("logo", AssetKind::Raster));
        assert_eq!(file.size, Size::new(80.0, 40.0));
    }

    #[test]
    fn svg_size() {
        let svg = br#"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg" width="300" height="100"><rect width="300" height="100" fill="red"/></svg>"#;
        let file = read_bytes("badge.svg", svg.to_vec()).unwrap();
        assert_eq!(file.kind, AssetKind::Svg);
        assert_eq!(file.size, Size::new(300.0, 100.0));
    }

    #[test]
    fn unsupported_and_damaged_files() {
        let gif = b"GIF89a\x01\x00\x01\x00".to_vec();
        assert_eq!(
            read_bytes("anim.gif", gif).unwrap_err().to_string(),
            "anim.gif: unsupported format"
        );
        let mut png = solid_png(4, 4, [0; 4]);
        png.truncate(20);
        assert!(read_bytes("broken.png", png).is_err());
    }
}
