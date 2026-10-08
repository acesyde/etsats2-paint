use std::path::Path;

use serde_json::json;
use tp_vehicles::sample;

use super::*;

/// A truck manifest whose first `(id, size, path)` is its main texture and
/// the others accessories.
fn manifest(templates: &[(&str, u32, &str)]) -> Value {
    let part = |(id, size, path): &(&str, u32, &str), game_ids: Vec<String>| {
        json!({
            "id": id,
            "name": id,
            "game_ids": game_ids,
            "texture": { "size": size, "template": path, "layout_version": 1 },
            "future_part_field": {"kept": true},
        })
    };
    json!({
        "format": 1,
        "id": "community.jdoe.my_truck",
        "version": "1.0.0",
        "name": "My Truck",
        "brand": "Jdoe",
        "kind": "truck",
        "game": { "id": "ets2", "versions": ">=1.50", "path": "jdoe.my_truck" },
        "future_field": {"kept": true},
        "paint_job": {
            "main": templates[..1].iter().map(|t| part(t, Vec::new())).collect::<Vec<_>>(),
            "accessories": templates[1..]
                .iter()
                .map(|t| part(t, vec![format!("{}.acc", t.0)]))
                .collect::<Vec<_>>(),
        },
    })
}

fn write(dir: &Path, path: &str, bytes: &[u8]) {
    let file = dir.join(path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, bytes).unwrap();
}

fn folder(manifest: &Value) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), MANIFEST, manifest.to_string().as_bytes());
    dir
}

fn packaged_manifest(bytes: &[u8]) -> Value {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let file = zip.by_name(MANIFEST).unwrap();
    serde_json::from_reader(file).unwrap()
}

#[test]
fn packs_referenced_files_reproducibly() {
    let dir = folder(&manifest(&[
        ("cabin", 1024, "templates/cabin.png"),
        ("chassis", 512, "templates/cabin.png"),
    ]));
    write(dir.path(), "templates/cabin.png", &sample::png(16));
    write(dir.path(), "notes.txt", b"todo");
    write(dir.path(), "work/cabin.psd", b"psd");
    write(dir.path(), ".DS_Store", b"x");
    let a = pack_folder(dir.path()).unwrap();
    let b = pack_folder(dir.path()).unwrap();
    assert_eq!(a.bytes, b.bytes, "identical bytes");
    assert_eq!(a.ignored, ["notes.txt", "work/cabin.psd"]);
    assert_eq!(a.file_name(), "community.jdoe.my_truck-1.0.0.tpv");
    let zip = zip::ZipArchive::new(Cursor::new(&a.bytes)).unwrap();
    let names: Vec<&str> = zip.file_names().collect();
    assert_eq!(names, [MANIFEST, "templates/cabin.png"]);
    // Unknown fields are kept, at every level.
    let m = packaged_manifest(&a.bytes);
    assert_eq!(m["future_field"]["kept"], true);
    assert_eq!(
        m["paint_job"]["accessories"][0]["future_part_field"]["kept"],
        true
    );
}

#[test]
fn missing_template_names_the_texture() {
    let dir = folder(&manifest(&[("cabin", 1024, "templates/cabin.png")]));
    let err = pack_folder(dir.path()).unwrap_err();
    assert!(matches!(err, PackError::MissingFile { .. }), "{err:?}");
    assert!(
        err.to_string().contains("cabin template is missing"),
        "{err}"
    );
}

#[test]
fn unsafe_paths_are_refused() {
    let dir = folder(&manifest(&[("cabin", 1024, "../cabin.png")]));
    write(dir.path().parent().unwrap(), "cabin.png", &sample::png(8));
    let err = pack_folder(dir.path()).unwrap_err();
    assert!(matches!(err, PackError::UnsafePath(_)), "{err:?}");
}

#[test]
fn no_manifest_and_bad_manifest() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        pack_folder(dir.path()),
        Err(PackError::NoManifest(_))
    ));
    write(dir.path(), MANIFEST, b"{ nope");
    assert!(matches!(
        pack_folder(dir.path()),
        Err(PackError::BadManifest(_))
    ));
}

#[test]
fn validation_errors_come_from_the_reader() {
    let mut m = manifest(&[("cabin", 1000, "templates/cabin.png")]);
    let dir = folder(&m);
    write(dir.path(), "templates/cabin.png", &sample::png(8));
    let err = pack_folder(dir.path()).unwrap_err();
    assert!(
        matches!(err, PackError::Package(PackageError::BadSize { .. })),
        "{err:?}"
    );
    m["id"] = json!("Bad Id");
    write(dir.path(), MANIFEST, m.to_string().as_bytes());
    assert!(err.to_string().contains("1000"));
    assert!(matches!(
        pack_folder(dir.path()),
        Err(PackError::Package(PackageError::BadId(_)))
    ));
}

