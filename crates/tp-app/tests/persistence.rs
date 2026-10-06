//! Headless tests for persistence: saving, opening, the unsaved-changes
//! prompt, recent projects and crash recovery.

mod common;

use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui::{Key, Modifiers, ViewportEvent, ViewportId};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::state::{Modal, Screen};
use tp_app::workspace::{SaveState, Workspace};
use tp_core::document::{CharStyle, Frame, Object, ObjectId, Rgba, ShapeKind, TextBlock};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

fn harness_in(recovery: Option<&Path>) -> H {
    let mut state = AppState::with_prefs(tp_app::prefs::Prefs::default(), None);
    if let Some(dir) = recovery {
        state.enable_recovery(dir);
    }
    Harness::builder()
        .with_size(common::SIZE)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state)
}

fn open() -> H {
    let mut h = harness_in(None);
    common::create_project(&mut h);
    h.run();
    h
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn dialogs(h: &mut H, save: &[PathBuf], open: &[PathBuf]) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        save: save.iter().cloned().collect(),
        open: open.iter().cloned().collect(),
        ..Default::default()
    });
}

/// Steps until background saves are applied.
fn wait_saves(h: &mut H) {
    for _ in 0..400 {
        h.step();
        let busy = h.state().workspace().is_some_and(|w| w.saving.is_some());
        if !busy {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.run();
}

fn save_shortcut(h: &mut H) {
    h.key_press_modifiers(Modifiers::COMMAND, Key::S);
    wait_saves(h);
}

fn add_rect(h: &mut H, x: f64) -> ObjectId {
    let now = h.ctx.input(|i| i.time);
    let frame = Frame::new(Point::new(x, 500.0), Size::new(200.0, 100.0), 0.0);
    let id = ws_mut(h).create_shape(ShapeKind::rectangle(), frame, now);
    h.run();
    id
}

fn status(h: &H) -> SaveState {
    ws(h).save_state()
}

fn message_text(h: &H) -> Option<String> {
    match &h.state().modal {
        Some(Modal::Message { text, .. }) => Some(text.clone()),
        _ => None,
    }
}

// ------------------------------------------------------------------ saving

#[test]
fn first_save() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open();
    ws_mut(&mut h).project.name = "ACE Logistics".into();
    // The extension is added when the user leaves it out.
    dialogs(&mut h, &[dir.path().join("ace")], &[]);
    assert_eq!(status(&h), SaveState::Unsaved);
    save_shortcut(&mut h);
    let path = dir.path().join("ace.truckpaint");
    assert!(path.is_file());
    assert_eq!(status(&h), SaveState::Saved);
    assert_eq!(ws(&h).path.as_deref(), Some(path.as_path()));
    assert!(h.query_by_label("Saved").is_some());
    assert_eq!(h.state().prefs.recent[0].path, path);
}

#[test]
fn save_after_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    let mut h = open();
    dialogs(&mut h, std::slice::from_ref(&path), &[]);
    save_shortcut(&mut h);
    add_rect(&mut h, 300.0);
    assert_eq!(status(&h), SaveState::Unsaved);
    // No second dialog: the scripted save queue is empty now.
    save_shortcut(&mut h);
    assert_eq!(status(&h), SaveState::Saved);
    let reread = tp_file::read(&path).unwrap().project;
    assert_eq!(reread.surface().objects.len(), 1);
}

#[test]
fn undo_back_to_the_saved_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open();
    dialogs(&mut h, &[dir.path().join("a.truckpaint")], &[]);
    save_shortcut(&mut h);
    add_rect(&mut h, 300.0);
    assert_eq!(status(&h), SaveState::Unsaved);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(status(&h), SaveState::Saved);
    assert!(h.query_by_label("Saved").is_some());
}

#[test]
fn saving_status_is_shown() {
    let mut h = open();
    ws_mut(&mut h).saving = Some(tp_app::workspace::PendingSave {
        id: 999,
        snapshot: ws(&h).snapshot(),
        path: "/tmp/x.truckpaint".into(),
    });
    h.run();
    assert!(h.query_by_label("Saving…").is_some());
}

#[cfg(unix)]
#[test]
fn save_to_a_read_only_location() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let locked = dir.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    let mut h = open();
    dialogs(&mut h, &[locked.join("ace.truckpaint")], &[]);
    save_shortcut(&mut h);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    let text = message_text(&h).expect("error message");
    assert!(text.contains("ace.truckpaint could not be saved"), "{text}");
    assert_eq!(status(&h), SaveState::Unsaved);
    assert!(!locked.join("ace.truckpaint").exists());
    h.key_press(Key::Enter);
    h.run();
    assert!(h.state().modal.is_none());
}

// ----------------------------------------------------------------- opening

