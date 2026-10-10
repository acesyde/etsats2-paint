//! Headless tests for editing the mod settings and pictures in the Project
//! space's Mod information column: one undo step per committed field,
//! Escape, Undo, the pictures chosen or dropped, and a field committed by
//! Export Mod….

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::layout::Space;
use tp_app::state::Modal;
use tp_app::workspace::{SaveState, Workspace};

type H = Harness<'static, AppState>;

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

/// A tall window showing, in the Project space, a saved project named "ACE
/// Logistics" for the sample truck painting its default textures.
fn open() -> H {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, 1800.0))
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(Default::default(), None),
        );
    common::create_project(&mut h);
    {
        let ws = ws_mut(&mut h);
        ws.project.name = "ACE Logistics".into();
        ws.project.mod_settings = tp_core::ModSettings::for_project("ACE Logistics");
        let snapshot = ws.snapshot();
        ws.saved = Some(snapshot);
    }
    common::show_space(&mut h, Space::Project);
    common::settle_renders(&mut h);
    h
}

/// The text of text field `name`.
fn field(h: &H, name: &str) -> String {
    h.get_by_role_and_label(Role::TextInput, name)
        .value()
        .unwrap_or_default()
}

/// Focuses field `name` and replaces its text with `text` (without leaving
/// it).
fn type_in(h: &mut H, name: &str, text: &str) {
    h.get_by_role_and_label(Role::TextInput, name).focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, name)
        .type_text(text);
    h.run();
}

/// Types `text` in field `name`, then presses `key`.
fn type_into(h: &mut H, name: &str, text: &str, key: Key) {
    type_in(h, name, text);
    h.key_press(key);
    h.run();
}

fn steps(h: &H) -> usize {
    ws(h).history.len()
}

#[test]
fn one_step_per_field() {
    let mut h = open();
    let before = steps(&h);
    type_into(&mut h, "Author", "Jane", Key::Enter);
    type_into(&mut h, "Price", "8000", Key::Tab);
    let s = &ws(&h).project.mod_settings;
    assert_eq!((s.author.as_str(), s.price), ("Jane", 8000));
    assert_eq!(steps(&h), before + 2);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-edit-mod-settings"));
    assert_eq!(ws(&h).save_state(), SaveState::Unsaved);
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.mod_settings.price, 5000);
    assert_eq!(ws(&h).project.mod_settings.author, "Jane");
    assert_eq!(ws(&h).history.undo_label(), Some("undo-edit-mod-settings"));
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.mod_settings.author, "");
}

#[test]
fn mod_settings_are_edited_here() {
    let mut h = open();
    h.get_by_role_and_label(Role::TextInput, "Name").focus();
    h.run();
    h.key_press(Key::End);
    h.get_by_role_and_label(Role::TextInput, "Name")
        .type_text(" Freight");
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(ws(&h).project.mod_settings.name, "ACE Logistics Freight");
}

#[test]
fn escape_restores_the_field() {
    let mut h = open();
    let before = steps(&h);
    type_into(&mut h, "Version", "1.1", Key::Escape);
    assert_eq!(field(&h, "Version"), "1.0");
    assert_eq!(ws(&h).project.mod_settings.version, "1.0");
    assert_eq!(steps(&h), before);
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
}

#[test]
fn unchanged_value_records_nothing() {
    let mut h = open();
    let before = steps(&h);
    h.get_by_role_and_label(Role::TextInput, "Name").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run();
    assert_eq!(steps(&h), before);
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
    // A number typed again as it is records nothing either; anything else
    // than a number restores it.
    type_into(&mut h, "Price", "5000", Key::Enter);
    type_into(&mut h, "Unlock level", "abc", Key::Enter);
    assert_eq!(field(&h, "Unlock level"), "0");
    assert_eq!(steps(&h), before);
}

#[test]
fn description_on_several_lines() {
    let mut h = open();
    let before = steps(&h);
    type_in(&mut h, "Description", "Fleet colors");
    h.key_press(Key::Enter);
    h.get_by_role_and_label(Role::TextInput, "Description")
        .type_text("Truck and trailer");
    h.run();
    // Enter started a new line: nothing recorded yet.
    assert_eq!(steps(&h), before);
    h.get_by_label("Mod information").click();
    h.run();
    assert_eq!(
        ws(&h).project.mod_settings.description,
        "Fleet colors\nTruck and trailer"
    );
    assert_eq!(steps(&h), before + 1);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-edit-mod-settings"));
}

#[test]
fn undo_shows_in_the_field() {
    let mut h = open();
    type_into(&mut h, "Version", "1.2", Key::Enter);
    assert_eq!(ws(&h).project.mod_settings.version, "1.2");
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(field(&h, "Version"), "1.0");
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    h.run();
    assert_eq!(field(&h, "Version"), "1.2");
}

