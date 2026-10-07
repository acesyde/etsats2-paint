//! Headless tests for text-and-images: the Text tool and on-canvas editing,
//! character settings, image import, the Assets panel and the Eyedropper on
//! images.

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::import::{read_bytes, solid_png};
use tp_app::layout::PanelKind;
use tp_app::prefs::Prefs;
use tp_app::tool::Tool;
use tp_app::workspace::Workspace;
use tp_core::document::{Object, ObjectId, Rgba, ShapeKind, StrokeStyle, TextAlign};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

/// Runs a few frames. Background threads (saves, recovery copies, SVG
/// renders) wake the UI at unpredictable times, so `Harness::run`, which
/// waits until nothing asks for a repaint, would be flaky.
fn settle(h: &mut H) {
    for _ in 0..4 {
        h.step();
    }
}

/// Tall window with every panel open and expanded.
fn open() -> H {
    let mut prefs = Prefs::default();
    for slot in &mut prefs.layout.panels {
        slot.open = slot.kind != PanelKind::Vehicle;
        slot.collapsed = false;
    }
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, 2400.0))
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(prefs, None),
        );
    common::create_project(&mut h);
    settle(&mut h);
    h
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

/// Imported files: the project's assets other than its vehicle templates.
fn imported(h: &H) -> Vec<std::sync::Arc<tp_core::Asset>> {
    let project = &ws(h).project;
    let templates: Vec<_> = project.template_assets().collect();
    project
        .assets
        .values()
        .filter(|a| !templates.contains(&a.id))
        .cloned()
        .collect()
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn obj(h: &H, id: ObjectId) -> Object {
    (**ws(h).project.surface().get(id).expect("object")).clone()
}

fn content(h: &H, id: ObjectId) -> String {
    obj(h, id).text.expect("text").content
}

fn screen(h: &H, x: f64, y: f64) -> Pos2 {
    ws(h)
        .screen_map(1.0)
        .expect("canvas")
        .to_screen(Point::new(x, y))
}

fn set_tool(h: &mut H, tool: Tool) {
    ws_mut(h).tool = tool;
    settle(h);
}

fn click_at(h: &mut H, at: Pos2, mods: Modifiers) {
    h.event(Event::ModifiersChanged(mods));
    h.event(Event::PointerMoved(at));
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: mods,
        });
    }
    h.step();
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    settle(h);
}

fn double_click_at(h: &mut H, at: Pos2) {
    h.event(Event::PointerMoved(at));
    for k in 0..2 {
        h.input_mut().time = Some(100.0 + 0.1 * f64::from(k));
        for pressed in [true, false] {
            h.event(Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        h.step();
    }
    h.input_mut().time = None;
    settle(h);
}

fn drag(h: &mut H, from: Pos2, to: Pos2, mods: Modifiers) {
    h.event(Event::ModifiersChanged(mods));
    h.event(Event::PointerMoved(from));
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: mods,
    });
    h.step();
    for i in 1..=8 {
        h.event(Event::PointerMoved(from + (to - from) * (i as f32 / 8.0)));
        h.step();
    }
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: mods,
    });
    h.step();
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    settle(h);
}

/// Types like a keyboard: a key press and the text it produces.
fn type_text(h: &mut H, text: &str) {
    for c in text.chars() {
        if let Some(key) = Key::from_name(&c.to_ascii_uppercase().to_string()) {
            h.event(Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            });
        }
        h.event(Event::Text(c.to_string()));
        h.step();
    }
    settle(h);
}

/// Creates a text with the Text tool at document point (x, y).
fn create_text(h: &mut H, x: f64, y: f64, text: &str) -> ObjectId {
    set_tool(h, Tool::Text);
    click_at(h, screen(h, x, y), Modifiers::NONE);
    type_text(h, text);
    let id = ws(h).editing_text().expect("editing");
    h.key_press(Key::Escape);
    settle(h);
    id
}

