//! Headless tests for canvas-core: navigation, shape tools, selection,
//! transforms, history and object commands.

mod common;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, MouseWheelUnit, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::tool::Tool;
use tp_app::workspace::Workspace;
use tp_core::document::{Frame, Object, ObjectId, ShapeKind};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

fn open() -> H {
    let mut h = common::harness();
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

fn screen(h: &H, x: f64, y: f64) -> Pos2 {
    ws(h)
        .screen_map(1.0)
        .expect("canvas shown")
        .to_screen(Point::new(x, y))
}

fn set_tool(h: &mut H, tool: Tool) {
    ws_mut(h).tool = tool;
    h.run();
}

/// Adds a rectangle directly (bypassing tools) and returns its id.
fn add_rect(h: &mut H, x: f64, y: f64, w: f64, hgt: f64) -> ObjectId {
    let id = ws_mut(h).project.add(Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(x, y), Size::new(w, hgt), 0.0),
    ));
    h.run();
    id
}

/// Presses at `from`, moves in steps to `to` and releases, with `mods` held.
fn drag(h: &mut H, from: Pos2, to: Pos2, mods: Modifiers) {
    drag_with(h, from, to, mods, PointerButton::Primary);
}

fn drag_with(h: &mut H, from: Pos2, to: Pos2, mods: Modifiers, button: PointerButton) {
    h.event(Event::ModifiersChanged(mods));
    h.event(Event::PointerMoved(from));
    h.event(Event::PointerButton {
        pos: from,
        button,
        pressed: true,
        modifiers: mods,
    });
    h.step();
    for i in 1..=6 {
        let t = i as f32 / 6.0;
        h.event(Event::PointerMoved(from + (to - from) * t));
        h.step();
    }
    h.event(Event::PointerButton {
        pos: to,
        button,
        pressed: false,
        modifiers: mods,
    });
    h.step();
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.run();
}

fn click(h: &mut H, at: Pos2, mods: Modifiers) {
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
    h.run();
}

fn frame_of(h: &H, id: ObjectId) -> Frame {
    ws(h).project.surface().get(id).expect("object").frame
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1.0
}

// --- shape tools -------------------------------------------------------------

