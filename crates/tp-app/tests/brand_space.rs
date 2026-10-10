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
    let mut h = common::builder()
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
        assert!(h.query_all_by_label(label).count() > 0, "{label}");
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
    assert_eq!(
        value(&h, "Company red"),
        "#C01020 · Unused",
        "its hex value and usage"
    );
    // New Color adds the current target's color.
    h.get_by_label("New Color").click();
    h.run();
    assert_eq!(ws(&h).project.palette.len(), 2);
    // Edit Swatch… from the card's menu opens the editor.
    card_action(&mut h, "Company red", "Edit Swatch…");
    assert!(ws(&h).panels.brand_edit.is_some());
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
fn switching_space_cancels_the_editor() {
    let mut h = open();
    let red = ws_mut(&mut h)
        .project
        .add_swatch(Rgba::rgb(0xC0, 0x10, 0x20), "Color")
        .0;
    ws_mut(&mut h).project.rename_swatch(red, "Company red");
    show_brand(&mut h);
    let steps = ws(&h).history.len();
    card_action(&mut h, "Company red", "Edit Swatch…");
    ws_mut(&mut h).set_edit_value(tp_app::brand_ops::EditValue::Color(Rgba::rgb(0x8B, 0, 0)));
    common::settle_renders(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num2);
    h.run();
    assert_eq!(space(&h), Space::Workshop);
    assert!(ws(&h).panels.brand_edit.is_none());
    assert_eq!(
        ws(&h).project.swatch(red).unwrap().color,
        Rgba::rgb(0xC0, 0x10, 0x20)
    );
    assert_eq!(ws(&h).history.len(), steps);
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
    assert_eq!(value(&h, "Symbol Logo"), "1 instance · 1 texture");
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
    assert_eq!(value(&h, "Symbol Symbol 1"), "1 instance · 1 texture");
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

// ── Before/after editor and usage ───────────────────────────────────────

const RED: Rgba = Rgba::rgb(0xC0, 0x10, 0x20);
const DARK_RED: Rgba = Rgba::rgb(0x8B, 0, 0);

/// Adds `o` to surface `surface`; returns its id.
fn add_on(h: &mut H, surface: usize, o: Object) -> ObjectId {
    let p = &mut ws_mut(h).project;
    let active = p.active_surface;
    p.active_surface = surface;
    let id = p.add(o);
    p.active_surface = active;
    id
}

fn get(h: &H, surface: usize, id: ObjectId) -> Object {
    (**ws(h).project.surfaces[surface].get(id).expect("object")).clone()
}

/// A rectangle filled with `color` over the middle of surface `surface`.
fn middle_of(h: &H, surface: usize, color: Rgba) -> Object {
    let side = ws(h).project.surfaces[surface].size;
    let mut o = rect_at(0.0, color);
    o.frame = Frame::new(
        Point::new(side / 2.0, side / 2.0),
        Size::new(side / 2.0, side / 2.0),
        0.0,
    );
    o
}

/// The swatch "Company red", linked by a rectangle over the middle of the
/// first two textures.
fn company_red(h: &mut H) -> (tp_core::document::SwatchId, [ObjectId; 2]) {
    let (red, _) = ws_mut(h).project.add_swatch(RED, "Color");
    ws_mut(h).project.rename_swatch(red, "Company red");
    let ids = [0, 1].map(|surface| {
        let mut o = middle_of(h, surface, RED);
        o.fill_swatch = Some(red);
        add_on(h, surface, o)
    });
    h.run();
    (red, ids)
}

/// Replaces the text of the field `label` with `text`, then presses
/// `then` (Enter commits a number field).
fn type_into(h: &mut H, label: &str, text: &str, then: Option<Key>) {
    h.get_by_role_and_label(Role::TextInput, label).focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, label)
        .type_text(text);
    h.run();
    if let Some(key) = then {
        h.key_press(key);
        h.run();
    }
}

