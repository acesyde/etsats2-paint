//! Headless tests for the Workshop's inspector: its sections by selection
//! kind, the texture's properties with nothing selected, the header's
//! symbol actions, the Style row, and the color popover of the Fill and
//! Stroke rows.

mod common;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::prefs::Prefs;
use tp_app::tool::Tool;
use tp_app::workspace::{ColorTarget, Workspace};
use tp_core::document::{
    CharStyle, Frame, Node, Object, ObjectId, Paint, PathData, Rgba, ShapeKind, StrokeStyle,
    Subpath, TextBlock,
};
use tp_core::kurbo::{Point, Size};
use tp_core::{CheckReason, TemplateStatus, TextureState};

type H = Harness<'static, AppState>;

/// Tall window, so no section of the inspector scrolls.
fn open() -> H {
    let mut h = common::builder()
        .with_size(Vec2::new(1440.0, 2400.0))
        .with_step_dt(1.0 / 4.0)
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(Prefs::default(), None),
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

fn add(h: &mut H, kind: ShapeKind, x: f64) -> ObjectId {
    let o = Object::new(
        ObjectId(0),
        kind,
        Frame::new(Point::new(x, 800.0), Size::new(400.0, 200.0), 0.0),
    );
    let id = ws_mut(h).project.add(o);
    h.run();
    id
}

fn add_text(h: &mut H, x: f64) -> ObjectId {
    let at = Point::new(x, 1600.0);
    let mut text = Object::text(ObjectId(0), TextBlock::new("ACE", CharStyle::default()), at);
    ws_mut(h).text.place_at(&mut text, at);
    let id = ws_mut(h).project.add(text);
    h.run();
    id
}

fn polygon(sides: u8) -> ShapeKind {
    ShapeKind::Polygon { sides, star: None }
}

fn select(h: &mut H, ids: &[ObjectId]) {
    ws_mut(h).selection = ids.to_vec();
    h.run();
}

fn has(h: &H, label: &str) -> bool {
    h.query_by_label(label).is_some()
}

/// A section heading (or another label) of the inspector.
fn heading(h: &H, label: &str) -> bool {
    h.query_by_role_and_label(Role::Label, label).is_some()
}

/// Whether the inspector's `nodes` (role and label) are shown in this
/// order, top to bottom. A section heading can share its name with the
/// header's kind ("Polygon") or the tool options bar ("Text"): the last
/// one is the heading.
fn in_order(h: &H, nodes: &[(Role, &str)]) -> bool {
    let tops: Vec<f32> = nodes
        .iter()
        .map(|(role, label)| {
            h.get_all_by_role_and_label(*role, label)
                .last()
                .unwrap_or_else(|| panic!("no {label:?}"))
                .rect()
                .top()
        })
        .collect();
    tops.is_sorted()
}

const LABEL: Role = Role::Label;
const FIELD: Role = Role::TextInput;
const BUTTON: Role = Role::Button;

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

/// What a row of the inspector (a button) reads.
fn row_value(h: &H, label: &str) -> String {
    h.get_by_role_and_label(BUTTON, label)
        .value()
        .unwrap_or_default()
}

/// A click at `at`. One frame (a quarter second) between press and
/// release: `run` may take four frames when background renders repaint, a
/// second, which egui takes for a long press, then a drag, not a click.
fn click_at(h: &mut H, at: Pos2) {
    h.event(Event::PointerMoved(at));
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run();
}

// --- Sections ------------------------------------------------------------------

#[test]
fn sections_of_a_text() {
    let mut h = open();
    let t = add_text(&mut h, 800.0);
    select(&mut h, &[t]);
    // The header (the text's name, first of its labels) on top.
    let name = obj(&h, t).name;
    let header = h.get_all_by_role_and_label(LABEL, &name).next().unwrap();
    assert!(header.rect().top() < h.get_by_role_and_label(LABEL, "Layout").rect().top());
    assert!(in_order(
        &h,
        &[
            (LABEL, "Layout"),
            (FIELD, "Font size"),
            (LABEL, "Appearance"),
            (BUTTON, "Style")
        ]
    ));
    assert!(!heading(&h, "Image") && !heading(&h, "Polygon"));
}

#[test]
fn sections_of_a_rectangle() {
    let mut h = open();
    let r = add(&mut h, ShapeKind::rectangle(), 800.0);
    select(&mut h, &[r]);
    assert!(in_order(
        &h,
        &[
            (LABEL, "Layout"),
            (LABEL, "Appearance"),
            (FIELD, "Corner radius"),
            (BUTTON, "Style")
        ]
    ));
    assert!(!heading(&h, "Text") && !has(&h, "Font size"));
}

#[test]
fn sections_of_a_polygon() {
    let mut h = open();
    let p = add(&mut h, polygon(6), 800.0);
    select(&mut h, &[p]);
    assert!(in_order(
        &h,
        &[
            (LABEL, "Layout"),
            (LABEL, "Appearance"),
            (LABEL, "Polygon"),
            (FIELD, "Sides"),
            (BUTTON, "Style")
        ]
    ));
    assert!(!heading(&h, "Text") && !heading(&h, "Image") && !has(&h, "Corner radius"));
}

#[test]
fn sections_of_an_image() {
    let mut h = open();
    let files = vec![tp_app::import::read_bytes(
        "logo.png",
        tp_app::import::solid_png(80, 40, [20, 60, 220, 255]),
    )];
    ws_mut(&mut h).place_files(files, Some(Point::new(800.0, 800.0)), 0.0);
    for _ in 0..4 {
        h.step();
    }
    assert!(in_order(
        &h,
        &[
            (LABEL, "Layout"),
            (LABEL, "Appearance"),
            (FIELD, "Opacity"),
            (LABEL, "Image")
        ]
    ));
    assert!(!has(&h, "Fill color") && !has(&h, "Stroke color"));
    assert!(
        h.query_by_role_and_label(BUTTON, "Style").is_none(),
        "images take no style"
    );
    assert!(has(&h, "Reset Size"));
}

#[test]
fn several_objects() {
    let mut h = open();
    let ids = [
        add(&mut h, ShapeKind::rectangle(), 400.0),
        add(&mut h, ShapeKind::Ellipse, 900.0),
        add(&mut h, ShapeKind::rectangle(), 1400.0),
    ];
    select(&mut h, &ids);
    assert!(has(&h, "3 objects"));
    // Not every object is a rectangle: no corner radius.
    assert!(!has(&h, "Corner radius"));
}

#[test]
fn no_corner_radius_for_an_ellipse() {
    let mut h = open();
    let e = add(&mut h, ShapeKind::Ellipse, 800.0);
    select(&mut h, &[e]);
    assert!(!has(&h, "Corner radius"));
    assert!(has(&h, "Fill color") && has(&h, "Stroke color"));
}

#[test]
fn no_polygon_section_for_a_mixed_selection() {
    let mut h = open();
    let p = add(&mut h, polygon(6), 800.0);
    let r = add(&mut h, ShapeKind::rectangle(), 1400.0);
    select(&mut h, &[p, r]);
    assert!(!heading(&h, "Polygon") && !has(&h, "Sides"));
}

#[test]
fn star_from_the_polygon_section() {
    let mut h = open();
    let a = add(&mut h, polygon(6), 800.0);
    let b = add(&mut h, polygon(6), 1400.0);
    select(&mut h, &[a, b]);
    assert_eq!(ws(&h).tool, Tool::Select);
    let steps = ws(&h).history.len();
    type_into(&mut h, "Sides", "5");
    h.get_by_role_and_label(Role::CheckBox, "Star").click();
    h.run();
    for id in [a, b] {
        assert_eq!(
            obj(&h, id).kind,
            ShapeKind::Polygon {
                sides: 5,
                star: Some(0.5)
            }
        );
    }
    assert_eq!(ws(&h).history.len(), steps + 2, "one step per change");
    // The Polygon tool's options bar shows the same values.
    ws_mut(&mut h).selection.clear();
    ws_mut(&mut h).tool = Tool::Polygon;
    h.run();
    assert_eq!(
        h.get_by_role_and_label(Role::TextInput, "Sides").value(),
        Some("5".into())
    );
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "Star")
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::True)
    );
}

