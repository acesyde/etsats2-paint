//! Headless tests for vehicles: the library dialog, the New Project vehicle
//! step, the sidebar tree, the template overlay, Textures…, Update Template
//! and export of vehicle textures.

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
use tp_app::workspace::{SaveState, Workspace};
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

/// Writes a package file of a truck whose first texture is its single main
/// texture and the others accessories; returns its path.
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
        slot.collapsed = !matches!(slot.kind, PanelKind::Layers | PanelKind::Properties);
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

/// Lets a dialog that changed size settle (egui re-centers a modal over a
/// few frames) and background template renders finish.
fn settle(h: &mut H) {
    h.run_steps(8);
    for _ in 0..1000 {
        if h.try_run().is_ok() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("the UI keeps repainting");
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
    settle(h);
    h.get_by_label("Next").click();
    h.run();
    h.get_by_label("Create").click();
    h.run();
}

/// Opens the ⋯ menu of `vehicle` in the sidebar and clicks `item`.
fn vehicle_action(h: &mut H, vehicle: &str, item: &str) {
    h.get_by_label(&format!("Actions for {vehicle}")).click();
    h.run();
    h.get_by_label(item).click();
    h.run();
}

/// Clicks a texture of the sidebar tree.
fn pick_texture(h: &mut H, path: &str) {
    h.get_by_label(&format!("Texture {path}")).click();
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
    settle(&mut h);
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
    assert!(
        toggled(&h, Role::CheckBox, "Chassis"),
        "and its checked textures"
    );
    // A single main texture is always painted.
    let cabin = h.get_by_role_and_label(Role::CheckBox, "Cabin");
    assert!(cabin.accesskit_node().is_disabled());
    assert!(toggled(&h, Role::CheckBox, "Cabin"));
    h.get_by_label("Next").click();
    h.run();
    h.get_by_label("Create").click();
    h.run();
    let p = &ws(&h).project;
    assert_eq!(p.name, "Sample Truck");
    let names: Vec<&str> = p.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Cabin", "Chassis", "Accessories"]);
    assert_eq!(p.vehicles[0].version, "1.2.0");
}

#[test]
fn the_tree_and_shortcuts_switch_textures() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    pick_texture(&mut h, "Sample Truck › Chassis");
    assert_eq!(ws(&h).project.active_surface, 1);
    // The status bar names the active texture with its vehicle.
    assert!(h.query_by_label("Sample Truck › Chassis").is_some());
    let id = ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(500.0, 500.0), Size::new(100.0, 100.0), 0.0),
        1.0,
    );
    h.run();
    assert!(ws(&h).project.surfaces[1].get(id).is_some());
    assert!(ws(&h).project.surfaces[0].objects.is_empty());
    // Next and previous texture from the keyboard.
    h.key_press_modifiers(Modifiers::COMMAND, Key::PageDown);
    h.run();
    assert_eq!(ws(&h).project.active_surface, 2);
    h.key_press_modifiers(Modifiers::COMMAND, Key::PageDown);
    h.run();
    assert_eq!(ws(&h).project.active_surface, 0, "wraps around");
    h.key_press_modifiers(Modifiers::COMMAND, Key::PageUp);
    h.run();
    assert_eq!(ws(&h).project.active_surface, 2);
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
    // The same setting in the Properties panel, with nothing selected.
    h.get_by_role_and_label(Role::CheckBox, "Show Template")
        .click();
    h.run();
    assert!(!visible(&h));
    assert!(
        h.query_by_role_and_label(Role::Slider, "Template opacity")
            .is_some()
    );
}

#[test]
fn update_from_the_vehicle_panel_and_undo() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    assert!(
        h.query_by_label("Update the template of Sample Truck")
            .is_none()
    );
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
    h.get_by_label("Update the template of Sample Truck")
        .click();
    h.run();
    assert!(h.query_by_label_contains("Cabin: layout changed").is_some());
    h.get_by_label("Update").click();
    h.run();
    let p = &ws(&h).project;
    assert_eq!(p.vehicles[0].version, "1.3.0");
    let status = |h: &H, i: usize| ws(h).project.surfaces[i].template.as_ref().unwrap().status;
    assert_eq!(status(&h, 0), TemplateStatus::LayoutChanged);
    assert_eq!(status(&h, 1), TemplateStatus::Current);
    // The Properties panel (nothing selected) says so for the active Cabin.
    assert!(
        h.query_by_label_contains("The layout of this texture changed in version 1.3.0")
            .is_some()
    );
    h.get_by_label("Dismiss layout change of Sample Truck › Cabin")
        .click();
    h.run();
    assert_eq!(status(&h, 0), TemplateStatus::Current);
    ws_mut(&mut h).undo();
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.vehicles[0].version, "1.2.0");
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
    assert!(
        h.query_by_label("Truck · 1.2.0").is_some(),
        "recorded version"
    );
    // Saved and reopened without the package: templates travel with it.
    let bytes = tp_file::to_bytes(&ws(&h).project).unwrap();
    let opened = tp_file::from_bytes(&bytes).unwrap().project;
    assert_eq!(opened.template_assets().count(), 3);
    assert_eq!(opened.vehicles[0].version, "1.2.0");
}

