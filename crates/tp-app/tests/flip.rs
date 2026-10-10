//! Headless tests for flip-objects: Flip Horizontal and Flip Vertical, and
//! mirrored images and texts on the canvas (eyedropper, editing, outlines).

mod common;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::commands::CommandId;
use tp_app::import::read_bytes;
use tp_app::prefs::Prefs;
use tp_app::tool::Tool;
use tp_app::workspace::Workspace;
use tp_core::document::{FlipAxis, Object, ObjectId, Paint, Rgba};
use tp_core::kurbo::{Point, Shape};

type H = Harness<'static, AppState>;

/// Runs a few frames (background threads make `Harness::run` flaky).
fn settle(h: &mut H) {
    for _ in 0..4 {
        h.step();
    }
}

/// The sample truck project in a tall window, so the whole inspector
/// shows; snapping off.
fn open() -> H {
    let mut prefs = Prefs::default();
    prefs.view_aids.snapping = false;
    let mut h = common::builder()
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

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

fn obj(h: &H, id: ObjectId) -> Object {
    (**ws(h).project.surface().get(id).expect("object")).clone()
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

fn click_at(h: &mut H, at: Pos2) {
    h.event(Event::PointerMoved(at));
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.step();
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

/// Creates a text with the Text tool at (x, y) and leaves the selection on it.
fn create_text(h: &mut H, x: f64, y: f64, text: &str) -> ObjectId {
    set_tool(h, Tool::Text);
    click_at(h, screen(h, x, y));
    type_text(h, text);
    let id = ws(h).editing_text().expect("editing");
    h.key_press(Key::Escape);
    settle(h);
    set_tool(h, Tool::Select);
    id
}

/// A 400×200 image, red on its left half and blue on its right half,
/// centered at (x, y); selected.
fn place_halves(h: &mut H, x: f64, y: f64) -> ObjectId {
    let mut pixels = image::RgbaImage::from_pixel(40, 20, image::Rgba([255, 0, 0, 255]));
    for px in 20..40 {
        for py in 0..20 {
            pixels.put_pixel(px, py, image::Rgba([0, 0, 255, 255]));
        }
    }
    let mut bytes = Vec::new();
    pixels
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    let now = h.ctx.input(|i| i.time);
    ws_mut(h).place_files(
        vec![read_bytes("halves.png", bytes)],
        Some(Point::new(x, y)),
        now,
    );
    settle(h);
    let id = ws(h).selection[0];
    let mut o = obj(h, id);
    o.frame.size = tp_core::kurbo::Size::new(400.0, 200.0);
    ws_mut(h).project.surface_mut().replace(&[o]);
    settle(h);
    id
}

/// Sets the mirrored state of `id` directly.
fn mirror(h: &mut H, id: ObjectId) {
    let mut o = obj(h, id);
    o.mirrored = true;
    ws_mut(h).project.surface_mut().replace(&[o]);
    settle(h);
}

// ------------------------------------------------------- mirrored objects

#[test]
fn the_eyedropper_picks_the_pixel_shown() {
    let mut h = open();
    let image = place_halves(&mut h, 2500.0, 2500.0);
    mirror(&mut h, image);
    let text = create_text(&mut h, 600.0, 600.0, "ACE");
    ws_mut(&mut h).selection = vec![text];
    set_tool(&mut h, Tool::Eyedropper);
    // The left half of the mirrored image shows its blue half.
    let at = screen(&h, 2400.0, 2500.0);
    click_at(&mut h, at);
    assert_eq!(obj(&h, text).fill, Paint::from(Rgba::rgb(0, 0, 255)));
}

#[test]
fn typing_at_the_end_of_a_mirrored_text() {
    let mut h = open();
    let id = create_text(&mut h, 800.0, 900.0, "TRANS");
    mirror(&mut h, id);
    let before = obj(&h, id);
    let anchor = ws_mut(&mut h).text.anchor(&before).unwrap();
    // Mirrored, the end of the text is on its left.
    let b = before.frame.bounding_box();
    let at = screen(&h, b.x0 + 2.0, b.center().y);
    double_click_at(&mut h, at);
    assert_eq!(ws(&h).editing_text(), Some(id));
    type_text(&mut h, "PORT");
    h.key_press(Key::Escape);
    settle(&mut h);
    let after = obj(&h, id);
    assert_eq!(after.text.as_ref().unwrap().content, "TRANSPORT");
    assert!(after.mirrored);
    let kept = ws_mut(&mut h).text.anchor(&after).unwrap();
    assert!((kept - anchor).hypot() < 1e-6, "{kept:?} vs {anchor:?}");
    // The anchor is the right edge on screen: the text grew to the left.
    let grown = after.frame.bounding_box();
    assert!((grown.x1 - b.x1).abs() < 1.0 && grown.x0 < b.x0 - 100.0);
}

#[test]
fn clicking_the_start_of_a_mirrored_text() {
    let mut h = open();
    let id = create_text(&mut h, 800.0, 900.0, "TRANS");
    mirror(&mut h, id);
    let b = obj(&h, id).frame.bounding_box();
    let at = screen(&h, b.x1 - 2.0, b.center().y);
    double_click_at(&mut h, at);
    type_text(&mut h, "X");
    assert_eq!(obj(&h, id).text.unwrap().content, "XTRANS");
}

#[test]
fn outlining_a_mirrored_text() {
    let mut h = open();
    let id = create_text(&mut h, 800.0, 900.0, "LT");
    mirror(&mut h, id);
    let text = obj(&h, id);
    let outline = ws_mut(&mut h).text.outline(&text).bounding_box();
    ws_mut(&mut h).selection = vec![id];
    let now = h.ctx.input(|i| i.time);
    ws_mut(&mut h).create_outlines(now);
    settle(&mut h);
    let group = obj(&h, id);
    assert!(group.is_group());
    let (l, t) = (&group.children[0], &group.children[1]);
    assert_eq!((l.name.as_str(), t.name.as_str()), ("L", "T"));
    assert!(
        l.bounding_box().center().x > t.bounding_box().center().x,
        "the L is on the right"
    );
    let shown = l.path().bounding_box().union(t.path().bounding_box());
    assert!(
        (shown.x0 - outline.x0).abs() < 1e-3 && (shown.x1 - outline.x1).abs() < 1e-3,
        "{shown:?} vs {outline:?}"
    );
}

// --------------------------------------------------------------- commands

#[test]
fn shift_h_flips_a_logo_and_undo_restores_it() {
    let mut h = open();
    let image = place_halves(&mut h, 1500.0, 1200.0);
    let before = obj(&h, image).frame;
    h.key_press_modifiers(Modifiers::SHIFT, Key::H);
    settle(&mut h);
    let flipped = obj(&h, image);
    assert!(flipped.mirrored);
    assert!(flipped.frame.center.distance(before.center) < 1e-9);
    assert_eq!(flipped.frame.size, before.size);
    assert_eq!(ws(&h).history.undo_label(), Some("op-flip-horizontal"));
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    settle(&mut h);
    assert!(!obj(&h, image).mirrored);
}

#[test]
fn flip_vertical_from_the_object_menu() {
    let mut h = open();
    let id = create_text(&mut h, 800.0, 900.0, "TRANS");
    ws_mut(&mut h).selection = vec![id];
    settle(&mut h);
    h.get_by_label("Object").click();
    settle(&mut h);
    // The menu's item; the inspector's Layout section has the same button.
    h.get_all_by_label("Flip Vertical").last().unwrap().click();
    settle(&mut h);
    let text = obj(&h, id);
    assert!(text.mirrored);
    assert_eq!(text.frame.rotation_deg, 180.0);
    assert_eq!(ws(&h).history.undo_label(), Some("op-flip-vertical"));
}

#[test]
fn flip_from_the_inspector() {
    let mut h = open();
    let image = place_halves(&mut h, 1500.0, 1200.0);
    h.get_by_role_and_label(Role::Button, "Flip Horizontal")
        .click();
    settle(&mut h);
    assert!(obj(&h, image).mirrored);
    assert_eq!(
        h.state().prefs.layout.inspector_width,
        tp_ui::tokens::size::INSPECTOR_DEFAULT,
        "the buttons fit in the column's default width"
    );
}

#[test]
fn flipping_needs_a_selection() {
    let mut h = open();
    assert!(ws(&h).selection.is_empty());
    let context = h.state().edit_context();
    for axis in FlipAxis::ALL {
        assert!(!tp_app::state::is_enabled(CommandId::Flip(axis), &context));
        let reason = tp_app::state::disabled_reason_for(CommandId::Flip(axis), &context);
        assert_eq!(
            tp_i18n::tr(reason.unwrap()),
            "Select one or more objects first."
        );
    }
    h.get_by_label("Object").click();
    settle(&mut h);
    for label in ["Flip Horizontal", "Flip Vertical"] {
        assert!(
            h.get_by_label(label).accesskit_node().is_disabled(),
            "{label}"
        );
    }
    // Shift+H does nothing either.
    h.key_press(Key::Escape);
    settle(&mut h);
    let steps = ws(&h).history.len();
    h.key_press_modifiers(Modifiers::SHIFT, Key::H);
    settle(&mut h);
    assert_eq!(ws(&h).history.len(), steps);
}