fn place_bytes(h: &mut H, files: Vec<(&str, Vec<u8>)>, at: Option<Point>) {
    let files = files
        .into_iter()
        .map(|(name, bytes)| read_bytes(name, bytes))
        .collect();
    let now = h.ctx.input(|i| i.time);
    ws_mut(h).place_files(files, at, now);
    settle(h);
}

const BLUE: [u8; 4] = [20, 60, 220, 255];

// ---------------------------------------------------------------- text tool

#[test]
fn click_and_type() {
    let mut h = open();
    set_tool(&mut h, Tool::Text);
    let at = screen(&h, 800.0, 900.0);
    click_at(&mut h, at, Modifiers::NONE);
    let id = ws(&h).editing_text().expect("editing a new text");
    type_text(&mut h, "ACE Logistics");
    assert_eq!(content(&h, id), "ACE Logistics");
    assert_eq!(ws(&h).selection, vec![id]);
    assert_eq!(ws(&h).tool, Tool::Text);
    // The anchor (left of the first baseline) stays at the click point.
    let object = obj(&h, id);
    let anchor = ws_mut(&mut h).text.anchor(&object).unwrap();
    let click = ws(&h).screen_map(1.0).unwrap().to_doc(at);
    assert!((anchor - click).hypot() < 1e-6, "{anchor:?} {click:?}");
    assert!(object.frame.size.width > 500.0);
}

#[test]
fn empty_text_is_removed() {
    let mut h = open();
    set_tool(&mut h, Tool::Text);
    let p = screen(&h, 800.0, 900.0);
    click_at(&mut h, p, Modifiers::NONE);
    assert!(ws(&h).is_editing_text());
    h.key_press(Key::Escape);
    settle(&mut h);
    assert!(ws(&h).project.surface().objects.is_empty());
    assert!(!ws(&h).history.can_undo());
}

#[test]
fn escape_leaves_edit_mode() {
    let mut h = open();
    let id = create_text(&mut h, 800.0, 900.0, "ACE");
    assert!(!ws(&h).is_editing_text());
    assert_eq!(ws(&h).selection, vec![id]);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-create-text"));
}

#[test]
fn typing_a_tool_letter_while_editing() {
    let mut h = open();
    set_tool(&mut h, Tool::Text);
    let p = screen(&h, 800.0, 900.0);
    click_at(&mut h, p, Modifiers::NONE);
    let id = ws(&h).editing_text().unwrap();
    type_text(&mut h, "V");
    assert_eq!(content(&h, id), "V");
    assert_eq!(ws(&h).tool, Tool::Text);
    // Delete/Backspace edit the text, not the selection.
    h.key_press(Key::Backspace);
    settle(&mut h);
    assert_eq!(content(&h, id), "");
    assert!(ws(&h).project.surface().get(id).is_some());
}

#[test]
fn select_all_and_replace() {
    let mut h = open();
    let id = create_text(&mut h, 800.0, 900.0, "Old name");
    set_tool(&mut h, Tool::Select);
    let center = obj(&h, id).frame.center;
    let p = screen(&h, center.x, center.y);
    double_click_at(&mut h, p);
    assert_eq!(ws(&h).editing_text(), Some(id));
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    settle(&mut h);
    type_text(&mut h, "New name");
    assert_eq!(content(&h, id), "New name");
    // Cmd+A did not select every object either.
    assert_eq!(ws(&h).selection, vec![id]);
}

#[test]
fn new_line() {
    let mut h = open();
    set_tool(&mut h, Tool::Text);
    let p = screen(&h, 800.0, 900.0);
    click_at(&mut h, p, Modifiers::NONE);
    let id = ws(&h).editing_text().unwrap();
    type_text(&mut h, "Line 1");
    h.key_press(Key::Enter);
    settle(&mut h);
    type_text(&mut h, "Line 2");
    assert_eq!(content(&h, id), "Line 1\nLine 2");
    let object = obj(&h, id);
    let layout = ws_mut(&mut h)
        .text
        .block_layout(object.text.as_ref().unwrap());
    assert_eq!(layout.lines.len(), 2);
}

