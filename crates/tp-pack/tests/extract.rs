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
        ("missing-dimension", "/electric/shared_a4cfe8ff.tobj"),
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
        write(
            &source,
            "def/vehicle/truck/example.good/accessory/mirror/other.sii",
            "SiiNunit\n{\naccessory_addon_data : other.example.good.mirror {\nmodel: \"/vehicle/shared/example.good.pmd\"\n}\n}\n",
        );
        write(
            &source,
            "def/vehicle/truck/example.good/paint_job/accessory/fill.sii",
            "SiiNunit\n{\nsimple_paint_job_data : .fill {\nacc_list[]: \"mirror.other\"\npaint_job_mask: \"/shared_00000000.tobj\"\n}\n}\n",
        );
        let result = run(root, Some("example.good"));
        assert!(!result.status.success());
        let batch = batches(root).pop().unwrap();
        let report: Value =
            serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
        assert_eq!(report["results"][0]["stage"], "coverage");
        assert_eq!(
            report["results"][0]["dimension_diagnostics"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            report["results"][0]["dimension_diagnostics"][1]["reference"],
            "mirror.other"
        );
        let problem = &report["results"][0]["dimension_diagnostics"][0];
        assert_eq!(problem["reference"], "mirror.high");
        assert_eq!(problem["disposition"], "blocking");
        assert_eq!(problem["masks"][0]["path"], mask);
        assert_eq!(
            problem["masks"][0]["referenced_by"][0]["source"],
            "version.scs"
        );
        if name == "ambiguous-dimension" {
            assert_eq!(problem["reason"], "conflicting_crop_dimensions");
            assert_eq!(
                problem["candidates"],
                serde_json::json!([[128, 64], [256, 64]])
            );
            assert_eq!(problem["masks"].as_array().unwrap().len(), 2);
        } else {
            assert_eq!(problem["reason"], "missing_crop_dimensions");
            assert_eq!(problem["candidates"], serde_json::json!([]));
            assert!(problem["masks"][0]["crop_size"].is_null());
        }
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

#[test]
fn missing_dependencies_are_reported_without_guessing_a_dlc() {
    let orphan_fixture = fixture();
    let root = orphan_fixture.path();
    write(
        &root.join("game/version.scs.files"),
        "def/vehicle/truck/example.good/paint_job/accessory/orphan.sii",
        "SiiNunit\n{\nsimple_paint_job_data : .orphan {\nacc_list[]: \"sideskirt.missing\"\nacc_list[]: \"mirror.missing\"\npaint_job_mask: \"/mask_size_256x64\"\n}\n}\n",
    );
    assert!(run(root, Some("example.good")).status.success());
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    let missing = report["results"][0]["missing_dependencies"]
        .as_array()
        .unwrap();
    assert_eq!(missing.len(), 2);
    assert_eq!(missing[0]["kind"], "accessory_definition");
    assert_eq!(missing[0]["reference"], "mirror.missing");
    assert_eq!(missing[0]["referenced_by"][0]["source"], "version.scs");
    assert!(missing.iter().all(|d| d["expected_dlc"].is_null()
        && d["dlc_origin"] == "unknown"
        && d["disposition"] == "ignored_reference"));
    assert_eq!(report["results"][0]["status"], "success");
    assert_eq!(std::fs::read_dir(&batch).unwrap().count(), 2);
    let clean_fixture = fixture();
    assert!(
        run(clean_fixture.path(), Some("example.good"))
            .status
            .success()
    );
    let filename = report["results"][0]["package"].as_str().unwrap();
    assert_eq!(
        std::fs::read(batch.join(filename)).unwrap(),
        std::fs::read(batches(clean_fixture.path()).pop().unwrap().join(filename)).unwrap()
    );

    write(
        &root.join("game/version.scs.files"),
        "def/vehicle/truck/example.good/accessory/mirror/missing.sii",
        "SiiNunit\n{\naccessory_addon_data : missing.example.good.mirror {\n}\n}\n",
    );
    assert!(!run(root, Some("example.good")).status.success());
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    let missing = &report["results"][0]["missing_dependencies"][0];
    assert_eq!(missing["reference"], "mirror.missing");
    assert_eq!(missing["kind"], "painted_exterior_surface");
    assert_eq!(missing["disposition"], "blocking");
    assert_eq!(std::fs::read_dir(&batch).unwrap().count(), 1);

    let fixture = fixture();
    let root = fixture.path();
    std::fs::remove_file(root.join("game/version.scs.files/vehicle/shared/example.good.pmd"))
        .unwrap();
    assert!(!run(root, Some("example.good")).status.success());
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    assert_eq!(
        report["results"][0]["missing_dependencies"][0]["kind"],
        "game_file"
    );
    assert_eq!(
        report["results"][0]["missing_dependencies"][0]["reference"],
        "/vehicle/shared/example.good.pmd"
    );
    assert_eq!(
        report["results"][0]["missing_dependencies"][0]["disposition"],
        "blocking"
    );
}

#[test]
fn cabin_uv_mode_drives_accessories_and_package_metadata() {
    for alternate in [false, true] {
        let fixture = fixture();
        let root = fixture.path();
        let source = root.join("game/version.scs.files");
        let pim = PIM.replace("Triangles {", "Stream {\nTag: \"_UV7\"\nAliases: \"_TEXCOORD2\"\n0 ( 0.25 0.25 )\n1 ( 0.75 0.25 )\n2 ( 0.25 0.75 )\n}\nTriangles {");
        let cabin_shader = if alternate {
            "eut2.truckpaint.tsnmapuv.altuv"
        } else {
            "eut2.truckpaint"
        };
        let accessory_shader = if alternate {
            "eut2.truckpaint.tsnmap"
        } else {
            "eut2.truckpaint.tsnmap.airbrush.altuv"
        };
        for (model, shader) in [("example.good", cabin_shader), ("mirror", accessory_shader)] {
            write(&source, &format!("vehicle/shared/{model}.pmd"), "binary");
            write(&source, &format!("vehicle/shared/{model}.pim"), &pim);
            write(
                &source,
                &format!("vehicle/shared/{model}.pit"),
                &PIT.replace("eut2.truckpaint.tsnmapuv", shader),
            );
        }
        write(
            &source,
            "def/vehicle/truck/example.good/common.sui",
            "detail_model: \"/vehicle/shared/example.good.pmd\"\nvariant_uk: default\n",
        );
        write(
            &source,
            "def/vehicle/truck/example.good/accessory/mirror/paint.sii",
            "SiiNunit\n{\naccessory_addon_data : paint.example.good.mirror {\nmodel: \"/vehicle/shared/mirror.pmd\"\n}\n}\n",
        );
        write(
            &source,
            "def/vehicle/truck/example.good/paint_job/accessory/mirror.sii",
            "SiiNunit\n{\nsimple_paint_job_data : .paint {\nacc_list[]: \"mirror.paint\"\npaint_job_mask: \"/mask_size_256x64\"\n}\n}\n",
        );
        let result = run(root, Some("example.good"));
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let batch = batches(root).pop().unwrap();
        let report: Value =
            serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
        let result = &report["results"][0];
        let bytes = std::fs::read(batch.join(result["package"].as_str().unwrap())).unwrap();
        let package = tp_vehicles::Package::read(&bytes).unwrap();
        assert_eq!(package.manifest.game.alt_uv, alternate);
        assert_eq!(package.manifest.paint_job.main.len(), 1);
        assert_eq!(package.manifest.paint_job.accessories.len(), 1);
        assert_eq!(
            package.manifest.paint_job.accessories[0].texture.size,
            [256, 64].into()
        );
        for texture in result["textures"].as_array().unwrap() {
            assert_eq!(texture["edges"], 3);
            for contributor in texture["contributors"].as_array().unwrap() {
                for selection in contributor["selections"].as_array().unwrap() {
                    for surface in selection["painted_surfaces"].as_array().unwrap() {
                        assert_eq!(
                            surface["uv_alias"],
                            if alternate {
                                "_TEXCOORD2"
                            } else {
                                "_TEXCOORD1"
                            }
                        );
                    }
                }
            }
        }
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut svg = String::new();
        std::io::Read::read_to_string(
            &mut archive.by_name("templates/mirror_paint.svg").unwrap(),
            &mut svg,
        )
        .unwrap();
        assert!(svg.contains(if alternate {
            "M64.000,16.000"
        } else {
            "M0.000,0.000"
        }));
        write(
            &source,
            "def/vehicle/truck/example.good/cabin/conflict.sii",
            "SiiNunit\n{\naccessory_cabin_data : conflict.example.good.cabin {\nname: \"Conflicting\"\nmodel: \"/vehicle/shared/mirror.pmd\"\n}\n}\n",
        );
        assert!(!run(root, Some("example.good")).status.success());
        let batch = batches(root).pop().unwrap();
        let report: Value =
            serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
        assert!(
            report["results"][0]["error"]
                .as_str()
                .unwrap()
                .contains("conflicting cabin paint UV modes")
        );
        assert_eq!(std::fs::read_dir(batch).unwrap().count(), 1);
    }
}

#[test]
fn metallic_shaders_and_finite_uv_overflow_produce_traceable_packages() {
    let fixture = fixture();
    let root = fixture.path();
    let source = root.join("game/version.scs.files");
    write(
        &source,
        "vehicle/shared/example.good.pim",
        &PIM.replace("0 ( 0 0 )", "0 ( -2.125 1.75 )"),
    );
    write(
        &source,
        "vehicle/shared/example.good.pit",
        &PIT.replace(
            "truckpaint.tsnmapuv",
            "truckpaint.tsnmapuv.airbrush.flipflake",
        ),
    );
    let result = run(root, Some("example.good"));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    let vehicle = &report["results"][0];
    let surface =
        &vehicle["textures"][0]["contributors"][0]["selections"][0]["painted_surfaces"][0];
    assert_eq!(surface["uv_alias"], "_TEXCOORD1");
    assert_eq!(surface["out_of_bounds_vertices"], 1);
    assert_eq!(
        surface["uv_bounds"],
        serde_json::json!({"min":[-2.125,0.0],"max":[1.0,1.75]})
    );
    let package = std::fs::read(batch.join(vehicle["package"].as_str().unwrap())).unwrap();
    tp_pack::check(&package).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(package)).unwrap();
    let manifest: Value = serde_json::from_reader(zip.by_name("vehicle.json").unwrap()).unwrap();
    assert_eq!(manifest["game"]["alt_uv"], false);
    let mut svg = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("templates/high.svg").unwrap(), &mut svg)
        .unwrap();
    assert!(svg.contains("-8704.000,7168.000"));
}

