//! `tpv catalog sync` and `tpv catalog check`.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tp_vehicles::catalog::Entry;

fn tpv(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tpv"))
        .args(args)
        .output()
        .expect("run tpv")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn s(path: &Path) -> &str {
    path.to_str().unwrap()
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

/// A copy of the Paintjob Packer fixture and of the repository's catalog.
fn sync_setup(dir: &Path) -> (PathBuf, PathBuf) {
    let ppk = dir.join("paintjob-packer");
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/paintjob-packer"),
        &ppk,
    );
    let catalog = dir.join("catalog");
    copy_dir(&repo("catalog"), &catalog);
    (ppk, catalog)
}

fn entry(catalog: &Path, path: &str) -> Entry {
    Entry::parse(&std::fs::read_to_string(catalog.join(path)).unwrap()).unwrap()
}

const DAF: &str = "ets2/daf.2021.json";
const KRONE: &str = "ets2/krone.coolliner.json";
const COMMIT: &str = "a7a7ed3902ca3fe6f9ff3eb4afd2d92db3f5db2d";

#[test]
fn accessory_id_added_upstream() {
    let dir = tempfile::tempdir().unwrap();
    let (ppk, catalog) = sync_setup(dir.path());
    let daf_json = ppk.join("vehicles/ets/scs/daf.2021.json");
    let json = std::fs::read_to_string(&daf_json).unwrap().replace(
        "\"sunshield.paint_xgp\"",
        "\"sunshield.paint_xgp\",\n\t\t\t\"sunshield.paint_xd\"",
    );
    std::fs::write(&daf_json, json).unwrap();
    let before = entry(&catalog, DAF);

    let out = tpv(&[
        "catalog",
        "sync",
        s(&ppk),
        "--commit",
        "0123abcd",
        "--catalog",
        s(&catalog),
    ]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("DAF NGD: gained sunshield.paint_xd\n"),
        "{stdout}"
    );
    assert!(stdout.contains("Krone Cool Liner: unchanged\n"), "{stdout}");

    let after = entry(&catalog, DAF);
    assert!(after.game_data.accessories["Sun Visor"].contains(&"sunshield.paint_xd".to_owned()));
    assert_eq!(after.source.paintjob_packer, "0123abcd");
    assert_eq!(after.source.pack, before.source.pack);
    // The mapping is unchanged: the new id is not covered yet.
    assert_eq!(
        (&after.main, &after.accessories, &after.left_out),
        (&before.main, &before.accessories, &before.left_out)
    );
    assert_eq!(after.version, before.version);
    assert!(tp_vehicles::catalog::completeness(&after).contains(
        &tp_vehicles::catalog::Problem::UncoveredAccessory("sunshield.paint_xd".into())
    ));
    assert_eq!(entry(&catalog, KRONE).source.paintjob_packer, "0123abcd");
}

#[test]
fn vehicles_not_in_the_catalog() {
    let dir = tempfile::tempdir().unwrap();
    let (ppk, catalog) = sync_setup(dir.path());
    let out = tpv(&[
        "catalog",
        "sync",
        s(&ppk),
        "--commit",
        COMMIT,
        "--catalog",
        s(&catalog),
    ]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    for line in [
        "not in the catalog: DAF XF105 (ets2, daf.xf)",
        "not in the catalog: DAF XF (ets2, daf.xf_euro6)",
        "not in the catalog: Krone Dry Liner (ets2, krone.dryliner)",
        "not in the catalog: Freightliner Cascadia 2019 (ats, freightliner.cascadia2019)",
    ] {
        assert!(stdout.contains(line), "{stdout}");
    }
    // Mods are not official vehicles; catalog vehicles are not listed.
    assert!(!stdout.contains("MB Aerodynamic"), "{stdout}");
    assert!(!stdout.contains("not in the catalog: DAF NGD"), "{stdout}");
}

#[test]
fn unchanged_run() {
    let dir = tempfile::tempdir().unwrap();
    let (ppk, catalog) = sync_setup(dir.path());
    let out = tpv(&[
        "catalog",
        "sync",
        s(&ppk),
        "--commit",
        COMMIT,
        "--catalog",
        s(&catalog),
    ]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(stdout.contains("DAF NGD: unchanged\n"), "{stdout}");
    assert!(stdout.contains("Krone Cool Liner: unchanged\n"), "{stdout}");
    for path in [DAF, KRONE] {
        assert_eq!(
            std::fs::read_to_string(catalog.join(path)).unwrap(),
            std::fs::read_to_string(repo("catalog").join(path)).unwrap(),
            "{path}"
        );
    }
}

#[test]
fn missing_vehicle_changes_no_entry() {
    let dir = tempfile::tempdir().unwrap();
    let (ppk, catalog) = sync_setup(dir.path());
    std::fs::remove_file(ppk.join("vehicles/ets/scs/krone.coolliner.json")).unwrap();
    let out = tpv(&[
        "catalog",
        "sync",
        s(&ppk),
        "--commit",
        "0123abcd",
        "--catalog",
        s(&catalog),
    ]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("Krone Cool Liner"),
        "{}",
        text(&out.stderr)
    );
    for path in [DAF, KRONE] {
        assert_eq!(
            std::fs::read_to_string(catalog.join(path)).unwrap(),
            std::fs::read_to_string(repo("catalog").join(path)).unwrap(),
        );
    }
    // An unreadable file fails too.
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/paintjob-packer/vehicles/ets/scs/krone.coolliner.json"),
        ppk.join("vehicles/ets/scs/krone.coolliner.json"),
    )
    .unwrap();
    std::fs::write(ppk.join("vehicles/ets/am/broken.json"), "{ not json").unwrap();
    let out = tpv(&[
        "catalog",
        "sync",
        s(&ppk),
        "--commit",
        "0123abcd",
        "--catalog",
        s(&catalog),
    ]);
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("broken.json"));
    assert_eq!(entry(&catalog, DAF).source.paintjob_packer, COMMIT);
}

/// A trailer entry "Reefer" in folder "Brand Reefer": Reefer.png 128 × 64
/// (also painting `body.x`) and Accessories/Box.png 64 × 64.
const REEFER: &str = r#"{
  "format": 1,
  "game": "ets2",
  "path": "brand.reefer",
  "name": "Reefer",
  "brand": "Brand",
  "kind": "trailer",
  "version": "1.0.0",
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
      "layout_version": 1, "ids": ["box.l", "box.r"] }
  ]
}"#;

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

