use serde_json::json;

use super::sample::{self, SampleTexture};
use super::*;

fn ok_package() -> Vec<u8> {
    sample::package(
        "scs.sample.truck",
        "Sample Truck",
        "1.2.0",
        &sample::truck_textures(),
    )
}

fn with_manifest(edit: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
    let textures = sample::truck_textures();
    let mut m = sample::manifest("scs.sample.truck", "Sample Truck", "1.2.0", &textures);
    edit(&mut m);
    let files: Vec<(String, Vec<u8>)> = textures
        .iter()
        .map(|t| (format!("templates/{}.png", t.id), sample::png(32)))
        .collect();
    sample::zip(&m, &files)
}

fn err(bytes: &[u8]) -> PackageError {
    Package::read(bytes).expect_err("refused")
}

#[test]
fn reads_a_valid_package() {
    let p = Package::read(&ok_package()).unwrap();
    assert_eq!(p.manifest.id, "scs.sample.truck");
    assert_eq!(p.manifest.version, semver::Version::new(1, 2, 0));
    assert_eq!(p.manifest.kind, Kind::Truck);
    assert_eq!(p.manifest.game, Game::Ets2);
    assert!(
        p.manifest
            .game_versions
            .matches(&semver::Version::new(1, 53, 0))
    );
    assert!(
        !p.manifest
            .game_versions
            .matches(&semver::Version::new(1, 60, 0))
    );
    let v = &p.manifest.variants[0];
    assert_eq!(v.textures.len(), 3);
    let cabin = p.template("standard", "cabin").unwrap();
    assert_eq!((cabin.kind, cabin.width), (ImageKind::Png, 64.0));
    assert_eq!(Package::read_manifest(&ok_package()).unwrap(), p.manifest);
}

#[test]
fn unknown_fields_and_optional_data() {
    let bytes = with_manifest(|m| {
        m["mirror"] = json!({ "axis": "x" });
        m["requires"] = json!([{ "name": "Some mod", "version": ">=2" }]);
        m["variants"][0]["textures"][0]["export"] = json!({ "def": "/def/vehicle/x.sii" });
    });
    let p = Package::read(&bytes).unwrap();
    assert_eq!(p.manifest.requires[0].name, "Some mod");
    assert!(p.manifest.variants[0].textures[0].export.is_some());
}

#[test]
fn svg_templates() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="200"><rect width="10" height="10"/></svg>"#;
    let t = [SampleTexture {
        id: "cabin",
        name: "Cabin",
        size: 4096,
        layout: 1,
    }];
    let mut m = sample::manifest("a.b", "A", "1.0.0", &t);
    m["variants"][0]["textures"][0]["template"] = json!("templates/cabin.svg");
    let bytes = sample::zip(&m, &[("templates/cabin.svg".into(), svg.to_vec())]);
    let p = Package::read(&bytes).unwrap();
    let i = p.template("standard", "cabin").unwrap();
    assert_eq!((i.kind, i.width, i.height), (ImageKind::Svg, 300.0, 200.0));
}