/// Opens `path` through File › Open… (no project open).
fn open_file(h: &mut H, path: &Path) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        open: [path.to_path_buf()].into(),
        ..Default::default()
    });
    h.key_press_modifiers(Modifiers::COMMAND, Key::O);
    settle(h);
}

#[test]
fn older_files_get_the_game_data_of_their_installed_version() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    let recorded = ws(&h).project.clone();
    assert!(recorded.vehicles[0].game_data.is_some());
    // A file written before game data was recorded.
    let mut old = recorded.clone();
    old.vehicles[0].game_data = None;
    for s in &mut old.surfaces {
        let t = s.template.as_mut().unwrap();
        t.game_ids.clear();
        t.main_index = None;
    }
    let path = dir.path().join("old.truckpaint");
    tp_file::write(&old, &path).unwrap();
    h.state_mut().close_project();
    open_file(&mut h, &path);
    let p = &ws(&h).project;
    assert_eq!(p.vehicles, recorded.vehicles);
    let templates = |p: &tp_core::Project| {
        p.surfaces
            .iter()
            .map(|s| s.template.clone().map(|t| (t.game_ids, t.main_index)))
            .collect::<Vec<_>>()
    };
    assert_eq!(templates(p), templates(&recorded));
    assert_eq!(ws(&h).save_state(), SaveState::Unsaved);

    // Without the package version, it opens as it is, editable and saved.
    h.state_mut().close_project();
    h.state_mut()
        .vehicles
        .remove(ID, &"1.2.0".parse().unwrap())
        .unwrap();
    open_file(&mut h, &path);
    assert!(ws(&h).project.vehicles[0].game_data.is_none());
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
    ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0), 0.0),
        1.0,
    );
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
}

#[test]
fn exporting_a_smaller_texture_without_its_template() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    create_vehicle_project(&mut h);
    pick_texture(&mut h, "Sample Truck › Accessories");
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
        vec!["Sample Truck - Sample Truck - Accessories.png".to_owned()]
    );
    let img = image::open(&out).unwrap().to_rgba8();
    assert_eq!(img.dimensions(), (1024, 1024));
    // The template's dark lines are not in the export: all white.
    assert!(img.pixels().all(|p| p.0 == [255, 255, 255, 255]));
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
            d.vehicle = Some(tp_app::ui::vehicle_dialogs::VehicleChoice {
                id: ID.into(),
                version: "1.2.0".parse().unwrap(),
                textures: vec!["chassis".into()],
            });
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
        let project =
            tp_app::vehicle_project::fleet_project("T", &package, &["chassis".to_owned()]).unwrap();
        h.state_mut().open_project(project);
        let v13 = package_file(
            dir.path(),
            "1.3.0",
            &[tex("cabin", "Cabin", 4096, 2), tex("new", "New", 1024, 1)],
        );
        h.state_mut().vehicles.install_file(&v13).unwrap();
        h.run();
        assert_no_raw_ids(&h, "vehicle workspace");
        h.state_mut().open_update_dialog(None);
        h.run();
        assert_no_raw_ids(&h, "update dialog");
        h.state_mut().modal = Some(Modal::AddVehicle(Default::default()));
        h.run();
        assert_no_raw_ids(&h, "add vehicle dialog");
        let dialog = {
            let p = &ws(&h).project;
            tp_app::ui::vehicle_dialogs::TexturesDialog::new(p, &p.vehicles[0])
        };
        h.state_mut().modal = Some(Modal::Textures(dialog));
        h.run();
        assert_no_raw_ids(&h, "textures dialog");
    }
}