/// What the editor's Before or After block shows.
fn block(h: &H, label: &str) -> String {
    h.get_all_by_label(label)
        .filter(|n| n.accesskit_node().role() != Role::Label)
        .find_map(|n| n.value())
        .expect("a block")
}

/// Number of texture tiles in the editor's impact box.
fn tiles(h: &H) -> usize {
    h.query_all_by_label_contains("Preview of ").count()
}

#[test]
fn edit_the_company_red_from_brand() {
    let mut h = open();
    let (red, [a, b]) = company_red(&mut h);
    show_brand(&mut h);
    let steps = ws(&h).history.len();
    card_action(&mut h, "Company red", "Edit Swatch…");
    // The editor is a panel of the Brand space: the sections stay, the
    // edited card is marked.
    assert_eq!(space(&h), Space::Brand);
    assert!(h.query_by_label("Edit color").is_some());
    assert!(h.query_by_label("Palette · 1").is_some());
    assert!(toggled(&h, "Company red"), "the edited card is marked");
    type_into(&mut h, "Swatch hex color", "#8B0000", None);
    h.get_by_label("Apply to Fleet").click();
    h.run();
    assert!(ws(&h).panels.brand_edit.is_none());
    assert_eq!(get(&h, 0, a).fill, Paint::Solid(DARK_RED));
    assert_eq!(get(&h, 1, b).fill, Paint::Solid(DARK_RED), "other texture");
    assert_eq!(get(&h, 1, b).fill_swatch, Some(red), "the link holds");
    assert_eq!(ws(&h).project.swatch(red).unwrap().color, DARK_RED);
    assert_eq!(ws(&h).history.len(), steps + 1, "one undo step");
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(get(&h, 0, a).fill, Paint::Solid(RED));
    assert_eq!(get(&h, 1, b).fill, Paint::Solid(RED));
}

#[test]
fn nothing_changes_before_apply_and_the_impact_shows() {
    let mut h = open();
    let (red, _) = company_red(&mut h);
    show_brand(&mut h);
    common::settle_renders(&mut h);
    let before = ws(&h).project.clone();
    let steps = ws(&h).history.len();
    card_action(&mut h, "Company red", "Edit Swatch…");
    assert!(h.query_by_label("Impact: 2 textures, 2 objects").is_some());
    assert_eq!(tiles(&h), 2, "a tile per affected texture");
    type_into(&mut h, "Swatch hex color", "#8B0000", None);
    common::settle_renders(&mut h);
    let ws = ws(&h);
    assert_eq!(ws.project, before, "the document is untouched");
    assert_eq!(ws.history.len(), steps);
    assert_eq!(ws.project.swatch(red).unwrap().color, RED);
    assert_eq!(block(&h, "Before"), "#C01020");
    assert_eq!(block(&h, "After"), "#8B0000");
    for surface in [0, 1] {
        assert_eq!(
            ws.thumbnails.color_at(surface, [0.5, 0.5]),
            Some(RED),
            "the texture keeps its red"
        );
        assert_eq!(
            ws.preview_thumbs.color_at(surface, [0.5, 0.5]),
            Some(DARK_RED),
            "its tile shows the new value"
        );
    }
}

#[test]
fn escape_cancels_the_edit() {
    let mut h = open();
    let (red, [a, _]) = company_red(&mut h);
    show_brand(&mut h);
    let steps = ws(&h).history.len();
    card_action(&mut h, "Company red", "Edit Swatch…");
    type_into(&mut h, "Swatch hex color", "#00FF00", None);
    h.key_press(Key::Escape);
    h.run();
    assert!(ws(&h).panels.brand_edit.is_none());
    assert!(h.query_by_label("Apply to Fleet").is_none(), "closed");
    assert_eq!(ws(&h).project.swatch(red).unwrap().color, RED);
    assert_eq!(get(&h, 0, a).fill, Paint::Solid(RED));
    assert_eq!(ws(&h).history.len(), steps);
    assert_eq!(space(&h), Space::Brand);
}

