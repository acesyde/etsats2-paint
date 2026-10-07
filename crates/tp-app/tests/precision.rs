//! Headless tests for precision-aids: rulers, guides, grid and snapping.

mod common;

use egui::accesskit::{Role, Toggled};
use egui::{Event, Key, Modifiers, PointerButton, Pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::commands::CommandId;
use tp_app::state::{Modal, is_enabled};
use tp_app::workspace::Workspace;
use tp_core::kurbo::Point;
use tp_core::{Axis, Guide};

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

/// Opens the View menu and returns whether `label` is checked.
fn view_checked(h: &mut H, label: &str) -> bool {
    h.get_by_label("View").click();
    h.run();
    let checked = h.get_by_label(label).accesskit_node().toggled() == Some(Toggled::True);
    h.key_press(Key::Escape);
    h.run();
    checked
}

// --- commands and preferences ------------------------------------------------

#[test]
fn show_grid_toggles_with_its_check_mark() {
    let mut h = open();
    assert!(!h.state().prefs.view_aids.grid, "hidden by default");
    assert!(!view_checked(&mut h, "Show Grid"));
    h.key_press_modifiers(Modifiers::COMMAND, Key::Quote);
    h.run();
    assert!(h.state().prefs.view_aids.grid);
    assert!(ws(&h).aids.grid, "the workspace sees the setting");
    assert!(view_checked(&mut h, "Show Grid"));
}

#[test]
fn hide_guides() {
    let mut h = open();
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Vertical, 500.0));
    assert!(view_checked(&mut h, "Show Guides"), "shown by default");
    h.key_press_modifiers(Modifiers::COMMAND, Key::Semicolon);
    h.run();
    assert!(!h.state().prefs.view_aids.guides);
    assert!(!view_checked(&mut h, "Show Guides"));
}

#[test]
fn snapping_toggle() {
    let mut h = open();
    assert!(view_checked(&mut h, "Snapping"), "on by default");
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Semicolon);
    h.run();
    assert!(!h.state().prefs.view_aids.snapping);
}

#[test]
fn clear_guides_is_one_undo_step() {
    let mut h = open();
    assert!(!is_enabled(
        CommandId::ClearGuides,
        &h.state().edit_context()
    ));
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Vertical, 500.0));
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Horizontal, 700.0));
    h.run();
    assert!(is_enabled(
        CommandId::ClearGuides,
        &h.state().edit_context()
    ));
    h.get_by_label("View").click();
    h.run();
    h.get_by_label("Clear Guides").click();
    h.run();
    assert!(ws(&h).project.surface().guides.is_empty());
    assert_eq!(ws(&h).history.undo_label(), Some("cmd-clear-guides"));
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(ws(&h).project.surface().guides.len(), 2);
}

#[test]
fn coarser_grid_and_reset() {
    let mut h = open();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::Preferences)));
    h.get_by_role_and_label(Role::SpinButton, "Grid spacing")
        .click();
    h.run();
    // The drag value is now a focused text field: type into it.
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.event(Event::Text("256".into()));
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().prefs.view_aids.grid_spacing, 256.0);
    h.get_by_label("Reset to defaults").click();
    h.run();
    assert_eq!(h.state().prefs.view_aids.grid_spacing, 64.0);
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
        let t = i as f32 / 8.0;
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

fn canvas(h: &H) -> egui::Rect {
    ws(h).canvas_rect.expect("canvas shown")
}

/// A point on the top ruler above texture x, or on the left ruler beside y.
fn on_top_ruler(h: &H, x: f64) -> Pos2 {
    Pos2::new(screen(h, x, 0.0).x, canvas(h).top() - 10.0)
}

fn on_left_ruler(h: &H, y: f64) -> Pos2 {
    Pos2::new(canvas(h).left() - 10.0, screen(h, 0.0, y).y)
}

fn guides(h: &H) -> Vec<Guide> {
    ws(h).project.surface().guides.clone()
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.5
}

// --- rulers and guides -------------------------------------------------------

#[test]
fn rulers_are_shown_and_mark_the_pointer() {
    let mut h = open();
    assert!(h.query_by_label("Horizontal ruler").is_some());
    assert!(h.query_by_label("Vertical ruler").is_some());
    let top = h.get_by_label("Horizontal ruler").rect();
    assert!(
        top.bottom() <= canvas(&h).top() + 0.5,
        "the canvas starts below the ruler"
    );
    // The marker positions are the pointer's texture coordinates.
    let p = screen(&h, 1200.0, 800.0);
    let map = ws(&h).screen_map(1.0).unwrap();
    use tp_app::ui::workspace::canvas::aids::ruler_value;
    assert!(near(ruler_value(Axis::Horizontal, &map, p), 1200.0));
    assert!(near(ruler_value(Axis::Vertical, &map, p), 800.0));
    h.event(Event::PointerMoved(p));
    h.run();
}

