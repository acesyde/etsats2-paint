//! Headless tests for vector-tools: Pen, Line and Polygon tools, Direct
//! Selection and Convert to Path.

mod common;

use egui::{Event, Key, Modifiers, PointerButton, Pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::commands::CommandId;
use tp_app::state::is_enabled;
use tp_app::tool::Tool;
use tp_app::workspace::Workspace;
use tp_core::document::{
    Frame, Node, NodeRef, Object, ObjectId, PathData, PointRef, ShapeKind, StrokeStyle, Subpath,
};
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
    // Keep consecutive clicks from being read as double-clicks.
    for _ in 0..30 {
        h.step();
    }
}

/// A click checked on the very next frame: hints are still showing
/// (`Harness::run` keeps stepping until they expire).
fn click_now(h: &mut H, at: Pos2) {
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
}

fn click_doc(h: &mut H, x: f64, y: f64) {
    let at = screen(h, x, y);
    click(h, at, Modifiers::NONE);
}

fn objects(h: &H) -> Vec<Object> {
    ws(h)
        .project
        .surface()
        .objects
        .iter()
        .map(|o| (**o).clone())
        .collect()
}

fn only_object(h: &H) -> Object {
    let all = objects(h);
    assert_eq!(all.len(), 1, "exactly one object expected");
    all.into_iter().next().unwrap()
}

/// Document-space points of a path's first subpath.
fn doc_nodes(o: &Object) -> Vec<Point> {
    let a = o.frame.affine();
    o.path_data().expect("a path").subpaths[0]
        .nodes
        .iter()
        .map(|n| a * n.point)
        .collect()
}

fn near(a: Point, b: Point) -> bool {
    a.distance(b) < 1.5
}

// --- Polygon -----------------------------------------------------------------

