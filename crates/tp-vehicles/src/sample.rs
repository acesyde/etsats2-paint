//! Small generated packages for tests and demos (no game asset involved).

use std::io::{Cursor, Write};

use serde_json::{Value, json};

/// A texture of a sample package: id, name, size, layout version.
#[derive(Clone, Debug)]
pub struct SampleTexture {
    pub id: &'static str,
    pub name: &'static str,
    pub size: u32,
    pub layout: u32,
}

/// Cabin 4096, Chassis 2048 and Accessories 1024, layout 1.
pub fn truck_textures() -> Vec<SampleTexture> {
    vec![
        SampleTexture {
            id: "cabin",
            name: "Cabin",
            size: 4096,
            layout: 1,
        },
        SampleTexture {
            id: "chassis",
            name: "Chassis",
            size: 2048,
            layout: 1,
        },
        SampleTexture {
            id: "accessories",
            name: "Accessories",
            size: 1024,
            layout: 1,
        },
    ]
}

/// The manifest of a one-variant truck with `textures`.
pub fn manifest(id: &str, name: &str, version: &str, textures: &[SampleTexture]) -> Value {
    json!({
        "format": 1,
        "id": id,
        "version": version,
        "name": name,
        "brand": "Sample",
        "kind": "truck",
        "game": "ets2",
        "game_versions": ">=1.50, <1.60",
        "authors": ["TruckPaint tests"],
        "variants": [{
            "id": "standard",
            "name": "Standard cabin",
            "textures": textures.iter().map(|t| json!({
                "id": t.id,
                "name": t.name,
                "size": t.size,
                "template": format!("templates/{}.png", t.id),
                "layout_version": t.layout,
            })).collect::<Vec<_>>(),
        }],
    })
}

/// A small template-like PNG (outline and diagonals on transparency).
pub fn png(side: u32) -> Vec<u8> {
    let mut img = image::RgbaImage::new(side, side);
    for i in 0..side {
        for p in [
            (i, 0),
            (i, side - 1),
            (0, i),
            (side - 1, i),
            (i, i),
            (i, side - 1 - i),
        ] {
            img.put_pixel(p.0, p.1, image::Rgba([20, 20, 20, 255]));
        }
    }
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("encode png");
    out
}

/// A ZIP holding `manifest` as `vehicle.json` and `files`.
pub fn zip(manifest: &Value, files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut w = zip::ZipWriter::new(Cursor::new(&mut out));
        let options = zip::write::SimpleFileOptions::default();
        w.start_file(super::MANIFEST, options).expect("zip");
        w.write_all(manifest.to_string().as_bytes()).expect("zip");
        for (name, bytes) in files {
            w.start_file(name.as_str(), options).expect("zip");
            w.write_all(bytes).expect("zip");
        }
        w.finish().expect("zip");
    }
    out
}

/// A complete valid package of a truck with `textures`.
pub fn package(id: &str, name: &str, version: &str, textures: &[SampleTexture]) -> Vec<u8> {
    let files: Vec<(String, Vec<u8>)> = textures
        .iter()
        .map(|t| (format!("templates/{}.png", t.id), png(64)))
        .collect();
    zip(&manifest(id, name, version, textures), &files)
}

/// A ZIP with a template but no manifest.
pub fn zip_without_manifest() -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut w = zip::ZipWriter::new(Cursor::new(&mut out));
        w.start_file(
            "templates/cabin.png",
            zip::write::SimpleFileOptions::default(),
        )
        .expect("zip");
        w.write_all(&png(8)).expect("zip");
        w.finish().expect("zip");
    }
    out
}

/// A package whose central directory declares an entry of `size` bytes
/// uncompressed (what a ZIP bomb looks like), without the data.
pub fn zip_with_declared_size(manifest: &Value, size: u64) -> Vec<u8> {
    let mut bytes = zip(manifest, &[("templates/cabin.png".into(), png(8))]);
    let size = u32::try_from(size).expect("fits the ZIP field");
    let header = bytes
        .windows(4)
        .rposition(|w| w == b"PK\x01\x02")
        .expect("central directory");
    // Uncompressed size: 24 bytes into a central directory header.
    bytes[header + 24..header + 28].copy_from_slice(&size.to_le_bytes());
    bytes
}
