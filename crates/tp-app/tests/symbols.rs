//! Headless tests for symbols: Convert to Symbol, the Symbols section, editing
//! a symbol in its own view, Detach Instance, and instances in the editor.

mod common;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::commands::CommandId;
use tp_app::prefs::Prefs;
use tp_app::workspace::{ColorTarget, Workspace};
use tp_core::document::{Frame, Object, ObjectId, Paint, Rgba, ShapeKind, SymbolId};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

/// Tall window with the Resources tab shown; the sample truck project
/// (Standard cab, Chassis, …).
fn open() -> H {
    let mut prefs = Prefs::default();
    prefs.layout.left_tab = tp_app::layout::LeftTab::Resources;
    prefs.view_aids.snapping = false;
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, 2400.0))
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

fn add(h: &mut H, kind: ShapeKind, x: f64) -> ObjectId {
    let id = ws_mut(h).project.add(Object::new(
        ObjectId(0),
        kind,
        Frame::new(Point::new(x, 800.0), Size::new(200.0, 200.0), 0.0),
    ));
    h.run();
    id
}

fn get(h: &H, surface: usize, id: ObjectId) -> Object {
    let list = &ws(h).project.surfaces[surface].objects;
    (**tp_core::document::tree::get(list, id).expect("object")).clone()
}

fn run_command(h: &mut H, id: CommandId) {
    h.state_mut().queue.push(id);
    h.run_steps(3);
    h.run();
}

fn screen(h: &H, x: f64, y: f64) -> Pos2 {
    ws(h)
        .screen_map(1.0)
        .expect("canvas shown")
        .to_screen(Point::new(x, y))
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
    h.run();
}

fn drag(h: &mut H, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
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
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.run();
}

/// A circle and a rectangle on the Standard cab converted to "Symbol 1";
/// returns the symbol and its instance (selected).
fn convert_a_logo(h: &mut H) -> (SymbolId, ObjectId) {
    let a = add(h, ShapeKind::Ellipse, 600.0);
    let b = add(h, ShapeKind::rectangle(), 900.0);
    ws_mut(h).selection = vec![a, b];
    h.run();
    run_command(h, CommandId::ConvertToSymbol);
    let ws = ws(h);
    (ws.project.symbols[0].id, ws.selection[0])
}

fn chassis(h: &H) -> usize {
    ws(h)
        .project
        .surfaces
        .iter()
        .position(|s| s.name == "Chassis")
        .unwrap()
}

#[test]
fn convert_a_logo_and_place_it_on_another_texture() {
    let mut h = open();
    let (symbol, instance) = convert_a_logo(&mut h);
    assert!(get(&h, 0, instance).is_instance());
    assert!(h.query_by_label("Symbol Symbol 1").is_some());
    assert!(h.query_by_label("1 instance").is_some());
    // Place on the Chassis, in the middle of the view.
    let chassis = chassis(&h);
    ws_mut(&mut h).set_active_surface(chassis);
    h.run();
    h.get_by_label("Place Symbol 1").click();
    h.run();
    let placed = ws(&h).selection[0];
    let o = get(&h, chassis, placed);
    assert!(o.is_instance());
    let center = ws(&h).view_center();
    let shown = o.bounding_box().center();
    assert!(shown.distance(center) < 1.0, "{shown:?} vs {center:?}");
    assert_eq!(ws(&h).project.instance_count(symbol), 2);
    assert!(h.query_by_label("2 instances").is_some());
}

#[test]
fn delete_a_used_symbol_and_undo() {
    let mut h = open();
    let (symbol, instance) = convert_a_logo(&mut h);
    for _ in 0..2 {
        h.get_by_label("Place Symbol 1").click();
        h.run();
    }
    assert_eq!(ws(&h).project.instance_count(symbol), 3);
    h.get_by_label("Symbol Symbol 1").click_secondary();
    h.run();
    h.get_by_label("Delete Symbol").click();
    h.run();
    assert!(
        h.query_by_label("Delete Symbol 1? Its 3 instances become groups that look the same.")
            .is_some()
    );
    h.get_by_label("Delete Symbol").click();
    h.run();
    assert!(ws(&h).project.symbols.is_empty());
    assert!(get(&h, 0, instance).is_group());
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.instance_count(symbol), 3);
    assert!(get(&h, 0, instance).is_instance());
}

#[test]
fn duplicate_for_a_variant() {
    let mut h = open();
    let (symbol, _) = convert_a_logo(&mut h);
    h.get_by_label("Symbol Symbol 1").click_secondary();
    h.run();
    h.get_by_label("Duplicate").click();
    h.run();
    let copy = &ws(&h).project.symbols[1];
    assert_eq!(copy.name, "Symbol 1 copy");
    assert_eq!(ws(&h).project.instance_count(copy.id), 0);
    assert!(h.query_by_label("0 instances").is_some());
    assert_ne!(copy.id, symbol);
}

