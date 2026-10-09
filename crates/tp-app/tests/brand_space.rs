//! Headless tests for the Brand space: the section index, Import from
//! Library… in the header, and the actions of the Palette, Graphic styles,
//! Text styles, Symbols and Images sections; only those that place
//! something or open an edit view show the Workshop.

mod common;

use egui::accesskit::{Role, Toggled};
use egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::file_dialogs::ScriptedDialogs;
use tp_app::layout::Space;
use tp_app::state::Modal;
use tp_app::workspace::Workspace;
use tp_core::document::{
    CharStyle, Frame, Object, ObjectId, Paint, Rgba, ShapeKind, SymbolId, TextBlock,
};
use tp_core::kurbo::{Point, Size};

type H = Harness<'static, AppState>;

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#1E8C3A"/></svg>"##;

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

/// A project for the sample truck in a window `height` high, in the
/// Workshop.
fn open_with_height(height: f32) -> H {
    let mut h = Harness::builder()
        .with_size(Vec2::new(1440.0, height))
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(Default::default(), None),
        );
    common::create_project(&mut h);
    h.run();
    h
}

fn open() -> H {
    open_with_height(2400.0)
}

fn space(h: &H) -> Space {
    ws(h).space
}

/// Shows the Brand space with Cmd/Ctrl+3.
fn show_brand(h: &mut H) {
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num3);
    h.run();
    assert_eq!(space(h), Space::Brand);
}

fn rect_at(x: f64, fill: Rgba) -> Object {
    let mut o = Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(x, 500.0), Size::new(200.0, 80.0), 0.0),
    );
    o.fill = Paint::Solid(fill);
    o
}

fn text_at(x: f64) -> Object {
    let mut o = rect_at(x, Rgba::rgb(0, 0, 0));
    o.kind = ShapeKind::Text;
    o.text = Some(TextBlock::new("ACE", CharStyle::default()));
    o
}

/// Adds a symbol "`name`" made of one rectangle, with one instance on the
/// active texture.
fn symbol(h: &mut H, name: &str) -> SymbolId {
    let p = &mut ws_mut(h).project;
    let id = p.add(rect_at(300.0, Rgba::rgb(10, 20, 30)));
    let (symbol, _) = p.convert_to_symbol(&[id], "Symbol").unwrap();
    p.rename_symbol(symbol, name);
    symbol
}

fn toggled(h: &H, label: &str) -> bool {
    h.get_by_label(label).accesskit_node().toggled() == Some(Toggled::True)
}

/// What the card `label` reads after its name (its detail).
fn value(h: &H, label: &str) -> String {
    h.get_by_label(label).value().unwrap_or_default()
}

/// Opens the ⋯ menu of the card `name` and clicks `item`.
fn card_action(h: &mut H, name: &str, item: &str) {
    h.get_by_label(&format!("Actions for {name}")).click();
    h.run();
    h.get_by_label(item).click();
    h.run();
}

#[test]
fn the_index_counts_every_section() {
    let mut h = open();
    let p = &mut ws_mut(&mut h).project;
    for i in 0..5u8 {
        p.add_swatch(Rgba::rgb(40 * i, 10, 10), "Color");
    }
    for i in 0..4 {
        let id = p.add(rect_at(
            300.0 + 250.0 * f64::from(i),
            Rgba::rgb(i as u8, 0, 0),
        ));
        p.new_graphic_style(id, "Style").unwrap();
    }
    for i in 0..3 {
        let id = p.add(text_at(300.0 + 250.0 * f64::from(i)));
        p.new_text_style(id, "Text style").unwrap();
    }
    p.add_asset(
        "logo",
        tp_core::AssetKind::Svg,
        std::sync::Arc::from(SVG),
        Size::new(40.0, 20.0),
    );
    symbol(&mut h, "Logo");
    symbol(&mut h, "Badge");
    ws_mut(&mut h).relayout_all_texts();
    h.run();
    show_brand(&mut h);
    for label in [
        "Palette · 5",
        "Graphic styles · 4",
        "Text styles · 3",
        "Symbols · 2",
        "Images · 1",
    ] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    assert!(toggled(&h, "Palette · 5"), "the first section is current");
}

#[test]
fn jumping_to_a_section_scrolls_it_into_view() {
    let mut h = open_with_height(700.0);
    let p = &mut ws_mut(&mut h).project;
    for i in 0..30u8 {
        p.add_swatch(Rgba::rgb(8 * i, 100, 10), "Color");
    }
    show_brand(&mut h);
    let heading = |h: &H| {
        h.get_all_by_label("Symbols")
            .find(|n| n.accesskit_node().role() == Role::Label)
            .unwrap()
            .rect()
            .top()
    };
    assert!(heading(&h) > 700.0, "below the fold at first");
    h.get_by_label("Symbols · 0").click();
    h.run();
    assert!(toggled(&h, "Symbols · 0"));
    assert!(!toggled(&h, "Palette · 30"));
    assert!(heading(&h) < 700.0, "scrolled into view");
}