#[test]
fn default_hexagon() {
    let mut h = open();
    set_tool(&mut h, Tool::Polygon);
    let (a, b) = (screen(&h, 100.0, 100.0), screen(&h, 500.0, 500.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let o = only_object(&h);
    assert_eq!(
        o.kind,
        ShapeKind::Polygon {
            sides: 6,
            star: None
        }
    );
    assert_eq!(o.name, "Polygon");
    let bounds = o.bounding_box();
    assert!(near(
        Point::new(bounds.x0, bounds.y0),
        Point::new(100.0, 100.0)
    ));
    assert!(near(
        Point::new(bounds.x1, bounds.y1),
        Point::new(500.0, 500.0)
    ));
    assert!(near(o.flattened(0.1)[0], Point::new(300.0, 100.0)));
    assert_eq!(ws(&h).selection, vec![o.id]);
    assert_eq!(ws(&h).tool, Tool::Polygon);
}

#[test]
fn shift_keeps_the_polygon_regular() {
    let mut h = open();
    set_tool(&mut h, Tool::Polygon);
    let (a, b) = (screen(&h, 100.0, 100.0), screen(&h, 900.0, 400.0));
    drag(&mut h, a, b, Modifiers::SHIFT);
    let points = only_object(&h).flattened(0.1);
    let sides: Vec<f64> = (0..points.len())
        .map(|i| points[i].distance(points[(i + 1) % points.len()]))
        .collect();
    for s in &sides {
        assert!((s - sides[0]).abs() < 0.5, "{sides:?}");
    }
}

#[test]
fn polygon_click_without_drag_creates_nothing() {
    let mut h = open();
    set_tool(&mut h, Tool::Polygon);
    click_doc(&mut h, 300.0, 300.0);
    assert!(objects(&h).is_empty());
}

// --- Line ----------------------------------------------------------------------

#[test]
fn horizontal_line_with_shift() {
    let mut h = open();
    set_tool(&mut h, Tool::Line);
    let (a, b) = (screen(&h, 100.0, 100.0), screen(&h, 500.0, 120.0));
    drag(&mut h, a, b, Modifiers::SHIFT);
    let o = only_object(&h);
    assert_eq!(o.name, "Line");
    let nodes = doc_nodes(&o);
    assert!(near(nodes[0], Point::new(100.0, 100.0)), "{nodes:?}");
    assert!(near(nodes[1], Point::new(500.0, 100.0)), "{nodes:?}");
    assert_eq!(ws(&h).history.undo_label(), Some("undo-create-line"));
}

#[test]
fn line_from_the_middle() {
    let mut h = open();
    set_tool(&mut h, Tool::Line);
    let (a, b) = (screen(&h, 500.0, 500.0), screen(&h, 600.0, 500.0));
    drag(&mut h, a, b, Modifiers::ALT);
    let nodes = doc_nodes(&only_object(&h));
    assert!(near(nodes[0], Point::new(400.0, 500.0)), "{nodes:?}");
    assert!(near(nodes[1], Point::new(600.0, 500.0)), "{nodes:?}");
}

#[test]
fn default_line() {
    let mut h = open();
    ws_mut(&mut h).style.fill = tp_core::document::Rgba::rgb(255, 0, 0).into();
    set_tool(&mut h, Tool::Line);
    let (a, b) = (screen(&h, 100.0, 100.0), screen(&h, 500.0, 300.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let line = only_object(&h);
    assert_eq!(
        line.fill,
        tp_core::document::Paint::from(tp_core::document::Rgba::rgb(255, 0, 0))
    );
    assert!(line.stroke.is_none(), "stroke off by default");
    assert_eq!(line.path_data().unwrap().line_width, 8.0);
}

#[test]
fn line_uses_the_current_stroke_as_an_outline() {
    let mut h = open();
    {
        let style = &mut ws_mut(&mut h).style;
        style.fill = tp_core::document::Rgba::rgb(255, 255, 255).into();
        style.stroke_enabled = true;
        style.stroke = StrokeStyle {
            paint: tp_core::document::Rgba::rgb(0, 0, 0).into(),
            width: 4.0,
            ..Default::default()
        };
    }
    set_tool(&mut h, Tool::Line);
    let (a, b) = (screen(&h, 100.0, 100.0), screen(&h, 500.0, 300.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let line = only_object(&h);
    assert_eq!(
        line.fill,
        tp_core::document::Paint::from(tp_core::document::Rgba::rgb(255, 255, 255))
    );
    assert_eq!(line.stroke.unwrap().width, 4.0);
}

// --- Pen -------------------------------------------------------------------------

#[test]
fn pen_three_clicks_and_enter() {
    let mut h = open();
    set_tool(&mut h, Tool::Pen);
    for (x, y) in [(100.0, 100.0), (500.0, 100.0), (500.0, 400.0)] {
        click_doc(&mut h, x, y);
    }
    assert!(objects(&h).is_empty(), "not in the document while drawing");
    assert_eq!(ws(&h).pen.as_ref().unwrap().nodes.len(), 3);
    h.key_press(Key::Enter);
    h.run();
    let o = only_object(&h);
    let path = o.path_data().unwrap();
    assert!(!path.subpaths[0].closed);
    assert!(path.subpaths[0].nodes.iter().all(|n| !n.smooth));
    assert!(near(doc_nodes(&o)[2], Point::new(500.0, 400.0)));
    assert_eq!(ws(&h).selection, vec![o.id]);
    assert_eq!(ws(&h).tool, Tool::Pen);
    assert!(ws(&h).hint.is_none(), "no 'not available' hint");
}

#[test]
fn pen_drag_makes_a_smooth_point() {
    let mut h = open();
    set_tool(&mut h, Tool::Pen);
    click_doc(&mut h, 100.0, 100.0);
    let (a, b) = (screen(&h, 500.0, 100.0), screen(&h, 600.0, 100.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let n = ws(&h).pen.as_ref().unwrap().nodes[1];
    assert!(n.smooth);
    assert!(near(n.point, Point::new(500.0, 100.0)));
    assert!(near(n.handle_out.unwrap(), Point::new(600.0, 100.0)));
    assert!(near(n.handle_in.unwrap(), Point::new(400.0, 100.0)));
}

#[test]
fn pen_shift_constrains_the_next_point() {
    let mut h = open();
    set_tool(&mut h, Tool::Pen);
    click_doc(&mut h, 100.0, 100.0);
    let at = screen(&h, 400.0, 130.0);
    click(&mut h, at, Modifiers::SHIFT);
    let p = ws(&h).pen.as_ref().unwrap().nodes[1].point;
    assert!((p.y - 100.0).abs() < 1e-6, "{p:?}");
}

#[test]
fn pen_closing_a_triangle() {
    let mut h = open();
    set_tool(&mut h, Tool::Pen);
    for (x, y) in [
        (100.0, 100.0),
        (500.0, 100.0),
        (300.0, 400.0),
        (100.0, 100.0),
    ] {
        click_doc(&mut h, x, y);
    }
    assert!(ws(&h).pen.is_none());
    let o = only_object(&h);
    let sub = &o.path_data().unwrap().subpaths[0];
    assert!(sub.closed);
    assert_eq!(sub.nodes.len(), 3);
    assert!(!o.has_open_path());
}

#[test]
fn pen_single_point_is_discarded() {
    let mut h = open();
    set_tool(&mut h, Tool::Pen);
    click_doc(&mut h, 100.0, 100.0);
    h.key_press(Key::Enter);
    h.run();
    assert!(objects(&h).is_empty());
    assert!(ws(&h).pen.is_none());
}

#[test]
fn pen_backspace_removes_the_last_point() {
    let mut h = open();
    set_tool(&mut h, Tool::Pen);
    for (x, y) in [
        (100.0, 100.0),
        (500.0, 100.0),
        (500.0, 400.0),
        (100.0, 400.0),
    ] {
        click_doc(&mut h, x, y);
    }
    h.key_press(Key::Backspace);
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(doc_nodes(&only_object(&h)).len(), 3);
}

#[test]
fn pen_undo_while_drawing_removes_a_point() {
    let mut h = open();
    set_tool(&mut h, Tool::Pen);
    for (x, y) in [(100.0, 100.0), (500.0, 100.0), (500.0, 400.0)] {
        click_doc(&mut h, x, y);
    }
    let history = ws(&h).history.len();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(ws(&h).pen.as_ref().unwrap().nodes.len(), 2);
    assert_eq!(ws(&h).history.len(), history);
    assert!(objects(&h).is_empty());
}

#[test]
fn pen_whole_path_is_one_undo_step() {
    let mut h = open();
    set_tool(&mut h, Tool::Pen);
    for (x, y) in [
        (100.0, 100.0),
        (500.0, 100.0),
        (500.0, 400.0),
        (300.0, 600.0),
        (100.0, 400.0),
    ] {
        click_doc(&mut h, x, y);
    }
    h.key_press(Key::Escape);
    h.run();
    assert_eq!(objects(&h).len(), 1);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert!(objects(&h).is_empty());
}

#[test]
fn switching_tool_finishes_the_pen_path() {
    let mut h = open();
    set_tool(&mut h, Tool::Pen);
    click_doc(&mut h, 100.0, 100.0);
    click_doc(&mut h, 500.0, 100.0);
    h.key_press(Key::V);
    h.run();
    assert_eq!(ws(&h).tool, Tool::Select);
    assert_eq!(objects(&h).len(), 1);
    assert!(ws(&h).pen.is_none());
}

#[test]
fn every_tool_is_available() {
    let mut h = open();
    for tool in [Tool::Pen, Tool::Line, Tool::Polygon, Tool::DirectSelect] {
        set_tool(&mut h, tool);
        let at = screen(&h, 1000.0, 1000.0);
        click_now(&mut h, at);
        assert!(ws(&h).hint.is_none(), "{tool:?} shows a hint");
        h.key_press(Key::Enter);
        h.run();
    }
    // The pen click placed a point that Enter discarded.
    assert!(objects(&h).is_empty());
}

// --- Direct Selection ----------------------------------------------------------------

fn add(h: &mut H, o: Object) -> ObjectId {
    let id = ws_mut(h).project.add(o);
    h.run();
    id
}

fn corner(x: f64, y: f64) -> Node {
    Node::corner(Point::new(x, y))
}

/// A closed square path (100..500).
fn add_square(h: &mut H) -> ObjectId {
    add(
        h,
        Object::from_path(
            ObjectId(0),
            PathData::new(vec![Subpath::new(
                vec![
                    corner(100.0, 100.0),
                    corner(500.0, 100.0),
                    corner(500.0, 500.0),
                    corner(100.0, 500.0),
                ],
                true,
            )]),
        ),
    )
}

fn direct(h: &mut H, id: ObjectId) {
    set_tool(h, Tool::DirectSelect);
    ws_mut(h).selection = vec![id];
    h.run();
}

fn point(id: ObjectId, node: usize) -> PointRef {
    PointRef::new(id, NodeRef::new(0, node))
}

fn path_of(h: &H, id: ObjectId) -> Object {
    (**ws(h).project.surface().get(id).expect("object")).clone()
}

fn double_click(h: &mut H, at: Pos2) {
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
    h.run();
    for _ in 0..30 {
        h.step();
    }
}

#[test]
fn points_of_a_path_inside_a_group() {
    let mut h = open();
    let square = add_square(&mut h);
    let other = add(
        &mut h,
        Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(Point::new(1500.0, 1500.0), Size::new(100.0, 100.0), 0.0),
        ),
    );
    ws_mut(&mut h).project.group(&[square, other]).unwrap();
    set_tool(&mut h, Tool::DirectSelect);
    click_doc(&mut h, 300.0, 300.0);
    assert_eq!(ws(&h).selection, vec![square]);
    assert_eq!(ws(&h).selected_paths().len(), 1);
}

#[test]
fn rectangle_hint() {
    let mut h = open();
    let r = add(
        &mut h,
        Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(Point::new(1500.0, 1500.0), Size::new(400.0, 400.0), 0.0),
        ),
    );
    set_tool(&mut h, Tool::DirectSelect);
    let at = screen(&h, 1500.0, 1500.0);
    click_now(&mut h, at);
    assert_eq!(ws(&h).selection, vec![r]);
    assert!(
        ws(&h)
            .hint
            .as_ref()
            .unwrap()
            .text
            .contains("Convert to Path")
    );
}

#[test]
fn marquee_selects_points() {
    let mut h = open();
    let id = add_square(&mut h);
    set_tool(&mut h, Tool::DirectSelect);
    let (a, b) = (screen(&h, 50.0, 50.0), screen(&h, 550.0, 200.0));
    drag(&mut h, a, b, Modifiers::NONE);
    assert_eq!(ws(&h).points, [point(id, 0), point(id, 1)].into());
    assert_eq!(ws(&h).selection, vec![id]);
}

#[test]
fn shift_click_toggles_a_point() {
    let mut h = open();
    let id = add_square(&mut h);
    direct(&mut h, id);
    click_doc(&mut h, 100.0, 100.0);
    assert_eq!(ws(&h).points, [point(id, 0)].into());
    let p3 = screen(&h, 500.0, 500.0);
    click(&mut h, p3, Modifiers::SHIFT);
    let p1 = screen(&h, 100.0, 100.0);
    click(&mut h, p1, Modifiers::SHIFT);
    assert_eq!(ws(&h).points, [point(id, 2)].into());
    // Escape clears the points first, then the objects.
    h.key_press(Key::Escape);
    h.run();
    assert!(ws(&h).points.is_empty() && ws(&h).selection == vec![id]);
    h.key_press(Key::Escape);
    h.run();
    assert!(ws(&h).selection.is_empty());
}

#[test]
fn move_one_point_and_undo() {
    let mut h = open();
    let id = add_square(&mut h);
    direct(&mut h, id);
    let (a, b) = (screen(&h, 500.0, 100.0), screen(&h, 520.0, 140.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let o = path_of(&h, id);
    let nodes = doc_nodes(&o);
    assert!(near(nodes[1], Point::new(520.0, 140.0)), "{nodes:?}");
    assert!(near(nodes[0], Point::new(100.0, 100.0)));
    assert!(near(nodes[2], Point::new(500.0, 500.0)));
    assert!((o.bounding_box().x1 - 520.0).abs() < 1.5, "bounds follow");
    assert_eq!(ws(&h).points, [point(id, 1)].into());
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert!(near(
        doc_nodes(&path_of(&h, id))[1],
        Point::new(500.0, 100.0)
    ));
    // Undo restores the point selection of that time.
    assert_eq!(ws(&h).points, [point(id, 1)].into());
}

fn add_smooth(h: &mut H) -> ObjectId {
    add(
        h,
        Object::from_path(
            ObjectId(0),
            PathData::new(vec![Subpath::new(
                vec![
                    corner(100.0, 500.0),
                    Node::smooth(Point::new(500.0, 500.0), Point::new(600.0, 500.0)),
                    corner(900.0, 800.0),
                ],
                false,
            )]),
        ),
    )
}

fn node(h: &H, id: ObjectId, i: usize) -> Node {
    let o = path_of(h, id);
    let a = o.frame.affine();
    let n = o.path_data().unwrap().subpaths[0].nodes[i];
    Node {
        point: a * n.point,
        handle_in: n.handle_in.map(|p| a * p),
        handle_out: n.handle_out.map(|p| a * p),
        smooth: n.smooth,
    }
}

#[test]
fn smooth_handle_stays_aligned() {
    let mut h = open();
    let id = add_smooth(&mut h);
    direct(&mut h, id);
    click_doc(&mut h, 500.0, 500.0);
    let (a, b) = (screen(&h, 600.0, 500.0), screen(&h, 500.0, 400.0));
    drag(&mut h, a, b, Modifiers::NONE);
    let n = node(&h, id, 1);
    assert!(n.smooth);
    assert!(near(n.handle_out.unwrap(), Point::new(500.0, 400.0)));
    assert!(
        near(n.handle_in.unwrap(), Point::new(500.0, 600.0)),
        "{n:?}"
    );
}

#[test]
fn alt_breaks_the_handle() {
    let mut h = open();
    let id = add_smooth(&mut h);
    direct(&mut h, id);
    click_doc(&mut h, 500.0, 500.0);
    let (a, b) = (screen(&h, 600.0, 500.0), screen(&h, 500.0, 400.0));
    drag(&mut h, a, b, Modifiers::ALT);
    let n = node(&h, id, 1);
    assert!(!n.smooth);
    assert!(near(n.handle_in.unwrap(), Point::new(400.0, 500.0)));
}

#[test]
fn nudge_points_is_one_step() {
    let mut h = open();
    let id = add_square(&mut h);
    direct(&mut h, id);
    click_doc(&mut h, 500.0, 500.0);
    let history = ws(&h).history.len();
    h.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    h.key_press(Key::ArrowRight);
    h.run();
    assert!(near(
        doc_nodes(&path_of(&h, id))[2],
        Point::new(511.0, 500.0)
    ));
    assert!(near(
        doc_nodes(&path_of(&h, id))[1],
        Point::new(500.0, 100.0)
    ));
    assert_eq!(ws(&h).history.len(), history + 1);
}

#[test]
fn insert_keeps_the_shape() {
    let mut h = open();
    let id = add_smooth(&mut h);
    direct(&mut h, id);
    let before = path_of(&h, id).path();
    // A point on the curved segment between the second and third points.
    let o = path_of(&h, id);
    let local = o.path_data().unwrap().subpaths[0].segment(1);
    let on_curve = o.frame.affine() * tp_core::kurbo::ParamCurve::eval(&local, 0.5);
    let at = screen(&h, on_curve.x, on_curve.y);
    double_click(&mut h, at);
    let after = path_of(&h, id);
    assert_eq!(after.path_data().unwrap().subpaths[0].nodes.len(), 4);
    let a = tp_core::document::flatten_closed(after.path(), 0.1);
    for p in a {
        let d = before
            .segments()
            .map(|s| {
                tp_core::kurbo::ParamCurveNearest::nearest(&s, p, 1e-6)
                    .distance_sq
                    .sqrt()
            })
            .fold(f64::INFINITY, f64::min);
        assert!(d < 0.2, "{d}");
    }
    assert_eq!(ws(&h).history.undo_label(), Some("undo-add-point"));
}

#[test]
fn delete_a_point_of_a_square() {
    let mut h = open();
    let id = add_square(&mut h);
    direct(&mut h, id);
    click_doc(&mut h, 500.0, 500.0);
    h.key_press(Key::Delete);
    h.run();
    let o = path_of(&h, id);
    let sub = &o.path_data().unwrap().subpaths[0];
    assert!(sub.closed);
    assert_eq!(sub.nodes.len(), 3);
    assert!(ws(&h).points.is_empty());
}

#[test]
fn deleting_every_point_deletes_the_path() {
    let mut h = open();
    let id = add_square(&mut h);
    direct(&mut h, id);
    let (a, b) = (screen(&h, 50.0, 50.0), screen(&h, 550.0, 550.0));
    drag(&mut h, a, b, Modifiers::NONE);
    assert_eq!(ws(&h).points.len(), 4);
    h.key_press(Key::Backspace);
    h.run();
    assert!(objects(&h).is_empty());
}

#[test]
fn corner_to_smooth() {
    let mut h = open();
    let id = add_square(&mut h);
    direct(&mut h, id);
    let at = screen(&h, 500.0, 100.0);
    double_click(&mut h, at);
    let n = node(&h, id, 1);
    assert!(n.smooth);
    let (hin, hout) = (n.handle_in.unwrap(), n.handle_out.unwrap());
    assert!(
        ((hin - n.point) + (hout - n.point)).hypot() < 1e-6,
        "aligned"
    );
}

// --- Convert to Path -----------------------------------------------------------

fn convert_from_menu(h: &mut H) {
    h.get_by_label("Object").click();
    h.run();
    h.get_by_label("Convert to Path").click();
    h.run();
}

#[test]
fn convert_a_rounded_rectangle() {
    let mut h = open();
    let id = add(
        &mut h,
        Object::new(
            ObjectId(0),
            ShapeKind::Rectangle {
                corner_radius: 40.0,
            },
            Frame::new(Point::new(800.0, 800.0), Size::new(400.0, 200.0), 0.0),
        ),
    );
    ws_mut(&mut h).selection = vec![id];
    h.run();
    convert_from_menu(&mut h);
    let o = path_of(&h, id);
    assert_eq!(o.kind, ShapeKind::Path);
    assert_eq!(o.path_data().unwrap().subpaths[0].nodes.len(), 8);
    assert_eq!(ws(&h).history.undo_label(), Some("cmd-convert-to-path"));
    set_tool(&mut h, Tool::DirectSelect);
    assert_eq!(ws(&h).selected_paths().len(), 1);
}

#[test]
fn convert_inside_a_group() {
    let mut h = open();
    let e = add(
        &mut h,
        Object::new(
            ObjectId(0),
            ShapeKind::Ellipse,
            Frame::new(Point::new(800.0, 800.0), Size::new(400.0, 200.0), 0.0),
        ),
    );
    let t = add(
        &mut h,
        Object::text(
            ObjectId(0),
            tp_core::document::TextBlock::new("ACE", Default::default()),
            Point::new(1500.0, 800.0),
        ),
    );
    let g = ws_mut(&mut h).project.group(&[e, t]).unwrap();
    ws_mut(&mut h).selection = vec![g];
    h.run();
    convert_from_menu(&mut h);
    assert_eq!(path_of(&h, e).kind, ShapeKind::Path);
    assert_eq!(path_of(&h, t).kind, ShapeKind::Text);
}

#[test]
fn convert_disabled_without_convertible_shapes() {
    let mut h = open();
    let t = add(
        &mut h,
        Object::text(
            ObjectId(0),
            tp_core::document::TextBlock::new("ACE", Default::default()),
            Point::new(1500.0, 800.0),
        ),
    );
    ws_mut(&mut h).selection = vec![t];
    h.run();
    assert!(!is_enabled(
        CommandId::ConvertToPath,
        &h.state().edit_context()
    ));
}

#[test]
#[ignore = "performance check; run with --release -- --ignored"]
fn pan_with_stars_and_paths_stays_fast() {
    let mut h = open();
    for i in 0..50 {
        let x = 200.0 + f64::from(i % 10) * 380.0;
        let y = 200.0 + f64::from(i / 10) * 380.0;
        let mut star = Object::new(
            ObjectId(0),
            ShapeKind::Polygon {
                sides: 7,
                star: Some(0.45),
            },
            Frame::new(Point::new(x, y), Size::new(300.0, 300.0), f64::from(i)),
        );
        star.stroke = Some(StrokeStyle {
            paint: tp_core::document::Rgba::rgb(0, 0, 0).into(),
            width: 6.0,
            ..Default::default()
        });
        ws_mut(&mut h).project.add(star);
        // A wavy closed path of 40 smooth points.
        let nodes = (0..40)
            .map(|k| {
                let a = f64::from(k) / 40.0 * std::f64::consts::TAU;
                let r = 150.0 + 30.0 * (a * 5.0).sin();
                let p = Point::new(x + r * a.cos(), y + 2000.0 + r * a.sin());
                let t = tp_core::kurbo::Vec2::new(-a.sin(), a.cos()) * 12.0;
                Node::smooth(p, p + t)
            })
            .collect();
        let mut path =
            Object::from_path(ObjectId(0), PathData::new(vec![Subpath::new(nodes, true)]));
        path.stroke = Some(StrokeStyle {
            paint: tp_core::document::Rgba::rgb(0, 0, 0).into(),
            width: 4.0,
            ..Default::default()
        });
        ws_mut(&mut h).project.add(path);
    }
    h.run();
    let frames = 60;
    let start = std::time::Instant::now();
    for _ in 0..frames {
        h.event(Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::Vec2::new(5.0, 2.0),
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