#[test]
fn edit_from_an_instance() {
    let mut h = open();
    let (symbol, instance) = convert_a_logo(&mut h);
    let chassis = chassis(&h);
    ws_mut(&mut h).set_active_surface(chassis);
    h.run();
    h.get_by_label("Place Symbol 1").click();
    h.run();
    let other = ws(&h).selection[0];
    ws_mut(&mut h).set_active_surface(0);
    h.run();
    // Double-click the circle of the instance.
    let at = screen(&h, 600.0, 800.0);
    double_click_at(&mut h, at);
    assert_eq!(ws(&h).project.editing_symbol, Some(symbol));
    assert!(h.query_by_label("Editing symbol Symbol 1").is_some());
    // Recolor the circle in the symbol, then Done.
    let first = ws(&h).project.surface().objects[0].id;
    {
        let ws = ws_mut(&mut h);
        ws.selection = vec![first];
        ws.apply_color(ColorTarget::Fill, Rgba::rgb(9, 99, 9));
        ws.commit_pending(1.0);
    }
    h.run();
    h.get_by_label("Done").click();
    h.run();
    assert!(!ws(&h).is_editing_symbol());
    let green = Paint::Solid(Rgba::rgb(9, 99, 9));
    assert_eq!(get(&h, 0, instance).children[0].fill, green);
    assert_eq!(get(&h, chassis, other).children[0].fill, green);
}

#[test]
fn escape_clears_the_selection_then_leaves() {
    let mut h = open();
    let (symbol, _) = convert_a_logo(&mut h);
    run_command(&mut h, CommandId::EditSymbol);
    assert_eq!(ws(&h).project.editing_symbol, Some(symbol));
    let first = ws(&h).project.surface().objects[0].id;
    ws_mut(&mut h).selection = vec![first];
    h.run();
    h.key_press(Key::Escape);
    h.run();
    assert!(ws(&h).selection.is_empty());
    assert!(
        ws(&h).is_editing_symbol(),
        "a first Escape clears the selection"
    );
    h.key_press(Key::Escape);
    h.run();
    assert!(!ws(&h).is_editing_symbol(), "the next one leaves");
}

#[test]
fn colors_dont_apply_to_an_instance() {
    let mut h = open();
    let _ = convert_a_logo(&mut h);
    assert!(
        !h.query_all_by_label(
            "An instance shows its symbol: edit the symbol to change its look, or detach the instance."
        )
        .collect::<Vec<_>>()
        .is_empty()
    );
    let context = h.state().edit_context();
    let reason = tp_app::state::disabled_reason_for(CommandId::ConvertToPath, &context);
    assert!(tp_i18n::tr(reason.unwrap()).contains("edit the symbol"));
}

#[test]
fn a_one_off_variant() {
    let mut h = open();
    let (symbol, instance) = convert_a_logo(&mut h);
    h.get_by_label("Place Symbol 1").click();
    h.run();
    let variant = ws(&h).selection[0];
    h.get_by_role_and_label(Role::Button, "Detach Instance")
        .click();
    h.run();
    let group = get(&h, 0, variant);
    assert!(group.is_group());
    // Recolor a shape of the detached copy: only it changes.
    {
        let ws = ws_mut(&mut h);
        ws.selection = vec![group.children[0].id];
        ws.apply_color(ColorTarget::Fill, Rgba::rgb(250, 250, 250));
        ws.commit_pending(1.0);
    }
    h.run();
    assert_eq!(
        get(&h, 0, variant).children[0].fill,
        Paint::Solid(Rgba::rgb(250, 250, 250))
    );
    assert_ne!(
        get(&h, 0, instance).children[0].fill,
        Paint::Solid(Rgba::rgb(250, 250, 250))
    );
    assert_eq!(ws(&h).project.instance_count(symbol), 1);
}

#[test]
fn resize_an_instance() {
    let mut h = open();
    let (_, instance) = convert_a_logo(&mut h);
    let before = get(&h, 0, instance);
    let f = before.frame;
    // Bottom-right handle, dragged until the size doubles (top-left fixed).
    let corner = f.affine() * Point::new(f.size.width / 2.0, f.size.height / 2.0);
    let target =
        corner + (corner - (f.affine() * Point::new(-f.size.width / 2.0, -f.size.height / 2.0)));
    let (from, to) = (
        screen(&h, corner.x, corner.y),
        screen(&h, target.x, target.y),
    );
    drag(&mut h, from, to);
    let after = get(&h, 0, instance);
    assert!(after.is_instance());
    assert!((after.frame.size.width - 2.0 * f.size.width).abs() < 2.0);
    let child = &after.children[0];
    assert!((child.frame.size.width - 2.0 * before.children[0].frame.size.width).abs() < 2.0);
    assert!(
        (child.frame.size.width / child.frame.size.height - 1.0).abs() < 0.01,
        "the circle keeps its proportions"
    );
    assert_eq!(child.fill, before.children[0].fill, "and its look");
}

#[test]
fn an_instance_in_the_layers_tab() {
    let mut h = open();
    let (_, instance) = convert_a_logo(&mut h);
    let object = get(&h, 0, instance);
    common::show_tab(&mut h, tp_app::layout::LeftTab::Textures);
    let elsewhere = h.query_all_by_label(&object.name).count();
    common::show_tab(&mut h, tp_app::layout::LeftTab::Layers);
    // One row named after the instance, with no expander, drawn with the
    // symbol icon in the link color.
    assert_eq!(h.query_all_by_label(&object.name).count(), elsewhere + 1);
    assert!(
        h.query_by_label(&format!("Expand {}", object.name))
            .is_none()
    );
    let (icon, name) = tp_app::ui::workspace::panels::layers::row_colors(&object, false, false);
    assert_eq!(
        (icon, name),
        (tp_ui::tokens::color::LINK, tp_ui::tokens::color::LINK)
    );
}