#[test]
fn horizontal_guide_from_the_top_ruler() {
    let mut h = open();
    let (from, to) = (on_top_ruler(&h, 1500.0), screen(&h, 1500.0, 1024.0));
    drag(&mut h, from, to, Modifiers::NONE);
    let g = guides(&h);
    assert_eq!(g.len(), 1);
    assert_eq!(g[0].axis, Axis::Horizontal);
    assert!(near(g[0].position, 1024.0), "{g:?}");
    assert_eq!(ws(&h).history.undo_label(), Some("undo-add-guide"));
}

#[test]
fn creating_a_guide_shows_hidden_guides() {
    let mut h = open();
    h.state_mut().prefs.view_aids.guides = false;
    h.run();
    let (from, to) = (on_left_ruler(&h, 1500.0), screen(&h, 700.0, 1500.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert_eq!(guides(&h)[0].axis, Axis::Vertical);
    assert!(h.state().prefs.view_aids.guides);
}

#[test]
fn released_back_on_the_ruler() {
    let mut h = open();
    let (from, to) = (on_left_ruler(&h, 1500.0), on_left_ruler(&h, 2500.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert!(guides(&h).is_empty());
}

#[test]
fn move_a_guide_and_undo() {
    let mut h = open();
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Vertical, 500.0));
    h.run();
    let (from, to) = (screen(&h, 500.0, 2000.0), screen(&h, 2048.0, 2100.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert!(near(guides(&h)[0].position, 2048.0), "{:?}", guides(&h));
    assert_eq!(ws(&h).history.undo_label(), Some("undo-move-guide"));
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(guides(&h)[0].position, 500.0);
}

#[test]
fn delete_by_dropping_on_a_ruler() {
    let mut h = open();
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Horizontal, 1000.0));
    h.run();
    let (from, to) = (screen(&h, 2000.0, 1000.0), on_top_ruler(&h, 2000.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert!(guides(&h).is_empty());
    assert_eq!(ws(&h).history.undo_label(), Some("undo-delete-guide"));
}

#[test]
fn objects_win_inside_their_shape_and_hidden_guides_are_inert() {
    use tp_core::document::{Frame, Object, ObjectId, ShapeKind};
    use tp_core::kurbo::Size;
    let mut h = open();
    let rect = ws_mut(&mut h).project.add(Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(2000.0, 2000.0), Size::new(400.0, 400.0), 0.0),
    ));
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Vertical, 2000.0));
    h.run();
    let (from, to) = (screen(&h, 2000.0, 2000.0), screen(&h, 2200.0, 2000.0));
    drag(&mut h, from, to, Modifiers::NONE);
    let moved = ws(&h).project.surface().get(rect).unwrap().frame.center.x;
    assert!((moved - 2200.0).abs() < 1.0, "the rectangle moved: {moved}");
    assert_eq!(guides(&h)[0].position, 2000.0);
    // Hidden guides cannot be grabbed: a drag over one draws a marquee.
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Horizontal, 600.0));
    h.state_mut().prefs.view_aids.guides = false;
    h.run();
    let (from, to) = (screen(&h, 1000.0, 600.0), screen(&h, 1000.0, 900.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert_eq!(guides(&h)[1].position, 600.0);
}

#[test]
fn grid_and_guides_are_not_exported() {
    let mut h = open();
    h.state_mut().prefs.view_aids.grid = true;
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Vertical, 1024.0));
    h.run();
    let ws = h.state_mut().workspace_mut().unwrap();
    let pixmap = tp_render::render(
        &ws.project,
        0,
        tp_render::RenderOptions {
            size: 1024,
            background: Some(tp_core::document::Rgba::rgb(255, 255, 255)),
        },
        &mut ws.text.fonts,
        &mut |_, _| true,
    )
    .unwrap();
    let img = tp_render::to_rgba(&pixmap);
    assert!(img.pixels().all(|p| p.0 == [255, 255, 255, 255]));
}

// --- snapping -------------------------------------------------------------------

use tp_app::snap::{SnapHit, Source};
use tp_core::document::{Frame, Node, Object, ObjectId, PathData, ShapeKind, Subpath};
use tp_core::kurbo::Size;

