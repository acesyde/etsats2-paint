//! Headless tests for vehicles: the library dialog, the New Project vehicle
//! step, texture tabs, the template overlay, the Vehicle panel, Update
//! Template and export of vehicle textures.

mod common;

use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::layout::PanelKind;
use tp_app::prefs::Prefs;
use tp_app::state::Modal;
use tp_app::vehicles::VehicleLibrary;
use tp_app::workspace::Workspace;
use tp_core::TemplateStatus;
use tp_core::document::{Frame, ShapeKind};
use tp_core::kurbo::{Point, Size};
use tp_vehicles::sample::{self, SampleTexture};

type H = Harness<'static, AppState>;

const ID: &str = "scs.sample.truck";

fn tex(id: &'static str, name: &'static str, size: u32, layout: u32) -> SampleTexture {
    SampleTexture {
        id,
        name,
        size,
        layout,
    }
}

/// Writes a package file; returns its path.
fn package_file(dir: &Path, version: &str, textures: &[SampleTexture]) -> PathBuf {
    let path = dir.join(format!("truck-{version}.tpv"));
    std::fs::write(
        &path,
        sample::package(ID, "Sample Truck", version, textures),
    )
    .unwrap();
    path
}

/// The app with a library in `dir/library` and every panel open.
fn app(dir: &Path) -> H {
    let mut prefs = Prefs::default();
    for slot in &mut prefs.layout.panels {
        slot.open = true;
        slot.collapsed = !matches!(
            slot.kind,
            PanelKind::Vehicle | PanelKind::Layers | PanelKind::Properties
        );
    }
    let mut state = AppState::with_prefs(prefs, None);
    state.vehicles = VehicleLibrary::open(&dir.join("library"));
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, 2000.0))
        .with_step_dt(1.0 / 4.0)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    h.run();
    h
}

fn script_packages(h: &mut H, files: &[PathBuf]) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        packages: [files.to_vec()].into(),
        ..Default::default()
    });
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

/// Creates a project for the installed sample truck through the wizard.
fn create_vehicle_project(h: &mut H) {
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::RadioButton, "Sample Truck")
        .click();
    h.run();
    h.get_by_label("Next").click();
    h.run();
    h.get_by_label("Create").click();
    h.run();
}

fn install_v120(h: &mut H, dir: &Path) {
    let file = package_file(dir, "1.2.0", &sample::truck_textures());
    h.state_mut().vehicles.install_file(&file).unwrap();
    h.run();
}

#[test]
fn install_from_the_library_dialog_and_two_versions() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    let v12 = package_file(dir.path(), "1.2.0", &sample::truck_textures());
    let v13 = package_file(dir.path(), "1.3.0", &sample::truck_textures());
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    assert!(h.query_by_label("No vehicle installed").is_some());
    script_packages(&mut h, &[v12, v13]);
    h.get_by_label("Install…").click();
    h.run();
    assert!(
        h.query_by_label_contains("Installed Sample Truck 1.3.0")
            .is_some()
    );
    let v = h.state().vehicles.get(ID).unwrap();
    assert_eq!(v.versions.len(), 2);
    assert_eq!(v.newest().manifest.version.to_string(), "1.3.0");
    // Persisted across sessions.
    assert_eq!(
        VehicleLibrary::open(&dir.path().join("library"))
            .get(ID)
            .unwrap()
            .versions
            .len(),
        2
    );
}

#[test]
fn invalid_package_shows_why() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    let bad = dir.path().join("bad.tpv");
    std::fs::write(&bad, b"not a package").unwrap();
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    script_packages(&mut h, &[bad]);
    h.get_by_label("Install…").click();
    h.run();
    assert!(
        h.query_by_label_contains("bad.tpv could not be installed: it is not a vehicle package.")
            .is_some()
    );
    assert!(h.state().vehicles.vehicles().is_empty());
}