/// A saved project with a group, a text and an image, at `path`.
fn rich_file(path: &Path) -> Workspace {
    let mut h = open();
    {
        let ws = ws_mut(&mut h);
        ws.project.name = "ACE Logistics".into();
        let mut a = Object::new(
            ObjectId(0),
            ShapeKind::Rectangle {
                corner_radius: 12.0,
            },
            Frame::new(Point::new(800.0, 600.0), Size::new(400.0, 200.0), 15.0),
        );
        a.fill = Rgba::rgb(0x7A, 0x1F, 0x2B);
        let a = ws.project.add(a);
        let b = ws.project.add(Object::new(
            ObjectId(0),
            ShapeKind::Ellipse,
            Frame::new(Point::new(1200.0, 600.0), Size::new(300.0, 300.0), 0.0),
        ));
        ws.project.group(&[a, b]);
        let anchor = Point::new(500.0, 1500.0);
        let mut text = Object::text(
            ObjectId(0),
            TextBlock::new("ACE", CharStyle::default()),
            anchor,
        );
        ws.text.place_at(&mut text, anchor);
        ws.project.add(text);
        let png = tp_app::import::solid_png(80, 40, [20, 60, 220, 255]);
        let file = tp_app::import::read_bytes("logo.png", png).unwrap();
        ws.place_files(vec![Ok(file)], Some(Point::new(2000.0, 2000.0)), 0.0);
        ws.project.add_to_palette(Rgba::rgb(1, 2, 3));
    }
    dialogs(&mut h, &[path.to_path_buf()], &[]);
    save_shortcut(&mut h);
    assert_eq!(status(&h), SaveState::Saved);
    let Screen::Workspace(ws) = std::mem::replace(&mut h.state_mut().screen, Screen::Home) else {
        unreachable!()
    };
    *ws
}

#[test]
fn round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    let original = rich_file(&path);
    let mut h = harness_in(None);
    dialogs(&mut h, &[], std::slice::from_ref(&path));
    h.key_press_modifiers(Modifiers::COMMAND, Key::O);
    h.run();
    let opened = &ws(&h).project;
    assert_eq!(opened.name, "ACE Logistics");
    assert_eq!(opened.palette, original.project.palette);
    assert_eq!(opened.assets, original.project.assets);
    assert_eq!(opened.surfaces, original.project.surfaces);
    assert_eq!(status(&h), SaveState::Saved);
    assert!(ws(&h).selection.is_empty());
    assert!(!ws(&h).history.can_undo());
}

#[test]
fn open_from_the_menu() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    rich_file(&path);
    let mut h = open();
    // Replacing an unsaved project asks first.
    dialogs(&mut h, &[], std::slice::from_ref(&path));
    h.get_by_label("File").click();
    h.run();
    h.get_by_label("Open Project…").click();
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::UnsavedChanges(_))));
    h.get_by_label("Don't Save").click();
    h.run();
    assert_eq!(ws(&h).project.name, "ACE Logistics");
    assert_eq!(h.state().title(), "ACE Logistics — TruckPaint");
    assert!(h.query_by_label("Saved").is_some());
}

#[test]
fn damaged_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    rich_file(&path);
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.truncate(bytes.len() / 2);
    std::fs::write(&path, bytes).unwrap();
    let mut h = open();
    let before = ws(&h).project.clone();
    dialogs(&mut h, &[], &[path]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::O);
    h.run();
    h.get_by_label("Don't Save").click();
    h.run();
    assert_eq!(
        message_text(&h).as_deref(),
        Some("ace.truckpaint is damaged and cannot be opened.")
    );
    assert_eq!(ws(&h).project, before);
}

#[test]
fn newer_format() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("future.truckpaint");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    let o = zip::write::SimpleFileOptions::default();
    zip.start_file("mimetype", o).unwrap();
    zip.write_all(b"application/x-truckpaint").unwrap();
    zip.start_file("project.ron", o).unwrap();
    zip.write_all(b"(format: 99, name: \"x\")").unwrap();
    zip.finish().unwrap();
    let mut h = harness_in(None);
    dialogs(&mut h, &[], &[path]);
    h.get_by_label("Open Project…").click();
    h.run();
    assert_eq!(
        message_text(&h).as_deref(),
        Some("future.truckpaint was created with a newer version of TruckPaint.")
    );
    assert!(!h.state().has_project());
}

#[test]
fn open_from_the_home_screen_and_recent_entries() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.truckpaint");
    rich_file(&path);
    let mut h = harness_in(None);
    dialogs(&mut h, &[], std::slice::from_ref(&path));
    h.get_by_label("Open Project…").click();
    h.run();
    assert_eq!(ws(&h).project.name, "ACE Logistics");
    // Saved project closes without a prompt and is listed first.
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    h.run();
    assert!(matches!(h.state().screen, Screen::Home));
    assert_eq!(h.state().prefs.recent[0].path, path);
    h.get_by_label("ACE Logistics").click();
    h.run();
    assert_eq!(ws(&h).path.as_deref(), Some(path.as_path()));
}