#[test]
fn polygon_sides_show_mixed() {
    let mut h = open();
    let a = add(&mut h, polygon(6), 800.0);
    let b = add(&mut h, polygon(8), 1400.0);
    select(&mut h, &[a, b]);
    assert_eq!(
        h.get_by_role_and_label(Role::TextInput, "Sides").value(),
        Some(String::new()),
        "empty, with the Mixed hint"
    );
}

#[test]
fn dashed_preset_for_a_line() {
    let mut h = open();
    let line = ws(&h).styled_path(
        PathData::new(vec![Subpath::new(
            vec![
                Node::corner(Point::new(500.0, 500.0)),
                Node::corner(Point::new(900.0, 500.0)),
            ],
            false,
        )]),
        "Line",
    );
    let id = ws_mut(&mut h).project.add(line);
    select(&mut h, &[id]);
    // The Width field, and the line settings in a popover opened from it.
    assert!(has(&h, "Line width"));
    common::open_line_settings(&mut h);
    h.get_by_label("Dash presets").click();
    h.run();
    h.get_by_label("Dashed 20/10").click();
    h.run();
    let dash = obj(&h, id).path_data().unwrap().line_style.dash;
    assert_eq!(
        dash,
        Some(tp_app::ui::workspace::panels::line_style::DASHED)
    );
    // Escape closes the popover.
    h.key_press(Key::Escape);
    h.run();
    assert!(!has(&h, "Dash presets"));
    assert_eq!(ws(&h).selection, vec![id], "Escape did not deselect");
}