#[test]
fn rename_a_swatch_only() {
    let mut h = open();
    let (red, _) = company_red(&mut h);
    show_brand(&mut h);
    card_action(&mut h, "Company red", "Edit Swatch…");
    type_into(&mut h, "Name", "Ardent red", None);
    h.get_by_label("Apply to Fleet").click();
    h.run();
    let s = ws(&h).project.swatch(red).unwrap();
    assert_eq!((s.name.as_str(), s.color), ("Ardent red", RED));
    assert!(h.query_by_label("Ardent red").is_some(), "listed");
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.swatch(red).unwrap().name, "Company red");
}

#[test]
fn an_empty_or_taken_name_disables_apply() {
    let mut h = open();
    let (red, _) = company_red(&mut h);
    show_brand(&mut h);
    card_action(&mut h, "Company red", "Edit Swatch…");
    type_into(&mut h, "Name", "", None);
    h.key_press(Key::Backspace);
    h.run();
    assert!(
        h.get_by_label("Apply to Fleet")
            .accesskit_node()
            .is_disabled()
    );
    assert!(h.query_by_label("Enter a name for the swatch.").is_some());
    h.key_press(Key::Escape);
    h.run();
    assert_eq!(ws(&h).project.swatch(red).unwrap().name, "Company red");
    // A style can't take the name of another one.
    let p = &mut ws_mut(&mut h).project;
    let a = p.add(rect_at(300.0, Rgba::rgb(200, 0, 0)));
    let b = p.add(rect_at(600.0, Rgba::rgb(0, 200, 0)));
    let stripe = p.new_graphic_style(a, "Style").unwrap();
    p.rename_style(stripe, "Stripe");
    p.new_graphic_style(b, "Style").unwrap();
    h.run();
    card_action(&mut h, "Stripe", "Edit Style…");
    type_into(&mut h, "Style name", "Style 1", None);
    assert!(
        h.get_by_label("Apply to Fleet")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        h.query_by_label("Another graphic style already has this name.")
            .is_some()
    );
}

/// "Stripe": a red fill with a 4 px stroke, followed by a rectangle on
/// each of the first three textures.
fn stripes(h: &mut H) -> (tp_core::document::StyleId, [ObjectId; 3]) {
    let ids = [0, 1, 2].map(|surface| {
        let mut o = middle_of(h, surface, Rgba::rgb(200, 0, 0));
        o.stroke = Some(tp_core::document::StrokeStyle {
            width: 4.0,
            ..Default::default()
        });
        add_on(h, surface, o)
    });
    let p = &mut ws_mut(h).project;
    let stripe = p.new_graphic_style(ids[0], "Style").unwrap();
    p.rename_style(stripe, "Stripe");
    for (surface, id) in [1, 2].into_iter().zip(&ids[1..]) {
        p.active_surface = surface;
        p.apply_graphic_style(stripe, &[*id]);
    }
    p.active_surface = 0;
    h.run();
    (stripe, ids)
}

#[test]
fn restyle_the_stripes_from_brand() {
    let mut h = open();
    let (stripe, ids) = stripes(&mut h);
    show_brand(&mut h);
    assert_eq!(value(&h, "Graphic style Stripe"), "3 textures · 3 objects");
    let steps = ws(&h).history.len();
    card_action(&mut h, "Stripe", "Edit Style…");
    // The editor opens on the right with the style's fill, stroke and
    // opacity, its usage as the impact and the textures' tiles.
    assert_eq!(space(&h), Space::Brand);
    assert!(h.query_by_label("Edit style").is_some());
    for label in ["Fill", "Stroke", "Stroke width", "Opacity"] {
        assert!(h.query_all_by_label(label).count() > 0, "{label}");
    }
    assert!(h.query_by_label("Impact: 3 textures, 3 objects").is_some());
    assert_eq!(tiles(&h), 3);
    type_into(&mut h, "Stroke width", "8", Some(Key::Enter));
    assert_eq!(get(&h, 0, ids[0]).stroke.unwrap().width, 4.0, "not yet");
    h.get_by_label("Apply to Fleet").click();
    h.run();
    for (surface, id) in ids.iter().enumerate() {
        let o = get(&h, surface, *id);
        assert_eq!(o.stroke.unwrap().width, 8.0);
        assert_eq!(o.style, Some(stripe), "still follows Stripe");
    }
    assert_eq!(ws(&h).history.len(), steps + 1);
    ws_mut(&mut h).undo();
    h.run();
    for (surface, id) in ids.iter().enumerate() {
        assert_eq!(get(&h, surface, *id).stroke.unwrap().width, 4.0);
    }
}

