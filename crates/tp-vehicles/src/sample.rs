//! Small generated packages for tests and demos (no game asset involved).

use std::io::{Cursor, Write};

use serde_json::{Value, json};

/// A part of a sample package: id, name, texture size, layout version.
#[derive(Clone, Debug)]
pub struct SampleTexture {
    pub id: &'static str,
    pub name: &'static str,
    pub size: u32,
    pub layout: u32,
}

/// Main texture Cabin 4096, accessories Chassis 2048 and Accessories 1024,
/// layout 1.
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

/// The JSON of a part: game ids `[id]` for a main texture when there are
/// several, `["<id>.sample"]` for an accessory.
fn part(t: &SampleTexture, game_ids: Vec<String>) -> Value {
    json!({
        "id": t.id,
        "name": t.name,
        "game_ids": game_ids,
        "texture": {
            "size": t.size,
            "template": format!("templates/{}.png", t.id),
            "layout_version": t.layout,
        },
    })
}

/// The manifest of a vehicle of `kind` with main textures `main` and
/// accessories `accessories`.
pub fn manifest_with(
    id: &str,
    name: &str,
    version: &str,
    kind: &str,
    main: &[SampleTexture],
    accessories: &[SampleTexture],
) -> Value {
    let several = main.len() > 1;
    json!({
        "format": 1,
        "id": id,
        "version": version,
        "name": name,
        "brand": "Sample",
        "kind": kind,
        "authors": ["TruckPaint tests"],
        "game": {
            "id": "ets2",
            "versions": ">=1.50, <1.60",
            "path": "sample.vehicle",
        },
        "paint_job": {
            "main": main.iter().map(|t| part(t, if several { vec![t.id.to_owned()] } else { Vec::new() })).collect::<Vec<_>>(),
            "accessories": accessories.iter().map(|t| part(t, vec![format!("{}.sample", t.id)])).collect::<Vec<_>>(),
        },
    })
}

/// The manifest of a truck whose first texture is its single main texture
/// and the others its accessories.
pub fn manifest(id: &str, name: &str, version: &str, textures: &[SampleTexture]) -> Value {
    let (main, accessories) = textures.split_at(textures.len().min(1));
    manifest_with(id, name, version, "truck", main, accessories)
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

/// The template files of `textures` (small PNGs).
pub fn templates(textures: &[SampleTexture]) -> Vec<(String, Vec<u8>)> {
    textures
        .iter()
        .map(|t| (format!("templates/{}.png", t.id), png(64)))
        .collect()
}

/// A complete valid package of a truck with `textures` (see [`manifest`]).
pub fn package(id: &str, name: &str, version: &str, textures: &[SampleTexture]) -> Vec<u8> {
    zip(&manifest(id, name, version, textures), &templates(textures))
}

/// A complete valid package of a vehicle of `kind` (see [`manifest_with`]).
pub fn package_with(
    id: &str,
    name: &str,
    version: &str,
    kind: &str,
    main: &[SampleTexture],
    accessories: &[SampleTexture],
) -> Vec<u8> {
    let all: Vec<SampleTexture> = main.iter().chain(accessories).cloned().collect();
    zip(
        &manifest_with(id, name, version, kind, main, accessories),
        &templates(&all),
    )
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