#[test]
fn import_from_library_in_the_header() {
    let mut h = open();
    show_brand(&mut h);
    h.get_by_role_and_label(Role::Button, "Import from Library…")
        .click();
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::ImportFromLibrary(_))));
}

#[test]
fn the_library_is_not_listed() {
    let mut h = open();
    let logo = symbol(&mut h, "Logo Ardent");
    let AppState {
        screen, library, ..
    } = h.state_mut();
    let tp_app::state::Screen::Workspace(ws) = screen else {
        unreachable!()
    };
    ws.add_to_library(library, tp_app::library::Element::Symbol(logo), 0.0);
    ws.delete_symbol(logo, 0.0);
    h.run();
    show_brand(&mut h);
    assert!(h.query_by_label("Symbols · 0").is_some());
    assert!(h.query_by_label_contains("Logo Ardent").is_none());
}

#[test]
fn palette_actions_stay_in_brand() {
    let mut h = open();
    let (red, _) = ws_mut(&mut h)
        .project
        .add_swatch(Rgba::rgb(0xC0, 0x10, 0x20), "Color");
    ws_mut(&mut h).project.rename_swatch(red, "Company red");
    show_brand(&mut h);
    assert_eq!(value(&h, "Company red"), "#C01020", "its hex value");
    // New Color adds the current target's color.
    h.get_by_label("New Color").click();
    h.run();
    assert_eq!(ws(&h).project.palette.len(), 2);
    // Edit Swatch… from the card's menu opens the popup.
    card_action(&mut h, "Company red", "Edit Swatch…");
    assert!(ws(&h).panels.editing_swatch.is_some());
    h.key_press(Key::Escape);
    h.run();
    // Delete Swatch, from the context menu, is one undo step.
    h.get_by_label("Company red").click_secondary();
    h.run();
    h.get_by_label("Delete Swatch").click();
    h.run();
    assert_eq!(ws(&h).project.palette.len(), 1);
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.palette.len(), 2);
    assert_eq!(space(&h), Space::Brand);
}

#[test]
fn select_users_shows_the_workshop() {
    let mut h = open();
    let p = &mut ws_mut(&mut h).project;
    let ids: Vec<ObjectId> = (0..3)
        .map(|i| p.add(rect_at(300.0 + 250.0 * f64::from(i), Rgba::rgb(200, 0, 0))))
        .collect();
    let stripe = p.new_graphic_style(ids[0], "Style").unwrap();
    p.rename_style(stripe, "Stripe");
    p.apply_graphic_style(stripe, &ids);
    h.run();
    show_brand(&mut h);
    // Nothing selected: New Style from Selection says why it is disabled.
    assert!(
        h.get_by_label("New Graphic Style from Selection")
            .accesskit_node()
            .is_disabled()
    );
    card_action(&mut h, "Stripe", "Select Users on This Texture");
    assert_eq!(space(&h), Space::Workshop);
    let mut selected = ws(&h).selection.clone();
    selected.sort();
    assert_eq!(selected, ids);
}

#[test]
fn followed_styles_are_marked() {
    let mut h = open();
    let p = &mut ws_mut(&mut h).project;
    let id = p.add(rect_at(300.0, Rgba::rgb(200, 0, 0)));
    let other = p.add(rect_at(600.0, Rgba::rgb(0, 200, 0)));
    let stripe = p.new_graphic_style(id, "Style").unwrap();
    p.rename_style(stripe, "Stripe");
    p.new_graphic_style(other, "Style").unwrap();
    ws_mut(&mut h).selection = vec![id];
    h.run();
    show_brand(&mut h);
    assert!(toggled(&h, "Graphic style Stripe"));
    assert!(!toggled(&h, "Graphic style Style 1"));
    // New Style from Selection is offered for the selected shape, and
    // stays in Brand.
    h.get_by_label("New Graphic Style from Selection").click();
    h.run();
    assert_eq!(ws(&h).project.graphic_styles.len(), 3);
    assert_eq!(space(&h), Space::Brand);
}

#[test]
fn edit_a_symbol_from_brand() {
    let mut h = open();
    symbol(&mut h, "Logo");
    h.run();
    show_brand(&mut h);
    h.get_by_label("Edit Logo").click();
    h.run();
    assert_eq!(space(&h), Space::Workshop);
    assert!(ws(&h).is_editing_symbol());
}