#[test]
fn install_the_sample_vehicles_from_new_project() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_label("Install the sample vehicles").click();
    settle(&mut h);
    assert!(
        h.query_by_label_contains("Installed TruckPaint Sample Truck 1.1.0")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("Installed TruckPaint Sample Trailer 1.0.0")
            .is_some()
    );
    // Installed, the truck selected, and the button is gone.
    assert!(h.query_by_label("Install the sample vehicles").is_none());
    let row = h.get_by_role_and_label(Role::RadioButton, "TruckPaint Sample Truck");
    assert_eq!(
        row.accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True),
        "the sample truck is selected"
    );
    h.get_by_label("Next").click();
    h.run();
    h.get_by_label("Create").click();
    h.run();
    let project = &ws(&h).project;
    assert_eq!(project.name, "TruckPaint Sample Truck");
    let names: Vec<&str> = project.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        ["Standard cab", "Chassis", "Cab accessories", "Side skirts"]
    );
}

#[test]
fn install_the_sample_vehicles_from_the_library() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    h.get_by_label("Install the sample vehicles").click();
    settle(&mut h);
    for id in [
        tp_app::vehicles::SAMPLE_ID,
        tp_app::vehicles::SAMPLE_TRAILER_ID,
    ] {
        assert!(h.state().vehicles.get(id).is_some(), "{id}");
    }
    assert!(h.query_by_label("No vehicle installed").is_none());
    assert!(h.query_by_label("Install the sample vehicles").is_none());
    // Each entry summarizes its paint job.
    assert!(
        h.query_by_label_contains("Standard cab, High roof · Accessories: 3")
            .is_some()
    );
    assert!(h.query_by_label_contains("Base · Accessories: 3").is_some());
}

#[test]
fn no_sample_button_once_a_vehicle_is_installed() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    install_v120(&mut h, dir.path());
    h.get_by_label("New Project").click();
    h.run();
    assert!(h.query_by_label("Install the sample vehicles").is_none());
    h.key_press(Key::Escape);
    h.run();
    h.state_mut().modal = Some(Modal::VehicleLibrary(Default::default()));
    h.run();
    assert!(h.query_by_label("Install the sample vehicles").is_none());
}

const SAMPLE_ID: &str = "community.truckpaint.sample_truck";

/// Writes a package of `game` and `kind` with a single 1024 px main
/// texture "Body"; returns its path.
fn other_package_file(dir: &Path, id: &str, name: &str, game: &str, kind: &str) -> PathBuf {
    let mut m = sample::manifest_with(
        id,
        name,
        "1.0.0",
        kind,
        &[tex("body", "Body", 1024, 1)],
        &[],
    );
    m["game"]["id"] = game.into();
    let path = dir.join(format!("{id}.tpv"));
    std::fs::write(
        &path,
        sample::zip(&m, &[("templates/body.png".into(), sample::png(16))]),
    )
    .unwrap();
    path
}

fn toggled(h: &H, role: Role, label: &str) -> bool {
    h.get_by_role_and_label(role, label)
        .accesskit_node()
        .toggled()
        .is_some_and(|t| t == egui::accesskit::Toggled::True)
}

/// Sets checkbox `label` to `on`.
fn set_checkbox(h: &mut H, label: &str, on: bool) {
    if toggled(h, Role::CheckBox, label) != on {
        h.get_by_role_and_label(Role::CheckBox, label).click();
        h.run();
    }
}

/// A project on the sample truck (1.1.0) through the wizard, with the main
/// textures `main` checked and the accessories `left_out` unchecked.
fn create_sample_project(h: &mut H, main: &[&str], left_out: &[&str]) {
    h.state_mut().vehicles.install_samples().unwrap();
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::RadioButton, "TruckPaint Sample Truck")
        .click();
    settle(h);
    assert!(
        toggled(h, Role::CheckBox, "Standard cab"),
        "first main texture checked"
    );
    // Check before unchecking: the last checked main texture is locked.
    for m in main {
        set_checkbox(h, m, true);
    }
    for m in ["Standard cab", "High roof"] {
        if !main.contains(&m) {
            set_checkbox(h, m, false);
        }
    }
    for a in left_out {
        set_checkbox(h, a, false);
    }
    h.get_by_label("Next").click();
    h.run();
    h.get_by_label("Create").click();
    h.run();
}