// --- Nothing selected ------------------------------------------------------------

#[test]
fn nothing_selected_describes_the_texture() {
    let mut h = open();
    assert!(has(&h, "Standard cab"));
    assert!(has(&h, "Main texture · 4096 × 4096 px"));
    assert!(has(
        &h,
        "Select an object to set its layout, fill and stroke."
    ));
    // The look of new objects, under its heading.
    assert!(in_order(
        &h,
        &[
            (LABEL, "New objects"),
            (BUTTON, "Fill color"),
            (BUTTON, "Stroke color")
        ]
    ));
    // No empty state, no layout, and the template settings are in the
    // status bar only.
    assert!(!has(&h, "Nothing to transform") && !has(&h, "X position"));
    assert!(!heading(&h, "Layout"));
    let inspector = h.get_by_label("Fill color").rect();
    for node in h.query_all_by_label("Template opacity") {
        assert!(node.rect().top() > inspector.bottom());
    }
    // An accessory texture.
    let side_skirts = ws(&h)
        .project
        .surfaces
        .iter()
        .position(|s| s.name == "Side skirts")
        .unwrap();
    ws_mut(&mut h).set_active_surface(side_skirts);
    h.run();
    assert!(has(&h, "Accessory texture · 1024 × 1024 px"));
}

/// Flags the active texture with `status`.
fn flag(h: &mut H, status: TemplateStatus) {
    ws_mut(h)
        .project
        .surface_mut()
        .template
        .as_mut()
        .unwrap()
        .status = status;
    h.run();
}

fn state(h: &H) -> TextureState {
    ws(h).project.surface().state()
}

