//! Headless tests for text-to-outlines: the Create Outlines command, its
//! result, fidelity and editing of the letters.

mod common;

use egui::{Event, Key, Modifiers, PointerButton};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::commands::CommandId;
use tp_app::state::is_enabled;
use tp_app::tool::Tool;
use tp_app::workspace::Workspace;
use tp_core::document::{
    CharStyle, Frame, Object, ObjectId, Rgba, ShapeKind, StrokeStyle, TextBlock,
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

fn add_text(h: &mut H, content: &str, at: Point) -> ObjectId {
    let ws = ws_mut(h);
    let mut text = Object::text(
        ObjectId(0),
        TextBlock::new(content, CharStyle::default()),
        at,
    );
    ws.text.place_at(&mut text, at);
    let id = ws.project.add(text);
    h.run();
    id
}

fn object(h: &H, id: ObjectId) -> Object {
    (**ws(h).project.surface().get(id).expect("object")).clone()
}

fn create_outlines_from_menu(h: &mut H) {
    h.get_by_label("Object").click();
    h.run();
    h.get_by_label("Create Outlines").click();
    h.run();
}

#[test]
fn disabled_without_texts() {
    let mut h = open();
    let r = ws_mut(&mut h).project.add(Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(500.0, 500.0), Size::new(100.0, 100.0), 0.0),
    ));
    ws_mut(&mut h).selection = vec![r];
    h.run();
    assert!(!is_enabled(
        CommandId::CreateOutlines,
        &h.state().edit_context()
    ));
}

#[test]
fn outline_from_the_menu_and_undo_restores_the_text() {
    let mut h = open();
    let id = add_text(&mut h, "ACE", Point::new(1000.0, 1000.0));
    ws_mut(&mut h).selection = vec![id];
    h.run();
    create_outlines_from_menu(&mut h);
    let group = object(&h, id);
    assert!(group.is_group());
    assert_eq!(group.name, "ACE");
    assert_eq!(group.children.len(), 3);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    let text = object(&h, id);
    assert_eq!(text.kind, ShapeKind::Text);
    assert_eq!(text.text.unwrap().content, "ACE");
}

#[test]
fn menu_while_editing_keeps_the_typed_text() {
    let mut h = open();
    ws_mut(&mut h).start_new_text(Point::new(800.0, 900.0), 0.0);
    h.run();
    h.event(Event::Text("TP".into()));
    h.run();
    let id = ws(&h).editing_text().expect("editing");
    // Canvas shortcuts are off while typing: the menu converts.
    create_outlines_from_menu(&mut h);
    assert!(!ws(&h).is_editing_text(), "the session ended");
    let group = object(&h, id);
    assert!(group.is_group(), "{:?}", group.kind);
    let names: Vec<&str> = group.children.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["T", "P"]);
}

/// Exports a rotated, stretched text, outlines it and exports again;
/// returns the largest channel difference and the number of differing
/// pixels.
fn export_difference(stroke: bool) -> (u8, usize) {
    let mut h = open();
    let id = add_text(&mut h, "ROAD", Point::new(1500.0, 1500.0));
    {
        let ws = ws_mut(&mut h);
        let mut text = (**ws.project.surface().get(id).unwrap()).clone();
        text.fill = Rgba::rgb(200, 30, 40).into();
        if stroke {
            text.stroke = Some(StrokeStyle {
                paint: Rgba::rgb(0, 0, 0).into(),
                width: 12.0,
                ..Default::default()
            });
        }
        text.frame.rotation_deg = -12.0;
        text.frame.size.width *= 1.4;
        text.sync_text_scale();
        ws.project.surface_mut().replace(&[text]);
        ws.selection = vec![id];
    }
    h.run();
    let render = |h: &mut H| {
        let ws = h.state_mut().workspace_mut().unwrap();
        let pixmap = tp_render::render(
            &ws.project,
            0,
            tp_render::RenderOptions {
                size: 1024,
                background: Some(Rgba::rgb(255, 255, 255)),
            },
            &mut ws.text.fonts,
            &mut |_, _| true,
        )
        .unwrap();
        tp_render::to_rgba(&pixmap)
    };
    let before = render(&mut h);
    ws_mut(&mut h).create_outlines(1.0);
    h.run();
    assert!(object(&h, id).is_group());
    let after = render(&mut h);
    let mut max = 0u8;
    let mut differing = 0usize;
    for (a, b) in before.pixels().zip(after.pixels()) {
        let d =
            a.0.iter()
                .zip(b.0)
                .map(|(x, y)| x.abs_diff(y))
                .max()
                .unwrap();
        max = max.max(d);
        if d > 0 {
            differing += 1;
        }
    }
    (max, differing)
}

#[test]
fn shortcut_outlines_the_selection() {
    let mut h = open();
    let id = add_text(&mut h, "GO", Point::new(1000.0, 1000.0));
    ws_mut(&mut h).selection = vec![id];
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::O);
    h.run();
    assert_eq!(object(&h, id).children.len(), 2);
}