#[test]
fn undo_an_editing_session() {
    let mut h = open();
    let id = create_text(&mut h, 800.0, 900.0, "ACE");
    set_tool(&mut h, Tool::Select);
    h.key_press(Key::Enter);
    settle(&mut h);
    assert_eq!(ws(&h).editing_text(), Some(id));
    type_text(&mut h, " Logistics");
    // Undo inside the session.
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    settle(&mut h);
    assert_eq!(content(&h, id), "ACE");
    type_text(&mut h, " Logistics");
    h.key_press(Key::Escape);
    settle(&mut h);
    assert_eq!(content(&h, id), "ACE Logistics");
    assert_eq!(ws(&h).history.undo_label(), Some("cmd-edit-text"));
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    settle(&mut h);
    assert_eq!(content(&h, id), "ACE");
}

#[test]
fn clicking_outside_ends_editing() {
    let mut h = open();
    set_tool(&mut h, Tool::Select);
    let id = create_text(&mut h, 800.0, 900.0, "ACE");
    set_tool(&mut h, Tool::Select);
    h.key_press(Key::Enter);
    settle(&mut h);
    assert!(ws(&h).is_editing_text());
    let p = screen(&h, 3000.0, 3000.0);
    click_at(&mut h, p, Modifiers::NONE);
    assert!(!ws(&h).is_editing_text());
    assert!(ws(&h).project.surface().get(id).is_some());
}

#[test]
fn anchor_stays_fixed_for_centered_text() {
    let mut h = open();
    ws_mut(&mut h).text_style.align = TextAlign::Center;
    set_tool(&mut h, Tool::Text);
    let p = screen(&h, 1500.0, 900.0);
    click_at(&mut h, p, Modifiers::NONE);
    let id = ws(&h).editing_text().unwrap();
    type_text(&mut h, "A");
    let before = obj(&h, id).frame.center;
    type_text(&mut h, "BCDEF");
    let after = obj(&h, id).frame.center;
    // Centered text grows on both sides: its center stays put.
    assert!((after.x - before.x).abs() < 1e-6);
}

// -------------------------------------------------------- character settings

#[test]
fn change_size_of_two_texts() {
    let mut h = open();
    let a = create_text(&mut h, 600.0, 600.0, "ACE");
    let b = create_text(&mut h, 600.0, 1200.0, "LOGISTICS");
    ws_mut(&mut h).selection = vec![a, b];
    settle(&mut h);
    h.get_by_role_and_label(Role::TextInput, "Font size")
        .focus();
    settle(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, "Font size")
        .type_text("300");
    settle(&mut h);
    h.key_press(Key::Enter);
    settle(&mut h);
    for id in [a, b] {
        assert_eq!(obj(&h, id).text.unwrap().style.size, 300.0);
    }
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    settle(&mut h);
    for id in [a, b] {
        assert_eq!(obj(&h, id).text.unwrap().style.size, 200.0);
    }
}

#[test]
fn centered_multi_line_text() {
    let mut h = open();
    set_tool(&mut h, Tool::Text);
    let p = screen(&h, 1000.0, 900.0);
    click_at(&mut h, p, Modifiers::NONE);
    let id = ws(&h).editing_text().unwrap();
    type_text(&mut h, "WIDE LINE");
    h.key_press(Key::Enter);
    type_text(&mut h, "ab");
    h.key_press(Key::Escape);
    settle(&mut h);
    h.get_by_label("Align center").click();
    settle(&mut h);
    let object = obj(&h, id);
    assert_eq!(object.text.as_ref().unwrap().style.align, TextAlign::Center);
    let layout = ws_mut(&mut h)
        .text
        .block_layout(object.text.as_ref().unwrap());
    let centers: Vec<f64> = layout.lines.iter().map(|l| l.x + l.width / 2.0).collect();
    assert!((centers[0] - centers[1]).abs() < 0.5, "{centers:?}");
    assert_eq!(ws(&h).history.undo_label(), Some("undo-change-alignment"));
}

