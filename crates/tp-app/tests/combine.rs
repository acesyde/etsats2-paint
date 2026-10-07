//! Headless tests for boolean-operations: Object › Combine and the
//! Transform panel buttons.

mod common;

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::workspace::Workspace;
use tp_core::document::{CharStyle, Frame, Object, ObjectId, Rgba, ShapeKind, TextBlock};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

fn harness(panels: bool) -> H {
    let mut prefs = tp_app::prefs::Prefs::default();
    if panels {
        for slot in &mut prefs.layout.panels {
            slot.open = !matches!(
                slot.kind,
                tp_app::layout::PanelKind::Assets | tp_app::layout::PanelKind::Vehicle
            );
            slot.collapsed = false;
        }
    }
    let mut h = Harness::builder()
        .with_size(egui::Vec2::new(1440.0, if panels { 2400.0 } else { 900.0 }))
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

fn shape(h: &mut H, kind: ShapeKind, x0: f64, y0: f64, w: f64, hh: f64, fill: Rgba) -> ObjectId {
    let mut o = Object::new(
        ObjectId(0),
        kind,
        Frame::new(
            Point::new(x0 + w / 2.0, y0 + hh / 2.0),
            Size::new(w, hh),
            0.0,
        ),
    );
    o.fill = fill.into();
    let id = ws_mut(h).project.add(o);
    h.run();
    id
}

fn select(h: &mut H, ids: &[ObjectId]) {
    ws_mut(h).selection = ids.to_vec();
    h.run();
}

fn only_object(h: &H) -> Object {
    let objects = &ws(h).project.surface().objects;
    assert_eq!(objects.len(), 1, "one object expected");
    (*objects[0]).clone()
}

fn from_combine_menu(h: &mut H, label: &str) {
    h.get_by_label("Object").click();
    h.run();
    h.get_by_label("Combine ⏵").click();
    h.run();
    h.get_by_label(label).click();
    h.run();
}

const BLUE: Rgba = Rgba::rgb(0, 0, 255);
const RED: Rgba = Rgba::rgb(255, 0, 0);

#[test]
fn unite_two_overlapping_squares() {
    let mut h = harness(false);
    let a = shape(
        &mut h,
        ShapeKind::rectangle(),
        100.0,
        100.0,
        400.0,
        400.0,
        BLUE,
    );
    let b = shape(
        &mut h,
        ShapeKind::rectangle(),
        300.0,
        300.0,
        400.0,
        400.0,
        RED,
    );
    select(&mut h, &[a, b]);
    from_combine_menu(&mut h, "Unite");
    let r = only_object(&h);
    assert_eq!(r.kind, ShapeKind::Path);
    assert_eq!(r.fill, RED.into(), "the topmost style");
    assert_eq!(
        r.path_data().unwrap().subpaths.len(),
        1,
        "no internal edges"
    );
    assert!(r.contains(Point::new(150.0, 150.0), 0.0) && r.contains(Point::new(650.0, 650.0), 0.0));
    assert!(!r.contains(Point::new(650.0, 150.0), 0.0));
}

#[test]
fn cut_a_notch_with_the_shortcut_and_undo() {
    let mut h = harness(false);
    let stripe = shape(
        &mut h,
        ShapeKind::rectangle(),
        100.0,
        1000.0,
        2000.0,
        300.0,
        BLUE,
    );
    let disk = shape(
        &mut h,
        ShapeKind::Ellipse,
        1950.0,
        1000.0,
        300.0,
        300.0,
        RED,
    );
    select(&mut h, &[stripe, disk]);
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Minus);
    h.run();
    let r = only_object(&h);
    assert_eq!(r.fill, BLUE.into(), "the bottom style for Minus Front");
    assert!(r.contains(Point::new(1000.0, 1150.0), 0.0));
    assert!(!r.contains(Point::new(2080.0, 1150.0), 0.0), "the notch");
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    let ids: Vec<ObjectId> = ws(&h)
        .project
        .surface()
        .objects
        .iter()
        .map(|o| o.id)
        .collect();
    assert_eq!(ids, vec![stripe, disk]);
}