#[test]
fn two_main_textures_and_the_tree_selects_textures() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab", "High roof"], &[]);
    let p = &ws(&h).project;
    let names: Vec<&str> = p.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Standard cab",
            "High roof",
            "Chassis",
            "Cab accessories",
            "Side skirts"
        ]
    );
    pick_texture(&mut h, "TruckPaint Sample Truck › High roof");
    assert_eq!(ws(&h).project.active_surface, 1);
    assert!(
        h.query_by_label("TruckPaint Sample Truck › High roof")
            .is_some(),
        "status bar breadcrumb"
    );
    // The next texture follows the project order.
    h.key_press_modifiers(Modifiers::COMMAND, Key::PageDown);
    h.run();
    assert_eq!(ws(&h).project.active_surface, 2);
    assert!(
        h.query_by_label("TruckPaint Sample Truck › Chassis")
            .is_some()
    );
}

#[test]
fn next_needs_a_vehicle_and_the_last_main_texture_stays_checked() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.get_by_label("New Project").click();
    h.run();
    assert!(h.get_by_label("Next").accesskit_node().is_disabled());
    assert!(h.query_by_label("Install the sample vehicles").is_some());
    assert!(h.query_by_label("Blank texture").is_none());
    h.get_by_label("Install the sample vehicles").click();
    settle(&mut h);
    assert!(!h.get_by_label("Next").accesskit_node().is_disabled());
    // The only checked main texture can't be unchecked.
    let locked = |h: &H, label: &str| {
        h.get_by_role_and_label(Role::CheckBox, label)
            .accesskit_node()
            .is_disabled()
    };
    assert!(locked(&h, "Standard cab"));
    set_checkbox(&mut h, "High roof", true);
    assert!(!locked(&h, "Standard cab"));
    set_checkbox(&mut h, "Standard cab", false);
    assert!(locked(&h, "High roof"));
    assert!(!h.get_by_label("Next").accesskit_node().is_disabled());
    // Accessories can all be left out.
    for a in ["Chassis", "Cab accessories", "Side skirts"] {
        set_checkbox(&mut h, a, false);
    }
    h.get_by_label("Next").click();
    h.run();
    assert!(
        h.query_by_label_contains("High roof · 4096 × 4096 px")
            .is_some()
    );
    assert!(h.query_by_label_contains("Chassis ·").is_none());
}

#[test]
fn a_trailer_needs_no_choice_of_main_texture() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().vehicles.install_samples().unwrap();
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::RadioButton, "TruckPaint Sample Trailer")
        .click();
    settle(&mut h);
    let base = h.get_by_role_and_label(Role::CheckBox, "Base");
    assert!(base.accesskit_node().is_disabled());
    assert!(toggled(&h, Role::CheckBox, "Base"));
    assert!(toggled(&h, Role::CheckBox, "Mudflaps"));
    h.get_by_label("Next").click();
    h.run();
    h.get_by_label("Create").click();
    h.run();
    let names: Vec<&str> = ws(&h)
        .project
        .surfaces
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "Base",
            "Curtain body 13.6 m",
            "Curtain body 10.5 m",
            "Mudflaps"
        ]
    );
}

#[test]
fn add_vehicle_lists_only_the_project_game() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    for (id, name, game, kind) in [
        (
            "scs.krone.cool_liner",
            "Krone Cool Liner",
            "ets2",
            "trailer",
        ),
        ("scs.peterbilt.579", "Peterbilt 579", "ats", "truck"),
    ] {
        let file = other_package_file(dir.path(), id, name, game, kind);
        h.state_mut().vehicles.install_file(&file).unwrap();
    }
    create_sample_project(&mut h, &["Standard cab"], &[]);
    h.get_by_label("Add Vehicle…").click();
    h.run();
    assert!(
        h.query_by_role_and_label(Role::RadioButton, "Krone Cool Liner")
            .is_some()
    );
    assert!(
        h.query_by_role_and_label(Role::RadioButton, "Peterbilt 579")
            .is_none()
    );
    assert!(
        h.query_by_role_and_label(Role::RadioButton, "TruckPaint Sample Truck")
            .is_none(),
        "vehicles already in the project are hidden"
    );
    h.get_by_role_and_label(Role::RadioButton, "Krone Cool Liner")
        .click();
    settle(&mut h);
    h.get_by_label("Add").click();
    settle(&mut h);
    assert!(h.state().modal.is_none());
    let p = &ws(&h).project;
    assert_eq!(p.vehicles.len(), 2);
    assert_eq!(p.surfaces.len(), 5);
    assert_eq!(p.active_surface, 4);
    assert!(h.query_by_label("Vehicles · ETS2").is_some());
    // The trailer can be removed (no artwork: no question); the last
    // vehicle cannot.
    vehicle_action(&mut h, "Krone Cool Liner", "Remove from Project");
    assert_eq!(ws(&h).project.vehicles.len(), 1);
    h.get_by_label("Actions for TruckPaint Sample Truck")
        .click();
    h.run();
    assert!(
        h.get_by_label("Remove from Project")
            .accesskit_node()
            .is_disabled()
    );
    h.key_press(Key::Escape);
    h.run();
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.vehicles.len(), 2);
}

