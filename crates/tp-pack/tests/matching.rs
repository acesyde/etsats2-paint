//! `tpv catalog match`: proposing entries from a fixture copying the real
//! names of the ETS2 template pack and of Paintjob Packer.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tp_pack::catalog::matching::{MatchReport, NO_TEMPLATE, VehicleMatch, match_vehicles};
use tp_vehicles::Game;
use tp_vehicles::catalog::{Entry, completeness};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/match")
        .join(path)
}

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
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

/// The DAF NGD's entry, as the catalog holds it.
fn daf_ngd() -> Entry {
    Entry::parse(&std::fs::read_to_string(repo("catalog/ets2/daf.2021.json")).unwrap()).unwrap()
}

fn run(pack: &Path, only: &[&str]) -> MatchReport {
    let only: Vec<String> = only.iter().map(|s| (*s).to_owned()).collect();
    match_vehicles(
        pack,
        &fixture("paintjob-packer"),
        Game::Ets2,
        "abc",
        &only,
        &[daf_ngd()],
    )
    .unwrap()
}

fn vehicle<'a>(report: &'a MatchReport, path: &str) -> &'a VehicleMatch {
    report
        .vehicles
        .iter()
        .find(|m| m.entry.path == path)
        .unwrap_or_else(|| panic!("{path} matched"))
}

fn all() -> MatchReport {
    run(&fixture("pack"), &[])
}

/// The main textures as (file, cabins).
fn mains(m: &VehicleMatch) -> Vec<(&str, Vec<&str>)> {
    m.entry
        .main
        .iter()
        .map(|t| {
            (
                t.file.as_str(),
                t.cabins.iter().map(String::as_str).collect(),
            )
        })
        .collect()
}

fn accessory<'a>(m: &'a VehicleMatch, file: &str) -> Vec<&'a str> {
    m.entry
        .accessories
        .iter()
        .find(|t| t.file == file)
        .unwrap_or_else(|| panic!("{file} used"))
        .ids
        .iter()
        .map(String::as_str)
        .collect()
}

fn group<'a>(m: &'a VehicleMatch, name: &str) -> Vec<&'a str> {
    m.entry.game_data.accessories[name]
        .iter()
        .map(String::as_str)
        .collect()
}

fn doubts_about<'a>(m: &'a VehicleMatch, text: &str) -> Vec<&'a String> {
    m.doubts.iter().filter(|d| d.contains(text)).collect()
}

#[test]
fn truck_matched_by_cabin_names() {
    let report = all();
    let m = vehicle(&report, "daf.xd");
    let e = &m.entry;
    assert_eq!(
        mains(m),
        [
            ("Cabin A (Sleeper High).png", vec!["slh"]),
            ("Cabin B (Sleeper).png", vec!["sl"]),
            ("Cabin C (Day Cab).png", vec!["day"]),
        ]
    );
    let names: Vec<&str> = e.main.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["Sleeper High", "Sleeper", "Day Cab"]);
    // The files' own sizes (4096² in the pack, 128² in the fixture).
    assert_eq!(e.main[0].size.label(), "128²");
    assert_eq!(
        (e.name.as_str(), e.brand.as_str(), e.folder.as_str()),
        ("DAF XD", "DAF", "DAF XD")
    );
    assert_eq!(e.version, semver::Version::new(1, 0, 0));
    assert_eq!(e.game_versions.to_string(), ">=1.59");
    assert_eq!(e.source.paintjob_packer, "abc");
    assert_eq!(e.source.pack, "1.59 (2026-05-28)");
    assert!(m.uncovered.is_empty(), "{:?}", m.uncovered);
    // The macOS resource file is neither read nor listed.
    assert!(!m.unused.iter().any(|f| f.contains("._")), "{:?}", m.unused);
}

#[test]
fn cabins_matched_by_name_despite_their_codes() {
    let report = all();
    let m = vehicle(&report, "volvo.fh_2024");
    let main = mains(m);
    assert_eq!(main.len(), 8);
    assert!(main.contains(&("Cabin A (Globetrotter XXL).png", vec!["l3h3", "l3h34_8x4"])));
    assert!(main.contains(&(
        "Cabin A Aero (Aero Globetrotter XXL).png",
        vec!["l3h3_aero", "l3h3_8x4_a"]
    )));
    assert!(doubts_about(m, "by its code only").is_empty());
}

#[test]
fn eight_by_four_cabin_with_the_name_of_another() {
    let report = all();
    let m = vehicle(&report, "mercedes.actros2014");
    let by_name: Vec<(&str, &str)> = m
        .entry
        .main
        .iter()
        .map(|t| (t.name.as_str(), t.file.as_str()))
        .collect();
    assert!(by_name.contains(&("GigaSpace", "Cabin A (GigaSpace).png")));
    assert!(by_name.contains(&("GigaSpace 8x4", "Cabin 8x4 (GigaSpace).png")));
    assert!(m.uncovered.is_empty());
}