/// A catalog holding `entry` and a pack matching [`REEFER`].
fn check_setup(dir: &Path, entry: &str) -> (PathBuf, PathBuf) {
    let catalog = dir.join("catalog");
    write(&catalog.join("ets2/brand.reefer.json"), entry.as_bytes());
    let pack = dir.join("templates");
    write(&pack.join("Brand Reefer/Reefer.png"), &png(128, 64));
    write(&pack.join("Brand Reefer/Accessories/Box.png"), &png(64, 64));
    write(
        &pack.join("Info and links.txt"),
        b"These templates are current for game version 1.59, updated 2026-05-28\n",
    );
    (catalog, pack)
}

fn check(catalog: &Path, pack: &Path) -> Output {
    tpv(&["catalog", "check", s(pack), "--catalog", s(catalog)])
}

#[test]
fn check_succeeds_when_the_pack_matches() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, pack) = check_setup(dir.path(), REEFER);
    let out = check(&catalog, &pack);
    assert!(out.status.success(), "{}", text(&out.stdout));
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("pack version: 1.59 (2026-05-28)"),
        "{stdout}"
    );
    assert!(stdout.contains("checked: Reefer"), "{stdout}");
    assert!(stdout.contains("no problem"), "{stdout}");
    assert!(
        !stdout.contains("unused") && !stdout.contains("info:"),
        "{stdout}"
    );
    // The same with --game, and from the folder holding the pack.
    let out = tpv(&[
        "catalog",
        "check",
        s(dir.path()),
        "--game",
        "ets2",
        "--catalog",
        s(&catalog),
    ]);
    assert!(out.status.success(), "{}", text(&out.stdout));
    // No vehicle of ATS in this catalog: nothing to check.
    let out = tpv(&[
        "catalog",
        "check",
        s(&pack),
        "--game",
        "ats",
        "--catalog",
        s(&catalog),
    ]);
    assert!(out.status.success());
    assert!(text(&out.stdout).contains("no problem"));
}

#[test]
fn check_uncovered_id() {
    let dir = tempfile::tempdir().unwrap();
    let entry = REEFER.replacen(r#""box.r"] }"#, r#""box.r", "box.m"] }"#, 1);
    assert_ne!(entry, REEFER);
    let (catalog, pack) = check_setup(dir.path(), &entry);
    let out = check(&catalog, &pack);
    assert!(!out.status.success());
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains(
            "problem: Reefer: the accessory id \"box.m\" is neither covered nor left out"
        ),
        "{stdout}"
    );
    assert!(text(&out.stderr).contains("1 problem(s) found"));
}

#[test]
fn check_missing_file_and_size_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, pack) = check_setup(dir.path(), REEFER);
    std::fs::remove_file(pack.join("Brand Reefer/Accessories/Box.png")).unwrap();
    write(&pack.join("Brand Reefer/Reefer.png"), &png(64, 64));
    let out = check(&catalog, &pack);
    assert!(!out.status.success());
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("problem: Reefer: \"Reefer.png\" is 64 × 64 instead of 128 × 64"),
        "{stdout}"
    );
    assert!(
        stdout.contains("problem: Reefer: \"Accessories/Box.png\" is missing"),
        "{stdout}"
    );
    // A missing vehicle folder.
    std::fs::remove_dir_all(pack.join("Brand Reefer")).unwrap();
    std::fs::create_dir(pack.join("Other")).unwrap();
    let out = check(&catalog, &pack);
    assert!(!out.status.success());
    assert!(text(&out.stdout).contains("problem: Reefer: no folder \"Brand Reefer\""));
}

#[test]
fn check_lists_unused_files_and_other_pack_versions() {
    let dir = tempfile::tempdir().unwrap();
    let (catalog, pack) = check_setup(dir.path(), REEFER);
    write(&pack.join("Brand Reefer/Accessories/Mid.png"), &png(8, 8));
    write(&pack.join("Brand Reefer/Reefer.psd"), b"psd");
    write(
        &pack.join("Info and links.txt"),
        b"These templates are current for game version 1.60, updated 2026-09-01\n",
    );
    let out = check(&catalog, &pack);
    assert!(out.status.success(), "{}", text(&out.stdout));
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("unused:  Brand Reefer/Accessories/Mid.png"),
        "{stdout}"
    );
    assert!(!stdout.contains("Reefer.psd"), "{stdout}");
    assert!(
        stdout.contains("pack version: 1.60 (2026-09-01)"),
        "{stdout}"
    );
    assert!(
        stdout.contains("info:    Reefer was validated with the pack 1.59 (2026-05-28)"),
        "{stdout}"
    );
}

#[test]
fn check_usage_errors() {
    let out = tpv(&["catalog", "check"]);
    assert!(!out.status.success());
    let dir = tempfile::tempdir().unwrap();
    let out = tpv(&["catalog", "check", s(dir.path()), "--game", "fs22"]);
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("unknown game"));
    let out = tpv(&["catalog", "sync", s(dir.path())]);
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("--commit"));
}