#[test]
fn outlines_export_like_the_text() {
    // Same geometry, same renderer: the stroked text is pixel-identical.
    let (max, differing) = export_difference(true);
    assert_eq!((max, differing), (0, 0));
}

#[test]
fn unstroked_outlines_differ_only_by_edge_antialiasing() {
    // The font's quadratic curves become exact cubics; the renderer
    // flattens the two curve types slightly differently, which moves a few
    // anti-aliased edge pixels by a fraction of their coverage.
    let (max, differing) = export_difference(false);
    assert!(max <= 64, "at most a quarter of a pixel's coverage: {max}");
    assert!(
        differing < 300,
        "a few edge pixels of the 1024² image: {differing}"
    );
}

#[test]
fn counters_stay_holes() {
    let mut h = open();
    let id = add_text(&mut h, "O", Point::new(1000.0, 1000.0));
    ws_mut(&mut h).selection = vec![id];
    ws_mut(&mut h).create_outlines(1.0);
    h.run();
    let letter = object(&h, id).children[0].clone();
    let path = letter.path_data().unwrap();
    assert_eq!(path.subpaths.len(), 2);
    assert!(path.subpaths.iter().all(|s| s.closed));
    // The center of the "O" is inside the counter: not part of the shape.
    let center = letter.bounding_box().center();
    assert!(!letter.contains(center, 0.0));
}

#[test]
fn reshape_a_letter_with_direct_selection() {
    let mut h = open();
    let id = add_text(&mut h, "SI", Point::new(1500.0, 1500.0));
    ws_mut(&mut h).selection = vec![id];
    ws_mut(&mut h).create_outlines(1.0);
    h.run();
    let group = object(&h, id);
    let (s, i) = (group.children[0].clone(), group.children[1].clone());
    let i_before = i.path();
    // Direct Selection on the "S" shows its points.
    ws_mut(&mut h).tool = Tool::DirectSelect;
    ws_mut(&mut h).selection = vec![s.id];
    h.run();
    assert_eq!(ws(&h).selected_paths().len(), 1);
    // Drag its first point.
    let first = s.frame.affine() * s.path_data().unwrap().subpaths[0].nodes[0].point;
    let map = ws(&h).screen_map(1.0).unwrap();
    let (from, to) = (
        map.to_screen(first),
        map.to_screen(first) + egui::vec2(20.0, 10.0),
    );
    h.event(Event::PointerMoved(from));
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    for k in 1..=6 {
        h.event(Event::PointerMoved(from + (to - from) * (k as f32 / 6.0)));
        h.step();
    }
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run();
    let s_after = ws(&h).project.surface().get(s.id).unwrap().clone();
    let moved = s_after.frame.affine() * s_after.path_data().unwrap().subpaths[0].nodes[0].point;
    assert!(moved.distance(first) > 10.0, "the point moved");
    let i_after = ws(&h).project.surface().get(i.id).unwrap().path();
    assert_eq!(i_after, i_before, "the other letter is unchanged");
}

#[test]
fn outlined_text_round_trips_through_a_file() {
    let mut h = open();
    let id = add_text(&mut h, "ACE", Point::new(1000.0, 1000.0));
    ws_mut(&mut h).selection = vec![id];
    ws_mut(&mut h).create_outlines(1.0);
    h.run();
    let project = ws(&h).project.clone();
    let bytes = tp_file::to_bytes(&project).unwrap();
    let opened = tp_file::from_bytes(&bytes).unwrap().project;
    assert_eq!(opened.surfaces, project.surfaces);
}

#[test]
fn outlined_gradient_text_keeps_the_gradient_in_place() {
    use tp_core::document::{ColorStop, Gradient, GradientKind, Paint};
    let mut h = open();
    let id = add_text(&mut h, "ROAD", Point::new(1500.0, 1500.0));
    let gradient = Gradient::new(
        GradientKind::Linear,
        &[
            ColorStop::new(0.0, Rgba::rgb(240, 180, 76)),
            ColorStop::new(1.0, Rgba::rgb(122, 31, 43)),
        ],
    );
    let text = {
        let ws = ws_mut(&mut h);
        let mut text = (**ws.project.surface().get(id).unwrap()).clone();
        text.fill = Paint::Gradient(gradient);
        text.frame.rotation_deg = -12.0;
        ws.project.surface_mut().replace(&[text.clone()]);
        ws.selection = vec![id];
        text
    };
    h.run();
    let want = gradient.document_points(&text.frame);
    ws_mut(&mut h).create_outlines(1.0);
    h.run();
    let group = object(&h, id);
    assert_eq!(group.children.len(), 4);
    for letter in &group.children {
        let got = letter
            .fill
            .gradient()
            .unwrap()
            .document_points(&letter.frame);
        assert!(
            (got.0 - want.0).hypot() < 1e-6 && (got.1 - want.1).hypot() < 1e-6,
            "{} {got:?} {want:?}",
            letter.name
        );
    }
}