#[test]
fn layout_changed_notice_with_mark_as_checked() {
    let mut h = open();
    add(&mut h, ShapeKind::Ellipse, 800.0);
    flag(&mut h, TemplateStatus::LayoutChanged);
    let version = ws(&h).project.vehicles[0].version.clone();
    assert!(
        h.query_by_label(&format!("To check: Layout changed in {version}"))
            .is_some()
    );
    assert_eq!(state(&h), TextureState::ToCheck(CheckReason::LayoutChanged));
    let mark = h
        .query_all_by_label_contains("as checked")
        .next()
        .expect("Mark as Checked")
        .rect();
    // In the inspector, under the texture's name.
    assert!(mark.left() > h.get_by_label("Canvas").rect().right());
    h.get_by_label("Mark TruckPaint Sample Truck › Standard cab as checked")
        .click();
    h.run();
    // Marking a texture as checked: it takes its objects' state, as one
    // undo step; Undo flags it again.
    assert_eq!(state(&h), TextureState::Modified);
    assert!(h.query_by_label_contains("To check:").is_none());
    assert_eq!(ws(&h).history.undo_label(), Some("undo-mark-checked"));
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(state(&h), TextureState::ToCheck(CheckReason::LayoutChanged));
    assert!(
        h.query_by_label_contains("To check: Layout changed")
            .is_some()
    );
}

#[test]
fn not_in_this_version_notice() {
    let mut h = open();
    flag(&mut h, TemplateStatus::Removed);
    assert!(
        h.query_by_label("To check: Not in this version, left out of the mod")
            .is_some()
    );
    assert!(h.query_by_label_contains("as checked").is_none());
    assert_eq!(state(&h), TextureState::ToCheck(CheckReason::NotInVersion));
}

/// The value of a count of On this texture.
fn count(h: &H, label: &str) -> String {
    h.get_by_label(label).value().unwrap_or_default()
}

/// Edits object `id` (one undo step).
fn edit(h: &mut H, id: ObjectId, f: impl FnOnce(&mut Object)) {
    ws_mut(h).edit_object(id, "undo-change-fill", 0.0, f);
    h.run();
}

fn fill(h: &mut H, id: ObjectId, color: Rgba) {
    edit(h, id, |o| o.fill = Paint::Solid(color));
}

#[test]
fn summary_of_a_texture() {
    let mut h = open();
    let ids: Vec<ObjectId> = (0..4)
        .map(|i| add(&mut h, ShapeKind::rectangle(), 400.0 + 500.0 * f64::from(i)))
        .collect();
    fill(&mut h, ids[0], Rgba::rgb(0x15, 0x15, 0x15));
    fill(&mut h, ids[1], Rgba::rgb(0x15, 0x15, 0x15));
    fill(&mut h, ids[2], Rgba::rgb(0xC2, 0x3B, 0x2A));
    let (swatch, _) = ws_mut(&mut h)
        .project
        .add_swatch(Rgba::rgb(0xC2, 0x3B, 0x2A), "Color");
    edit(&mut h, ids[3], |o| {
        o.fill = Paint::Solid(Rgba::rgb(0xC2, 0x3B, 0x2A));
        o.fill_swatch = Some(swatch);
    });
    h.run();
    assert!(has(&h, "On this texture"));
    assert_eq!(count(&h, "Objects"), "4");
    assert_eq!(count(&h, "Symbol instances"), "0");
    // The rectangle linked to the swatch doesn't count; the one of the same
    // value, not linked, does.
    assert_eq!(count(&h, "Off-palette colors"), "2");
    assert!(
        h.query_by_label("Select objects with off-palette colors")
            .is_some()
    );
    // The counts follow every edit.
    edit(&mut h, ids[0], |o| o.visible = false);
    edit(&mut h, ids[1], |o| o.visible = false);
    h.run();
    assert_eq!(count(&h, "Off-palette colors"), "1");
}

#[test]
fn never_empty_on_this_texture_follows_the_properties() {
    let h = open();
    let title = h.get_by_label("Standard cab").rect().top();
    let section = h.get_by_label("On this texture").rect().top();
    let new_objects = h.get_by_label("New objects").rect().top();
    assert!(title < section && section < new_objects);
    assert_eq!(count(&h, "Objects"), "0");
    assert_eq!(count(&h, "Off-palette colors"), "0");
    assert!(
        h.query_by_label("Select objects with off-palette colors")
            .is_none()
    );
    assert!(h.query_by_label_contains("Nothing to").is_none());
}

