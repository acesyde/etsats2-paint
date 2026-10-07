//! The `tpv` command: exit codes and written files.

use std::path::Path;
use std::process::{Command, Output};

use tp_vehicles::sample;

fn tpv(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tpv"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run tpv")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// A folder holding a valid sample manifest and its templates.
fn source(dir: &Path) {
    let manifest = sample::manifest(
        "community.jdoe.my_truck",
        "My Truck",
        "1.0.0",
        &sample::truck_textures(),
    );
    std::fs::create_dir_all(dir.join("src/templates")).unwrap();
    std::fs::write(dir.join("src/vehicle.json"), manifest.to_string()).unwrap();
    for t in sample::truck_textures() {
        std::fs::write(
            dir.join(format!("src/templates/{}.png", t.id)),
            sample::png(16),
        )
        .unwrap();
    }
}

#[test]
fn pack_writes_the_default_file_and_check_reads_it() {
    let dir = tempfile::tempdir().unwrap();
    source(dir.path());
    let out = tpv(dir.path(), &["pack", "src"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let file = "community.jdoe.my_truck-1.0.0.tpv";
    assert!(dir.path().join(file).is_file());
    assert!(text(&out.stdout).contains("packed    community.jdoe.my_truck-1.0.0.tpv"));

    let out = tpv(dir.path(), &["check", file]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("community.jdoe.my_truck 1.0.0"));

    let out = tpv(dir.path(), &["pack", "src", "-o", "custom.tpv"]);
    assert!(out.status.success());
    assert!(dir.path().join("custom.tpv").is_file());
}

#[test]
fn failed_pack_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    source(dir.path());
    std::fs::remove_file(dir.path().join("src/templates/cabin.png")).unwrap();
    let out = tpv(dir.path(), &["pack", "src", "-o", "out.tpv"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("Cabin template is missing"),
        "{}",
        text(&out.stderr)
    );
    assert!(!dir.path().join("out.tpv").exists());
}

#[test]
fn check_refuses_a_corrupt_file_and_usage_errors_fail() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("bad.tpv"), b"junk").unwrap();
    let out = tpv(dir.path(), &["check", "bad.tpv"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stderr).contains("not a ZIP file"));
    assert_eq!(tpv(dir.path(), &[]).status.code(), Some(1));
    assert_eq!(tpv(dir.path(), &["pack"]).status.code(), Some(1));
    let help = tpv(dir.path(), &["--help"]);
    assert!(help.status.success());
    assert!(text(&help.stdout).contains("Usage:"));
}
