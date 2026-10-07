//! Headless tests for gradients: the paint kind control, the gradient bar
//! and fields of the Colors panel, and the Gradient tool.

mod common;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::layout::PanelKind;
use tp_app::prefs::Prefs;
use tp_app::workspace::{ColorTarget, Workspace};
use tp_core::document::{
    ColorStop, Frame, Gradient, GradientKind, Object, ObjectId, Paint, Rgba, ShapeKind, StrokeStyle,
};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

const RED: Rgba = Rgba::rgb(255, 0, 0);
const BLUE: Rgba = Rgba::rgb(0, 0, 255);

/// Tall window with the editing panels open and expanded.
fn open() -> H {
    let mut prefs = Prefs::default();
    for slot in &mut prefs.layout.panels {
        slot.open = !matches!(slot.kind, PanelKind::Assets | PanelKind::Vehicle);
        slot.collapsed = false;
    }
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, 2400.0))
        .with_step_dt(1.0 / 4.0)
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(prefs, None),
        );
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

fn obj(h: &H, id: ObjectId) -> Object {
    (**ws(h).project.surface().get(id).expect("object")).clone()
}

fn red_blue() -> Gradient {
    Gradient::new(
        GradientKind::Linear,
        &[ColorStop::new(0.0, RED), ColorStop::new(1.0, BLUE)],
    )
}

/// Adds a 400 × 100 rectangle filled with `fill` and selects it.
fn add_rect(h: &mut H, center: (f64, f64), fill: Paint) -> ObjectId {
    let mut o = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(center.0, center.1), Size::new(400.0, 100.0), 0.0),
    );
    o.fill = fill;
    let id = ws_mut(h).project.add(o);
    ws_mut(h).selection = vec![id];
    h.run();
    id
}

fn gradient(h: &H, id: ObjectId) -> Gradient {
    *obj(h, id).fill.gradient().expect("a gradient fill")
}

fn selected(h: &H, name: &str) -> bool {
    h.get_by_label(name)
        .accesskit_node()
        .toggled()
        .is_some_and(|t| t == egui::accesskit::Toggled::True)
}

fn type_into(h: &mut H, name: &str, text: &str) {
    h.get_by_role_and_label(Role::TextInput, name).focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, name)
        .type_text(text);
    h.run();
    h.key_press(Key::Enter);
    h.run();
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
        h.run();
    }
}

fn undo(h: &mut H) {
    ws_mut(h).undo();
    h.run();
}

#[test]
fn make_a_fill_linear_and_undo() {
    let mut h = open();
    let id = add_rect(&mut h, (500.0, 500.0), RED.into());
    assert!(selected(&h, "Solid paint"));
    h.get_by_label("Linear gradient").click();
    h.run();
    let g = gradient(&h, id);
    assert_eq!(g.kind, GradientKind::Linear);
    assert_eq!(g.stops()[0].color, RED);
    assert_eq!(g.stops()[1].color, Rgba::with_alpha(255, 0, 0, 0));
    assert!(selected(&h, "Linear gradient"));
    assert_eq!(ws(&h).history.undo_label(), Some("Change Fill Type"));
    undo(&mut h);
    assert_eq!(obj(&h, id).fill, Paint::Solid(RED));
}

#[test]
fn add_a_middle_stop() {
    let mut h = open();
    let id = add_rect(&mut h, (500.0, 500.0), Paint::Gradient(red_blue()));
    let bar = h.get_by_label("Gradient bar").rect();
    // The markers' centers span the bar, inset by half a marker (6 px).
    click_at(&mut h, Pos2::new(bar.center().x, bar.top() + 5.0));
    let g = gradient(&h, id);
    assert_eq!(g.stops().len(), 3);
    let middle = g.stops()[1];
    assert!((middle.offset - 0.5).abs() < 0.02, "{}", middle.offset);
    let c = middle.color;
    assert!(
        c.r.abs_diff(128) <= 3 && c.g == 0 && c.b.abs_diff(128) <= 3,
        "{c:?}"
    );
    assert_eq!(ws(&h).panels.gradient_stop, 1, "the new stop is selected");
    assert_eq!(ws(&h).history.undo_label(), Some("Change Fill Gradient"));
}