// ------------------------------------------------------- unsaved changes

#[test]
fn cancel_keeps_working() {
    let mut h = open();
    add_rect(&mut h, 300.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    h.run();
    assert!(h.query_by_label_contains("Save changes to").is_some());
    h.key_press(Key::Escape);
    h.run();
    assert!(h.state().modal.is_none());
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
    assert_eq!(status(&h), SaveState::Unsaved);
}

#[test]
fn no_prompt_when_saved() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open();
    dialogs(&mut h, &[dir.path().join("a.truckpaint")], &[]);
    save_shortcut(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    h.run();
    assert!(matches!(h.state().screen, Screen::Home));
}

#[test]
fn save_from_the_prompt_then_close() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.truckpaint");
    let mut h = open();
    add_rect(&mut h, 300.0);
    dialogs(&mut h, std::slice::from_ref(&path), &[]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    h.run();
    h.get_by_role_and_label(Role::Button, "Save").click();
    h.run();
    for _ in 0..200 {
        h.step();
        if !h.state().has_project() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(matches!(h.state().screen, Screen::Home));
    assert_eq!(
        tp_file::read(&path)
            .unwrap()
            .project
            .surface()
            .objects
            .len(),
        1
    );
}

#[test]
fn new_project_asks_first() {
    let mut h = open();
    add_rect(&mut h, 300.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::N);
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::UnsavedChanges(_))));
    h.get_by_label("Don't Save").click();
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::NewProject(_))));
}

#[test]
fn quit_with_unsaved_changes() {
    let mut h = open();
    add_rect(&mut h, 300.0);
    h.input_mut()
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
    h.step();
    assert!(matches!(h.state().modal, Some(Modal::UnsavedChanges(_))));
    assert!(h.state().has_project());
    h.run();
    h.get_by_label("Cancel").click();
    h.run();
    assert!(h.state().modal.is_none());
    assert!(h.state().has_project());
}

// ----------------------------------------------------------- crash recovery

fn copies(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "truckpaint"))
        .collect()
}

/// Edits for more than the recovery interval.
fn edit_for_a_while(h: &mut H) {
    h.input_mut().time = Some(1000.0);
    h.step();
    add_rect(h, 300.0);
    h.input_mut().time = Some(1000.0 + tp_app::recovery::INTERVAL + 1.0);
    h.step();
    h.input_mut().time = None;
    h.run();
    // Recovery copies are written in the background.
    for _ in 0..200 {
        if !copies(h.state().recovery_dir().unwrap()).is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn recovery_copy_while_editing_and_normal_close() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = harness_in(Some(dir.path()));
    common::create_project(&mut h);
    edit_for_a_while(&mut h);
    assert_eq!(copies(dir.path()).len(), 1);
    assert_eq!(status(&h), SaveState::Unsaved);
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    h.run();
    h.get_by_label("Don't Save").click();
    h.run();
    assert!(copies(dir.path()).is_empty());
}

/// Leaves a recovery copy of "ACE Logistics" as a crashed session would.
fn crashed_session(dir: &Path) {
    let mut h = harness_in(Some(dir));
    common::create_project(&mut h);
    ws_mut(&mut h).project.name = "ACE Logistics".into();
    edit_for_a_while(&mut h);
    assert_eq!(copies(dir).len(), 1);
    // Dropped without shutdown: like a crash.
    drop(h);
}

#[test]
fn restore_unsaved_work() {
    let dir = tempfile::tempdir().unwrap();
    crashed_session(dir.path());
    let mut h = harness_in(Some(dir.path()));
    h.run();
    let recovered = h.get_by_label("Recovered projects").rect();
    let recent = h.get_by_label("Recent projects").rect();
    assert!(recovered.top() < recent.top(), "recovery shown first");
    assert!(h.query_by_label("Recovered ACE Logistics").is_some());
    h.get_by_label("Restore").click();
    h.run();
    assert_eq!(ws(&h).project.name, "ACE Logistics");
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
    assert_eq!(status(&h), SaveState::Unsaved);
    assert!(h.state().recovered.is_empty());
    // The copy now belongs to this session: closing removes it.
    h.key_press_modifiers(Modifiers::COMMAND, Key::W);
    h.run();
    h.get_by_label("Don't Save").click();
    h.run();
    assert!(copies(dir.path()).is_empty());
}

#[test]
fn discard_a_recovered_project() {
    let dir = tempfile::tempdir().unwrap();
    crashed_session(dir.path());
    let mut h = harness_in(Some(dir.path()));
    h.run();
    h.get_by_label("Discard").click();
    h.run();
    assert!(h.query_by_label("Recovered projects").is_none());
    assert!(copies(dir.path()).is_empty());
    drop(h);
    let h = harness_in(Some(dir.path()));
    assert!(h.state().recovered.is_empty());
}
