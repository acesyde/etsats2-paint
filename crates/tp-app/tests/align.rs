//! Headless tests for align-and-distribute: commands, menu, shortcuts,
//! Transform panel buttons and the key object.

mod common;

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::arrange::AlignTo;
use tp_app::commands::CommandId;
use tp_app::state::is_enabled;
use tp_app::workspace::Workspace;
use tp_core::document::{DistributeAxis, DistributeMode, Edge, Frame, Object, ObjectId, ShapeKind};
use tp_core::kurbo::{Point, Rect, Size};

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

/// A rectangle by its top-left corner and size.
fn rect(h: &mut H, x0: f64, y0: f64, w: f64, hh: f64) -> ObjectId {
    let id = ws_mut(h).project.add(Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(
            Point::new(x0 + w / 2.0, y0 + hh / 2.0),
            Size::new(w, hh),
            0.0,
        ),
    ));
    h.run();
    id
}

fn bounds(h: &H, id: ObjectId) -> Rect {
    ws(h).project.surface().get(id).unwrap().bounding_box()
}

fn select(h: &mut H, ids: &[ObjectId]) {
    ws_mut(h).selection = ids.to_vec();
    h.run();
}

fn enabled(h: &H, id: CommandId) -> bool {
    is_enabled(id, &h.state().edit_context())
}

fn from_align_menu(h: &mut H, label: &str) {
    h.get_by_label("Object").click();
    h.run();
    // egui labels submenu buttons with an arrow.
    h.get_by_label("Align ⏵").click();
    h.run();
    h.get_by_label(label).click();
    h.run();
}

#[test]
fn center_a_logo_on_the_texture() {
    let mut h = open();
    let logo = rect(&mut h, 300.0, 500.0, 400.0, 200.0);
    select(&mut h, &[logo]);
    from_align_menu(&mut h, "Align Horizontal Centers");
    assert_eq!(bounds(&h, logo).center().x, 2048.0);
    assert_eq!(bounds(&h, logo).y0, 500.0, "only x moved");
    assert_eq!(
        ws(&h).history.undo_label(),
        Some("op-align-horizontal-centers")
    );
}

#[test]
fn several_objects_to_the_artboard() {
    let mut h = open();
    let a = rect(&mut h, 100.0, 100.0, 200.0, 200.0);
    let b = rect(&mut h, 900.0, 600.0, 100.0, 300.0);
    ws_mut(&mut h).panels.align_to = AlignTo::Artboard;
    select(&mut h, &[a, b]);
    from_align_menu(&mut h, "Align Bottom");
    assert_eq!(bounds(&h, a).y1, 4096.0);
    assert_eq!(bounds(&h, b).y1, 4096.0);
}

#[test]
fn align_left_edges_to_the_selection() {
    let mut h = open();
    let ids = [
        rect(&mut h, 100.0, 100.0, 100.0, 100.0),
        rect(&mut h, 300.0, 400.0, 50.0, 50.0),
        rect(&mut h, 500.0, 900.0, 80.0, 20.0),
    ];
    select(&mut h, &ids);
    h.key_press_modifiers(Modifiers::ALT, Key::A);
    h.run();
    for id in ids {
        assert_eq!(bounds(&h, id).x0, 100.0);
    }
    assert_eq!(bounds(&h, ids[1]).y0, 400.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert_eq!(bounds(&h, ids[2]).x0, 500.0, "one undo step restores all");
}

#[test]
fn equal_spacing_shortcut() {
    let mut h = open();
    let ids = [
        rect(&mut h, 0.0, 0.0, 100.0, 50.0),
        rect(&mut h, 150.0, 200.0, 200.0, 50.0),
        rect(&mut h, 900.0, 400.0, 100.0, 50.0),
    ];
    select(&mut h, &ids);
    h.key_press_modifiers(Modifiers::ALT | Modifiers::SHIFT, Key::H);
    h.run();
    assert_eq!(bounds(&h, ids[1]).x0, 400.0);
    assert_eq!(bounds(&h, ids[0]).x0, 0.0);
    assert_eq!(bounds(&h, ids[2]).x0, 900.0);
    assert_eq!(
        ws(&h).history.undo_label(),
        Some("op-distribute-horizontal-spacing")
    );
}

#[test]
fn not_enough_objects_and_nothing_selected() {
    let mut h = open();
    let a = rect(&mut h, 0.0, 0.0, 100.0, 50.0);
    let b = rect(&mut h, 300.0, 0.0, 100.0, 50.0);
    let all_align = Edge::ALL.map(CommandId::Align);
    let distribute = CommandId::Distribute(DistributeAxis::Horizontal, DistributeMode::Spacing);
    assert!(
        all_align.iter().all(|c| !enabled(&h, *c)),
        "nothing selected"
    );
    assert!(!enabled(&h, distribute));
    select(&mut h, &[a, b]);
    assert!(all_align.iter().all(|c| enabled(&h, *c)));
    assert!(
        !enabled(&h, distribute),
        "two objects cannot be distributed"
    );
    // Key object mode needs two objects.
    ws_mut(&mut h).panels.align_to = AlignTo::KeyObject;
    select(&mut h, &[a]);
    assert!(!enabled(&h, CommandId::Align(Edge::Left)));
}

/// Tall window with the Transform panel open and expanded.
fn open_with_panels() -> H {
    let mut prefs = tp_app::prefs::Prefs::default();
    for slot in &mut prefs.layout.panels {
        slot.open = !matches!(
            slot.kind,
            tp_app::layout::PanelKind::Assets | tp_app::layout::PanelKind::Vehicle
        );
        slot.collapsed = false;
    }
    let mut h = Harness::builder()
        .with_size(egui::Vec2::new(1440.0, 2400.0))
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(prefs, None),
        );
    common::create_project(&mut h);
    h.run();
    h
}