#[test]
fn creating_a_project_for_a_vehicle_and_back() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::RadioButton, "Sample Truck")
        .click();
    h.run();
    h.get_by_label("Next").click();
    h.run();
    assert!(
        h.query_by_label_contains("Cabin · 4096 × 4096 px")
            .is_some()
    );
    h.get_by_label("Back").click();
    h.run();
    assert!(
        h.get_by_role_and_label(Role::RadioButton, "Sample Truck")
            .accesskit_node()
            .toggled()
            .is_some_and(|t| t == egui::accesskit::Toggled::True),
        "the previous choice is kept"
    );
    h.get_by_label("Next").click();
    h.run();
    h.get_by_label("Create").click();
    h.run();
    let p = &ws(&h).project;
    assert_eq!(p.name, "Sample Truck");
    let names: Vec<&str> = p.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Cabin", "Chassis", "Accessories"]);
    assert_eq!(p.vehicle.as_ref().unwrap().version, "1.2.0");
}

#[test]
fn tabs_switch_textures() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    h.get_by_label("Chassis").click();
    h.run();
    assert_eq!(ws(&h).project.active_surface, 1);
    // The tab and the status bar both name the active texture.
    assert!(h.query_all_by_label("Chassis").count() >= 2);
    let id = ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(500.0, 500.0), Size::new(100.0, 100.0), 0.0),
        1.0,
    );
    h.run();
    assert!(ws(&h).project.surfaces[1].get(id).is_some());
    assert!(ws(&h).project.surfaces[0].objects.is_empty());
}

#[test]
fn hide_the_template_with_shift_t() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    let visible = |h: &H| ws(h).project.surface().template.as_ref().unwrap().visible;
    assert!(visible(&h));
    h.key_press_modifiers(Modifiers::SHIFT, Key::T);
    h.run();
    assert!(!visible(&h));
    assert!(!ws(&h).history.can_undo(), "not an undo step");
    h.key_press_modifiers(Modifiers::SHIFT, Key::T);
    h.run();
    assert!(visible(&h));
}

#[test]
fn update_from_the_vehicle_panel_and_undo() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    assert!(h.query_by_label_contains("is available").is_none());
    let v13 = package_file(
        dir.path(),
        "1.3.0",
        &[
            tex("cabin", "Cabin", 4096, 2),
            tex("chassis", "Chassis", 2048, 1),
            tex("accessories", "Accessories", 1024, 1),
        ],
    );
    h.state_mut().vehicles.install_file(&v13).unwrap();
    h.run();
    assert!(h.query_by_label("Version 1.3.0 is available").is_some());
    h.get_by_label("Update Template…").click();
    h.run();
    assert!(h.query_by_label_contains("Cabin: layout changed").is_some());
    h.get_by_label("Update").click();
    h.run();
    let p = &ws(&h).project;
    assert_eq!(p.vehicle.as_ref().unwrap().version, "1.3.0");
    let status = |h: &H, i: usize| ws(h).project.surfaces[i].template.as_ref().unwrap().status;
    assert_eq!(status(&h, 0), TemplateStatus::LayoutChanged);
    assert_eq!(status(&h, 1), TemplateStatus::Current);
    assert!(h.query_by_label("Layout changed").is_some());
    h.get_by_label("Dismiss layout change of Cabin").click();
    h.run();
    assert_eq!(status(&h, 0), TemplateStatus::Current);
    ws_mut(&mut h).undo();
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.vehicle.as_ref().unwrap().version, "1.2.0");
}

#[test]
fn removing_a_version_keeps_open_projects_and_files_work_without_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    h.state_mut()
        .vehicles
        .remove(ID, &"1.2.0".parse().unwrap())
        .unwrap();
    h.run();
    assert!(ws(&h).project.surfaces.iter().all(|s| s.template.is_some()));
    assert!(h.query_by_label("Package 1.2.0 (not installed)").is_some());
    // Saved and reopened without the package: templates travel with it.
    let bytes = tp_file::to_bytes(&ws(&h).project).unwrap();
    let opened = tp_file::from_bytes(&bytes).unwrap().project;
    assert_eq!(opened.template_assets().count(), 3);
    assert_eq!(opened.vehicle.as_ref().unwrap().version, "1.2.0");
}