fn rect(h: &mut H, x: f64, y: f64, w: f64, hh: f64) -> ObjectId {
    let id = ws_mut(h).project.add(Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(x, y), Size::new(w, hh), 0.0),
    ));
    h.run();
    id
}

fn bounds(h: &H, id: ObjectId) -> tp_core::kurbo::Rect {
    ws(h).project.surface().get(id).unwrap().bounding_box()
}

/// Texture pixels per screen point at the current zoom.
fn px_per_pt(h: &H) -> f64 {
    ws(h).screen_map(1.0).unwrap().doc_len(1.0)
}

/// Presses at `from` and moves to `to` in steps with `during` held after the
/// press, without releasing.
fn press_and_move(h: &mut H, from: Pos2, to: Pos2, during: Modifiers) {
    h.event(Event::PointerMoved(from));
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.event(Event::ModifiersChanged(during));
    for i in 1..=8 {
        let t = i as f32 / 8.0;
        h.event(Event::PointerMoved(from + (to - from) * t));
        h.step();
    }
}

fn release(h: &mut H, at: Pos2) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.run();
}

#[test]
fn snap_to_a_guide() {
    let mut h = open();
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Vertical, 1000.0));
    let r = rect(&mut h, 700.0, 1500.0, 200.0, 100.0);
    // Left edge from 600 to 3 screen points before the guide.
    let dx = 400.0 - 3.0 * px_per_pt(&h);
    let (from, to) = (screen(&h, 700.0, 1500.0), screen(&h, 700.0 + dx, 1500.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert!(
        (bounds(&h, r).x0 - 1000.0).abs() < 1e-6,
        "{:?}",
        bounds(&h, r)
    );
}

#[test]
fn snap_to_the_artboard_center() {
    let mut h = open();
    let r = rect(&mut h, 1000.0, 1500.0, 300.0, 100.0);
    let (from, to) = (screen(&h, 1000.0, 1500.0), screen(&h, 2040.0, 1500.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert!(
        (bounds(&h, r).center().x - 2048.0).abs() < 1e-6,
        "{:?}",
        bounds(&h, r)
    );
}

#[test]
fn turn_snapping_off_and_temporary_bypass() {
    let mut h = open();
    let r = rect(&mut h, 1000.0, 1500.0, 300.0, 100.0);
    // Cmd/Ctrl held during the drag: no snapping.
    let (from, to) = (screen(&h, 1000.0, 1500.0), screen(&h, 2040.0, 1500.0));
    press_and_move(&mut h, from, to, Modifiers::COMMAND);
    release(&mut h, to);
    let x = bounds(&h, r).center().x;
    assert!(
        (x - 2048.0).abs() > 2.0,
        "not snapped while Cmd/Ctrl is held: {x}"
    );
    // Snapping off: no snapping either.
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.state_mut().prefs.view_aids.snapping = false;
    h.run();
    drag(&mut h, from, to, Modifiers::NONE);
    let x = bounds(&h, r).center().x;
    assert!((x - 2048.0).abs() > 2.0, "{x}");
}

#[test]
fn hidden_grid_does_not_snap() {
    let mut h = open();
    let r = rect(&mut h, 1000.0, 1000.0, 100.0, 100.0);
    // Left edge to 1290 (grid 64 would give 1280; no other target nearby).
    let (from, to) = (screen(&h, 1000.0, 1000.0), screen(&h, 1340.0, 1000.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert!(
        (bounds(&h, r).x0 - 1290.0).abs() < 1.0,
        "{:?}",
        bounds(&h, r)
    );
}

#[test]
fn drawing_from_a_grid_intersection() {
    let mut h = open();
    h.state_mut().prefs.view_aids.grid = true;
    ws_mut(&mut h).tool = tp_app::tool::Tool::Rectangle;
    h.run();
    let off = 2.0 * px_per_pt(&h);
    let (from, to) = (
        screen(&h, 128.0 + off, 256.0 + off),
        screen(&h, 700.0, 600.0),
    );
    drag(&mut h, from, to, Modifiers::NONE);
    let id = ws(&h).selection[0];
    let b = bounds(&h, id);
    assert!(
        (b.x0 - 128.0).abs() < 1e-6 && (b.y0 - 256.0).abs() < 1e-6,
        "{b:?}"
    );
}

#[test]
fn pen_point_on_another_paths_point() {
    let mut h = open();
    ws_mut(&mut h).project.add(Object::from_path(
        ObjectId(0),
        PathData::new(vec![Subpath::new(
            vec![
                Node::corner(Point::new(1500.0, 1500.0)),
                Node::corner(Point::new(2500.0, 1700.0)),
            ],
            false,
        )]),
    ));
    ws_mut(&mut h).tool = tp_app::tool::Tool::Pen;
    h.run();
    let off = 3.0 * px_per_pt(&h);
    let at = screen(&h, 1500.0 + off, 1500.0);
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
    h.run();
    let p = ws(&h).pen.as_ref().unwrap().nodes[0].point;
    assert_eq!(p, Point::new(1500.0, 1500.0));
}

#[test]
fn smart_guide_between_two_objects() {
    let mut h = open();
    rect(&mut h, 600.0, 1000.0, 200.0, 200.0); // top edge at 900
    let b = rect(&mut h, 1300.0, 1300.0, 200.0, 200.0); // top edge at 1200
    let dy = -300.0 + 2.0 * px_per_pt(&h);
    let (from, to) = (screen(&h, 1300.0, 1300.0), screen(&h, 1300.0, 1300.0 + dy));
    press_and_move(&mut h, from, to, Modifiers::NONE);
    let hits = ws(&h).snap_hits.clone();
    assert!(
        hits.iter().any(|hit| matches!(
            hit,
            SnapHit::Line {
                axis: Axis::Horizontal,
                value,
                extent: Some(_),
                source: Source::Object(_),
            } if (*value - 900.0).abs() < 1e-9
        )),
        "{hits:?}"
    );
    release(&mut h, to);
    assert!((bounds(&h, b).y0 - 900.0).abs() < 1e-6);
    assert!(
        ws(&h).snap_hits.is_empty(),
        "lines disappear when the drag ends"
    );
}

#[test]
fn dragged_guide_snaps_to_the_artboard_center() {
    let mut h = open();
    let off = 3.0 * px_per_pt(&h);
    let (from, to) = (on_left_ruler(&h, 1500.0), screen(&h, 2048.0 + off, 1500.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert_eq!(guides(&h)[0].position, 2048.0);
}

#[test]
fn resizing_snaps_the_dragged_handle() {
    let mut h = open();
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Vertical, 1200.0));
    let r = rect(&mut h, 1000.0, 1500.0, 200.0, 200.0); // right edge 1100
    ws_mut(&mut h).selection = vec![r];
    h.run();
    let off = 3.0 * px_per_pt(&h);
    // Right-middle handle to 3 points before the guide.
    let (from, to) = (screen(&h, 1100.0, 1500.0), screen(&h, 1200.0 - off, 1500.0));
    drag(&mut h, from, to, Modifiers::NONE);
    assert!(
        (bounds(&h, r).x1 - 1200.0).abs() < 1e-6,
        "{:?}",
        bounds(&h, r)
    );
}

#[test]
fn direct_selection_point_snaps() {
    let mut h = open();
    ws_mut(&mut h)
        .project
        .add_guide(Guide::new(Axis::Horizontal, 1000.0));
    let id = ws_mut(&mut h).project.add(Object::from_path(
        ObjectId(0),
        PathData::new(vec![Subpath::new(
            vec![
                Node::corner(Point::new(1500.0, 1500.0)),
                Node::corner(Point::new(2500.0, 1700.0)),
            ],
            false,
        )]),
    ));
    ws_mut(&mut h).tool = tp_app::tool::Tool::DirectSelect;
    ws_mut(&mut h).selection = vec![id];
    h.run();
    let off = 3.0 * px_per_pt(&h);
    let (from, to) = (screen(&h, 1500.0, 1500.0), screen(&h, 1500.0, 1000.0 + off));
    drag(&mut h, from, to, Modifiers::NONE);
    let o = ws(&h).project.surface().get(id).unwrap().clone();
    let p = o.frame.affine() * o.path_data().unwrap().subpaths[0].nodes[0].point;
    assert!((p.y - 1000.0).abs() < 1e-6, "{p:?}");
}

#[test]
#[ignore = "performance check; run with --release -- --ignored"]
fn pan_with_grid_and_guides_stays_fast() {
    let mut h = open();
    h.state_mut().prefs.view_aids.grid = true;
    h.state_mut().prefs.view_aids.grid_spacing = 16.0;
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
    for i in 0..20 {
        let axis = if i % 2 == 0 {
            Axis::Vertical
        } else {
            Axis::Horizontal
        };
        ws_mut(&mut h)
            .project
            .add_guide(Guide::new(axis, f64::from(i) * 200.0));
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