#[test]
fn solid_colour_override_does_not_invent_a_paint_job_uv_mapping() {
    let fixture = fixture();
    let root = fixture.path();
    let source = root.join("game/version.scs.files");
    for (extension, text) in [
        ("pmd", "binary"),
        ("pim", PIM),
        (
            "pit",
            &PIT.replace(
                "eut2.truckpaint.tsnmapuv",
                "eut2.dif.spec.weight.add.env.tsnmapuv.paint.a",
            ),
        ),
    ] {
        write(&source, &format!("vehicle/shared/solid.{extension}"), text);
    }
    write(
        &source,
        "def/vehicle/truck/example.good/accessory/guard/painted.sii",
        "SiiNunit\n{\naccessory_addon_data : painted.example.good.guard {\nexterior_model: \"/vehicle/shared/solid.pmd\"\n}\n}\n",
    );
    write(
        &source,
        "def/vehicle/truck/example.good/paint_job/accessory/guard.sii",
        "SiiNunit\n{\nsimple_paint_job_data : .guard {\nacc_list[]: \"guard.painted\"\npaint_job_mask: \"/mask_size_128x64.tobj\"\n}\n}\n",
    );
    let result = run(root, Some("example.good"));
    assert!(!result.status.success());
    let batch = batches(root).pop().unwrap();
    let report: Value =
        serde_json::from_slice(&std::fs::read(batch.join("report.json")).unwrap()).unwrap();
    let vehicle = &report["results"][0];
    assert_eq!(vehicle["stage"], "coverage");
    assert_eq!(
        vehicle["missing_dependencies"][0]["reference"],
        "guard.painted"
    );
    assert_eq!(
        vehicle["missing_dependencies"][0]["kind"],
        "painted_exterior_surface"
    );
    assert_eq!(
        vehicle["missing_dependencies"][0]["disposition"],
        "blocking"
    );
    assert_eq!(std::fs::read_dir(batch).unwrap().count(), 1);
}
