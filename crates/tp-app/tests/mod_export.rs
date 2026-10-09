//! Headless tests for the Export Mod dialog and the Project section's
//! version.

mod common;

use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::commands::CommandId;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::mod_export::Picture;
use tp_app::state::{Modal, is_enabled};
use tp_app::ui::mod_export_dialog::ModExportDialog;
use tp_app::workspace::{SaveState, Workspace};
use tp_core::ModSettings;
use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

/// The sample truck project (Standard cab, Chassis, Cab accessories, Side
/// skirts) named "ACE Logistics", with an ellipse on the Standard cab.
fn open() -> H {
    let mut h = common::harness();
    common::create_project(&mut h);
    {
        let ws = ws_mut(&mut h);
        ws.project.name = "ACE Logistics".into();
        ws.project.mod_settings = ModSettings::for_project("ACE Logistics");
        let mut o = Object::new(
            ObjectId(0),
            ShapeKind::Ellipse,
            Frame::new(Point::new(2048.0, 2048.0), Size::new(2000.0, 1000.0), 0.0),
        );
        o.fill = Rgba::rgb(200, 30, 40).into();
        ws.project.add(o);
    }
    h.run();
    h
}

/// Runs a few frames (background renders make `Harness::run` unusable
/// while the dialog is open), enough for the dialog to settle in place as
/// egui sizes and centers it.
fn settle(h: &mut H) {
    for _ in 0..8 {
        h.step();
    }
}

/// Steps until `done` holds (background work), with a time limit.
fn wait_until(h: &mut H, mut done: impl FnMut(&H) -> bool) {
    for _ in 0..6000 {
        h.step();
        if done(h) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.step();
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn dialog(h: &H) -> Option<&ModExportDialog> {
    match &h.state().modal {
        Some(Modal::ExportMod(d)) => Some(d),
        _ => None,
    }
}

fn dialog_mut(h: &mut H) -> &mut ModExportDialog {
    match &mut h.state_mut().modal {
        Some(Modal::ExportMod(d)) => d,
        _ => panic!("Export Mod dialog open"),
    }
}

fn open_dialog(h: &mut H) {
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    settle(h);
    assert!(dialog(h).is_some(), "Export Mod dialog open");
}

fn script(h: &mut H, mod_save: &[PathBuf], mod_images: &[PathBuf]) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        mod_save: mod_save.iter().cloned().collect(),
        mod_images: mod_images.iter().cloned().collect(),
        ..Default::default()
    });
}

/// Clicks Export… and waits for the export to end.
fn export(h: &mut H) {
    common::last(h, "Export…").click();
    settle(h);
    wait_until(h, |h| dialog(h).is_none_or(|d| d.job.is_none()));
    settle(h);
}

/// Whether the Project space's Mod information column shows `value` (its
/// values are read only: labels, not text fields).
fn shows_field(h: &H, value: &str) -> bool {
    h.query_all_by_label(value).count() > 0
}

fn field_value(h: &H, name: &str) -> String {
    h.get_by_role_and_label(Role::TextInput, name)
        .value()
        .unwrap_or_default()
}

#[test]
fn the_command_needs_a_texture() {
    let mut h = open();
    let enabled = |h: &H| is_enabled(CommandId::ExportMod, &h.state().edit_context());
    assert!(enabled(&h));
    let now = h.ctx.input(|i| i.time);
    let ws = ws_mut(&mut h);
    let id = ws.project.surface().objects[0].id;
    ws.selection = vec![id];
    let symbol = ws.convert_to_symbol(now).expect("symbol");
    ws.edit_symbol(symbol, now);
    h.run();
    assert!(!enabled(&h), "disabled while a symbol is edited");
}

#[test]
fn defaults_and_summary() {
    let mut h = open();
    open_dialog(&mut h);
    assert_eq!(
        dialog(&h).unwrap().settings,
        ModSettings::for_project("ACE Logistics")
    );
    assert_eq!(field_value(&h, "Name"), "ACE Logistics");
    // Titled with the project's name and game.
    assert!(
        h.query_by_label("ACE Logistics · Euro Truck Simulator 2")
            .is_some()
    );
    // The internal name is under Advanced, collapsed.
    assert!(
        h.query_by_role_and_label(Role::TextInput, "Internal name")
            .is_none()
    );
    h.get_by_label("Advanced").click();
    settle(&mut h);
    // The sample truck has two main textures: 10 characters at most.
    assert_eq!(field_value(&h, "Internal name"), "ace_logist");
    for line in [
        "Standard cab: cabins standard",
        "High roof: not painted",
        "Accessories: Chassis, Cab accessories, Side skirts",
    ] {
        assert!(h.query_by_label(line).is_some(), "{line}");
    }
    assert!(!common::last(&h, "Export…").accesskit_node().is_disabled());
}

#[test]
fn an_empty_name_blocks_the_export() {
    let mut h = open();
    open_dialog(&mut h);
    dialog_mut(&mut h).settings.name.clear();
    settle(&mut h);
    assert!(h.query_by_label("The mod needs a name.").is_some());
    assert!(common::last(&h, "Export…").accesskit_node().is_disabled());
}

#[test]
fn escape_keeps_the_project_settings() {
    let mut h = open();
    let undo_steps = ws(&h).history.len();
    open_dialog(&mut h);
    dialog_mut(&mut h).settings.price = 9000;
    settle(&mut h);
    h.key_press(Key::Escape);
    settle(&mut h);
    assert!(dialog(&h).is_none());
    assert_eq!(ws(&h).project.mod_settings.price, 5000);
    assert_eq!(ws(&h).history.len(), undo_steps);
}