#[test]
fn recolor_a_stop_with_the_hex_field() {
    let mut h = open();
    let id = add_rect(&mut h, (500.0, 500.0), Paint::Gradient(red_blue()));
    h.get_by_label("Stop 2 at 100%").click();
    h.run();
    type_into(&mut h, "Hex color", "#00ff00");
    let g = gradient(&h, id);
    assert_eq!(g.stops()[0].color, RED);
    assert_eq!(g.stops()[1].color, Rgba::rgb(0, 255, 0));
}

#[test]
fn recent_color_on_a_stop() {
    let mut h = open();
    let id = add_rect(&mut h, (500.0, 500.0), Paint::Gradient(red_blue()));
    // Applying a color to the first stop makes it recent.
    type_into(&mut h, "Hex color", "#123456");
    h.get_by_label("Stop 2 at 100%").click();
    h.run();
    h.get_by_label("Recent color #123456").click();
    h.run();
    let g = gradient(&h, id);
    assert_eq!(g.stops()[0].color, Rgba::rgb(0x12, 0x34, 0x56));
    assert_eq!(g.stops()[1].color, Rgba::rgb(0x12, 0x34, 0x56));
}

#[test]
fn two_stops_minimum() {
    let mut h = open();
    let id = add_rect(&mut h, (500.0, 500.0), Paint::Gradient(red_blue()));
    h.get_by_label("Stop 1 at 0%").click();
    h.run();
    h.key_press(Key::Delete);
    h.run();
    assert_eq!(gradient(&h, id).stops().len(), 2);
    assert_eq!(ws(&h).project.surface().objects.len(), 1, "nothing deleted");
}

#[test]
fn delete_and_move_stops_with_the_keyboard() {
    let mut h = open();
    let mut g = red_blue();
    g.set_stops(&[
        ColorStop::new(0.0, RED),
        ColorStop::new(0.5, Rgba::rgb(0, 255, 0)),
        ColorStop::new(1.0, BLUE),
    ]);
    let id = add_rect(&mut h, (500.0, 500.0), Paint::Gradient(g));
    h.get_by_label("Stop 2 at 50%").click();
    h.run();
    h.key_press(Key::ArrowRight);
    h.run();
    assert!((gradient(&h, id).stops()[1].offset - 0.51).abs() < 1e-4);
    h.get_by_label("Stop 2 at 51%").click();
    h.run();
    h.key_press(Key::Delete);
    h.run();
    let g = gradient(&h, id);
    assert_eq!(g.stops().len(), 2);
    assert_eq!((g.stops()[0].color, g.stops()[1].color), (RED, BLUE));
}

#[test]
fn location_angle_aspect_and_reverse() {
    let mut h = open();
    let id = add_rect(&mut h, (500.0, 500.0), Paint::Gradient(red_blue()));
    h.get_by_label("Stop 2 at 100%").click();
    h.run();
    type_into(&mut h, "Stop location", "30");
    assert!((gradient(&h, id).stops()[1].offset - 0.3).abs() < 1e-6);
    type_into(&mut h, "Gradient angle", "90");
    let o = obj(&h, id);
    assert!((gradient(&h, id).angle(&o.frame) - 90.0).abs() < 1e-6);
    h.get_by_label("Reverse gradient").click();
    h.run();
    let g = gradient(&h, id);
    assert_eq!(g.stops()[0].color, BLUE);
    assert!((g.stops()[0].offset - 0.7).abs() < 1e-6);
    h.get_by_label("Radial gradient").click();
    h.run();
    type_into(&mut h, "Gradient aspect ratio", "50");
    let o = obj(&h, id);
    assert!((gradient(&h, id).aspect(&o.frame) - 0.5).abs() < 1e-6);
}

