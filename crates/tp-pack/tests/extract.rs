//! No game required: synthetic archives and a converter stand-in.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

const CONVERTER: &str = r#"#!/bin/sh
set -eu
mount=''
export=''
action=''
path=''
while [ "$#" -gt 0 ]; do
    case "$1" in
        -b) mount="$2"; shift 2 ;;
        -e) export="$2"; shift 2 ;;
        *) action="$1"; path="$2"; shift 2 ;;
    esac
done
root="$mount.files"
case "$action" in
    --list-directory)
        if [ ! -d "$root$path" ]; then
            echo '<error> [system] : readDir returned null!'
            exit 1
        fi
        for file in "$root${path%/}"/*; do
            [ -e "$file" ] || continue
            relative="${file#"$root"}"
            if [ -d "$file" ]; then echo "[D] $relative"; else echo "[F] $relative"; fi
        done
        ;;
    --extract-directory|--extract-file)
        mkdir -p "$(dirname "$export$path")"
        cp -R "$root$path" "$export$path"
        ;;
    -m)
        case "$path" in
            *broken*) echo '[err] model conversion failed'; exit 0 ;;
        esac
        mkdir -p "$(dirname "$export$path")"
        cp "$root$path.pim" "$export$path.pim"
        cp "$root$path.pit" "$export$path.pit"
        ;;
    *) echo '[err] unknown action'; exit 1 ;;
esac
"#;

const PIM: &str = "Material {\nAlias: \"paint\"\n}\nPiece {\nIndex: 0\nMaterial: 0\nVertexCount: 3\nTriangleCount: 1\nStream {\nAliases: \"_TEXCOORD1\"\n0 ( 0 0 )\n1 ( 1 0 )\n2 ( 0 1 )\n}\nTriangles {\n0 ( 0 1 2 )\n}\n}\nPart {\nName: \"body\"\nPieces: 0\n}\n";
const PIT: &str = "Look {\nName: \"default\"\nMaterial {\nAlias: \"paint\"\nEffect: \"eut2.truckpaint.tsnmapuv\"\n}\n}\nVariant {\nName: \"default\"\nPart {\nName: \"body\"\nAttribute {\nTag: \"visible\"\nValue: ( 1 )\n}\n}\n}\n";

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "converter", CONVERTER);
    std::fs::set_permissions(
        root.join("converter"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    for archive in ["base.scs", "def.scs", "version.scs"] {
        write(root, &format!("game/{archive}"), "fake archive");
    }
    // The stand-in reads the last mounted archive; mirror a complete mounted view here.
    let archive = root.join("game/version.scs.files");
    write(
        &archive,
        "version.sii",
        "SiiNunit\n{\nfs_pack_set : .version {\napplication: eut2\nversion: \"1.61.1.1\"\n}\n}\n",
    );
    for vehicle in ["example.good", "example.broken"] {
        write(
            &archive,
            &format!("def/vehicle/truck/{vehicle}/paint_job/accessory/flake.sii"),
            "SiiNunit\n{\nsimple_paint_job_data : .flake {\nacc_list[]: \"obsolete.accessory\"\nflake_uvscale: 2\n}\n}\n",
        );
        write(
            &archive,
            &format!("def/vehicle/truck/{vehicle}/data.sii"),
            &format!(
                "SiiNunit\n{{\naccessory_truck_data : vehicle.{vehicle} {{\ninfo[]: \"Example\"\ninfo[]: \"Truck\"\n}}\n}}\n"
            ),
        );
        write(
            &archive,
            &format!("def/vehicle/truck/{vehicle}/cabin/high.sii"),
            &format!(
                "SiiNunit\n{{\naccessory_cabin_data : high.{vehicle}.cabin {{\nname: \"High\"\n@include \"../common.sui\"\n}}\n}}\n"
            ),
        );
        write(
            &archive,
            &format!("def/vehicle/truck/{vehicle}/common.sui"),
            &format!("model: \"/vehicle/shared/{vehicle}.pmd\"\n"),
        );
        for (extension, content) in [("pmd", "binary"), ("pim", PIM), ("pit", PIT)] {
            write(
                &archive,
                &format!("vehicle/shared/{vehicle}.{extension}"),
                content,
            );
        }
    }
    temp
}

fn run(root: &Path, vehicle: Option<&str>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tpv"));
    command
        .current_dir(root)
        .arg("extract")
        .arg(root.join("game"))
        .arg("--converter")
        .arg(root.join("converter"))
        .args(["--package-version", "2.0.0"]);
    if let Some(vehicle) = vehicle {
        command.args(["--vehicle", vehicle]);
    }
    command.output().unwrap()
}

fn batches(root: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = std::fs::read_dir(root.join("dist"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

#[test]
fn isolated_failures_reports_and_unique_complete_outputs() {
    let fixture = fixture();
    let root = fixture.path();
    let result = run(root, None);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("incomplete batch"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let first = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(first.join("report.json")).unwrap()).unwrap();
    assert_eq!(report["failed"], 1, "{report:#}");
    let results = report["results"].as_array().unwrap();
    let good = results
        .iter()
        .find(|r| r["vehicle"] == "example.good")
        .unwrap();
    assert_eq!(good["status"], "success");
    assert_eq!(good["technical_validation"], "passed");
    assert_eq!(good["in_game_validation"], "required");
    assert!(
        good["sources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["path"].as_str().unwrap().ends_with("common.sui"))
    );
    let package = first.join(good["package"].as_str().unwrap());
    assert_eq!(
        tp_pack::check(&std::fs::read(&package).unwrap())
            .unwrap()
            .id,
        "official.ets2.example.good"
    );
    assert_eq!(std::fs::read_dir(&first).unwrap().count(), 2);
    let result = run(root, Some("example.good"));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(batches(root).len(), 2);
    assert!(package.is_file());
    let other = batches(root).into_iter().find(|p| p != &first).unwrap();
    assert_eq!(
        std::fs::read(package).unwrap(),
        std::fs::read(other.join("official.ets2.example.good-2.0.0.tpv")).unwrap()
    );
}

#[test]
fn invalid_version_and_excluded_vehicle_publish_nothing() {
    let fixture = fixture();
    let root = fixture.path();
    let result = Command::new(env!("CARGO_BIN_EXE_tpv"))
        .current_dir(root)
        .arg("extract")
        .arg(root.join("game"))
        .arg("--converter")
        .arg(root.join("converter"))
        .args(["--package-version", "not-a-version"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!root.join("dist").exists());
    let result = run(root, Some("trailer.example"));
    assert!(!result.status.success());
    assert!(!root.join("dist").exists());
    write(
        &root.join("game/version.scs.files"),
        "version.sii",
        "SiiNunit\n{\nfs_pack_set : .version {\napplication: ats\nversion: \"1.61.1.1\"\n}\n}\n",
    );
    let result = run(root, None);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("only ETS2"));
    assert!(!root.join("dist").exists());
}

#[test]
fn ambiguous_masks_and_missing_model_outputs_refuse_whole_vehicle() {
    for (name, mask) in [
        ("missing-dimension", "/mask"),
        ("ambiguous-dimension", "/mask_size_128x64"),
    ] {
        let fixture = fixture();
        let root = fixture.path();
        let source = root.join("game/version.scs.files");
        write(
            &source,
            "def/vehicle/truck/example.good/accessory/mirror/high.sii",
            "SiiNunit\n{\naccessory_addon_data : high.example.good.mirror {\nmodel: \"/vehicle/shared/example.good.pmd\"\n}\n}\n",
        );
        write(
            &source,
            "def/vehicle/truck/example.good/paint_job/accessory/first.sii",
            &format!(
                "SiiNunit\n{{\nsimple_paint_job_data : .first {{\nacc_list[]: \"mirror.high\"\npaint_job_mask: \"{mask}\"\n}}\n}}\n"
            ),
        );
        if name == "ambiguous-dimension" {
            write(
                &source,
                "def/vehicle/truck/example.good/paint_job/accessory/second.sii",
                "SiiNunit\n{\nsimple_paint_job_data : .second {\nacc_list[]: \"mirror.high\"\npaint_job_mask: \"/mask_size_256x64\"\n}\n}\n",
            );
        }
        let result = run(root, Some("example.good"));
        assert!(!result.status.success());
        let batch = batches(root).pop().unwrap();
        let report: Value =
            serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
        assert_eq!(report["results"][0]["stage"], "coverage");
        assert_eq!(std::fs::read_dir(batch).unwrap().count(), 1);
    }
    let fixture = fixture();
    let root = fixture.path();
    let script = CONVERTER.replace("cp \"$root$path.pit\" \"$export$path.pit\"", ":");
    write(root, "converter", &script);
    assert!(!run(root, Some("example.good")).status.success());
    assert_eq!(
        std::fs::read_dir(batches(root).pop().unwrap())
            .unwrap()
            .count(),
        1
    );
}