#[test]
fn export_while_typing() {
    let mut h = open();
    let before = steps(&h);
    type_in(&mut h, "Name", "ACE Freight");
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    for _ in 0..4 {
        h.step();
    }
    assert_eq!(ws(&h).project.mod_settings.name, "ACE Freight");
    assert_eq!(steps(&h), before + 1);
    let Some(Modal::ExportMod(dialog)) = &h.state().modal else {
        panic!("Export Mod dialog open");
    };
    assert_eq!(
        dialog.destination.file_name().unwrap().to_str(),
        Some("ACE Freight.scs")
    );
    // The field then sees it lost the focus: nothing more is recorded.
    for _ in 0..4 {
        h.step();
    }
    assert_eq!(steps(&h), before + 1);
}

#[test]
fn a_field_left_for_another_space_is_committed() {
    let mut h = open();
    type_in(&mut h, "Author", "Jane");
    common::show_space(&mut h, Space::Workshop);
    h.run();
    assert_eq!(ws(&h).project.mod_settings.author, "Jane");
    assert!(ws(&h).mod_draft.is_none());
    common::show_space(&mut h, Space::Project);
    h.run();
    assert_eq!(field(&h, "Author"), "Jane");
}

#[test]
fn fields_stay_editable_while_a_symbol_is_edited() {
    let mut h = open();
    let w = ws_mut(&mut h);
    let id = w.project.add(tp_core::document::Object::new(
        tp_core::document::ObjectId(0),
        tp_core::document::ShapeKind::rectangle(),
        tp_core::document::Frame::new(
            tp_core::kurbo::Point::new(500.0, 500.0),
            tp_core::kurbo::Size::new(100.0, 100.0),
            0.0,
        ),
    ));
    w.selection = vec![id];
    let symbol = w.convert_to_symbol(0.0).unwrap();
    w.edit_symbol(symbol, 0.0);
    h.run();
    for name in ["Name", "Author", "Version", "Description", "Price"] {
        assert!(
            !h.get_by_role_and_label(Role::TextInput, name)
                .accesskit_node()
                .is_disabled(),
            "{name}"
        );
    }
    type_into(&mut h, "Author", "Jane", Key::Enter);
    assert_eq!(ws(&h).project.mod_settings.author, "Jane");
}

/// Writes a PNG of `w` × `h` pixels of `rgb` to `path`.
fn png_file(path: &Path, w: u32, h: u32, rgb: [u8; 3]) {
    image::RgbaImage::from_pixel(w, h, image::Rgba([rgb[0], rgb[1], rgb[2], 255]))
        .save(path)
        .unwrap();
}

fn script_images(h: &mut H, images: &[PathBuf]) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        mod_images: images.iter().cloned().collect(),
        ..Default::default()
    });
}

#[test]
fn choosing_a_mod_manager_image() {
    let dir = tempfile::tempdir().unwrap();
    let picture = dir.path().join("red.png");
    png_file(&picture, 1280, 720, [255, 0, 0]);
    let mut h = open();
    let before = steps(&h);
    let renders = ws(&h).mod_previews.renders;
    assert!(h.query_by_label("Use Generated Image").is_none());
    script_images(&mut h, std::slice::from_ref(&picture));
    h.get_by_label("Choose Image…").click();
    common::settle_renders(&mut h);
    let project = &ws(&h).project;
    let asset = project.mod_settings.image.expect("chosen image");
    assert_eq!(
        &*project.assets[&asset].bytes,
        std::fs::read(&picture).unwrap()
    );
    assert_eq!(project.mod_settings.icon, None);
    assert_eq!(steps(&h), before + 1);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-edit-mod-settings"));
    // The preview is rendered again from the chosen picture.
    assert_eq!(ws(&h).mod_previews.renders, renders + 1);
    assert!(h.query_by_label("Use Generated Image").is_some());
    assert!(h.query_by_label("Use Generated Icon").is_none());
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.mod_settings.image, None);
}

#[test]
fn back_to_the_generated_picture() {
    let dir = tempfile::tempdir().unwrap();
    let picture = dir.path().join("red.png");
    png_file(&picture, 300, 200, [255, 0, 0]);
    let mut h = open();
    script_images(&mut h, std::slice::from_ref(&picture));
    h.get_by_label("Choose Image…").click();
    common::settle_renders(&mut h);
    assert!(ws(&h).project.mod_settings.image.is_some());
    let assets = ws(&h).project.assets.len();
    h.get_by_label("Use Generated Image").click();
    common::settle_renders(&mut h);
    assert_eq!(ws(&h).project.mod_settings.image, None);
    assert_eq!(ws(&h).project.assets.len(), assets - 1);
    assert!(h.query_by_label("Use Generated Image").is_none());
}

