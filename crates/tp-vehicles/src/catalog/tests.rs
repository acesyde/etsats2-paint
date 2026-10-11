use std::collections::BTreeMap;
use std::io::Cursor;

use super::*;
use crate::Package;

/// A complete truck: cabins `a` (internal names `a1`, `a2`) and `b`, two
/// main textures, one accessory covering `mirror.x`, and `bumper.x` left
/// out.
fn inline_entry() -> Entry {
    Entry::parse(
        r#"{
  "format": 1,
  "game": "ets2",
  "path": "brand.model",
  "name": "Model",
  "brand": "Brand",
  "kind": "truck",
  "version": "1.0.0",
  "game_versions": ">=1.59",
  "folder": "Brand Model",
  "source": { "paintjob_packer": "abc", "pack": "1.59 (2026-05-28)" },
  "game_data": {
    "alt_uv": false,
    "colour_picker": true,
    "cabins": [
      { "code": "a", "name": "A", "internal_names": ["a1", "a2"] },
      { "code": "b", "name": "B", "internal_names": ["b"] }
    ],
    "accessories": { "Mirrors": ["mirror.x"], "Bumper": ["bumper.x"] }
  },
  "main": [
    { "id": "a", "name": "Cabin A", "file": "Cabin A.png", "size": [256, 256],
      "layout_version": 1, "cabins": ["a1", "a2"] },
    { "id": "b", "name": "Cabin B", "file": "Cabin B.png", "size": [256, 256],
      "layout_version": 2, "cabins": ["b"] }
  ],
  "accessories": [
    { "id": "mirrors", "name": "Mirrors", "file": "Accessories/Mirrors.png",
      "size": [128, 64], "layout_version": 1, "ids": ["mirror.x"] }
  ],
  "left_out": [{ "ids": ["bumper.x"], "reason": "no template in the pack" }]
}"#,
    )
    .unwrap()
}

/// A PNG of `size` (blank, grey).
fn placeholder_png(size: TextureSize) -> Vec<u8> {
    let img = image::GrayImage::new(size.width, size.height);
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// The package of `entry`, its templates placeholder PNGs of their sizes.
fn package(entry: &Entry) -> Vec<u8> {
    let files: Vec<(String, Vec<u8>)> = entry
        .textures()
        .map(|t| (Entry::template_path(t.id), placeholder_png(t.size)))
        .collect();
    crate::sample::zip(&manifest(entry), &files)
}

#[test]
fn a_complete_entry_gives_a_valid_manifest() {
    let entry = inline_entry();
    assert_eq!(completeness(&entry), []);
    let p = Package::read(&package(&entry)).unwrap();
    let m = &p.manifest;
    assert_eq!(m.id, "official.ets2.brand.model");
    assert_eq!(m.version, semver::Version::new(1, 0, 0));
    assert_eq!((m.name.as_str(), m.brand.as_str()), ("Model", "Brand"));
    assert_eq!(m.kind, Kind::Truck);
    assert_eq!(m.game.path, "brand.model");
    assert!(m.game.colour_picker && !m.game.alt_uv);
    assert_eq!(m.paint_job.main[0].game_ids, ["a1", "a2"]);
    assert_eq!(m.paint_job.main[1].texture.layout_version, 2);
    let mirrors = &m.paint_job.accessories[0];
    assert_eq!(mirrors.game_ids, ["mirror.x"]);
    assert_eq!(mirrors.texture.size, TextureSize::new(128, 64));
    assert_eq!(mirrors.texture.template, "templates/mirrors.png");
    assert!(is_official(&m.id) && !is_official("custom.a.b"));
}

#[test]
fn incomplete_entries_are_reported() {
    // An uncovered cabin.
    let mut e = inline_entry();
    e.main[1].cabins.clear();
    e.main[0].cabins = vec!["a1".into(), "a2".into()];
    assert_eq!(
        completeness(&e),
        [
            Problem::MainWithoutCabins("Cabin B".into()),
            Problem::UncoveredCabin("b".into())
        ]
    );
    let mut e = inline_entry();
    e.main[0].cabins.pop();
    assert_eq!(completeness(&e), [Problem::UncoveredCabin("a2".into())]);
    // An uncovered accessory id.
    let mut e = inline_entry();
    e.left_out.clear();
    assert_eq!(
        completeness(&e),
        [Problem::UncoveredAccessory("bumper.x".into())]
    );
    // An id covered twice: by two textures, or by a texture and left out.
    let mut e = inline_entry();
    e.main[0].also_paints = vec!["mirror.x".into()];
    assert_eq!(completeness(&e), [Problem::CoveredTwice("mirror.x".into())]);
    let mut e = inline_entry();
    e.left_out[0].ids.push("mirror.x".into());
    assert_eq!(completeness(&e), [Problem::CoveredTwice("mirror.x".into())]);
    let mut e = inline_entry();
    e.main[1].cabins.push("a1".into());
    assert_eq!(completeness(&e), [Problem::CoveredTwice("a1".into())]);
    // A covered id missing from the game data.
    let mut e = inline_entry();
    e.accessories[0].ids.push("mirror.gone".into());
    e.main[0].cabins.push("gone".into());
    assert_eq!(
        completeness(&e),
        [
            Problem::NotInGameData("gone".into()),
            Problem::NotInGameData("mirror.gone".into())
        ]
    );
}

#[test]
fn a_single_main_texture_covers_every_cabin() {
    let mut e = inline_entry();
    e.main.truncate(1);
    e.main[0].cabins.clear();
    assert_eq!(completeness(&e), []);
    e.main.clear();
    assert!(completeness(&e).contains(&Problem::NoMainTexture));
}

#[test]
fn complete_entries_only() {
    assert_eq!(embedded().len(), ENTRIES.len(), "every entry reads");
    for (path, json) in ENTRIES {
        let entry = Entry::parse(json).unwrap();
        assert_eq!(completeness(&entry), [], "{path}");
        assert_eq!(entry.catalog_path(), *path);
        // Written as `tpv catalog sync` writes it.
        assert_eq!(entry.to_json(), *json, "{path} is not in canonical form");
    }
    // Every official ETS2 vehicle of Paintjob Packer: 26 trucks, 40 trailers.
    let ets2 = entries(Game::Ets2);
    assert_eq!(ets2.len(), 66);
    let trucks = ets2.iter().filter(|e| e.kind == Kind::Truck).count();
    assert_eq!((trucks, ets2.len() - trucks), (26, 40));
    for path in ["daf.2021", "krone.coolliner", "volvo.fh_2024", "scs.box"] {
        assert!(ets2.iter().any(|e| e.path == path), "{path}");
    }
    assert!(entries(Game::Ats).is_empty());
}

#[test]
fn every_catalog_file_is_embedded() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog");
    let mut files = Vec::new();
    for game in std::fs::read_dir(root).unwrap() {
        let game = game.unwrap();
        let name = game.file_name().to_string_lossy().into_owned();
        if !game.path().is_dir() {
            files.push(name);
            continue;
        }
        for file in std::fs::read_dir(game.path()).unwrap() {
            let file = file.unwrap().file_name().to_string_lossy().into_owned();
            files.push(format!("{name}/{file}"));
        }
    }
    files.sort();
    let mut embedded: Vec<String> = ENTRIES.iter().map(|(p, _)| (*p).to_owned()).collect();
    embedded.push(PAINTJOB_PACKER_LICENSE_FILE.into());
    embedded.sort();
    assert_eq!(files, embedded);
    assert!(PAINTJOB_PACKER_LICENSE.starts_with("MIT License"));
    assert!(PAINTJOB_PACKER_LICENSE.contains("Carsmaniac"));
}