#[test]
fn the_tree_groups_main_textures_and_accessories() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab", "High roof"], &[]);
    h.get_by_label("Add Vehicle…").click();
    h.run();
    h.get_by_role_and_label(Role::RadioButton, "TruckPaint Sample Trailer")
        .click();
    settle(&mut h);
    h.get_by_label("Add").click();
    settle(&mut h);
    // One heading of each per vehicle: both vehicles have main textures and
    // accessories.
    assert_eq!(h.query_all_by_label("Main textures").count(), 2);
    assert_eq!(h.query_all_by_label("Accessories").count(), 2);
    for row in [
        "TruckPaint Sample Truck › Standard cab",
        "TruckPaint Sample Truck › High roof",
        "TruckPaint Sample Truck › Side skirts",
        "TruckPaint Sample Trailer › Base",
    ] {
        assert!(
            h.query_by_label(&format!("Texture {row}")).is_some(),
            "{row}"
        );
    }
    pick_texture(&mut h, "TruckPaint Sample Trailer › Mudflaps");
    let p = &ws(&h).project;
    assert_eq!(p.surfaces[p.active_surface].name, "Mudflaps");
    assert!(
        h.query_by_label("TruckPaint Sample Trailer › Mudflaps")
            .is_some(),
        "status bar breadcrumb"
    );
}

#[test]
fn removing_a_texture_with_artwork_asks_and_undoes() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab", "High roof"], &[]);
    let id = ws_mut(&mut h).create_shape(
        ShapeKind::rectangle(),
        Frame::new(Point::new(500.0, 500.0), Size::new(100.0, 100.0), 0.0),
        1.0,
    );
    h.run();
    vehicle_action(&mut h, "TruckPaint Sample Truck", "Textures…");
    set_checkbox(&mut h, "Standard cab", false);
    h.get_by_label("Apply").click();
    h.run();
    assert!(
        h.query_by_label_contains("The artwork of Standard cab")
            .is_some()
    );
    assert_eq!(ws(&h).project.surfaces.len(), 5, "nothing removed yet");
    h.get_by_label("Remove").click();
    h.run();
    assert!(h.state().modal.is_none());
    assert_eq!(ws(&h).project.surfaces.len(), 4);
    assert_eq!(ws(&h).project.surfaces[0].name, "High roof");
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.surfaces.len(), 5);
    assert!(ws(&h).project.surfaces[0].get(id).is_some());
}

#[test]
fn adding_an_accessory_later_keeps_the_package_order() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab"], &["Cab accessories"]);
    let names = |h: &H| -> Vec<String> {
        ws(h)
            .project
            .surfaces
            .iter()
            .map(|s| s.name.clone())
            .collect()
    };
    assert_eq!(names(&h), ["Standard cab", "Chassis", "Side skirts"]);
    vehicle_action(&mut h, "TruckPaint Sample Truck", "Textures…");
    // The only main texture painted can't be unchecked here either.
    assert!(
        h.get_by_role_and_label(Role::CheckBox, "Standard cab")
            .accesskit_node()
            .is_disabled()
    );
    set_checkbox(&mut h, "Cab accessories", true);
    h.get_by_label("Apply").click();
    h.run();
    assert_eq!(
        names(&h),
        ["Standard cab", "Chassis", "Cab accessories", "Side skirts"]
    );
}

#[test]
fn textures_need_the_recorded_version() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab"], &[]);
    h.state_mut()
        .vehicles
        .remove(SAMPLE_ID, &"1.1.0".parse().unwrap())
        .unwrap();
    h.run();
    vehicle_action(&mut h, "TruckPaint Sample Truck", "Textures…");
    assert!(
        h.query_by_label_contains("Version 1.1.0 of this vehicle is not installed")
            .is_some()
    );
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "High roof")
            .is_none()
    );
}