#[test]
fn single_main_texture_covers_every_cabin() {
    let report = all();
    let m = vehicle(&report, "iveco.sway");
    assert_eq!(mains(m), [("Iveco S-Way.png", vec![])]);
    assert_eq!(m.entry.main[0].name, "Active Space");
    assert_eq!(completeness(&m.entry), []);
}

#[test]
fn shared_folder() {
    let report = all();
    for (path, prefix, other) in [("scania.r_2016", "R ", "S "), ("scania.s_2016", "S ", "R ")] {
        let m = vehicle(&report, path);
        assert_eq!(m.entry.folder, "Scania R & S");
        assert_eq!(m.entry.main.len(), 3);
        for t in &m.entry.main {
            assert!(t.file.starts_with(prefix), "{path}: {}", t.file);
        }
        assert!(!doubts_about(m, "\"Scania R & S\" is shared").is_empty());
        // The other vehicle's cabins are neither used nor unused here.
        assert!(
            !m.unused.iter().any(|f| f.starts_with(other)),
            "{:?}",
            m.unused
        );
    }
}

#[test]
fn trailer_with_several_bodies() {
    let report = all();
    let m = vehicle(&report, "kogel.cargo");
    let e = &m.entry;
    assert_eq!(e.kind, tp_vehicles::Kind::Trailer);
    assert_eq!(e.main.len(), 1);
    assert_eq!(e.main[0].file, "Body.png");
    assert!(e.main[0].cabins.is_empty());
    assert_eq!(e.main[0].also_paints, group(m, "Body"));
    assert_eq!(accessory(m, "Body (Long).png"), group(m, "Body (Long)"));
    assert_eq!(completeness(e), []);
    assert!(m.doubts.is_empty(), "{:?}", m.doubts);
}

#[test]
fn bodies_named_without_body() {
    let report = all();
    let m = vehicle(&report, "scs.fueltank");
    assert_eq!(m.entry.main[0].file, "2 Axles.png");
    assert_eq!(m.entry.main[0].also_paints, group(m, "Body (2 Axles)"));
    assert_eq!(accessory(m, "3 Axles.png"), group(m, "Body (3 Axles)"));
    assert!(doubts_about(m, "Axles.png").is_empty(), "{:?}", m.doubts);
    // Left and right read on the id's last one-letter word.
    assert_eq!(
        accessory(m, "Accessories/Chassis Front (Left) - Painted Box.png"),
        ["chs_f_l.paint_box"]
    );
    assert_eq!(
        accessory(m, "Accessories/Chassis Front (Right) - Painted Box.png"),
        ["chs_f_r.paint_box"]
    );
}

#[test]
fn group_without_a_template() {
    let report = all();
    let m = vehicle(&report, "krone.coolliner");
    let bumper = m
        .entry
        .left_out
        .iter()
        .find(|l| l.ids.contains(&"r_bumper.bumper01p".to_owned()))
        .expect("rear bumper left out");
    assert_eq!(bumper.ids, group(m, "Rear Bumper"));
    assert_eq!(bumper.reason, NO_TEMPLATE);
    assert!(report.text().contains(&format!(
        "left out:  r_bumper.bumper01p, r_bumper.bumper02p ({NO_TEMPLATE})"
    )));
}

#[test]
fn ids_no_file_names_go_to_the_plainest_file() {
    let report = all();
    let m = vehicle(&report, "volvo.fh_2024");
    let visor = group(m, "Sun Visor");
    assert_eq!(accessory(m, "Accessories/Sun Visor - Painted.png"), visor);
    assert_eq!(doubts_about(m, "as the group shares one layout").len(), 5);
    assert!(
        doubts_about(m, "sunshld.dual")
            .iter()
            .any(|d| d.contains("Sun Visor - Painted.png"))
    );
    // A file naming its ids wins: Standard and Camera on the DAF NGD.
    let only = run(&fixture("pack"), &["daf.xd"]);
    let xd = vehicle(&only, "daf.xd");
    assert_eq!(
        accessory(xd, "Accessories/Mirrors (Camera).png"),
        ["mirror.cam_semi", "mirror.cam_semi_h"]
    );
    assert_eq!(completeness(&m.entry), []);
}

#[test]
fn split_body_and_typo_are_doubts() {
    let report = all();
    let m = vehicle(&report, "wielton.curtainm");
    assert_eq!(m.entry.main[0].file, "Body (Curtain).png");
    assert!(!doubts_about(m, "the group \"Body\" has no file of its name").is_empty());
    let m = vehicle(&report, "wielton.dropsidem");
    assert_eq!(accessory(m, "Body (835 mm).png"), group(m, "Body (825 mm)"));
    assert!(
        !doubts_about(
            m,
            "\"Body (835 mm).png\" matched to the group \"Body (825 mm)\""
        )
        .is_empty()
    );
}