/// A symbol "Logo" made of a circle filled with an unlinked color, with
/// two instances on the texture, one of them inside a group.
fn logo(h: &mut H) -> tp_core::document::SymbolId {
    let circle = add(h, ShapeKind::Ellipse, 800.0);
    fill(h, circle, Rgba::rgb(0x12, 0x34, 0x56));
    select(h, &[circle]);
    let symbol = ws_mut(h).convert_to_symbol(1.0).expect("a symbol");
    ws_mut(h).place_symbol(symbol, Some(Point::new(2000.0, 2000.0)), 2.0);
    ws_mut(h).group_selection(3.0);
    ws_mut(h).selection.clear();
    h.run();
    symbol
}

#[test]
fn instances_counted_their_colors_not() {
    let mut h = open();
    logo(&mut h);
    assert_eq!(count(&h, "Objects"), "2");
    assert_eq!(count(&h, "Symbol instances"), "2");
    assert_eq!(count(&h, "Off-palette colors"), "0");
}

#[test]
fn editing_a_symbol_counts_its_content() {
    let mut h = open();
    let symbol = logo(&mut h);
    ws_mut(&mut h).edit_symbol(symbol, 4.0);
    ws_mut(&mut h).selection.clear();
    h.run();
    assert!(has(&h, "In this symbol"));
    assert!(!has(&h, "On this texture"));
    assert_eq!(count(&h, "Objects"), "1");
    assert_eq!(count(&h, "Off-palette colors"), "1");
    assert!(h.query_by_label("Symbol instances").is_none());
}

#[test]
fn selecting_the_off_palette_objects() {
    let mut h = open();
    let a = add(&mut h, ShapeKind::rectangle(), 400.0);
    let b = add(&mut h, ShapeKind::rectangle(), 1000.0);
    fill(&mut h, a, Rgba::rgb(0x15, 0x15, 0x15));
    fill(&mut h, b, Rgba::rgb(0xEE, 0xEE, 0xEE));
    select(&mut h, &[a, b]);
    ws_mut(&mut h).group_selection(1.0);
    let group = ws(&h).selection[0];
    edit(&mut h, group, |o| o.name = "Stripes".into());
    let text = add_text(&mut h, 2000.0);
    let (swatch, _) = ws_mut(&mut h)
        .project
        .add_swatch(Rgba::rgb(0, 0, 0), "Color");
    edit(&mut h, text, |o| {
        o.fill = Paint::Solid(Rgba::rgb(0, 0, 0));
        o.fill_swatch = Some(swatch);
    });
    ws_mut(&mut h).selection.clear();
    h.run();
    assert_eq!(count(&h, "Off-palette colors"), "2");
    h.get_by_label("Select objects with off-palette colors")
        .click();
    h.run();
    // The rectangles themselves, not their group nor the text.
    let mut selection = ws(&h).selection.clone();
    selection.sort();
    let mut expected = vec![a, b];
    expected.sort();
    assert_eq!(selection, expected);
    assert!(has(&h, "2 objects"));
}

#[test]
fn locked_objects_are_left_out_of_the_selection() {
    let mut h = open();
    let a = add(&mut h, ShapeKind::rectangle(), 400.0);
    let b = add(&mut h, ShapeKind::rectangle(), 1000.0);
    fill(&mut h, a, Rgba::rgb(0x15, 0x15, 0x15));
    fill(&mut h, b, Rgba::rgb(0x15, 0x15, 0x15));
    ws_mut(&mut h).set_locked(b, true, 1.0);
    h.run();
    h.get_by_label("Select objects with off-palette colors")
        .click();
    h.run();
    assert_eq!(ws(&h).selection, [a]);
    // Every one of them locked: the count is plain text.
    ws_mut(&mut h).selection.clear();
    ws_mut(&mut h).set_locked(a, true, 2.0);
    h.run();
    assert_eq!(count(&h, "Off-palette colors"), "1");
    assert!(
        h.query_by_label("Select objects with off-palette colors")
            .is_none()
    );
}

