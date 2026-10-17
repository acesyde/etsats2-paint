//! Manual installation and app workflows for generated game packages.

mod common;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::kittest::Queryable;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::state::Modal;

#[test]
fn no_uninstalled_official_vehicle_is_offered() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = common::harness();
    h.state_mut().vehicles = tp_app::vehicles::VehicleLibrary::open(dir.path());
    h.get_by_label("New Project").click();
    h.run();
    assert!(h.query_by_label("DAF NGD").is_none());
    assert!(h.query_by_label("Krone Cool Liner").is_none());
    assert!(h.query_by_label("Install…").is_some());
    assert!(
        h.query_by_label(&tp_i18n::tr("vehicles-install-sample"))
            .is_some()
    );
    assert!(h.query_by_label("Custom Vehicle…").is_some());
}

#[test]
#[ignore = "needs a freshly generated MAN package: TRUCKPAINT_MAN_PACKAGE"]
fn man_manual_install_project_reload_and_mod_export() {
    check_generated_package("TRUCKPAINT_MAN_PACKAGE", "man.tgx_2020");
}

#[test]
#[ignore = "needs a freshly generated DAF package: TRUCKPAINT_DAF_PACKAGE"]
fn daf_manual_install_project_reload_and_mod_export() {
    check_generated_package("TRUCKPAINT_DAF_PACKAGE", "daf.2021");
}

fn check_generated_package(variable: &str, vehicle: &str) {
    use std::io::Read;
    use std::path::PathBuf;
    use tp_app::mod_export::{self, Entry};

    let path =
        PathBuf::from(std::env::var_os(variable).unwrap_or_else(|| panic!("set {variable}")));
    let package = tp_vehicles::Package::read(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(package.manifest.id, format!("official.ets2.{vehicle}"));
    let temp = tempfile::tempdir().unwrap();
    let mut h = common::harness();
    h.state_mut().vehicles = tp_app::vehicles::VehicleLibrary::open(&temp.path().join("library"));
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        packages: [vec![path]].into(),
        ..Default::default()
    });
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    h.get_by_label("Install…").click();
    h.run();
    assert!(h.state().vehicles.get(&package.manifest.id).is_some());
    h.state_mut().modal = None;
    h.run();
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::CheckBox, &package.manifest.name)
        .click();
    common::settle_renders(&mut h);
    for cabin in package.manifest.paint_job.main.iter().skip(1) {
        h.get_by_role_and_label(Role::CheckBox, &cabin.name).click();
        common::settle_renders(&mut h);
    }
    h.get_by_label("Create Project").click();
    common::settle_renders(&mut h);
    let project = &h.state().workspace().unwrap().project;
    assert_eq!(
        project.surfaces.len(),
        package.manifest.paint_job.parts().count()
    );
    let bytes = tp_file::to_bytes(project).unwrap();
    let reopened = tp_file::from_bytes(&bytes).unwrap().project;
    assert_eq!(reopened.surfaces.len(), project.surfaces.len());
    let plan = mod_export::plan(&reopened).unwrap();
    std::fs::write(temp.path().join("probe.truckpaint"), bytes).unwrap();
    common::open_project(&mut h, reopened);
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    h.run_steps(8);
    let output = temp.path().join("probe.scs");
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        mod_save: [output.clone()].into(),
        ..Default::default()
    });
    h.get_by_label("Change…").click();
    h.run_steps(8);
    common::last(&h, "Export").click();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while std::time::Instant::now() < deadline {
        h.step();
        if (h.state().modal.is_none()
            || matches!(&h.state().modal, Some(Modal::ExportMod(d)) if d.job.is_none()))
            && output.is_file()
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&output).unwrap()).unwrap();
    for (path, entry) in &plan.files {
        let mut bytes = Vec::new();
        archive
            .by_name(path)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        match entry {
            Entry::Text(text) => assert_eq!(bytes, text.as_bytes()),
            Entry::Bytes(expected) => assert_eq!(&bytes, expected),
            Entry::Texture { width, height, .. } => {
                assert_eq!(&bytes[..4], b"DDS ");
                assert_eq!(
                    u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
                    *height
                );
                assert_eq!(
                    u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
                    *width
                );
            }
            _ => {}
        }
    }
}

/// A package shaped like `tpv extract` output for an owned trailer: a single
/// `trailer` main texture without game ids, and one accessory texture shared
/// by two bodies the game paints from the same region.
#[test]
fn extracted_trailer_installs_and_exports_its_grouped_bodies() {
    use tp_app::mod_export;
    use tp_vehicles::TextureSize;
    use tp_vehicles::sample::{self, SampleTexture};
    let main = SampleTexture {
        id: "trailer",
        name: "Trailer",
        size: TextureSize::new(4096, 4096),
        layout: 1,
    };
    let bodies = SampleTexture {
        id: "body_dry_van",
        name: "body.dry_van, body.ins",
        size: TextureSize::new(512, 256),
        layout: 1,
    };
    let mut manifest = sample::manifest_with(
        "official.ets2.scs.box",
        "SCS Box",
        "2.0.0",
        "trailer",
        std::slice::from_ref(&main),
        std::slice::from_ref(&bodies),
    );
    manifest["game"]["path"] = "scs.box".into();
    manifest["paint_job"]["accessories"][0]["game_ids"] =
        serde_json::json!(["body.dry_van", "body.ins"]);
    let bytes = sample::zip(&manifest, &sample::templates(&[main, bodies]));
    let dir = tempfile::tempdir().unwrap();
    let mut library = tp_app::vehicles::VehicleLibrary::open(dir.path());
    library.install_bytes(&bytes).unwrap();
    let package = library
        .load("official.ets2.scs.box", &"2.0.0".parse().unwrap())
        .unwrap();
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let mut project =
        tp_app::vehicle_project::fleet_project("ACE", &[(&package, &textures)]).unwrap();
    assert_eq!(project.surfaces.len(), 2);
    project.mod_settings.internal_name = Some("ace".into());
    let plan = mod_export::plan(&project).unwrap();
    let def = "def/vehicle/trailer_owned/scs.box/paint_job";
    let unit = plan.text(&format!("{def}/ace.sii")).unwrap();
    assert!(
        unit.contains("accessory_paint_job_data : ace.scs.box.paint_job"),
        "{unit}"
    );
    assert!(!unit.contains("suitable_for"), "{unit}");
    let overrides = plan.text(&format!("{def}/accessory/ace.sii")).unwrap();
    assert_eq!(
        overrides,
        "SiiNunit\n{\n\nsimple_paint_job_data : .ovr0\n{\n\
         \tpaint_job_mask: \"/vehicle/trailer_owned/upgrade/paintjob/ace/scs.box/body_dry_van.tobj\"\n\
         \tacc_list[]: \"body.dry_van\"\n\tacc_list[]: \"body.ins\"\n}\n}\n"
    );
}