#[test]
fn place_a_symbol_from_brand() {
    let mut h = open();
    let logo = symbol(&mut h, "Logo");
    let chassis = ws(&h)
        .project
        .surfaces
        .iter()
        .position(|s| s.name == "Chassis")
        .unwrap();
    ws_mut(&mut h).set_active_surface(chassis);
    h.run();
    show_brand(&mut h);
    assert_eq!(value(&h, "Symbol Logo"), "1 instance");
    h.get_by_label("Place Logo").click();
    h.run();
    let ws = ws(&h);
    assert_eq!(ws.space, Space::Workshop);
    assert_eq!(ws.project.active_surface, chassis);
    let [id] = ws.selection[..] else {
        panic!("one instance selected")
    };
    let o = ws.project.surface().get(id).unwrap();
    assert!(matches!(o.kind, ShapeKind::Instance { symbol, .. } if symbol == logo));
    assert_eq!(ws.project.instance_count(logo), 2);
}

#[test]
fn create_a_symbol_from_brand() {
    let mut h = open();
    let p = &mut ws_mut(&mut h).project;
    let circle = p.add(rect_at(300.0, Rgba::rgb(0, 0, 200)));
    let text = p.add(text_at(600.0));
    ws_mut(&mut h).relayout_all_texts();
    ws_mut(&mut h).selection = vec![circle, text];
    h.run();
    show_brand(&mut h);
    assert!(h.query_by_label("Symbols · 0").is_some());
    h.get_by_label("Create from Selection").click();
    h.run();
    assert_eq!(space(&h), Space::Brand, "creating places nothing new");
    assert_eq!(value(&h, "Symbol Symbol 1"), "1 instance");
}

#[test]
fn place_is_disabled_while_editing_a_symbol() {
    let mut h = open();
    let logo = symbol(&mut h, "Logo");
    ws_mut(&mut h).edit_symbol(logo, 0.0);
    h.run();
    show_brand(&mut h);
    assert!(h.get_by_label("Place Logo").accesskit_node().is_disabled());
    assert!(toggled(&h, "Symbol Logo"), "the edited symbol is marked");
}

#[test]
fn import_an_image_from_brand() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("logo.svg");
    std::fs::write(&path, SVG).unwrap();
    let mut h = open();
    h.state_mut().dialogs = Box::new(ScriptedDialogs {
        images: [vec![path]].into(),
        ..Default::default()
    });
    show_brand(&mut h);
    h.get_by_label("Import…").click();
    h.run_steps(3);
    let ws = ws(&h);
    assert_eq!(ws.space, Space::Workshop);
    assert_eq!(ws.selection.len(), 1);
    common::show_space(&mut h, Space::Brand);
    h.run();
    assert_eq!(value(&h, "Asset logo"), "Vector · 1 use");
    assert!(h.query_by_label("Images · 1").is_some());
}

#[test]
fn image_actions_from_brand() {
    let mut h = open();
    let (badge, _) = ws_mut(&mut h).project.add_asset(
        "badge",
        tp_core::AssetKind::Svg,
        std::sync::Arc::from(SVG),
        Size::new(40.0, 20.0),
    );
    h.run();
    show_brand(&mut h);
    // Removing an unused image keeps Brand shown, and is undone.
    h.get_by_label("Remove Asset").click();
    h.run();
    assert!(!ws(&h).project.assets.contains_key(&badge));
    assert_eq!(space(&h), Space::Brand);
    ws_mut(&mut h).undo();
    h.run();
    assert!(ws(&h).project.assets.contains_key(&badge));
    // Place adds it to the active texture and shows the Workshop.
    h.get_by_label("Place Asset").click();
    h.run();
    assert_eq!(space(&h), Space::Workshop);
    assert_eq!(ws(&h).selection.len(), 1);
}

#[test]
fn redefine_a_style_from_brand() {
    let mut h = open();
    let p = &mut ws_mut(&mut h).project;
    let ids: Vec<ObjectId> = (0..2)
        .map(|i| p.add(rect_at(300.0 + 250.0 * f64::from(i), Rgba::rgb(200, 0, 0))))
        .collect();
    let stripe = p.new_graphic_style(ids[0], "Style").unwrap();
    p.rename_style(stripe, "Stripe");
    p.apply_graphic_style(stripe, &ids);
    // The first rectangle is selected and turned blue.
    let blue = Rgba::rgb(0, 0, 200);
    let mut o = (**p.surface().get(ids[0]).unwrap()).clone();
    o.fill = Paint::Solid(blue);
    p.surface_mut().replace(&[o]);
    ws_mut(&mut h).selection = vec![ids[0]];
    h.run();
    show_brand(&mut h);
    card_action(&mut h, "Stripe", "Redefine from Selection");
    let ws = ws(&h);
    assert_eq!(ws.space, Space::Brand, "nothing to show in the Workshop");
    assert_eq!(
        ws.project.surface().get(ids[1]).unwrap().fill,
        Paint::Solid(blue)
    );
}