#[test]
fn text_section_for_new_texts_with_the_text_tool() {
    let mut h = open();
    assert!(!has(&h, "Font size"));
    ws_mut(&mut h).tool = Tool::Text;
    h.run();
    assert!(in_order(
        &h,
        &[
            (LABEL, "New objects"),
            (LABEL, "Text"),
            (FIELD, "Font size")
        ]
    ));
    type_into(&mut h, "Font size", "300");
    assert_eq!(ws(&h).text_style.size, 300.0);
}

#[test]
fn red_fill_for_new_objects() {
    let mut h = open();
    common::open_color_popover(&mut h, ColorTarget::Fill);
    type_into(&mut h, "Hex color", "#FF0000");
    assert_eq!(row_value(&h, "Fill color"), "#FF0000");
    let frame = Frame::new(Point::new(800.0, 800.0), Size::new(200.0, 100.0), 0.0);
    let id = ws_mut(&mut h).create_shape(ShapeKind::rectangle(), frame, 1.0);
    h.run();
    assert_eq!(obj(&h, id).fill, Paint::Solid(Rgba::rgb(255, 0, 0)));
}

// --- Header ----------------------------------------------------------------------

fn convert_a_logo(h: &mut H) -> ObjectId {
    let a = add(h, ShapeKind::Ellipse, 600.0);
    let t = add_text(h, 900.0);
    select(h, &[a, t]);
    h.get_by_role_and_label(Role::Button, "Convert to Symbol")
        .click();
    h.run();
    ws(h).selection[0]
}

#[test]
fn convert_from_the_header() {
    let mut h = open();
    let instance = convert_a_logo(&mut h);
    let o = obj(&h, instance);
    assert!(o.is_instance());
    assert_eq!(ws(&h).project.symbols.len(), 1);
    assert_eq!(ws(&h).project.surface().objects.len(), 1);
}

#[test]
fn instance_header() {
    let mut h = open();
    let instance = convert_a_logo(&mut h);
    let name = ws(&h).project.symbols[0].name.clone();
    // The symbol's name with its icon, Edit Symbol and Detach Instance; no
    // Convert to Symbol, no color rows.
    let label = format!("Instance of {name}");
    assert!(has(&h, &label));
    assert!(has(&h, "Edit Symbol") && has(&h, "Detach Instance"));
    assert!(!has(&h, "Convert to Symbol"));
    assert!(!has(&h, "Fill color") && !has(&h, "Stroke color"));
    assert!(has(
        &h,
        "An instance shows its symbol: edit the symbol to change its look, or detach the instance."
    ));
    // The instance's opacity is still there.
    assert!(h.query_by_role_and_label(FIELD, "Opacity").is_some());
    // Detach Instance from the header.
    h.get_by_role_and_label(Role::Button, "Detach Instance")
        .click();
    h.run();
    assert!(obj(&h, instance).is_group());
    assert!(!has(&h, "Edit Symbol"));
}

#[test]
fn edit_symbol_from_the_header() {
    let mut h = open();
    convert_a_logo(&mut h);
    h.get_by_role_and_label(Role::Button, "Edit Symbol").click();
    h.run();
    assert!(ws(&h).is_editing_symbol());
}

// --- Style row -------------------------------------------------------------------

/// A graphic style "Stripe" made from a red rectangle.
fn stripe(h: &mut H) -> tp_core::document::StyleId {
    let r = add(h, ShapeKind::rectangle(), 2000.0);
    {
        let ws = ws_mut(h);
        ws.selection = vec![r];
        ws.apply_color(ColorTarget::Fill, Rgba::rgb(200, 0, 0));
        ws.commit_pending(0.1);
    }
    let id = ws_mut(h).new_graphic_style(0.2).expect("style");
    ws_mut(h).project.rename_style(id, "Stripe");
    h.run();
    id
}