#[test]
fn only_the_vehicle_with_a_newer_version_offers_an_update() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    let trailer = other_package_file(
        dir.path(),
        "scs.krone.cool_liner",
        "Krone Cool Liner",
        "ets2",
        "trailer",
    );
    h.state_mut().vehicles.install_file(&trailer).unwrap();
    let v100 = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/vehicles/community.truckpaint.sample_truck-1.0.0.tpv");
    h.state_mut().vehicles.install_file(&v100).unwrap();
    // A project on 1.0.0 (Standard cab, Chassis, Cab accessories) with the
    // trailer, then 1.1.0 installed.
    let package = h
        .state()
        .vehicles
        .load(SAMPLE_ID, &"1.0.0".parse().unwrap())
        .unwrap();
    let textures = tp_app::vehicle_project::default_textures(&package.manifest);
    let project = tp_app::vehicle_project::fleet_project("F", &package, &textures).unwrap();
    h.state_mut().open_project(project);
    let krone = h
        .state()
        .vehicles
        .load("scs.krone.cool_liner", &"1.0.0".parse().unwrap())
        .unwrap();
    ws_mut(&mut h).add_vehicle(&krone, &[], 1.0).unwrap();
    h.state_mut().vehicles.install_samples().unwrap();
    h.run();
    assert!(
        h.query_by_label("Update the template of TruckPaint Sample Truck")
            .is_some()
    );
    assert!(
        h.query_by_label("Update the template of Krone Cool Liner")
            .is_none()
    );
    // The menu command applies to the active surface's vehicle: the
    // trailer, which has no update.
    assert_eq!(ws(&h).project.active_surface, 3);
    h.get_by_label("Update the template of TruckPaint Sample Truck")
        .click();
    h.run();
    // The new accessory is offered checked; the High roof (a main texture
    // not painted) unchecked.
    assert!(h.query_by_label("New in this version:").is_some());
    assert!(toggled(&h, Role::CheckBox, "Side skirts"));
    assert!(!toggled(&h, Role::CheckBox, "High roof"));
    h.get_by_label("Update").click();
    h.run();
    let p = &ws(&h).project;
    assert_eq!(p.vehicles[0].version, "1.1.0");
    assert_eq!(p.vehicles[1].version, "1.0.0");
    let names: Vec<&str> = p.surfaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Standard cab",
            "Chassis",
            "Cab accessories",
            "Side skirts",
            "Body"
        ]
    );
    assert_eq!(
        p.surfaces[0].template.as_ref().unwrap().status,
        TemplateStatus::LayoutChanged
    );
    assert_eq!(p.surfaces[1].size, 4096.0, "the chassis grew");
}

#[test]
fn the_vehicles_sidebar_hides_and_comes_back() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    create_sample_project(&mut h, &["Standard cab"], &[]);
    let tree_row = "Texture TruckPaint Sample Truck › Chassis";
    assert!(h.query_by_label(tree_row).is_some(), "open by default");
    h.get_by_label("Hide Sidebar").click();
    h.run();
    assert!(h.query_by_label(tree_row).is_none());
    assert!(!h.state().prefs.layout.vehicles_open, "remembered");
    h.get_by_label("Show Sidebar").click();
    h.run();
    assert!(h.query_by_label(tree_row).is_some());
    // F5 toggles it; Vehicle › Vehicle Information shows it.
    h.key_press(Key::F5);
    h.run();
    assert!(h.query_by_label(tree_row).is_none());
    h.state_mut()
        .queue
        .push(tp_app::commands::CommandId::VehicleInfo);
    // The queued command runs at the end of a frame, the sidebar shows on
    // the next one.
    h.step();
    h.run();
    assert!(h.state().prefs.layout.vehicles_open);
    assert!(h.query_by_label(tree_row).is_some());
}

#[test]
fn projects_open_showing_the_canvas() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = app(dir.path());
    h.state_mut().prefs.layout.view_mode = tp_app::layout::ViewMode::ThreeD;
    create_sample_project(&mut h, &["Standard cab"], &[]);
    assert_eq!(
        h.state().prefs.layout.view_mode,
        tp_app::layout::ViewMode::TwoD
    );
    // Split shows the canvas: kept.
    h.state_mut().close_project();
    h.state_mut().prefs.layout.view_mode = tp_app::layout::ViewMode::Split;
    h.run();
    create_sample_project(&mut h, &["Standard cab"], &[]);
    assert_eq!(
        h.state().prefs.layout.view_mode,
        tp_app::layout::ViewMode::Split
    );
}
