use std::io::Cursor;

use super::*;
use tp_vehicles::Package;

/// A trailer "Reefer" whose Body (128 × 64) also paints `body.x`, with an
/// accessory "Box" (64 × 64) in `Accessories/`.
fn entry() -> Entry {
    Entry::parse(
        r#"{
  "format": 1,
  "game": "ets2",
  "path": "brand.reefer",
  "name": "Reefer",
  "brand": "Brand",
  "kind": "trailer",
  "version": "1.2.0",
  "game_versions": ">=1.59",
  "folder": "Brand Reefer",
  "source": { "paintjob_packer": "abc", "pack": "1.59 (2026-05-28)" },
  "game_data": {
    "alt_uv": false,
    "colour_picker": false,
    "cabins": [],
    "accessories": { "Body": ["body.x"], "Box": ["box.l", "box.r"] }
  },
  "main": [
    { "id": "body", "name": "Body", "file": "Reefer.png", "size": [128, 64],
      "layout_version": 1, "also_paints": ["body.x"] }
  ],
  "accessories": [
    { "id": "box", "name": "Box", "file": "Accessories/Box.png", "size": [64, 64],
      "layout_version": 3, "ids": ["box.l", "box.r"] }
  ]
}"#,
    )
    .unwrap()
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::new();
    image::GrayImage::new(width, height)
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn write(path: &Path, bytes: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

/// A pack at `root` holding the files of [`entry`] at their sizes.
fn pack(root: &Path) {
    let dir = root.join("Brand Reefer");
    write(&dir.join("Reefer.png"), &png(128, 64));
    write(&dir.join("Accessories/Box.png"), &png(64, 64));
}

#[test]
fn pack_found_directly_or_one_level_down() {
    let e = entry();
    let dir = tempfile::tempdir().unwrap();
    pack(dir.path());
    assert_eq!(
        find_pack_of(dir.path(), &[&e]),
        Some(dir.path().to_path_buf())
    );
    let outer = tempfile::tempdir().unwrap();
    let inner = outer.path().join("ETS2 templates");
    pack(&inner);
    std::fs::create_dir(outer.path().join("Other")).unwrap();
    assert_eq!(find_pack_of(outer.path(), &[&e]), Some(inner.clone()));
    // A wrong folder: no vehicle folder, even two levels down.
    let wrong = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(wrong.path().join("Downloads/a")).unwrap();
    pack(&wrong.path().join("a/b"));
    assert_eq!(find_pack_of(wrong.path(), &[&e]), None);
    assert_eq!(find_pack_of(&wrong.path().join("missing"), &[&e]), None);
}

#[test]
fn builds_a_valid_package() {
    let e = entry();
    let dir = tempfile::tempdir().unwrap();
    pack(dir.path());
    let mut steps = Vec::new();
    let packed = build_with(
        &e,
        dir.path(),
        &mut |done, total| steps.push((done, total)),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(steps, [(1, 2), (2, 2)]);
    let p = Package::read(&packed.bytes).unwrap();
    let m = &p.manifest;
    assert_eq!(m.id, "official.ets2.brand.reefer");
    assert_eq!(m.version, semver::Version::new(1, 2, 0));
    assert_eq!(m.kind, tp_vehicles::Kind::Trailer);
    assert_eq!(m.paint_job.main[0].also_paints, ["body.x"]);
    assert!(m.paint_job.main[0].game_ids.is_empty());
    assert_eq!(m.paint_job.accessories[0].game_ids, ["box.l", "box.r"]);
    assert_eq!(m.paint_job.accessories[0].texture.layout_version, 3);
    let body = p.template("body").unwrap();
    assert_eq!((body.width, body.height), (128.0, 64.0));
    assert_eq!(packed.file_name(), "official.ets2.brand.reefer-1.2.0.tpv");
    // Reproducible.
    assert_eq!(build(&e, dir.path()).unwrap().bytes, packed.bytes);
}

#[test]
fn missing_file_or_other_size_refused() {
    let e = entry();
    let dir = tempfile::tempdir().unwrap();
    pack(dir.path());
    std::fs::remove_file(dir.path().join("Brand Reefer/Accessories/Box.png")).unwrap();
    let err = build(&e, dir.path()).unwrap_err();
    assert!(
        matches!(&err, CatalogError::MissingFile { vehicle, file }
            if vehicle == "Reefer" && file == "Accessories/Box.png"),
        "{err:?}"
    );
    assert!(err.to_string().contains("Accessories/Box.png"));

    pack(dir.path());
    write(&dir.path().join("Brand Reefer/Reefer.png"), &png(64, 64));
    let err = build(&e, dir.path()).unwrap_err();
    assert!(
        matches!(&err, CatalogError::WrongSize { file, expected, actual, .. }
            if file == "Reefer.png"
                && *expected == TextureSize::new(128, 64)
                && *actual == (64, 64)),
        "{err:?}"
    );
    assert!(err.to_string().contains("Reefer.png"));

    write(&dir.path().join("Brand Reefer/Reefer.png"), b"not a png");
    assert!(matches!(
        build(&e, dir.path()),
        Err(CatalogError::Unreadable { .. })
    ));
    let empty = tempfile::tempdir().unwrap();
    assert!(matches!(
        build(&e, empty.path()),
        Err(CatalogError::NoVehicleFolder { .. })
    ));
}

#[test]
fn cancelled_build() {
    let e = entry();
    let dir = tempfile::tempdir().unwrap();
    pack(dir.path());
    let cancel = AtomicBool::new(false);
    let err = build_with(
        &e,
        dir.path(),
        &mut |_, _| cancel.store(true, Ordering::Relaxed),
        &cancel,
    )
    .unwrap_err();
    assert!(matches!(err, CatalogError::Cancelled));
}

#[test]
fn pack_versions() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(pack_version(dir.path()), None);
    std::fs::write(
        dir.path().join(PACK_INFO),
        "Hey there!\n\nThese templates are current for game version 1.59, updated 2026-05-28\n",
    )
    .unwrap();
    assert_eq!(
        pack_version(dir.path()).as_deref(),
        Some("1.59 (2026-05-28)")
    );
}

#[test]
fn check_lists_problems_and_unused_files() {
    let e = entry();
    let dir = tempfile::tempdir().unwrap();
    pack(dir.path());
    write(
        &dir.path().join("Brand Reefer/Accessories/Extra.png"),
        &png(8, 8),
    );
    let report = check(dir.path(), &[&e]);
    assert_eq!(report.problems, Vec::<String>::new());
    assert_eq!(report.unused, ["Brand Reefer/Accessories/Extra.png"]);
    assert_eq!(report.pack_version, None);
    assert!(report.text().contains("no problem"));

    let mut incomplete = entry();
    incomplete.accessories[0].ids.pop();
    std::fs::remove_file(dir.path().join("Brand Reefer/Reefer.png")).unwrap();
    let report = check(dir.path(), &[&incomplete]);
    assert_eq!(
        report.problems,
        [
            "Reefer: the accessory id \"box.r\" is neither covered nor left out",
            "Reefer: \"Reefer.png\" is missing"
        ]
    );
}

#[test]
fn accented_folder_found_in_either_unicode_form() {
    let mut e = entry();
    e.folder = "K\u{f6}gel Cargo".into();
    let dir = tempfile::tempdir().unwrap();
    let vehicle = dir.path().join("Ko\u{308}gel Cargo");
    write(&vehicle.join("Reefer.png"), &png(128, 64));
    write(&vehicle.join("Accessories/Box.png"), &png(64, 64));
    assert_eq!(
        find_pack_of(dir.path(), &[&e]),
        Some(dir.path().to_path_buf())
    );
    build(&e, dir.path()).expect("built from the decomposed folder");
    let report = check(dir.path(), &[&e]);
    assert_eq!(report.problems, Vec::<String>::new());
    assert_eq!(report.unused, Vec::<String>::new());
}

#[test]
fn macos_resource_files_ignored() {
    let e = entry();
    let dir = tempfile::tempdir().unwrap();
    pack(dir.path());
    write(
        &dir.path().join("Brand Reefer/Accessories/._Box.png"),
        b"not a png",
    );
    let report = check(dir.path(), &[&e]);
    assert_eq!(report.problems, Vec::<String>::new());
    assert_eq!(report.unused, Vec::<String>::new());
}

#[test]
fn shared_folder_file_used_by_another_entry_is_not_unused() {
    let r = entry();
    let mut s = entry();
    s.path = "brand.reefer_s".into();
    s.name = "Reefer S".into();
    s.main[0].file = "S Reefer.png".into();
    let dir = tempfile::tempdir().unwrap();
    pack(dir.path());
    write(&dir.path().join("Brand Reefer/S Reefer.png"), &png(128, 64));
    write(&dir.path().join("Brand Reefer/Spare.png"), &png(64, 64));
    let report = check(dir.path(), &[&r, &s]);
    assert_eq!(report.problems, Vec::<String>::new());
    assert_eq!(report.unused, ["Brand Reefer/Spare.png"]);
}