#[test]
fn apply_a_style_from_the_style_row() {
    let mut h = open();
    stripe(&mut h);
    let a = add(&mut h, ShapeKind::rectangle(), 400.0);
    let b = add(&mut h, ShapeKind::rectangle(), 900.0);
    select(&mut h, &[a, b]);
    assert_eq!(row_value(&h, "Style"), "None");
    let steps = ws(&h).history.len();
    h.get_by_role_and_label(BUTTON, "Style").click();
    h.run();
    h.get_by_label("Stripe").click();
    h.run();
    for id in [a, b] {
        assert_eq!(obj(&h, id).fill, Paint::Solid(Rgba::rgb(200, 0, 0)));
    }
    assert_eq!(row_value(&h, "Style"), "Stripe");
    assert_eq!(ws(&h).history.len(), steps + 1, "one undo step");
    ws_mut(&mut h).undo();
    h.run();
    for id in [a, b] {
        assert_ne!(obj(&h, id).fill, Paint::Solid(Rgba::rgb(200, 0, 0)));
    }
}

#[test]
fn own_change_unlinks_the_style() {
    let mut h = open();
    stripe(&mut h);
    let a = add(&mut h, ShapeKind::rectangle(), 400.0);
    select(&mut h, &[a]);
    h.get_by_role_and_label(BUTTON, "Style").click();
    h.run();
    h.get_by_label("Stripe").click();
    h.run();
    assert_eq!(row_value(&h, "Style"), "Stripe");
    type_into(&mut h, "Opacity", "40");
    assert_eq!(row_value(&h, "Style"), "None");
}

#[test]
fn style_row_reads_mixed() {
    let mut h = open();
    let styled = stripe(&mut h);
    let a = add(&mut h, ShapeKind::rectangle(), 400.0);
    let styled_object = ws(&h)
        .project
        .surface()
        .objects
        .iter()
        .find(|o| o.style == Some(styled))
        .map(|o| o.id)
        .unwrap();
    select(&mut h, &[a, styled_object]);
    assert_eq!(row_value(&h, "Style"), "Mixed");
}

// --- Color popover ----------------------------------------------------------------

#[test]
fn open_and_close_the_color_popover() {
    let mut h = open();
    let r = add(&mut h, ShapeKind::rectangle(), 800.0);
    select(&mut h, &[r]);
    h.get_by_label("Fill color").click();
    h.run();
    assert!(common::color_popover_open(&h));
    let row = h.get_by_label("Fill color").rect();
    let kind = h.get_by_label("Solid paint").rect();
    assert!(kind.top() > row.bottom(), "under the Fill row");
    type_into(&mut h, "Hex color", "#00FF00");
    h.key_press(Key::Escape);
    h.run();
    assert!(!common::color_popover_open(&h));
    assert_eq!(ws(&h).selection, vec![r], "Escape closed the popover only");
    assert_eq!(obj(&h, r).fill, Paint::Solid(Rgba::rgb(0, 255, 0)));
    assert_eq!(row_value(&h, "Fill color"), "#00FF00");
}

#[test]
fn a_click_on_the_pasteboard_closes_it() {
    let mut h = open();
    common::open_color_popover(&mut h, ColorTarget::Fill);
    let canvas = h.get_by_label("Canvas").rect();
    click_at(&mut h, canvas.left_top() + Vec2::new(30.0, 60.0));
    assert!(!common::color_popover_open(&h));
}

#[test]
fn x_moves_the_popover_to_the_stroke_row() {
    let mut h = open();
    let r = add(&mut h, ShapeKind::rectangle(), 800.0);
    let mut o = obj(&h, r);
    o.stroke = Some(StrokeStyle::default());
    ws_mut(&mut h).project.surface_mut().replace(&[o]);
    select(&mut h, &[r]);
    common::open_color_popover(&mut h, ColorTarget::Fill);
    h.key_press(Key::X);
    h.run();
    assert_eq!(ws(&h).panels.color_target, ColorTarget::Stroke);
    assert!(common::color_popover_open(&h));
    let row = h.get_by_label("Stroke color").rect();
    let kind = h.get_by_label("Solid paint").rect();
    assert!(kind.top() > row.bottom(), "under the Stroke row");
    // The stroke target offers None.
    assert!(has(&h, "No stroke"));
}