#[test]
fn edit_style_is_offered_for_graphic_styles_only() {
    let mut h = open();
    stripes(&mut h);
    let p = &mut ws_mut(&mut h).project;
    let t = p.add(text_at(300.0));
    let lettering = p.new_text_style(t, "Text style").unwrap();
    p.rename_style(lettering, "Lettering");
    ws_mut(&mut h).relayout_all_texts();
    h.run();
    show_brand(&mut h);
    h.get_by_label("Actions for Lettering").click();
    h.run();
    assert!(h.query_by_label("Edit Style…").is_none());
    assert!(h.query_by_label("Redefine from Selection").is_some());
    h.key_press(Key::Escape);
    h.run();
    h.get_by_label("Actions for Stripe").click();
    h.run();
    assert!(h.query_by_label("Edit Style…").is_some());
}

#[test]
fn usage_on_the_cards() {
    let mut h = open();
    let p = &mut ws_mut(&mut h).project;
    let (cream, _) = p.add_swatch(Rgba::rgb(0xF2, 0xE8, 0xD5), "Color");
    p.rename_swatch(cream, "Cream");
    let (black, _) = p.add_swatch(Rgba::rgb(0, 0, 0), "Color");
    p.rename_swatch(black, "Black");
    for surface in [0, 0, 1] {
        let mut o = middle_of(&h, surface, Rgba::rgb(0xF2, 0xE8, 0xD5));
        o.fill_swatch = Some(cream);
        add_on(&mut h, surface, o);
    }
    let t = ws_mut(&mut h).project.add(text_at(300.0));
    let p = &mut ws_mut(&mut h).project;
    let lettering = p.new_text_style(t, "Text style").unwrap();
    p.rename_style(lettering, "Lettering");
    let size = p.text_style(lettering).unwrap().style.size;
    ws_mut(&mut h).relayout_all_texts();
    h.run();
    show_brand(&mut h);
    assert_eq!(value(&h, "Cream"), "#F2E8D5 · 2 textures · 3 objects");
    assert_eq!(value(&h, "Black"), "#000000 · Unused");
    assert_eq!(
        value(&h, "Text style Lettering"),
        format!("{size} px · 1 texture · 1 object")
    );
}

#[test]
fn instances_and_textures_of_a_symbol() {
    let mut h = open();
    let logo = symbol(&mut h, "Logo");
    let chassis = ws(&h)
        .project
        .surfaces
        .iter()
        .position(|s| s.name == "Chassis")
        .unwrap();
    for _ in 0..3 {
        ws_mut(&mut h).place_symbol(logo, None, 0.0);
    }
    ws_mut(&mut h).set_active_surface(chassis);
    for _ in 0..2 {
        ws_mut(&mut h).place_symbol(logo, None, 0.0);
    }
    h.run();
    show_brand(&mut h);
    assert_eq!(value(&h, "Symbol Logo"), "6 instances · 2 textures");
    // The Resources tab keeps the number of instances.
    common::show_space(&mut h, Space::Workshop);
    common::show_tab(&mut h, tp_app::layout::LeftTab::Resources);
    assert!(h.query_by_label("6 instances").is_some());
}