#[test]
fn outlined_lettering() {
    let mut h = open();
    let id = create_text(&mut h, 800.0, 900.0, "ACE");
    let mut o = obj(&h, id);
    o.fill = Rgba::rgb(255, 255, 255).into();
    o.stroke = Some(StrokeStyle {
        paint: Rgba::rgb(0, 0, 0).into(),
        width: 6.0,
        ..Default::default()
    });
    ws_mut(&mut h).project.surface_mut().replace(&[o]);
    settle(&mut h);
    let object = ws(&h).project.surface().get(id).unwrap().clone();
    let mesh = ws_mut(&mut h).text.mesh(&object, 0, 0.25);
    assert!(mesh.stroke.as_ref().is_some_and(|m| !m.is_empty()));
}

#[test]
fn search_a_font() {
    let mut h = open();
    create_text(&mut h, 800.0, 900.0, "ACE");
    h.get_by_label_contains("Font family").click();
    settle(&mut h);
    h.get_by_role_and_label(Role::TextInput, "Search fonts")
        .type_text("bebas");
    settle(&mut h);
    assert!(h.query_by_label("Bebas Neue").is_some());
    assert!(h.query_by_label("Oswald").is_none());
    assert!(h.query_by_label("Inter").is_none());
    // Keyboard choice.
    h.key_press(Key::Enter);
    settle(&mut h);
    let id = ws(&h).selection[0];
    let style = obj(&h, id).text.unwrap().style;
    assert_eq!(style.family, "Bebas Neue");
    // Bebas Neue only has a regular weight: the nearest one is used.
    assert_eq!(style.weight, 400);
    assert!(ws(&h).panels.font_picker.is_none());
}

#[test]
fn font_not_installed() {
    let mut h = open();
    let id = create_text(&mut h, 800.0, 900.0, "ACE");
    let mut o = obj(&h, id);
    o.text.as_mut().unwrap().style.family = "Some Missing Font".into();
    ws_mut(&mut h).project.surface_mut().replace(&[o]);
    settle(&mut h);
    assert!(
        h.query_by_label_contains("Font not found: Some Missing Font")
            .is_some()
    );
    let object = obj(&h, id);
    let layout = ws_mut(&mut h)
        .text
        .block_layout(object.text.as_ref().unwrap());
    assert!(layout.missing_font && !layout.glyphs.is_empty());
    // The original name is kept.
    assert_eq!(object.text.unwrap().style.family, "Some Missing Font");
}

// ----------------------------------------------------------------- images

#[test]
fn place_a_png() {
    let mut h = open();
    place_bytes(&mut h, vec![("logo.png", solid_png(800, 400, BLUE))], None);
    let selected = ws(&h).selected_objects();
    assert_eq!(selected.len(), 1);
    let image = &selected[0];
    assert_eq!(image.name, "logo");
    assert!(matches!(image.kind, ShapeKind::Image { .. }));
    assert_eq!(image.frame.size, Size::new(800.0, 400.0));
    assert_eq!(image.frame.center, ws(&h).view_center());
    assert_eq!(ws(&h).history.undo_label(), Some("undo-place"));
}

#[test]
fn large_image_is_scaled_down() {
    let mut h = open();
    assert_eq!(ws(&h).project.surface().size, 4096.0);
    place_bytes(&mut h, vec![("big.png", solid_png(6000, 3000, BLUE))], None);
    assert_eq!(
        ws(&h).selected_objects()[0].frame.size,
        Size::new(2048.0, 1024.0)
    );
}

#[test]
fn unsupported_file() {
    let mut h = open();
    place_bytes(
        &mut h,
        vec![
            ("anim.gif", b"GIF89a\x01\x00\x01\x00".to_vec()),
            ("logo.png", solid_png(80, 40, BLUE)),
        ],
        None,
    );
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
    let hint = ws(&h).hint.as_ref().expect("message shown");
    assert_eq!(hint.text, "anim.gif: unsupported format");
}

