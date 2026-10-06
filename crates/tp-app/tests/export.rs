//! Headless tests for the Export Texture dialog.

mod common;

use std::path::PathBuf;

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::state::Modal;
use tp_app::ui::export_dialog::ExportDialog;
use tp_app::workspace::{SaveState, Workspace};
use tp_core::document::{Frame, Object, ObjectId, Rgba, ShapeKind};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

fn open() -> H {
    let mut h = common::harness();
    common::create_project(&mut h);
    {
        let ws = h.state_mut().workspace_mut().unwrap();
        ws.project.name = "ACE Logistics".into();
        let mut o = Object::new(
            ObjectId(0),
            ShapeKind::Ellipse,
            Frame::new(Point::new(2048.0, 2048.0), Size::new(2000.0, 1000.0), 0.0),
        );
        o.fill = Rgba::rgb(200, 30, 40);
        ws.project.add(o);
    }
    h.run();
    h
}

/// Runs a few frames. Background work (preview renders, exports) wakes the
/// UI at unpredictable times, so `Harness::run` (which waits until nothing
/// asks for a repaint) is not usable while the dialog is open.
fn settle(h: &mut H) {
    for _ in 0..4 {
        h.step();
    }
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn dialog(h: &H) -> Option<&ExportDialog> {
    match &h.state().modal {
        Some(Modal::Export(d)) => Some(d),
        _ => None,
    }
}

fn open_dialog(h: &mut H) {
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    settle(h);
    assert!(dialog(h).is_some(), "export dialog open");
}

/// Steps until `done` holds (background work), with a time limit.
fn wait_until(h: &mut H, mut done: impl FnMut(&H) -> bool) {
    for _ in 0..3000 {
        h.step();
        if done(h) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.step();
}

fn export_to(h: &mut H, path: &std::path::Path) {
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        export: [path.to_path_buf()].into(),
        ..Default::default()
    });
    h.get_by_label("Export…").click();
    // The export repaints as it progresses: step rather than run.
    h.step();
    h.step();
}

#[test]
fn default_export_settings() {
    let mut h = open();
    open_dialog(&mut h);
    let d = dialog(&h).unwrap();
    assert_eq!(d.settings, tp_app::export::ExportSettings::default());
    assert!(h.query_by_label_contains("4096 × 4096 px · PNG").is_some());
    wait_until(&mut h, |h| {
        dialog(h).is_some_and(ExportDialog::preview_ready)
    });
    assert!(dialog(&h).unwrap().preview_ready());
    h.key_press(Key::Escape);
    settle(&mut h);
    assert!(h.state().modal.is_none());
}

#[test]
fn export_to_png() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.png");
    let mut h = open();
    open_dialog(&mut h);
    export_to(&mut h, &dir.path().join("ace"));
    assert_eq!(
        h.state().dialogs_suggested(),
        vec!["ACE Logistics.png".to_owned()]
    );
    wait_until(&mut h, |h| h.state().modal.is_none());
    assert_eq!(image::image_dimensions(&path).unwrap(), (4096, 4096));
    let img = image::open(&path).unwrap().to_rgba8();
    assert_eq!(img.get_pixel(2048, 2048).0, [200, 30, 40, 255]);
    assert_eq!(img.get_pixel(10, 10).0, [255, 255, 255, 255]);
    assert_eq!(ws(&h).hint.as_ref().unwrap().text, "Exported ace.png");
}

#[test]
fn export_does_not_change_the_project() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open();
    let path = dir.path().join("ace.truckpaint");
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        save: [path].into(),
        ..Default::default()
    });
    h.key_press_modifiers(Modifiers::COMMAND, Key::S);
    wait_until(&mut h, |h| ws(h).saving.is_none());
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
    let history = ws(&h).history.len();
    open_dialog(&mut h);
    dialog_quarter_size(&mut h);
    export_to(&mut h, &dir.path().join("ace.png"));
    wait_until(&mut h, |h| h.state().modal.is_none());
    assert!(dir.path().join("ace.png").is_file());
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
    assert_eq!(ws(&h).history.len(), history);
}

fn dialog_quarter_size(h: &mut H) {
    h.get_by_label("1024 × 1024").click();
    settle(h);
}

#[test]
fn unwritable_destination() {
    let mut h = open();
    open_dialog(&mut h);
    dialog_quarter_size(&mut h);
    let path = PathBuf::from("/nonexistent-dir/ace.png");
    export_to(&mut h, &path);
    wait_until(&mut h, |h| {
        matches!(h.state().modal, Some(Modal::Message { .. }))
    });
    let Some(Modal::Message { text, .. }) = &h.state().modal else {
        panic!("error message expected");
    };
    assert!(text.starts_with("ace.png could not be written"), "{text}");
    assert!(!path.exists());
}

#[test]
fn cancel_during_export_leaves_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open();
    open_dialog(&mut h);
    h.get_by_label("DDS").click();
    settle(&mut h);
    export_to(&mut h, &dir.path().join("ace.dds"));
    assert!(dialog(&h).unwrap().job.is_some());
    // The modal re-centers once its content (progress bar) is laid out.
    h.step();
    h.get_by_label("Cancel").click();
    h.step();
    wait_until(&mut h, |h| dialog(h).is_some_and(|d| d.job.is_none()));
    assert!(
        dialog(&h).is_some(),
        "the dialog stays open after cancelling"
    );
    assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
}

#[test]
fn transparent_background_preview_and_size_info() {
    let mut h = open();
    open_dialog(&mut h);
    h.get_by_label("Transparent").click();
    settle(&mut h);
    assert_eq!(dialog(&h).unwrap().settings.background, None);
    wait_until(&mut h, |h| {
        dialog(h).is_some_and(ExportDialog::preview_ready)
    });
    assert_eq!(dialog(&h).unwrap().preview_for.unwrap().background, None);
    h.get_by_label("2048 × 2048").click();
    settle(&mut h);
    assert!(h.query_by_label_contains("2048 × 2048 px · PNG").is_some());
    // Settings are remembered for the next export.
    h.key_press(Key::Escape);
    settle(&mut h);
    open_dialog(&mut h);
    assert_eq!(dialog(&h).unwrap().settings.divisor, 2);
}

#[test]
fn dds_export_has_mipmaps() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ace.dds");
    let mut h = open();
    open_dialog(&mut h);
    h.get_by_label("DDS").click();
    settle(&mut h);
    dialog_quarter_size(&mut h);
    export_to(&mut h, &path);
    wait_until(&mut h, |h| h.state().modal.is_none());
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[84..88], b"DXT5");
    assert_eq!(u32::from_le_bytes(bytes[28..32].try_into().unwrap()), 11);
}