#[test]
fn new_shapes_use_the_current_gradient() {
    let mut h = open();
    ws_mut(&mut h).selection.clear();
    h.run();
    h.get_by_label("Radial gradient").click();
    h.run();
    let frame = Frame::new(Point::new(800.0, 800.0), Size::new(300.0, 200.0), 0.0);
    let id = ws_mut(&mut h).create_shape(ShapeKind::Ellipse, frame, 1.0);
    h.run();
    assert_eq!(
        obj(&h, id).fill.gradient().map(|g| g.kind),
        Some(GradientKind::Radial)
    );
}

#[test]
fn mixed_kinds_and_editing_applies_the_first_gradient() {
    let mut h = open();
    let a = add_rect(&mut h, (500.0, 500.0), Paint::Gradient(red_blue()));
    let b = add_rect(&mut h, (500.0, 900.0), RED.into());
    ws_mut(&mut h).selection = vec![a, b];
    h.run();
    for name in ["Solid paint", "Linear gradient", "Radial gradient"] {
        assert!(!selected(&h, name), "{name}: Mixed");
    }
    h.get_by_label("Reverse gradient").click();
    h.run();
    assert_eq!(gradient(&h, a).stops()[0].color, BLUE);
    assert_eq!(gradient(&h, b), gradient(&h, a));
}

#[test]
fn stroke_gradient_from_the_panel() {
    let mut h = open();
    let id = add_rect(&mut h, (500.0, 500.0), RED.into());
    ws_mut(&mut h).map_selected_shapes("x", |o| {
        o.stroke = Some(StrokeStyle {
            width: 8.0,
            ..Default::default()
        });
    });
    ws_mut(&mut h).commit_pending(0.5);
    ws_mut(&mut h).panels.color_target = ColorTarget::Stroke;
    h.run();
    h.get_by_label("Linear gradient").click();
    h.run();
    let o = obj(&h, id);
    assert_eq!(o.fill, Paint::Solid(RED));
    let stroke = o.stroke.unwrap();
    assert_eq!(stroke.width, 8.0);
    assert!(stroke.paint.gradient().is_some());
}

// --- Gradient tool -------------------------------------------------------------

fn screen(h: &H, p: Point) -> Pos2 {
    ws(h).screen_map(1.0).expect("canvas").to_screen(p)
}

/// Presses at `from`, moves in steps to `to` and releases, with `mods` held.
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
    for i in 1..=6 {
        let t = i as f32 / 6.0;
        h.event(Event::PointerMoved(from + (to - from) * t));
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
    h.run();
}

fn close(a: Point, b: Point, tol: f64) -> bool {
    (a - b).hypot() <= tol
}

#[test]
fn g_activates_the_gradient_tool_after_the_eyedropper() {
    let mut h = open();
    h.key_press(Key::G);
    h.run();
    assert_eq!(ws(&h).tool, tp_app::tool::Tool::Gradient);
    let y = |name: &str| h.get_by_label(name).rect().center().y;
    assert!(y("Eyedropper") < y("Gradient") && y("Gradient") < y("Zoom"));
}

#[test]
fn drag_a_gradient_across_a_rectangle() {
    let mut h = open();
    let blue = Rgba::rgb(0, 0, 255);
    let id = add_rect(&mut h, (1000.0, 1000.0), blue.into());
    ws_mut(&mut h).tool = tp_app::tool::Tool::Gradient;
    h.run();
    // From the top edge to the bottom edge (the rectangle is 100 px tall).
    let (top, bottom) = (Point::new(1000.0, 950.0), Point::new(1000.0, 1050.0));
    let (a, b) = (screen(&h, top), screen(&h, bottom));
    drag(&mut h, a, b, Modifiers::NONE);
    let o = obj(&h, id);
    let g = gradient(&h, id);
    assert_eq!(g.kind, GradientKind::Linear);
    assert_eq!(g.stops()[0].color, blue);
    assert_eq!(g.stops()[1].color, Rgba::with_alpha(0, 0, 255, 0));
    let (s, e, _) = g.document_points(&o.frame);
    let tol = 1.0 / f64::from(ws(&h).screen_map(1.0).unwrap().scale);
    assert!(
        close(s, top, tol * 2.0) && close(e, bottom, tol * 2.0),
        "{s:?} {e:?}"
    );
    assert_eq!(ws(&h).history.undo_label(), Some("Change Fill Gradient"));
    undo(&mut h);
    assert_eq!(obj(&h, id).fill, Paint::Solid(blue));
}