#[test]
fn align_from_the_panel() {
    let mut h = open_with_panels();
    let a = rect(&mut h, 100.0, 100.0, 100.0, 100.0);
    let b = rect(&mut h, 600.0, 450.0, 100.0, 100.0);
    select(&mut h, &[a, b]);
    h.get_by_label("Align Top").click();
    h.run();
    assert_eq!(bounds(&h, a).y0, 100.0);
    assert_eq!(bounds(&h, b).y0, 100.0);
    assert_eq!(ws(&h).history.undo_label(), Some("op-align-top"));
    // Two objects: the distribute buttons are disabled.
    let distribute = h.get_by_label("Distribute Horizontal Spacing");
    assert!(distribute.accesskit_node().is_disabled());
}

#[test]
fn align_to_selector_in_the_panel() {
    let mut h = open_with_panels();
    let a = rect(&mut h, 100.0, 100.0, 100.0, 100.0);
    select(&mut h, &[a]);
    h.get_by_label("Align to").click();
    h.run();
    h.get_by_label("Key object").click();
    h.run();
    assert_eq!(ws(&h).panels.align_to, AlignTo::KeyObject);
    // One object with a key object: nothing to align to.
    assert!(h.get_by_label("Align Top").accesskit_node().is_disabled());
}

fn click_doc(h: &mut H, x: f64, y: f64, mods: Modifiers) {
    let at = ws(h).screen_map(1.0).unwrap().to_screen(Point::new(x, y));
    h.event(egui::Event::ModifiersChanged(mods));
    h.event(egui::Event::PointerMoved(at));
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: mods,
        });
    }
    h.step();
    h.event(egui::Event::ModifiersChanged(Modifiers::NONE));
    h.run();
    for _ in 0..30 {
        h.step();
    }
}

#[test]
fn align_to_a_key_object() {
    let mut h = open();
    let stripe = rect(&mut h, 300.0, 1500.0, 1200.0, 100.0);
    let logo = rect(&mut h, 2500.0, 800.0, 400.0, 400.0);
    ws_mut(&mut h).panels.align_to = AlignTo::KeyObject;
    h.run();
    click_doc(&mut h, 900.0, 1550.0, Modifiers::NONE);
    click_doc(&mut h, 2700.0, 1000.0, Modifiers::SHIFT);
    assert_eq!(ws(&h).selection, vec![stripe, logo]);
    assert_eq!(ws(&h).key_object(), Some(logo));
    h.key_press_modifiers(Modifiers::ALT, Key::W);
    h.run();
    assert_eq!(
        bounds(&h, stripe).y0,
        800.0,
        "the stripe's top moved to the logo's"
    );
    assert_eq!(
        bounds(&h, logo),
        Rect::new(2500.0, 800.0, 2900.0, 1200.0),
        "the key object did not move"
    );
}