#[test]
fn same_logo_twice() {
    let mut h = open();
    let png = solid_png(80, 40, BLUE);
    place_bytes(&mut h, vec![("logo.png", png.clone())], None);
    place_bytes(&mut h, vec![("logo.png", png)], None);
    assert_eq!(ws(&h).project.surface().objects.len(), 2);
    assert_eq!(imported(&h).len(), 1);
    assert!(h.query_by_label("80 × 40 px · 2 uses").is_some());
}

#[test]
fn undo_an_import() {
    let mut h = open();
    place_bytes(&mut h, vec![("logo.png", solid_png(80, 40, BLUE))], None);
    assert!(h.query_by_label("Asset logo").is_some());
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    settle(&mut h);
    assert!(ws(&h).project.surface().objects.is_empty());
    assert!(imported(&h).is_empty());
    assert!(h.query_by_label("No assets").is_some());
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

#[test]
fn drop_two_files() {
    let mut h = open();
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="100"><rect width="300" height="100" fill="red"/></svg>"#;
    let at = screen(&h, 1200.0, 1500.0);
    h.event(Event::PointerMoved(at));
    h.step();
    h.input_mut().dropped_files = vec![
        Arc::new(TestFile {
            path: "/tmp/logo.svg".into(),
            bytes: svg.to_vec(),
        }),
        Arc::new(TestFile {
            path: "/tmp/badge.jpg".into(),
            bytes: {
                let mut out = Vec::new();
                image::RgbImage::from_pixel(40, 40, image::Rgb([200, 0, 0]))
                    .write_to(
                        &mut std::io::Cursor::new(&mut out),
                        image::ImageFormat::Jpeg,
                    )
                    .unwrap();
                out
            },
        }),
    ];
    h.step();
    h.input_mut().dropped_files.clear();
    settle(&mut h);
    let selected = ws(&h).selected_objects();
    assert_eq!(selected.len(), 2);
    let names: Vec<_> = selected.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(names, ["logo", "badge"]);
    let drop = ws(&h).screen_map(1.0).unwrap().to_doc(at);
    assert!((selected[0].frame.center - drop).hypot() < 1e-6);
}

#[test]
fn image_tool_click_places_at_the_click() {
    let mut h = open();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("logo.png");
    std::fs::write(&path, solid_png(80, 40, BLUE)).unwrap();
    h.state_mut().dialogs = Box::new(tp_app::file_dialogs::ScriptedDialogs {
        images: [vec![path]].into(),
        ..Default::default()
    });
    set_tool(&mut h, Tool::Image);
    let at = screen(&h, 900.0, 700.0);
    click_at(&mut h, at, Modifiers::NONE);
    let image = &ws(&h).selected_objects()[0];
    assert_eq!(image.name, "logo");
    let click = ws(&h).screen_map(1.0).unwrap().to_doc(at);
    assert!((image.frame.center - click).hypot() < 1e-6);
}

#[test]
fn reset_a_distorted_image() {
    let mut h = open();
    place_bytes(
        &mut h,
        vec![("logo.png", solid_png(800, 400, BLUE))],
        Some(Point::new(1500.0, 1500.0)),
    );
    let mut o = ws(&h).selected_objects()[0].clone();
    o.frame.size = Size::new(800.0, 800.0);
    ws_mut(&mut h).project.surface_mut().replace(&[o]);
    settle(&mut h);
    assert!(
        h.query_by_label("Fill color").is_none(),
        "no fill for images"
    );
    h.get_by_label("Reset Size").click();
    settle(&mut h);
    let o = &ws(&h).selected_objects()[0];
    assert_eq!(o.frame.size, Size::new(800.0, 400.0));
    assert_eq!(o.frame.center, Point::new(1500.0, 1500.0));
}

#[test]
fn proportional_resize_with_shift() {
    let mut h = open();
    place_bytes(
        &mut h,
        vec![("logo.png", solid_png(800, 400, BLUE))],
        Some(Point::new(1500.0, 1500.0)),
    );
    set_tool(&mut h, Tool::Select);
    let corner = screen(&h, 1900.0, 1700.0);
    let to = screen(&h, 2300.0, 1750.0);
    drag(&mut h, corner, to, Modifiers::SHIFT);
    let size = ws(&h).selected_objects()[0].frame.size;
    assert!(size.width > 800.0);
    assert!((size.width / size.height - 2.0).abs() < 1e-6, "{size:?}");
}

// ------------------------------------------------------------ assets panel

#[test]
fn asset_listed_after_import() {
    let mut h = open();
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="100"><circle cx="50" cy="50" r="40" fill="blue"/></svg>"#;
    place_bytes(&mut h, vec![("logo.svg", svg.to_vec())], None);
    assert!(h.query_by_label("Asset logo").is_some());
    assert!(h.query_by_label("Vector · 1 use").is_some());
}

#[test]
fn remove_an_unused_asset() {
    let mut h = open();
    place_bytes(&mut h, vec![("badge.png", solid_png(80, 40, BLUE))], None);
    h.key_press(Key::Delete);
    settle(&mut h);
    assert!(ws(&h).project.surface().objects.is_empty());
    h.get_by_label("Remove Asset").click();
    settle(&mut h);
    assert!(imported(&h).is_empty());
    assert!(h.query_by_label("Asset badge").is_none());
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    settle(&mut h);
    assert_eq!(imported(&h).len(), 1);
    assert!(h.query_by_label("Asset badge").is_some());
}

#[test]
fn remove_is_disabled_while_used() {
    let mut h = open();
    place_bytes(&mut h, vec![("badge.png", solid_png(80, 40, BLUE))], None);
    let remove = h.get_by_label("Remove Asset");
    assert!(remove.accesskit_node().is_disabled());
    remove.click();
    settle(&mut h);
    assert_eq!(imported(&h).len(), 1);
}

#[test]
fn place_and_rename_from_the_assets_panel() {
    let mut h = open();
    place_bytes(&mut h, vec![("badge.png", solid_png(80, 40, BLUE))], None);
    h.get_by_label("Place Asset").click();
    settle(&mut h);
    assert_eq!(ws(&h).project.surface().objects.len(), 2);
    h.get_by_label("Rename Asset").click();
    settle(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, "Asset name")
        .type_text("ACE badge");
    settle(&mut h);
    h.key_press(Key::Enter);
    settle(&mut h);
    let asset = imported(&h)[0].clone();
    assert_eq!(asset.name, "ACE badge");
    // Existing objects keep their names.
    assert!(
        ws(&h)
            .project
            .surface()
            .objects
            .iter()
            .all(|o| o.name == "badge")
    );
}

#[test]
fn empty_assets_panel_offers_place() {
    let mut h = open();
    assert!(h.query_by_label("No assets").is_some());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("logo.png");
    std::fs::write(&path, solid_png(80, 40, BLUE)).unwrap();
    h.state_mut().dialogs = Box::new(tp_app::file_dialogs::ScriptedDialogs {
        images: [vec![path]].into(),
        ..Default::default()
    });
    h.get_by_label("Place…").click();
    settle(&mut h);
    assert_eq!(imported(&h).len(), 1);
}

// --------------------------------------------------------------- eyedropper

#[test]
fn pick_a_color_from_a_logo() {
    let mut h = open();
    place_bytes(
        &mut h,
        vec![("logo.png", solid_png(400, 200, BLUE))],
        Some(Point::new(2500.0, 2500.0)),
    );
    let text = create_text(&mut h, 600.0, 600.0, "ACE");
    ws_mut(&mut h).selection = vec![text];
    set_tool(&mut h, Tool::Eyedropper);
    let p = screen(&h, 2500.0, 2500.0);
    click_at(&mut h, p, Modifiers::NONE);
    assert_eq!(
        obj(&h, text).fill,
        tp_core::document::Paint::from(Rgba::rgb(BLUE[0], BLUE[1], BLUE[2]))
    );
}

/// Frame-time budget with 50 stroked texts while panning (release only).
#[test]
#[ignore = "performance check; run with --release -- --ignored"]
fn pan_with_50_texts_stays_fast() {
    pan_with_50_texts(tp_core::document::StrokeAlign::Center);
}

/// The same with outside strokes (stroke regions clipped by the letters).
#[test]
#[ignore = "performance check; run with --release -- --ignored"]
fn pan_with_50_outside_stroked_texts_stays_fast() {
    pan_with_50_texts(tp_core::document::StrokeAlign::Outside);
}

/// The same with gradient fills and strokes (ramp textures).
#[test]
#[ignore = "performance check; run with --release -- --ignored"]
fn pan_with_50_gradient_texts_stays_fast() {
    pan_with_50_texts_with(tp_core::document::StrokeAlign::Outside, true);
}

fn pan_with_50_texts(align: tp_core::document::StrokeAlign) {
    pan_with_50_texts_with(align, false);
}

/// The same in French (translated labels every frame).
#[test]
#[ignore = "performance check; run with --release -- --ignored"]
fn pan_with_50_texts_in_french_stays_fast() {
    pan_with_50_texts_in(
        tp_core::document::StrokeAlign::Center,
        false,
        tp_i18n::Language::French,
    );
}

fn pan_with_50_texts_with(align: tp_core::document::StrokeAlign, gradients: bool) {
    pan_with_50_texts_in(align, gradients, tp_i18n::Language::English);
}

fn pan_with_50_texts_in(
    align: tp_core::document::StrokeAlign,
    gradients: bool,
    language: tp_i18n::Language,
) {
    use tp_core::document::{CharStyle, TextBlock};
    use tp_core::document::{ColorStop, Gradient, GradientKind, Paint};
    let mut h = open();
    h.state_mut().prefs.set_language(Some(language));
    {
        let ws = ws_mut(&mut h);
        for i in 0..50 {
            let anchor = Point::new(
                100.0 + f64::from(i % 5) * 800.0,
                200.0 + f64::from(i / 5) * 380.0,
            );
            let mut o = Object::text(
                ObjectId(0),
                TextBlock::new("ACE Logistics 24", CharStyle::default()),
                anchor,
            );
            o.stroke = Some(StrokeStyle {
                paint: Rgba::rgb(0, 0, 0).into(),
                width: 6.0,
                align,
                ..Default::default()
            });
            o.frame.rotation_deg = f64::from(i % 7) * 5.0;
            ws.text.place_at(&mut o, anchor);
            if gradients {
                // Five distinct gradients: five ramp textures of each kind.
                let tint = (i % 5) as u8 * 40;
                let stops = [
                    ColorStop::new(0.0, Rgba::rgb(255, tint, 0)),
                    ColorStop::new(1.0, Rgba::rgb(0, tint, 255)),
                ];
                o.fill = Paint::Gradient(Gradient::new(GradientKind::Radial, &stops));
                if let Some(s) = &mut o.stroke {
                    s.paint = Paint::Gradient(Gradient::new(GradientKind::Linear, &stops));
                }
            }
            ws.project.add(o);
        }
    }
    let align = (align, gradients, language);
    let first = std::time::Instant::now();
    h.step();
    println!(
        "{align:?}: first frame (meshes built): {:?}",
        first.elapsed()
    );
    settle(&mut h);
    let frames = 60;
    let start = std::time::Instant::now();
    for _ in 0..frames {
        h.event(Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: Vec2::new(5.0, 2.0),
            modifiers: Modifiers::NONE,
            phase: egui::TouchPhase::Move,
        });
        h.step();
    }
    let per_frame = start.elapsed() / frames;
    println!("{align:?}: average frame: {per_frame:?}");
    assert!(
        per_frame < std::time::Duration::from_millis(16),
        "{per_frame:?}"
    );
}