#[test]
fn exporting_a_smaller_texture_without_its_template() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    h.get_by_label("Accessories").click();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    for _ in 0..4 {
        h.step();
    }
    assert!(h.query_by_label_contains("1024 × 1024 px · PNG").is_some());
    let out = dir.path().join("out.png");
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        export: [out.clone()].into(),
        ..Default::default()
    });
    h.get_by_label("Export…").click();
    for _ in 0..3000 {
        h.step();
        if h.state().modal.is_none() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        h.state().dialogs_suggested(),
        vec!["Sample Truck - Accessories.png".to_owned()]
    );
    let img = image::open(&out).unwrap().to_rgba8();
    assert_eq!(img.dimensions(), (1024, 1024));
    // The template's dark lines are not in the export: all white.
    assert!(img.pixels().all(|p| p.0 == [255, 255, 255, 255]));
}

#[test]
fn blank_projects_are_unchanged_and_the_panel_says_so() {
    let mut h = common::harness();
    common::create_project(&mut h);
    let p = &h.state().workspace().unwrap().project;
    assert!(p.vehicle.is_none());
    assert_eq!(p.surfaces.len(), 1);
    h.state_mut()
        .prefs
        .layout
        .slot_mut(PanelKind::Vehicle)
        .collapsed = false;
    h.run();
    assert!(h.query_by_label("No vehicle").is_some());
}

fn looks_like_id(text: &str) -> bool {
    text.len() > 3
        && text.contains('-')
        && text.starts_with(|c: char| c.is_ascii_lowercase())
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn assert_no_raw_ids(h: &H, screen: &str) {
    use egui_kittest::kittest::By;
    let leaks: Vec<String> = h
        .query_all(By::new().include_labels().predicate(|node| {
            [node.label(), node.value()]
                .into_iter()
                .flatten()
                .any(|t| looks_like_id(&t))
        }))
        .map(|n| format!("{:?}", n.accesskit_node().label()))
        .collect();
    assert!(
        leaks.is_empty(),
        "{screen}: raw message ids shown: {leaks:#?}"
    );
}

#[test]
fn vehicle_screens_show_no_raw_message_id() {
    for language in tp_i18n::Language::ALL {
        let dir = tempfile::tempdir().unwrap();
        let mut h = app(dir.path());
        h.state_mut().prefs.set_language(Some(language));
        install_v120(&mut h, dir.path());
        h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
        h.run();
        assert_no_raw_ids(&h, "library");
        h.state_mut().modal = Some(Modal::NewProject(Default::default()));
        h.run();
        assert_no_raw_ids(&h, "wizard vehicle step");
        if let Some(Modal::NewProject(d)) = &mut h.state_mut().modal {
            d.vehicle = Some((ID.into(), "1.2.0".parse().unwrap(), "standard".into()));
            d.step = 1;
        }
        h.run();
        assert_no_raw_ids(&h, "wizard name step");
        h.state_mut().modal = None;
        let package = h
            .state()
            .vehicles
            .load(ID, &"1.2.0".parse().unwrap())
            .unwrap();
        let project = tp_app::vehicle_project::vehicle_project("T", &package, "standard").unwrap();
        h.state_mut().open_project(project);
        let v13 = package_file(
            dir.path(),
            "1.3.0",
            &[tex("cabin", "Cabin", 4096, 2), tex("new", "New", 1024, 1)],
        );
        h.state_mut().vehicles.install_file(&v13).unwrap();
        h.run();
        assert_no_raw_ids(&h, "vehicle workspace");
        h.state_mut().open_update_dialog();
        h.run();
        assert_no_raw_ids(&h, "update dialog");
    }
}