#[test]
fn drawing_a_rectangle() {
    let mut h = open();
    set_tool(&mut h, Tool::Rectangle);
    let (a, b) = (screen(&h, 100.0, 100.0), screen(&h, 500.0, 300.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let ws = ws(&h);
    assert_eq!(ws.project.surface().objects.len(), 1);
    let f = ws.project.surface().objects[0].frame;
    assert!(near(f.center.x, 300.0) && near(f.center.y, 200.0), "{f:?}");
    assert!(
        near(f.size.width, 400.0) && near(f.size.height, 200.0),
        "{f:?}"
    );
    assert_eq!(ws.selection.len(), 1);
    assert_eq!(ws.tool, Tool::Rectangle);
}

#[test]
fn click_without_drag_creates_nothing() {
    let mut h = open();
    set_tool(&mut h, Tool::Ellipse);
    let p = screen(&h, 1000.0, 1000.0);
    click(&mut h, p, Modifiers::NONE);
    assert!(ws(&h).project.surface().objects.is_empty());
}

#[test]
fn shift_draws_a_circle_and_alt_draws_from_center() {
    let mut h = open();
    set_tool(&mut h, Tool::Ellipse);
    let (a, b) = (screen(&h, 1000.0, 1000.0), screen(&h, 1600.0, 1200.0));
    drag(&mut h, a, b, Modifiers::SHIFT);
    let f = ws(&h).project.surface().objects[0].frame;
    assert!(near(f.size.width, f.size.height), "{f:?}");

    set_tool(&mut h, Tool::Rectangle);
    let (a, b) = (screen(&h, 500.0, 500.0), screen(&h, 600.0, 550.0));
    drag(&mut h, a, b, Modifiers::ALT);
    let f = ws(&h).project.surface().objects[1].frame;
    assert!(near(f.center.x, 500.0) && near(f.center.y, 500.0), "{f:?}");
    assert!(
        near(f.size.width, 200.0) && near(f.size.height, 100.0),
        "{f:?}"
    );
}

#[test]
fn escape_cancels_drawing() {
    let mut h = open();
    set_tool(&mut h, Tool::Rectangle);
    let a = screen(&h, 100.0, 100.0);
    h.event(Event::PointerMoved(a));
    h.event(Event::PointerButton {
        pos: a,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.event(Event::PointerMoved(a + Vec2::new(80.0, 60.0)));
    h.step();
    h.event(Event::PointerMoved(a + Vec2::new(90.0, 70.0)));
    h.step();
    assert!(ws(&h).gesture.is_active());
    h.key_press(Key::Escape);
    h.step();
    h.event(Event::PointerButton {
        pos: a + Vec2::new(90.0, 70.0),
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run();
    assert!(ws(&h).project.surface().objects.is_empty());
}

// --- selection -------------------------------------------------------------

#[test]
fn shift_click_toggles_selection() {
    let mut h = open();
    let a = add_rect(&mut h, 500.0, 500.0, 300.0, 300.0);
    let b = add_rect(&mut h, 1500.0, 500.0, 300.0, 300.0);
    let (pa, pb) = (screen(&h, 500.0, 500.0), screen(&h, 1500.0, 500.0));
    click(&mut h, pa, Modifiers::NONE);
    assert_eq!(ws(&h).selection, vec![a]);
    click(&mut h, pb, Modifiers::SHIFT);
    click(&mut h, pa, Modifiers::SHIFT);
    assert_eq!(ws(&h).selection, vec![b]);
    // Clicking empty canvas clears.
    let empty = screen(&h, 3000.0, 3000.0);
    click(&mut h, empty, Modifiers::NONE);
    assert!(ws(&h).selection.is_empty());
}

#[test]
fn marquee_selects_touched_objects() {
    let mut h = open();
    let a = add_rect(&mut h, 500.0, 500.0, 300.0, 300.0);
    let b = add_rect(&mut h, 1200.0, 500.0, 300.0, 300.0);
    let _c = add_rect(&mut h, 3000.0, 3000.0, 300.0, 300.0);
    let (from, to) = (screen(&h, 100.0, 100.0), screen(&h, 1100.0, 600.0));
    drag(&mut h, from, to, Modifiers::NONE);
    let mut sel = ws(&h).selection.clone();
    sel.sort();
    assert_eq!(sel, vec![a, b]);
}

#[test]
fn select_all_and_escape() {
    let mut h = open();
    add_rect(&mut h, 500.0, 500.0, 300.0, 300.0);
    add_rect(&mut h, 1500.0, 500.0, 300.0, 300.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    assert_eq!(ws(&h).selection.len(), 2);
    h.key_press(Key::Escape);
    h.run();
    assert!(ws(&h).selection.is_empty());
}

// --- transforms --------------------------------------------------------------

#[test]
fn dragging_moves_the_whole_selection_in_one_undo_step() {
    let mut h = open();
    let a = add_rect(&mut h, 500.0, 500.0, 300.0, 300.0);
    let b = add_rect(&mut h, 1500.0, 500.0, 300.0, 300.0);
    ws_mut(&mut h).selection = vec![a, b];
    h.run();
    let (from, to) = (screen(&h, 500.0, 500.0), screen(&h, 900.0, 500.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert!(near(frame_of(&h, a).center.x, 900.0));
    assert!(near(frame_of(&h, b).center.x, 1900.0));
    assert_eq!(ws(&h).history.undo_label(), Some("Move"));
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert!(near(frame_of(&h, a).center.x, 500.0));
    assert!(near(frame_of(&h, b).center.x, 1500.0));
}

#[test]
fn proportional_resize_from_corner_handle() {
    let mut h = open();
    let a = add_rect(&mut h, 1000.0, 1000.0, 1000.0, 500.0);
    ws_mut(&mut h).selection = vec![a];
    h.run();
    // Bottom-right handle at (1500, 1250); drag until width doubles.
    let (from, to) = (screen(&h, 1500.0, 1250.0), screen(&h, 2500.0, 1300.0));
    drag(&mut h, from, to, Modifiers::SHIFT);
    let f = frame_of(&h, a);
    assert!(
        near(f.size.width, 2000.0) && near(f.size.height, 1000.0),
        "{f:?}"
    );
    assert_eq!(ws(&h).history.undo_label(), Some("Resize"));
}

#[test]
fn shift_rotation_snaps_to_15_degrees() {
    let mut h = open();
    let a = add_rect(&mut h, 1000.0, 1000.0, 800.0, 400.0);
    ws_mut(&mut h).selection = vec![a];
    h.run();
    // Just outside the top-right corner (1400, 800), in the rotation zone.
    let corner = screen(&h, 1400.0, 800.0);
    let from = corner + Vec2::new(12.0, -12.0);
    let to = screen(&h, 1500.0, 1300.0);
    drag(&mut h, from, to, Modifiers::SHIFT);
    let rot = frame_of(&h, a).rotation_deg;
    assert!(rot.abs() > 1.0, "rotated: {rot}");
    assert!((rot / 15.0 - (rot / 15.0).round()).abs() < 1e-6, "{rot}");
    assert_eq!(ws(&h).history.undo_label(), Some("Rotate"));
}

#[test]
fn shift_arrow_nudges_ten_pixels() {
    let mut h = open();
    let a = add_rect(&mut h, 1000.0, 1000.0, 100.0, 100.0);
    ws_mut(&mut h).selection = vec![a];
    h.run();
    h.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    h.run();
    assert_eq!(frame_of(&h, a).center.x, 1010.0);
    h.key_press(Key::ArrowDown);
    h.run();
    assert_eq!(frame_of(&h, a).center.y, 1001.0);
}

// --- history ------------------------------------------------------------------

#[test]
fn undo_and_redo_a_creation() {
    let mut h = open();
    set_tool(&mut h, Tool::Rectangle);
    let (a, b) = (screen(&h, 100.0, 100.0), screen(&h, 500.0, 300.0));
    drag(&mut h, a, b, Modifiers::NONE);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert!(ws(&h).project.surface().objects.is_empty());
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    h.run();
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
    assert_eq!(ws(&h).selection.len(), 1);
    assert_eq!(
        h.state().workspace().unwrap().save_state(),
        tp_app::workspace::SaveState::Unsaved
    );
}

#[test]
fn zoom_is_not_undone() {
    let mut h = open();
    let a = add_rect(&mut h, 1000.0, 1000.0, 100.0, 100.0);
    ws_mut(&mut h).selection = vec![a];
    h.run();
    h.key_press(Key::ArrowRight);
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    h.run();
    let zoom = ws(&h).viewport.unwrap().zoom;
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(frame_of(&h, a).center.x, 1000.0);
    assert_eq!(ws(&h).viewport.unwrap().zoom, zoom);
}

#[test]
fn edit_menu_names_the_undo_step() {
    let mut h = open();
    let a = add_rect(&mut h, 1000.0, 1000.0, 1000.0, 500.0);
    ws_mut(&mut h).selection = vec![a];
    h.run();
    let (from, to) = (screen(&h, 1500.0, 1250.0), screen(&h, 1800.0, 1400.0));
    drag(&mut h, from, to, Modifiers::NONE);
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Undo Resize");
}

// --- object commands ------------------------------------------------------------

#[test]
fn duplicate_offsets_copy() {
    let mut h = open();
    let a = add_rect(&mut h, 100.0, 100.0, 50.0, 50.0);
    ws_mut(&mut h).selection = vec![a];
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::D);
    h.run();
    let sel = ws(&h).selection.clone();
    assert_eq!(sel.len(), 1);
    assert_ne!(sel[0], a);
    assert_eq!(frame_of(&h, sel[0]).center, Point::new(120.0, 120.0));
}

#[test]
fn paste_is_disabled_until_something_is_copied() {
    let mut h = open();
    let a = add_rect(&mut h, 100.0, 100.0, 50.0, 50.0);
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Paste").click();
    h.run();
    assert_eq!(ws(&h).project.surface().objects.len(), 1);

    ws_mut(&mut h).selection = vec![a];
    h.key_press_modifiers(Modifiers::COMMAND, Key::C);
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::V);
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::V);
    h.run();
    let objects = &ws(&h).project.surface().objects;
    assert_eq!(objects.len(), 3);
    assert_eq!(objects[1].frame.center, Point::new(100.0, 100.0));
    assert_eq!(objects[2].frame.center, Point::new(120.0, 120.0));
}

#[test]
fn delete_with_backspace_and_reorder() {
    let mut h = open();
    let a = add_rect(&mut h, 100.0, 100.0, 50.0, 50.0);
    let b = add_rect(&mut h, 100.0, 100.0, 50.0, 50.0);
    ws_mut(&mut h).selection = vec![a];
    h.key_press_modifiers(Modifiers::COMMAND, Key::CloseBracket);
    h.run();
    assert_eq!(ws(&h).project.surface().index_of(a), Some(1));
    h.key_press(Key::Backspace);
    h.run();
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
    assert!(ws(&h).project.surface().get(b).is_some());
}

#[test]
fn context_menu_on_object() {
    let mut h = open();
    let a = add_rect(&mut h, 1000.0, 1000.0, 400.0, 400.0);
    let p = screen(&h, 1000.0, 1000.0);
    let deletes_before = h.get_all_by_label("Delete").count();
    h.event(Event::PointerMoved(p));
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: p,
            button: PointerButton::Secondary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run();
    assert_eq!(ws(&h).selection, vec![a]);
    for label in ["Cut", "Copy", "Duplicate", "Bring Forward", "Send Backward"] {
        h.get_by_label(label);
    }
    // The Layers footer also has a Delete button: the menu adds one more.
    assert_eq!(h.get_all_by_label("Delete").count(), deletes_before + 1);
}

// --- navigation -------------------------------------------------------------------

#[test]
fn mouse_wheel_zooms_toward_pointer_and_trackpad_pans() {
    let mut h = open();
    let p = screen(&h, 1000.0, 1000.0);
    let zoom = ws(&h).viewport.unwrap().zoom;
    h.event(Event::PointerMoved(p));
    h.event(Event::MouseWheel {
        unit: MouseWheelUnit::Line,
        delta: Vec2::new(0.0, 3.0),
        modifiers: Modifiers::NONE,
        phase: egui::TouchPhase::Move,
    });
    h.run();
    let view = ws(&h).viewport.unwrap();
    assert!(view.zoom > zoom);
    let doc = ws(&h).screen_map(1.0).unwrap().to_doc(p);
    assert!(doc.distance(Point::new(1000.0, 1000.0)) < 0.5, "{doc:?}");

    let zoom = view.zoom;
    h.event(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: Vec2::new(30.0, -20.0),
        modifiers: Modifiers::NONE,
        phase: egui::TouchPhase::Move,
    });
    h.run();
    let after = ws(&h).viewport.unwrap();
    assert_eq!(after.zoom, zoom);
    let moved = ws(&h)
        .screen_map(1.0)
        .unwrap()
        .to_screen(Point::new(1000.0, 1000.0));
    assert!((moved - p - Vec2::new(30.0, -20.0)).length() < 0.5);
}

#[test]
fn cmd_plus_zooms_canvas_not_interface() {
    let mut h = open();
    let zoom = ws(&h).viewport.unwrap().zoom;
    h.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    h.run();
    assert!(ws(&h).viewport.unwrap().zoom > zoom);
    assert!((h.ctx.zoom_factor() - 1.0).abs() < 1e-6);
    h.get_by_label_contains("Zoom 25%");

    h.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    h.run();
    assert_eq!(ws(&h).viewport.unwrap().zoom, 1.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num0);
    h.run();
    assert!(ws(&h).viewport.unwrap().fitted);
}

#[test]
fn space_drag_pans_without_drawing() {
    let mut h = open();
    set_tool(&mut h, Tool::Rectangle);
    let center_before = ws(&h).viewport.unwrap().center;
    h.key_down(Key::Space);
    h.run();
    assert_eq!(ws(&h).tool, Tool::Hand);
    let (a, b) = (screen(&h, 1000.0, 1000.0), screen(&h, 1500.0, 1200.0));
    drag(&mut h, a, b, Modifiers::NONE);
    h.key_up(Key::Space);
    h.run();
    assert!(ws(&h).project.surface().objects.is_empty());
    assert_ne!(ws(&h).viewport.unwrap().center, center_before);
    assert_eq!(ws(&h).tool, Tool::Rectangle);
}

#[test]
fn middle_drag_pans() {
    let mut h = open();
    let center_before = ws(&h).viewport.unwrap().center;
    let (a, b) = (screen(&h, 1000.0, 1000.0), screen(&h, 1500.0, 1200.0));
    drag_with(&mut h, a, b, Modifiers::NONE, PointerButton::Middle);
    assert_ne!(ws(&h).viewport.unwrap().center, center_before);
    assert!(ws(&h).selection.is_empty());
}

#[test]
fn zoom_tool_drag_fits_area() {
    let mut h = open();
    set_tool(&mut h, Tool::Zoom);
    let (a, b) = (screen(&h, 1000.0, 1000.0), screen(&h, 1400.0, 1300.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let view = ws(&h).viewport.unwrap();
    assert!(
        near(view.center.x, 1200.0) && near(view.center.y, 1150.0),
        "{view:?}"
    );
    let canvas = ws(&h).canvas_rect.unwrap();
    // 400×300 texture px now fill the canvas in at least one dimension.
    let map = ws(&h).screen_map(1.0).unwrap();
    let w = map.to_screen(Point::new(1400.0, 0.0)).x - map.to_screen(Point::new(1000.0, 0.0)).x;
    let hh = map.to_screen(Point::new(0.0, 1300.0)).y - map.to_screen(Point::new(0.0, 1000.0)).y;
    assert!((w - canvas.width()).abs() < 1.0 || (hh - canvas.height()).abs() < 1.0);
}

#[test]
fn canvas_has_accessible_label() {
    let h = open();
    h.get_by_role_and_label(Role::Unknown, "Canvas");
}

#[test]
fn status_bar_coordinates_follow_navigation() {
    let mut h = open();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    h.run();
    h.event(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: Vec2::new(-120.0, 80.0),
        modifiers: Modifiers::NONE,
        phase: egui::TouchPhase::Move,
    });
    h.run();
    // A texture pixel near the view center, after zooming and panning.
    let c = ws(&h).viewport.unwrap().center;
    let (x, y) = ((c.x + 100.0).floor(), (c.y + 50.0).floor());
    let p = screen(&h, x + 0.5, y + 0.5);
    h.hover_at(p);
    h.run();
    h.get_by_label_contains(&format!("X {x}  Y {y} px"));
    h.get_by_label_contains("Zoom 100%");
}

#[test]
fn panning_reuses_cached_geometry() {
    let mut h = open();
    for i in 0..20 {
        let id = ws_mut(&mut h).project.add(Object::new(
            ObjectId(0),
            ShapeKind::Ellipse,
            Frame::new(
                Point::new(200.0 + 150.0 * f64::from(i), 1000.0),
                Size::new(120.0, 80.0),
                0.0,
            ),
        ));
        let _ = id;
    }
    h.run();
    let misses = ws(&h).geometry.misses;
    for _ in 0..5 {
        h.event(Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: Vec2::new(7.0, 3.0),
            modifiers: Modifiers::NONE,
            phase: egui::TouchPhase::Move,
        });
        h.run();
    }
    assert_eq!(
        ws(&h).geometry.misses,
        misses,
        "panning must not recompute outlines"
    );
}

#[test]
fn cmd_y_redoes() {
    let mut h = open();
    let a = add_rect(&mut h, 1000.0, 1000.0, 100.0, 100.0);
    ws_mut(&mut h).selection = vec![a];
    h.key_press(Key::ArrowLeft);
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(frame_of(&h, a).center.x, 1000.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Y);
    h.run();
    assert_eq!(frame_of(&h, a).center.x, 999.0);
}

/// Frame-time budget with 1000 shapes while panning (release builds only).
#[test]
#[ignore = "performance check; run with --release -- --ignored"]
fn pan_with_1000_shapes_stays_fast() {
    let mut h = open();
    for i in 0..1000 {
        let x = 100.0 + f64::from(i % 40) * 95.0;
        let y = 100.0 + f64::from(i / 40) * 150.0;
        let kind = if i % 2 == 0 {
            ShapeKind::Ellipse
        } else {
            ShapeKind::Rectangle {
                corner_radius: 10.0,
            }
        };
        ws_mut(&mut h).project.add(Object::new(
            ObjectId(0),
            kind,
            Frame::new(Point::new(x, y), Size::new(80.0, 60.0), f64::from(i % 90)),
        ));
    }
    h.run();
    let frames = 60;
    let start = std::time::Instant::now();
    for _ in 0..frames {
        h.event(Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: Vec2::new(5.0, 2.0),
            modifiers: Modifiers::NONE,
            phase: egui::TouchPhase::Move,
        });
        h.step();
    }
    let per_frame = start.elapsed() / frames;
    println!("average frame: {per_frame:?}");
    assert!(
        per_frame < std::time::Duration::from_millis(16),
        "{per_frame:?}"
    );
}
