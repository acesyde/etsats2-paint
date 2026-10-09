//! Headless tests for stroke-options: the inspector's Stroke row and its
//! popover (alignment, dashes, caps and joins), and the line settings of
//! lines.

mod common;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::prefs::Prefs;
use tp_app::workspace::Workspace;
use tp_core::document::{
    Cap, CharStyle, Dash, Frame, Join, Node, Object, ObjectId, PathData, ShapeKind, StrokeAlign,
    StrokeStyle, Subpath, TextBlock,
};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

/// Tall window, so the inspector and its popovers show whole.
fn open() -> H {
    let prefs = Prefs::default();
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

/// Adds `o` with a 6 px black stroke and selects it.
fn add_stroked(h: &mut H, mut o: Object) -> ObjectId {
    o.stroke = Some(StrokeStyle {
        width: 6.0,
        ..Default::default()
    });
    let id = ws_mut(h).project.add(o);
    ws_mut(h).selection = vec![id];
    h.run();
    id
}

fn shape(kind: ShapeKind) -> Object {
    Object::new(
        ObjectId(0),
        kind,
        Frame::new(Point::new(800.0, 800.0), Size::new(400.0, 300.0), 0.0),
    )
}

fn add_line(h: &mut H) -> ObjectId {
    let line = ws(h).styled_path(
        PathData::new(vec![Subpath::new(
            vec![
                Node::corner(Point::new(500.0, 500.0)),
                Node::corner(Point::new(900.0, 500.0)),
            ],
            false,
        )]),
        "Line",
    );
    let id = ws_mut(h).project.add(line);
    ws_mut(h).selection = vec![id];
    h.run();
    id
}

fn click(h: &mut H, name: &str) {
    h.get_by_label(name).click();
    h.run();
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

/// The Stroke row's summary.
fn summary(h: &H) -> String {
    h.get_by_label("Stroke options").value().expect("a summary")
}

fn selected(h: &H, name: &str) -> bool {
    h.get_by_label(name)
        .accesskit_node()
        .toggled()
        .is_some_and(|t| t == egui::accesskit::Toggled::True)
}

#[test]
fn outside_outline_in_one_click() {
    let mut h = open();
    let at = Point::new(1000.0, 1000.0);
    let mut text = Object::text(ObjectId(0), TextBlock::new("ACE", CharStyle::default()), at);
    ws_mut(&mut h).text.place_at(&mut text, at);
    let id = add_stroked(&mut h, text);
    common::open_stroke_popover(&mut h);
    assert!(selected(&h, "Center stroke"));
    click(&mut h, "Outside stroke");
    assert_eq!(obj(&h, id).stroke.unwrap().align, StrokeAlign::Outside);
    assert_eq!(summary(&h), "6 px · Outside");
    assert_eq!(
        ws(&h).history.undo_label(),
        Some("undo-change-stroke-alignment")
    );
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(obj(&h, id).stroke.unwrap().align, StrokeAlign::Center);
}

#[test]
fn dotted_preset() {
    let mut h = open();
    let id = add_stroked(&mut h, shape(ShapeKind::Ellipse));
    common::open_stroke_popover(&mut h);
    click(&mut h, "Dash presets");
    click(&mut h, "Dotted 0/12");
    let line = obj(&h, id).stroke.unwrap().line;
    assert_eq!(
        line.dash,
        Some(Dash {
            dash: 0.0,
            gap: 12.0
        })
    );
    assert_eq!(line.cap, Cap::Round);
    assert_eq!(
        ws(&h).history.undo_label(),
        Some("undo-change-stroke-dashes")
    );
}

#[test]
fn round_corners() {
    let mut h = open();
    let id = add_stroked(&mut h, shape(ShapeKind::rectangle()));
    common::open_stroke_popover(&mut h);
    assert!(selected(&h, "Miter join"));
    assert!(h.query_by_label("Miter limit").is_some());
    click(&mut h, "Round join");
    assert_eq!(obj(&h, id).stroke.unwrap().line.join, Join::Round);
    assert!(selected(&h, "Round join"));
    assert!(
        h.query_by_label("Miter limit").is_none(),
        "only for miter joins"
    );
}

#[test]
fn differing_joins_show_no_segment() {
    let mut h = open();
    let a = add_stroked(&mut h, shape(ShapeKind::rectangle()));
    let b = add_stroked(&mut h, shape(ShapeKind::Ellipse));
    let mut round = obj(&h, b);
    round.stroke.as_mut().unwrap().line.join = Join::Round;
    ws_mut(&mut h).project.surface_mut().replace(&[round]);
    ws_mut(&mut h).selection = vec![a, b];
    h.run();
    common::open_stroke_popover(&mut h);
    assert!(!selected(&h, "Miter join") && !selected(&h, "Round join"));
    // One click sets both, as one undo step.
    let steps = ws(&h).history.len();
    click(&mut h, "Bevel join");
    for id in [a, b] {
        assert_eq!(obj(&h, id).stroke.unwrap().line.join, Join::Bevel);
    }
    assert_eq!(ws(&h).history.len(), steps + 1);
}

#[test]
fn current_style_with_nothing_selected() {
    let mut h = open();
    ws_mut(&mut h).selection.clear();
    h.run();
    common::open_stroke_popover(&mut h);
    click(&mut h, "Inside stroke");
    click(&mut h, "Square cap");
    let style = ws(&h).style.stroke;
    assert_eq!(
        (style.align, style.line.cap),
        (StrokeAlign::Inside, Cap::Square)
    );
    // New shapes get it.
    ws_mut(&mut h).style.stroke_enabled = true;
    ws_mut(&mut h).create_shape(
        ShapeKind::Ellipse,
        Frame::new(Point::new(300.0, 300.0), Size::new(100.0, 100.0), 0.0),
        1.0,
    );
    let created = ws(&h).selected_objects()[0].stroke.unwrap();
    assert_eq!(created.align, StrokeAlign::Inside);
}

#[test]
fn dashed_line_from_the_inspector() {
    let mut h = open();
    let id = add_line(&mut h);
    common::open_line_settings(&mut h);
    // Lines use their own line settings, not the stroke popover's.
    assert_eq!(h.get_all_by_label("Dashed").count(), 1);
    click(&mut h, "Dashed");
    type_into(&mut h, "Dash length", "30");
    type_into(&mut h, "Gap length", "15");
    let line = obj(&h, id);
    assert_eq!(
        line.path_data().unwrap().line_style.dash,
        Some(Dash {
            dash: 30.0,
            gap: 15.0
        })
    );
    assert!(line.stroke.is_none(), "no stroke needed");
    // Turning Dashed off and on again restores the lengths.
    click(&mut h, "Dashed");
    assert_eq!(obj(&h, id).path_data().unwrap().line_style.dash, None);
    click(&mut h, "Dashed");
    assert_eq!(
        obj(&h, id).path_data().unwrap().line_style.dash,
        Some(Dash {
            dash: 30.0,
            gap: 15.0
        })
    );
}

#[test]
fn butt_ends_and_the_next_line() {
    let mut h = open();
    let id = add_line(&mut h);
    common::open_line_settings(&mut h);
    assert!(selected(&h, "Round cap"));
    click(&mut h, "Butt cap");
    assert_eq!(obj(&h, id).path_data().unwrap().line_style.cap, Cap::Butt);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-change-line-caps"));
    // The next line uses the last values.
    let next = add_line(&mut h);
    assert_eq!(obj(&h, next).path_data().unwrap().line_style.cap, Cap::Butt);
}

#[test]
fn stroke_row_summary_and_popover() {
    let mut h = open();
    let id = add_stroked(&mut h, shape(ShapeKind::rectangle()));
    // A 6 px stroke, centered.
    assert_eq!(summary(&h), "6 px · Center");
    click(&mut h, "Stroke options");
    // The stroke popover opens under the Stroke row with every setting.
    let row = h.get_by_label("Stroke color").rect();
    let width = h
        .get_by_role_and_label(Role::TextInput, "Stroke width")
        .rect();
    assert!(width.top() > row.bottom(), "{row:?} {width:?}");
    for name in [
        "Stroke enabled",
        "Outside stroke",
        "Dashed",
        "Butt cap",
        "Bevel join",
    ] {
        assert!(h.query_by_label(name).is_some(), "{name}");
    }
    type_into(&mut h, "Stroke width", "12");
    assert_eq!(obj(&h, id).stroke.unwrap().width, 12.0);
    assert_eq!(summary(&h), "12 px · Center");
    // Escape closes it, keeping the change and the selection.
    h.key_press(Key::Escape);
    h.run();
    assert!(h.query_by_label("Stroke width").is_none());
    assert_eq!(ws(&h).selection, vec![id]);
    assert_eq!(obj(&h, id).stroke.unwrap().width, 12.0);
}

#[test]
fn no_stroke_reads_none() {
    let mut h = open();
    let id = ws_mut(&mut h).project.add(shape(ShapeKind::Ellipse));
    ws_mut(&mut h).selection = vec![id];
    h.run();
    assert_eq!(summary(&h), "None");
}