#[test]
fn cabin_that_cant_be_placed() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture("pack"), dir.path());
    let xd = dir.path().join("DAF XD");
    std::fs::rename(
        xd.join("Cabin C (Day Cab).png"),
        xd.join("Cabin Z (Night Cab).png"),
    )
    .unwrap();
    let report = run(dir.path(), &["daf.xd"]);
    let m = vehicle(&report, "daf.xd");
    assert_eq!(m.uncovered, ["day"]);
    assert!(report.text().contains("uncovered: day"));
    assert!(m.unused.contains(&"Cabin Z (Night Cab).png".to_owned()));
    assert!(!completeness(&m.entry).is_empty());
}

#[test]
fn existing_entry_kept_and_missing_folders_listed() {
    let report = all();
    assert!(report.vehicles.iter().all(|m| m.entry.path != "daf.2021"));
    assert_eq!(report.kept, ["DAF NGD"]);
    assert_eq!(
        report.not_found,
        [("Renault Magnum".to_owned(), "renault.magnum".to_owned())]
    );
    assert_eq!(report.spare_folders, ["SCS Livestock"]);
    // Mods of Paintjob Packer are never matched.
    assert!(
        report
            .vehicles
            .iter()
            .all(|m| !m.entry.path.starts_with("am."))
    );
}

#[test]
fn same_drafts_twice_and_each_builds() {
    let first = all();
    let second = all();
    let json =
        |r: &MatchReport| -> Vec<String> { r.vehicles.iter().map(|m| m.entry.to_json()).collect() };
    assert_eq!(json(&first), json(&second));
    for m in &first.vehicles {
        tp_pack::catalog::build(&m.entry, &fixture("pack"))
            .unwrap_or_else(|e| panic!("{}: {e}", m.entry.name));
    }
}

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

#[test]
fn command_writes_missing_drafts_only() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = dir.path().join("catalog");
    std::fs::create_dir_all(catalog.join("ets2")).unwrap();
    let daf = repo("catalog/ets2/daf.2021.json");
    std::fs::copy(&daf, catalog.join("ets2/daf.2021.json")).unwrap();
    // A draft written by an earlier run, then edited by hand.
    let mut edited = daf_ngd();
    edited.path = "daf.xd".into();
    edited.name = "Edited".into();
    std::fs::write(catalog.join("ets2/daf.xd.json"), edited.to_json()).unwrap();
    let out = tpv(&[
        "catalog",
        "match",
        s(&fixture("pack")),
        s(&fixture("paintjob-packer")),
        "--commit",
        "abc",
        "--catalog",
        s(&catalog),
    ]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert_eq!(
        std::fs::read_to_string(catalog.join("ets2/daf.2021.json")).unwrap(),
        std::fs::read_to_string(&daf).unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(catalog.join("ets2/daf.xd.json")).unwrap(),
        edited.to_json()
    );
    let kogel = std::fs::read_to_string(catalog.join("ets2/kogel.cargo.json")).unwrap();
    Entry::parse(&kogel).unwrap();
    for line in [
        "== Kögel Cargo (kogel.cargo) in \"Kögel Cargo\"",
        "kept:      DAF NGD (already in the catalog)",
        "not found: Renault Magnum (renault.magnum): no folder in the pack",
        "no vehicle: folder \"SCS Livestock\"",
        "doubt:     folder \"Scania R & S\" is shared",
        "unused:    ",
    ] {
        assert!(stdout.contains(line), "{line}\n{stdout}");
    }
}

#[test]
fn command_one_vehicle_and_usage_errors() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = dir.path().join("catalog");
    let out = tpv(&[
        "catalog",
        "match",
        s(&fixture("pack")),
        s(&fixture("paintjob-packer")),
        "--commit",
        "abc",
        "--vehicle",
        "iveco.sway",
        "--catalog",
        s(&catalog),
    ]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let written: Vec<String> = std::fs::read_dir(catalog.join("ets2"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(written, ["iveco.sway.json"]);
    for args in [
        vec!["catalog", "match", s(&fixture("pack"))],
        vec![
            "catalog",
            "match",
            s(&fixture("pack")),
            s(&fixture("paintjob-packer")),
        ],
        vec![
            "catalog",
            "match",
            s(&fixture("pack")),
            s(&fixture("paintjob-packer")),
            "--commit",
            "abc",
            "--colour",
            "red",
        ],
    ] {
        let out = tpv(&args);
        assert!(!out.status.success(), "{args:?}");
        assert!(text(&out.stderr).starts_with("error: "), "{args:?}");
    }
}