#[test]
fn exporting_records_the_settings_as_one_step() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace");
    let mut h = open();
    // Export works from the Project space, which shows the version.
    common::show_space(&mut h, tp_app::layout::Space::Project);
    common::settle_renders(&mut h);
    assert!(
        shows_field(&h, "1.0"),
        "the Project space shows the version"
    );
    open_dialog(&mut h);
    dialog_mut(&mut h).settings.version = "1.2".into();
    settle(&mut h);
    script(&mut h, std::slice::from_ref(&path), &[]);
    export(&mut h);
    assert!(dialog(&h).is_none(), "closed once written");
    // The extension is added when the user leaves it out.
    let written = dir.path().join("ace.scs");
    assert!(written.is_file());
    assert_eq!(h.state().dialogs.suggested(), ["ACE Logistics.scs"]);
    let names: Vec<String> = zip::ZipArchive::new(std::fs::File::open(&written).unwrap())
        .unwrap()
        .file_names()
        .map(str::to_owned)
        .collect();
    assert!(names.iter().any(|n| n == "manifest.sii"), "{names:?}");
    assert_eq!(ws(&h).project.mod_settings.version, "1.2");
    assert_eq!(ws(&h).history.undo_label(), Some("undo-edit-mod-settings"));
    h.run();
    assert!(shows_field(&h, "1.2"));
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.mod_settings.version, "1.0");
    assert!(shows_field(&h, "1.0"));
}

#[test]
fn exporting_unchanged_settings_keeps_the_project_saved() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ACE Logistics.scs");
    let mut h = open();
    let snapshot = ws(&h).snapshot();
    ws_mut(&mut h).saved = Some(snapshot);
    let undo_steps = ws(&h).history.len();
    open_dialog(&mut h);
    script(&mut h, std::slice::from_ref(&path), &[]);
    export(&mut h);
    assert!(path.is_file());
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
    assert_eq!(ws(&h).history.len(), undo_steps);
}

/// Writes a red PNG of `w` × `h` pixels to `path`.
fn red_png(path: &Path, w: u32, h: u32) {
    image::RgbaImage::from_pixel(w, h, image::Rgba([255, 0, 0, 255]))
        .save(path)
        .unwrap();
}

#[test]
fn choosing_an_image_updates_its_preview() {
    let dir = tempfile::tempdir().unwrap();
    let picture = dir.path().join("picture.png");
    red_png(&picture, 1280, 720);
    let mut h = open();
    open_dialog(&mut h);
    wait_until(&mut h, |h| dialog(h).unwrap().preview_ready());
    script(&mut h, &[], std::slice::from_ref(&picture));
    h.get_by_label("Choose Image…").click();
    settle(&mut h);
    assert!(matches!(dialog(&h).unwrap().image, Picture::File(_)));
    assert_eq!(dialog(&h).unwrap().icon, Picture::Generated);
    wait_until(&mut h, |h| dialog(h).unwrap().preview_ready());
    assert!(dialog(&h).unwrap().preview_ready());
    assert!(h.query_by_label("Use Generated Image").is_some());
    assert!(h.query_by_label("Use Generated Icon").is_none());

    // Exported, the picture becomes the project's.
    let out = dir.path().join("ACE.scs");
    script(&mut h, std::slice::from_ref(&out), &[]);
    export(&mut h);
    let project = &ws(&h).project;
    let asset = project.mod_settings.image.expect("chosen image");
    assert_eq!(
        &*project.assets[&asset].bytes,
        std::fs::read(&picture).unwrap()
    );
    assert_eq!(project.mod_settings.icon, None);
}

#[test]
fn an_unreadable_picture_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.png");
    std::fs::write(&bad, b"not an image").unwrap();
    let mut h = open();
    open_dialog(&mut h);
    script(&mut h, &[], std::slice::from_ref(&bad));
    h.get_by_label("Choose Icon…").click();
    settle(&mut h);
    let d = dialog(&h).unwrap();
    assert_eq!(d.icon, Picture::Generated);
    assert!(d.picture_error.as_deref().unwrap().contains("bad.png"));
}

#[test]
fn advanced_opens_on_an_internal_name_problem() {
    let mut h = open();
    // Set while the project held only a trailer (12 characters were
    // allowed); the truck, with two main textures, allows 10.
    ws_mut(&mut h).project.mod_settings.internal_name = Some("ace_logistic".into());
    open_dialog(&mut h);
    assert!(dialog(&h).unwrap().advanced, "Advanced is open");
    assert_eq!(field_value(&h, "Internal name"), "ace_logistic");
    let problem = "The internal name can have at most 10 characters.";
    let problem_top = h.get_by_label(problem).rect().top();
    let export = common::last(&h, "Export…");
    assert!(export.accesskit_node().is_disabled());
    // Listed just above the buttons.
    assert!(problem_top < export.rect().top());
    assert!(problem_top > h.get_by_label("In the mod").rect().top());
}

#[test]
fn export_from_the_brand_space() {
    let mut h = open();
    common::show_space(&mut h, tp_app::layout::Space::Brand);
    h.run();
    h.get_by_role_and_label(Role::Button, "Export…").click();
    settle(&mut h);
    assert!(dialog(&h).is_some(), "the Export Mod dialog opens");
    assert!(!dialog(&h).unwrap().advanced, "Advanced is collapsed");
}