#[test]
fn move_the_end_handle() {
    let mut h = open();
    let id = add_rect(&mut h, (1000.0, 1000.0), Paint::Gradient(red_blue()));
    ws_mut(&mut h).tool = tp_app::tool::Tool::Gradient;
    h.run();
    let o = obj(&h, id);
    let (s, e, _) = gradient(&h, id).document_points(&o.frame);
    let halfway = s.midpoint(e);
    let (a, b) = (screen(&h, e), screen(&h, halfway));
    drag(&mut h, a, b, Modifiers::NONE);
    let o = obj(&h, id);
    let (s2, e2, _) = gradient(&h, id).document_points(&o.frame);
    let tol = 2.0 / f64::from(ws(&h).screen_map(1.0).unwrap().scale);
    assert!(close(s2, s, 1e-6), "the start stays");
    assert!(close(e2, halfway, tol), "{e2:?} {halfway:?}");
    assert_eq!(ws(&h).history.len(), 1, "one undo step");
}

#[test]
fn constrained_angle() {
    let mut h = open();
    let id = add_rect(&mut h, (1000.0, 1000.0), RED.into());
    ws_mut(&mut h).tool = tp_app::tool::Tool::Gradient;
    h.run();
    let from = Point::new(900.0, 1000.0);
    // About 40° below the horizontal (y points down).
    let to = from + tp_core::kurbo::Vec2::from_angle(40f64.to_radians()) * 150.0;
    let (a, b) = (screen(&h, from), screen(&h, to));
    drag(&mut h, a, b, Modifiers::SHIFT);
    let o = obj(&h, id);
    assert!((gradient(&h, id).angle(&o.frame) - 45.0).abs() < 1e-6);
}

#[test]
fn locked_objects_are_not_edited() {
    let mut h = open();
    let red = add_rect(&mut h, (1000.0, 1000.0), RED.into());
    let blue = add_rect(&mut h, (1000.0, 1300.0), BLUE.into());
    let group = ws_mut(&mut h).project.group(&[red, blue]).unwrap();
    {
        let ws = ws_mut(&mut h);
        let mut locked = (**ws.project.surface().get(blue).unwrap()).clone();
        locked.locked = true;
        ws.project.surface_mut().replace(&[locked]);
        ws.selection = vec![group];
        ws.tool = tp_app::tool::Tool::Gradient;
    }
    h.run();
    let (from, to) = (Point::new(800.0, 1000.0), Point::new(1200.0, 1000.0));
    let (a, b) = (screen(&h, from), screen(&h, to));
    drag(&mut h, a, b, Modifiers::NONE);
    assert!(obj(&h, red).fill.gradient().is_some());
    assert_eq!(obj(&h, blue).fill, Paint::Solid(BLUE));
}

#[test]
fn click_selects_with_the_gradient_tool() {
    let mut h = open();
    let a = add_rect(&mut h, (1000.0, 1000.0), RED.into());
    ws_mut(&mut h).selection.clear();
    ws_mut(&mut h).tool = tp_app::tool::Tool::Gradient;
    h.run();
    let at = screen(&h, Point::new(1000.0, 1000.0));
    click_at(&mut h, at);
    assert_eq!(ws(&h).selection, vec![a]);
}

#[test]
fn eyedropper_copies_a_gradient() {
    let mut h = open();
    let mut radial = red_blue();
    radial.set_kind(GradientKind::Radial);
    let source = add_rect(&mut h, (1000.0, 1000.0), Paint::Gradient(radial));
    let target = add_rect(&mut h, (1000.0, 1500.0), BLUE.into());
    ws_mut(&mut h).selection = vec![target];
    ws_mut(&mut h).tool = tp_app::tool::Tool::Eyedropper;
    h.run();
    let at = screen(&h, Point::new(1000.0, 1000.0));
    click_at(&mut h, at);
    assert_eq!(ws(&h).selection, vec![target], "selection unchanged");
    assert_eq!(obj(&h, target).fill, obj(&h, source).fill);
}