#[test]
fn dds_templates_become_png() {
    let dir = folder(&manifest(&[
        ("cabin", 256, "templates/cabin.dds"),
        ("chassis", 256, "templates/chassis.DDS"),
    ]));
    let image = dds::build::quadrants();
    write(
        dir.path(),
        "templates/cabin.dds",
        &dds::build::bc(&image, texpresso::Format::Bc3, b"DXT5"),
    );
    write(
        dir.path(),
        "templates/chassis.DDS",
        &dds::build::bgra(&image),
    );
    let packed = pack_folder(dir.path()).unwrap();
    assert_eq!(
        packed.converted,
        [
            ("templates/cabin.dds".into(), "templates/cabin.png".into()),
            (
                "templates/chassis.DDS".into(),
                "templates/chassis.png".into()
            ),
        ]
    );
    let m = packaged_manifest(&packed.bytes);
    // A main texture and an accessory are both rewritten.
    assert_eq!(
        m["paint_job"]["main"][0]["texture"]["template"],
        "templates/cabin.png"
    );
    assert_eq!(
        m["paint_job"]["accessories"][0]["texture"]["template"],
        "templates/chassis.png"
    );
    let package = Package::read(&packed.bytes).unwrap();
    let chassis = package.template("chassis").unwrap();
    assert_eq!((chassis.width, chassis.height), (16.0, 16.0));
    let decoded = image::load_from_memory(&chassis.bytes).unwrap().to_rgba8();
    assert_eq!(decoded, image, "uncompressed DDS converts exactly");
    // The source folder is untouched.
    assert!(dir.path().join("templates/cabin.dds").exists());
    assert!(!dir.path().join("templates/cabin.png").exists());
}

#[test]
fn unsupported_dds_names_the_texture() {
    let dir = folder(&manifest(&[("cabin", 256, "templates/cabin.dds")]));
    let image = dds::build::quadrants();
    write(
        dir.path(),
        "templates/cabin.dds",
        &dds::build::dx10(&image, texpresso::Format::Bc3, 98),
    );
    let err = pack_folder(dir.path()).unwrap_err();
    assert!(matches!(err, PackError::UnsupportedDds { .. }), "{err:?}");
    assert!(err.to_string().contains("cabin template"), "{err}");
}

#[test]
fn dds_and_png_with_the_same_name_collide() {
    let dir = folder(&manifest(&[
        ("cabin", 256, "templates/cabin.dds"),
        ("chassis", 256, "templates/cabin.png"),
    ]));
    let image = dds::build::quadrants();
    write(dir.path(), "templates/cabin.dds", &dds::build::bgra(&image));
    write(dir.path(), "templates/cabin.png", &sample::png(8));
    assert!(matches!(
        pack_folder(dir.path()),
        Err(PackError::DuplicateEntry(_))
    ));
}

#[test]
fn check_and_summary() {
    let bytes = sample::package(
        "scs.sample.truck",
        "Sample Truck",
        "1.2.0",
        &sample::truck_textures(),
    );
    let m = check(&bytes).unwrap();
    let text = summary(&m);
    assert!(text.starts_with("scs.sample.truck 1.2.0"), "{text}");
    assert!(text.contains("game path: sample.vehicle"), "{text}");
    assert!(text.contains("  main textures:\n"), "{text}");
    assert!(
        text.contains("cabin (Cabin): 4096×4096, layout 1, templates/cabin.png [whole vehicle]"),
        "{text}"
    );
    assert!(text.contains("  accessories:\n"), "{text}");
    assert!(
        text.contains(
            "chassis (Chassis): 2048×2048, layout 1, templates/chassis.png [chassis.sample]"
        ),
        "{text}"
    );
    assert!(matches!(
        check(b"junk"),
        Err(PackError::Package(PackageError::NotAZip))
    ));
}

#[test]
fn packs_from_memory() {
    let image = dds::build::quadrants();
    let files = BTreeMap::from([
        (
            "templates/cabin.dds".to_owned(),
            dds::build::bc(&image, texpresso::Format::Bc3, b"DXT5"),
        ),
        ("templates/mirrors.png".to_owned(), sample::png(8)),
        ("notes.txt".to_owned(), b"not referenced".to_vec()),
    ]);
    let mut count = 0;
    let packed = pack_entries_with(
        manifest(&[
            ("cabin", 256, "templates/cabin.dds"),
            ("mirrors", 256, "templates/mirrors.png"),
        ]),
        files,
        &mut || count += 1,
    )
    .unwrap();
    assert_eq!(count, 1, "one DDS converted");
    assert!(packed.ignored.is_empty());
    let zip = zip::ZipArchive::new(Cursor::new(&packed.bytes)).unwrap();
    let names: Vec<&str> = zip.file_names().collect();
    assert_eq!(
        names,
        [MANIFEST, "templates/cabin.png", "templates/mirrors.png"]
    );
    let m = packaged_manifest(&packed.bytes);
    assert_eq!(
        m["paint_job"]["main"][0]["texture"]["template"],
        "templates/cabin.png"
    );
    let package = Package::read(&packed.bytes).unwrap();
    let cabin = package.template("cabin").unwrap();
    assert_eq!((cabin.width, cabin.height), (16.0, 16.0));

    let err = pack_entries(
        manifest(&[("cabin", 256, "templates/cabin.png")]),
        BTreeMap::new(),
    )
    .unwrap_err();
    assert!(matches!(err, PackError::MissingFile { .. }), "{err:?}");
}