#[test]
fn palette_before_recent_colors() {
    let mut h = open();
    ws_mut(&mut h)
        .project
        .add_swatch(Rgba::rgb(10, 20, 30), "Color");
    h.state_mut().prefs.push_recent_color([1, 2, 3, 255]);
    h.run();
    common::open_color_popover(&mut h, ColorTarget::Fill);
    let palette = h.get_by_label("Brand palette").rect();
    let recent = h.get_by_label("Recent").rect();
    assert!(palette.top() < recent.top());
    assert!(h.get_by_label("Color 1").rect().top() < recent.top());
}

#[test]
fn gradient_fill_row() {
    let mut h = open();
    let r = add(&mut h, ShapeKind::rectangle(), 800.0);
    select(&mut h, &[r]);
    common::open_color_popover(&mut h, ColorTarget::Fill);
    h.get_by_label("Linear gradient").click();
    h.run();
    assert_eq!(row_value(&h, "Fill color"), "Linear");
    assert!(obj(&h, r).fill.gradient().is_some());
}

#[test]
fn shift_x_and_d_keep_working_with_the_popover_open() {
    let mut h = open();
    let r = add(&mut h, ShapeKind::rectangle(), 800.0);
    let mut o = obj(&h, r);
    o.fill = Paint::Solid(Rgba::rgb(255, 0, 0));
    o.stroke = Some(StrokeStyle {
        paint: Rgba::rgb(0, 0, 255).into(),
        ..StrokeStyle::default()
    });
    ws_mut(&mut h).project.surface_mut().replace(&[o]);
    select(&mut h, &[r]);
    common::open_color_popover(&mut h, ColorTarget::Fill);
    h.key_press_modifiers(Modifiers::SHIFT, Key::X);
    h.run();
    let o = obj(&h, r);
    assert_eq!(o.fill, Paint::Solid(Rgba::rgb(0, 0, 255)));
    assert_eq!(o.stroke.unwrap().paint, Paint::Solid(Rgba::rgb(255, 0, 0)));
    h.key_press(Key::D);
    h.run();
    assert!(obj(&h, r).stroke.is_none());
    assert!(common::color_popover_open(&h));
}

#[test]
fn swatch_context_menu_in_the_color_popover() {
    let mut h = open();
    let r = add(&mut h, ShapeKind::rectangle(), 800.0);
    select(&mut h, &[r]);
    ws_mut(&mut h)
        .project
        .add_swatch(Rgba::rgb(10, 20, 30), "Color");
    h.run();
    common::open_color_popover(&mut h, ColorTarget::Fill);
    // The popover's swatch is drawn after the Resources tab's.
    common::last(&h, "Color 1").click_secondary();
    h.run();
    for item in ["Edit Swatch…", "Delete Swatch", "Add to Library"] {
        assert!(has(&h, item), "{item}");
    }
    h.get_by_label("Delete Swatch").click();
    h.run();
    assert!(ws(&h).project.palette.is_empty());
    assert!(
        common::color_popover_open(&h),
        "the menu's click stays inside"
    );
    // Clicking a swatch applies and links it.
    ws_mut(&mut h)
        .project
        .add_swatch(Rgba::rgb(10, 20, 30), "Color");
    h.run();
    common::last(&h, "Color 1").click();
    h.run();
    assert_eq!(obj(&h, r).fill, Paint::Solid(Rgba::rgb(10, 20, 30)));
    assert!(obj(&h, r).fill_swatch.is_some());
    assert_eq!(row_value(&h, "Fill color"), "Color 1");
}