#[test]
fn invalid_packages_are_refused() {
    assert_eq!(err(b"not a zip"), PackageError::NotAZip);
    assert_eq!(
        err(&sample::zip_without_manifest()),
        PackageError::NoManifest
    );
    assert!(matches!(
        err(&with_manifest(|m| m["name"] = json!(3))),
        PackageError::BadManifest(_)
    ));
    assert!(matches!(
        err(&with_manifest(|m| {
            m.as_object_mut().unwrap().remove("game");
        })),
        PackageError::BadManifest(_)
    ));
    assert!(matches!(
        err(&with_manifest(|m| m["version"] = json!("1.2"))),
        PackageError::BadManifest(_)
    ));
    assert!(matches!(
        err(&with_manifest(|m| m["game_versions"] = json!("about 1.5"))),
        PackageError::BadManifest(_)
    ));
    assert!(matches!(
        err(&with_manifest(|m| m["kind"] = json!("boat"))),
        PackageError::BadManifest(_)
    ));
    assert_eq!(
        err(&with_manifest(|m| m["format"] = json!(2))),
        PackageError::NewerFormat(2)
    );
    assert_eq!(
        err(&with_manifest(|m| m["id"] = json!("Volvo"))),
        PackageError::BadId("Volvo".into())
    );
    assert_eq!(
        err(&with_manifest(|m| m["variants"] = json!([]))),
        PackageError::NoVariant
    );
    assert_eq!(
        err(&with_manifest(|m| m["variants"][0]["textures"] = json!([]))),
        PackageError::EmptyVariant("Standard cabin".into())
    );
    assert!(matches!(
        err(&with_manifest(
            |m| m["variants"][0]["textures"][1]["id"] = json!("cabin")
        )),
        PackageError::DuplicateTexture { .. }
    ));
    assert_eq!(
        err(&with_manifest(
            |m| m["variants"][0]["textures"][0]["size"] = json!(3000)
        )),
        PackageError::BadSize {
            texture: "Cabin".into(),
            size: 3000
        }
    );
    assert_eq!(
        err(&with_manifest(
            |m| m["variants"][0]["textures"][1]["template"] = json!("templates/missing.png")
        )),
        PackageError::MissingTemplate {
            texture: "Chassis".into(),
            path: "templates/missing.png".into()
        }
    );
    assert!(matches!(
        err(&with_manifest(
            |m| m["variants"][0]["textures"][0]["template"] = json!("templates/cabin.jpg")
        )),
        PackageError::MissingTemplate { .. } | PackageError::BadTemplate { .. }
    ));
}

#[test]
fn wrong_image_type_and_unsafe_paths() {
    let t = &sample::truck_textures()[..1];
    let m = sample::manifest("a.b", "A", "1.0.0", t);
    // A PNG name holding something else.
    let bad = sample::zip(&m, &[("templates/cabin.png".into(), b"GIF89a".to_vec())]);
    assert!(matches!(err(&bad), PackageError::BadTemplate { .. }));
    // Any entry escaping the package, even unused.
    let evil = sample::zip(
        &m,
        &[
            ("templates/cabin.png".into(), sample::png(8)),
            ("../../evil.png".into(), sample::png(8)),
        ],
    );
    assert_eq!(
        err(&evil),
        PackageError::UnsafePath("../../evil.png".into())
    );
    assert!(!is_safe_path("/etc/passwd") && !is_safe_path("C:/x") && !is_safe_path("a/../b"));
    assert!(is_safe_path("templates/cabin.png"));
}

#[test]
fn oversized_images_and_packages() {
    let t = &sample::truck_textures()[..1];
    let m = sample::manifest("a.b", "A", "1.0.0", t);
    // A PNG header claiming 20000 × 20000 pixels.
    let mut huge = sample::png(8);
    huge[16..20].copy_from_slice(&20_000u32.to_be_bytes());
    huge[20..24].copy_from_slice(&20_000u32.to_be_bytes());
    let bytes = sample::zip(&m, &[("templates/cabin.png".into(), huge)]);
    assert!(matches!(
        err(&bytes),
        PackageError::TemplateTooLarge { .. } | PackageError::BadTemplate { .. }
    ));
    // Declared sizes beyond the limit (a zip bomb) are refused before reading.
    let big = sample::zip_with_declared_size(&m, MAX_UNCOMPRESSED + 1);
    assert_eq!(err(&big), PackageError::TooLarge);
}

#[test]
fn ids() {
    assert!(is_valid_id("scs.volvo.fh16_2012"));
    assert!(is_valid_id("community.jdoe.mighty-hauler"));
    assert!(!is_valid_id("volvo"));
    assert!(!is_valid_id("SCS.volvo"));
    assert!(!is_valid_id("scs..volvo"));
}

#[test]
fn documented_example_manifest_is_valid() {
    let doc = include_str!("../../../docs/vehicle-package-format.md");
    let start = doc.find("```json\n").expect("json example") + "```json\n".len();
    let end = start + doc[start..].find("```").expect("end of example");
    let manifest: Manifest = serde_json::from_str(&doc[start..end]).expect("parses");
    validate(&manifest).expect("valid");
    assert_eq!(manifest.variants[0].textures.len(), 3);
}