#[test]
fn each_entry_builds_a_valid_package() {
    for entry in embedded() {
        let p = Package::read(&package(entry)).unwrap();
        let m = &p.manifest;
        assert_eq!(m.id, entry.package_id());
        assert_eq!(m.kind, entry.kind);
        for t in entry.textures() {
            let (_, part) = m.paint_job.part(t.id).unwrap();
            assert_eq!(part.texture.size, t.size);
            let image = p.template(t.id).unwrap();
            assert_eq!(
                (image.width, image.height),
                (f64::from(t.size.width), f64::from(t.size.height))
            );
        }
    }
}

#[test]
fn the_first_catalog() {
    let daf = by_package_id("official.ets2.daf.2021").unwrap();
    assert_eq!((daf.name.as_str(), daf.kind), ("DAF NGD", Kind::Truck));
    let names = |e: &Entry| e.textures().map(|t| t.name.to_owned()).collect::<Vec<_>>();
    assert_eq!(
        names(daf),
        [
            "XG+",
            "XG",
            "XF",
            "Sun Visor",
            "Sideskirts",
            "Mirrors (standard)",
            "Mirrors (camera)"
        ]
    );
    let krone = by_package_id("official.ets2.krone.coolliner").unwrap();
    assert_eq!(krone.kind, Kind::Trailer);
    assert_eq!(krone.main[0].size, TextureSize::new(8192, 1024));
    assert_eq!(krone.main[0].also_paints, ["body.reefer", "body.reef_13ct"]);
    let left: BTreeMap<&str, &str> = krone
        .left_out
        .iter()
        .flat_map(|l| l.ids.iter().map(|id| (id.as_str(), l.reason.as_str())))
        .collect();
    assert_eq!(
        left,
        BTreeMap::from([
            ("r_bumper.bumper01p", "no template in the pack"),
            ("r_bumper.bumper02p", "no template in the pack")
        ])
    );
}

#[test]
fn entries_sort_by_brand_then_name_ignoring_accents() {
    let named = |brand: &str, name: &str| Entry {
        brand: brand.to_owned(),
        name: name.to_owned(),
        ..inline_entry()
    };
    let all = [
        named("SCS", "SCS Box Trailer"),
        named("Krone", "Krone Dry Liner"),
        named("Kögel", "Kögel Cargo"),
        named("Schwarzmüller", "Schwarzmüller Platform"),
        named("Kässbohrer", "Kässbohrer K.STC"),
        named("Krone", "Krone Cool Liner"),
    ];
    let mut entries: Vec<&Entry> = all.iter().collect();
    sort(&mut entries);
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Kässbohrer K.STC",
            "Kögel Cargo",
            "Krone Cool Liner",
            "Krone Dry Liner",
            "Schwarzmüller Platform",
            "SCS Box Trailer",
        ]
    );
}