#[test]
fn exclude_makes_a_hole_in_the_export() {
    let mut h = harness(false);
    let big = shape(
        &mut h,
        ShapeKind::rectangle(),
        0.0,
        0.0,
        1024.0,
        1024.0,
        BLUE,
    );
    let small = shape(
        &mut h,
        ShapeKind::rectangle(),
        256.0,
        256.0,
        512.0,
        512.0,
        RED,
    );
    select(&mut h, &[big, small]);
    from_combine_menu(&mut h, "Exclude");
    let ws = h.state_mut().workspace_mut().unwrap();
    let pixmap = tp_render::render(
        &ws.project,
        0,
        tp_render::RenderOptions {
            size: 2048,
            background: Some(Rgba::rgb(255, 255, 255)),
        },
        &mut ws.text.fonts,
        &mut |_, _| true,
    )
    .unwrap();
    let img = tp_render::to_rgba(&pixmap);
    assert_eq!(
        img.get_pixel(100, 100).0,
        [255, 0, 0, 255],
        "the topmost style"
    );
    assert_eq!(img.get_pixel(512, 512).0, [255, 255, 255, 255], "the hole");
}

#[test]
fn text_in_the_selection_disables_with_a_reason() {
    let mut h = harness(true);
    let r = shape(&mut h, ShapeKind::rectangle(), 0.0, 0.0, 100.0, 100.0, BLUE);
    let t = {
        let ws = ws_mut(&mut h);
        let at = Point::new(500.0, 500.0);
        let mut text = Object::text(ObjectId(0), TextBlock::new("ACE", CharStyle::default()), at);
        ws.text.place_at(&mut text, at);
        ws.project.add(text)
    };
    select(&mut h, &[r, t]);
    let button = h.get_by_label("Unite");
    assert!(button.accesskit_node().is_disabled());
    let reason = tp_app::state::disabled_reason_for(
        tp_app::commands::CommandId::Combine(tp_core::document::BooleanOp::Unite),
        &h.state().edit_context(),
    );
    assert!(tp_i18n::tr(reason.unwrap()).contains("Create Outlines"));
}

#[test]
fn groups_count_as_one_shape() {
    let mut h = harness(false);
    let c1 = shape(&mut h, ShapeKind::Ellipse, 0.0, 0.0, 400.0, 400.0, BLUE);
    let c2 = shape(&mut h, ShapeKind::Ellipse, 1000.0, 0.0, 400.0, 400.0, BLUE);
    let g = ws_mut(&mut h).project.group(&[c1, c2]).unwrap();
    let bar = shape(
        &mut h,
        ShapeKind::rectangle(),
        0.0,
        150.0,
        1400.0,
        100.0,
        RED,
    );
    select(&mut h, &[g, bar]);
    from_combine_menu(&mut h, "Intersect");
    let r = only_object(&h);
    assert!(
        r.contains(Point::new(200.0, 200.0), 0.0),
        "bar ∩ first circle"
    );
    assert!(
        r.contains(Point::new(1200.0, 200.0), 0.0),
        "bar ∩ second circle"
    );
    assert!(
        !r.contains(Point::new(700.0, 200.0), 0.0),
        "bar between the circles"
    );
    assert_eq!(r.path_data().unwrap().subpaths.len(), 2);
}

#[test]
fn unite_from_the_panel() {
    let mut h = harness(true);
    let a = shape(
        &mut h,
        ShapeKind::rectangle(),
        100.0,
        100.0,
        400.0,
        400.0,
        BLUE,
    );
    let b = shape(
        &mut h,
        ShapeKind::rectangle(),
        300.0,
        300.0,
        400.0,
        400.0,
        RED,
    );
    select(&mut h, &[a, b]);
    h.get_by_label("Unite").click();
    h.run();
    let r = only_object(&h);
    assert_eq!(r.kind, ShapeKind::Path);
    assert_eq!(ws(&h).history.undo_label(), Some("op-unite"));
}