#[test]
fn chosen_mod_manager_image_is_exported() {
    let dir = tempfile::tempdir().unwrap();
    let picture = dir.path().join("red.png");
    png_file(&picture, 1280, 720, [255, 0, 0]);
    let mut h = open();
    script_images(&mut h, std::slice::from_ref(&picture));
    h.get_by_label("Choose Image…").click();
    common::settle_renders(&mut h);
    // The export uses the project's picture.
    let plan = tp_app::mod_export::plan(&{
        let mut p = ws(&h).project.clone();
        p.mod_settings.internal_name = Some("ace".into());
        p
    })
    .unwrap();
    assert!(plan.files.contains_key("icon.jpg"));
    let project = &ws(&h).project;
    let bytes = tp_app::mod_export::Picture::of(project.mod_settings.image)
        .bytes(project)
        .map(<[u8]>::to_vec)
        .unwrap();
    let mut fonts = tp_text::FontLibrary::bundled();
    let image = tp_app::mod_export::picture(
        project,
        Some(&bytes),
        tp_app::mod_export::IMAGE_SIZE,
        &mut fonts,
    )
    .unwrap();
    let rgba = tp_render::to_rgba(&image);
    let [r, g, b, _] = rgba.get_pixel(138, 81).0;
    assert!(r > 240 && g < 20 && b < 20, "{r} {g} {b}");
}

#[test]
fn an_unreadable_picked_file_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.png");
    std::fs::write(&bad, b"not an image").unwrap();
    let mut h = open();
    let before = steps(&h);
    script_images(&mut h, std::slice::from_ref(&bad));
    h.get_by_label("Choose Icon…").click();
    h.run();
    assert_eq!(ws(&h).project.mod_settings.icon, None);
    assert_eq!(steps(&h), before);
    assert!(h.query_by_label_contains("bad.png can't be used").is_some());
}

/// A file dropped from the operating system.
#[derive(Debug)]
struct TestFile {
    path: PathBuf,
    bytes: Vec<u8>,
}

impl egui::DroppedFile for TestFile {
    fn path(&self) -> &Path {
        &self.path
    }

    fn bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.bytes.clone())
    }
}

/// Drops file `name` holding `bytes` at `at`.
fn drop_file(h: &mut H, name: &str, bytes: Vec<u8>, at: egui::Pos2) {
    h.event(Event::PointerMoved(at));
    h.step();
    h.input_mut().dropped_files = vec![Arc::new(TestFile {
        path: PathBuf::from("/tmp").join(name),
        bytes,
    })];
    h.step();
    h.input_mut().dropped_files.clear();
    common::settle_renders(h);
}

fn jpeg(w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    image::RgbImage::from_pixel(w, h, image::Rgb([0, 0, 200]))
        .write_to(
            &mut std::io::Cursor::new(&mut out),
            image::ImageFormat::Jpeg,
        )
        .unwrap();
    out
}

#[test]
fn dropping_an_icon() {
    let mut h = open();
    let before = steps(&h);
    let objects = ws(&h).project.surface().objects.len();
    let icon = h
        .get_by_role_and_label(Role::Image, "Shop icon")
        .rect()
        .center();
    drop_file(&mut h, "logo.jpg", jpeg(400, 100), icon);
    let project = &ws(&h).project;
    assert!(project.mod_settings.icon.is_some());
    assert_eq!(project.mod_settings.image, None);
    assert_eq!(steps(&h), before + 1);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-edit-mod-settings"));
    assert_eq!(ws(&h).project.surface().objects.len(), objects);
}

#[test]
fn an_unreadable_dropped_file_is_reported() {
    let mut h = open();
    let before = steps(&h);
    let image = h
        .get_by_role_and_label(Role::Image, "Mod Manager image")
        .rect()
        .center();
    drop_file(&mut h, "logo.png", b"just some text".to_vec(), image);
    assert_eq!(ws(&h).project.mod_settings.image, None);
    assert_eq!(steps(&h), before);
    assert!(
        h.query_by_label_contains("logo.png can't be used")
            .is_some()
    );
    // Until the next picture change.
    let icon = h
        .get_by_role_and_label(Role::Image, "Shop icon")
        .rect()
        .center();
    drop_file(&mut h, "logo.jpg", jpeg(400, 100), icon);
    assert!(
        h.query_by_label_contains("logo.png can't be used")
            .is_none()
    );
}

#[test]
fn a_file_dropped_elsewhere_is_placed_on_the_texture() {
    let mut h = open();
    let objects = ws(&h).project.surface().objects.len();
    let elsewhere = h.get_by_label("Vehicles").rect().center();
    drop_file(&mut h, "logo.jpg", jpeg(40, 40), elsewhere);
    assert_eq!(ws(&h).project.surface().objects.len(), objects + 1);
    assert_eq!(ws(&h).project.mod_settings.icon, None);
    assert_eq!(ws(&h).project.mod_settings.image, None);
}
