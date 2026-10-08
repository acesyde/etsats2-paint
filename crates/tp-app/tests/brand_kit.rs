//! Headless tests for the brand kit: linked palette swatches and Edit
//! Swatch…, the Styles panel, and Copy From Cabin….

mod common;

use egui::accesskit::{Role, Toggled};
use egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::commands::CommandId;
use tp_app::layout::PanelKind;
use tp_app::prefs::Prefs;
use tp_app::workspace::{ColorTarget, Workspace};
use tp_core::document::{
    CharStyle, Frame, Object, ObjectId, Paint, Rgba, ShapeKind, StrokeStyle, SwatchId, TextBlock,
};
use tp_core::kurbo::{Point, Size};
use tp_vehicles::Package;

type H = Harness<'static, AppState>;

const RED: Rgba = Rgba::rgb(0xC0, 0x10, 0x20);
const DARK_RED: Rgba = Rgba::rgb(0x8B, 0, 0);

/// Tall window with every panel open and expanded; a project for the
/// sample truck (Standard cab, Chassis, Cab accessories, Side skirts).
fn open() -> H {
    let mut prefs = Prefs::default();
    for slot in &mut prefs.layout.panels {
        slot.open = !matches!(slot.kind, PanelKind::Assets | PanelKind::Transform);
        slot.collapsed = false;
    }
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

fn rect(x: f64) -> Object {
    Object::new(
        ObjectId(0),
        ShapeKind::rectangle(),
        Frame::new(Point::new(x, 300.0), Size::new(200.0, 50.0), 0.0),
    )
}

fn text(x: f64) -> Object {
    let mut o = rect(x);
    o.kind = ShapeKind::Text;
    o.text = Some(TextBlock::new("ACE", CharStyle::default()));
    o
}

/// Adds `o` to surface `surface`; returns its id.
fn add_on(h: &mut H, surface: usize, o: Object) -> ObjectId {
    let ws = ws_mut(h);
    let active = ws.project.active_surface;
    ws.project.active_surface = surface;
    let id = ws.project.add(o);
    ws.project.active_surface = active;
    h.run();
    id
}

fn get(h: &H, surface: usize, id: ObjectId) -> Object {
    let list = &ws(h).project.surfaces[surface].objects;
    (**tp_core::document::tree::get(list, id).expect("object")).clone()
}

fn select(h: &mut H, ids: &[ObjectId]) {
    ws_mut(h).selection = ids.to_vec();
    h.run();
}

fn toggled(h: &H, label: &str) -> bool {
    h.get_by_label(label)
        .accesskit_node()
        .toggled()
        .is_some_and(|t| t == Toggled::True)
}

/// A swatch "Company red" and a rectangle linked to it on the active
/// texture and on surface `other`.
fn company_red(h: &mut H, other: usize) -> (SwatchId, ObjectId, ObjectId) {
    let ws = ws_mut(h);
    let (id, _) = ws.project.add_swatch(RED, "Color");
    ws.project.rename_swatch(id, "Company red");
    let linked = |x| {
        let mut o = rect(x);
        o.fill = Paint::Solid(RED);
        o.fill_swatch = Some(id);
        o
    };
    let a = add_on(h, 0, linked(300.0));
    let b = add_on(h, other, linked(600.0));
    (id, a, b)
}

// ── Palette ─────────────────────────────────────────────────────────────

#[test]
fn linked_swatch_marked_and_picking_another_color_unlinks() {
    let mut h = open();
    let (_, a, _) = company_red(&mut h, 1);
    select(&mut h, &[a]);
    ws_mut(&mut h).panels.color_target = ColorTarget::Fill;
    h.run();
    assert!(toggled(&h, "Company red"), "the linked swatch has a ring");
    assert!(h.query_by_label("Linked to Company red").is_some());
    // A recent color: no swatch marked, the fill no longer linked.
    h.state_mut().prefs.push_recent_color([1, 2, 3, 255]);
    h.run();
    h.get_by_label("Recent color #010203").click();
    h.run();
    assert!(!toggled(&h, "Company red"));
    assert!(h.query_by_label("Linked to Company red").is_none());
    assert_eq!(get(&h, 0, a).fill_swatch, None);
}

#[test]
fn edit_the_company_red_across_textures() {
    let mut h = open();
    let (id, a, b) = company_red(&mut h, 1);
    h.get_by_label("Company red").click_secondary();
    h.run();
    h.get_by_label("Edit Swatch…").click();
    h.run();
    let hex = h.get_by_role_and_label(Role::TextInput, "Swatch hex color");
    hex.focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, "Swatch hex color")
        .type_text("#8B0000");
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(get(&h, 0, a).fill, Paint::Solid(DARK_RED));
    assert_eq!(get(&h, 1, b).fill, Paint::Solid(DARK_RED), "other texture");
    assert_eq!(ws(&h).project.swatch(id).unwrap().color, DARK_RED);
    assert_eq!(get(&h, 1, b).fill_swatch, Some(id));
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(get(&h, 0, a).fill, Paint::Solid(RED));
    assert_eq!(get(&h, 1, b).fill, Paint::Solid(RED));
}

#[test]
fn cancel_an_edit() {
    let mut h = open();
    let (id, a, _) = company_red(&mut h, 1);
    let steps = ws(&h).history.len();
    ws_mut(&mut h).start_swatch_edit(id);
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Swatch hex color")
        .focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, "Swatch hex color")
        .type_text("#00FF00");
    h.run();
    assert_eq!(
        get(&h, 0, a).fill,
        Paint::Solid(Rgba::rgb(0, 255, 0)),
        "live"
    );
    h.key_press(Key::Escape);
    h.run();
    assert!(ws(&h).panels.editing_swatch.is_none());
    assert_eq!(get(&h, 0, a).fill, Paint::Solid(RED));
    assert_eq!(ws(&h).project.swatch(id).unwrap().color, RED);
    assert_eq!(ws(&h).history.len(), steps);
}

// ── Styles panel ────────────────────────────────────────────────────────

fn red_stroked(x: f64) -> Object {
    let mut o = rect(x);
    o.fill = Paint::Solid(Rgba::rgb(255, 0, 0));
    o.stroke = Some(StrokeStyle {
        width: 4.0,
        ..StrokeStyle::default()
    });
    o.opacity = 0.8;
    o
}

#[test]
fn new_style_from_a_selected_object_and_apply_by_clicking() {
    let mut h = open();
    let source = add_on(&mut h, 0, red_stroked(300.0));
    let a = add_on(&mut h, 0, rect(600.0));
    let b = add_on(&mut h, 0, rect(900.0));
    select(&mut h, &[source]);
    h.get_by_label("New Graphic Style from Selection").click();
    h.run();
    let style = &ws(&h).project.graphic_styles[0];
    assert_eq!(style.name, "Style 1");
    assert_eq!(style.look.fill, Paint::Solid(Rgba::rgb(255, 0, 0)));
    assert_eq!(
        (style.look.stroke.unwrap().width, style.look.opacity),
        (4.0, 0.8)
    );
    let id = style.id;
    assert_eq!(get(&h, 0, source).style, Some(id));
    assert!(toggled(&h, "Graphic style Style 1"), "marked");

    select(&mut h, &[a, b]);
    assert!(!toggled(&h, "Graphic style Style 1"));
    h.get_by_label("Graphic style Style 1").click();
    h.run();
    for o in [a, b] {
        let o = get(&h, 0, o);
        assert_eq!((o.style, o.opacity), (Some(id), 0.8));
        assert_eq!(o.fill, Paint::Solid(Rgba::rgb(255, 0, 0)));
    }
    assert!(toggled(&h, "Graphic style Style 1"));
}

#[test]
fn delete_keeps_the_looks() {
    let mut h = open();
    let source = add_on(&mut h, 0, red_stroked(300.0));
    select(&mut h, &[source]);
    ws_mut(&mut h).new_graphic_style(1.0);
    h.run();
    h.get_by_label("Graphic style Style 1").click_secondary();
    h.run();
    h.get_by_label("Delete Style").click();
    h.run();
    assert!(ws(&h).project.graphic_styles.is_empty());
    let o = get(&h, 0, source);
    assert_eq!((o.style, o.opacity), (None, 0.8));
    assert_eq!(o.fill, Paint::Solid(Rgba::rgb(255, 0, 0)));
}

#[test]
fn text_style_needs_a_text() {
    let mut h = open();
    let r = add_on(&mut h, 0, rect(300.0));
    select(&mut h, &[r]);
    let button = h.get_by_label("New Text Style from Selection");
    assert!(button.accesskit_node().is_disabled());
    assert!(
        !h.get_by_label("New Graphic Style from Selection")
            .accesskit_node()
            .is_disabled()
    );
    assert_eq!(tp_i18n::tr("styles-need-text"), "Select one text.");
}

#[test]
fn restyle_the_fleets_lettering() {
    let mut h = open();
    let texts: Vec<(usize, ObjectId)> = (0..3)
        .map(|s| (s, add_on(&mut h, s, text(400.0))))
        .collect();
    // Laid out once, as the canvas does.
    ws_mut(&mut h).relayout_all_texts();
    select(&mut h, &[texts[0].1]);
    h.get_by_label("New Text Style from Selection").click();
    h.run();
    let style = ws(&h).project.text_styles[0].id;
    for (s, t) in &texts[1..] {
        let ws = ws_mut(&mut h);
        ws.project.active_surface = *s;
        ws.selection = vec![*t];
        ws.apply_style(style, 2.0);
    }
    ws_mut(&mut h).project.active_surface = 0;
    select(&mut h, &[texts[0].1]);
    let before = get(&h, 2, texts[2].1).text.unwrap().layout_size;
    // The user changes the selected text, then redefines the style.
    {
        let ws = ws_mut(&mut h);
        let mut o = get_ws(ws, texts[0].1);
        o.text.as_mut().unwrap().style.size = 400.0;
        ws.project.surface_mut().replace(&[o]);
    }
    h.run();
    h.get_by_label("Text style Text style 1").click_secondary();
    h.run();
    h.get_by_label("Redefine from Selection").click();
    h.run();
    for (s, t) in &texts {
        let t = get(&h, *s, *t).text.unwrap();
        assert_eq!((t.style.size, t.style_id), (400.0, Some(style)));
    }
    let after = get(&h, 2, texts[2].1).text.unwrap().layout_size;
    assert!(
        after.height > before.height,
        "laid out again on every texture"
    );
    ws_mut(&mut h).undo();
    h.run();
    for (s, t) in &texts[1..] {
        assert_eq!(get(&h, *s, *t).text.unwrap().style.size, 200.0);
    }
}

fn get_ws(ws: &Workspace, id: ObjectId) -> Object {
    (**ws.project.surface().get(id).unwrap()).clone()
}

// ── Copy from cabin ─────────────────────────────────────────────────────

/// A project for the sample truck painting both cabins (Standard cab and
/// High roof) and its accessories.
fn two_cabins(h: &mut H) {
    let package = Package::read(tp_app::vehicles::SAMPLES[0].bytes).unwrap();
    let mut textures = tp_app::vehicle_project::default_textures(&package.manifest);
    textures.push("high_roof".into());
    let project =
        tp_app::vehicle_project::fleet_project(common::SAMPLE, &package, &textures).unwrap();
    h.state_mut().open_project(project);
    h.run();
}

#[test]
fn copy_the_standard_cab_onto_the_high_roof() {
    let mut h = open();
    two_cabins(&mut h);
    let names: Vec<String> = ws(&h)
        .project
        .surfaces
        .iter()
        .map(|s| s.name.clone())
        .collect();
    let high_roof = names.iter().position(|n| n == "High roof").unwrap();
    let mut logo = rect(500.0);
    logo.name = "Logo".into();
    let logo = add_on(&mut h, 0, logo);
    let lettering = add_on(&mut h, 0, text(900.0));
    ws_mut(&mut h).set_active_surface(high_roof);
    h.run();
    h.state_mut().queue.push(CommandId::CopyFromCabin);
    // The command runs at the end of a frame; the dialog shows in the next.
    h.run_steps(3);
    assert!(h.query_by_label("Standard cab").is_some());
    h.get_by_label("Copy").click();
    h.run();
    let copies = ws(&h).project.surfaces[high_roof].objects.clone();
    assert_eq!(copies.len(), 2);
    assert_eq!(copies[0].frame, get(&h, 0, logo).frame);
    assert_eq!(copies[1].text, get(&h, 0, lettering).text);
    assert_eq!(ws(&h).selection.len(), 2);
    assert_eq!(
        ws(&h).project.surfaces[0].objects.len(),
        2,
        "source unchanged"
    );
    ws_mut(&mut h).undo();
    assert!(ws(&h).project.surfaces[high_roof].objects.is_empty());
}

#[test]
fn not_offered_for_accessories() {
    let mut h = open();
    two_cabins(&mut h);
    let chassis = ws(&h)
        .project
        .surfaces
        .iter()
        .position(|s| s.name == "Chassis")
        .unwrap();
    ws_mut(&mut h).set_active_surface(chassis);
    h.run();
    let context = h.state().edit_context();
    assert_eq!(context.cabin_sources, 0);
    let reason = tp_app::state::disabled_reason_for(CommandId::CopyFromCabin, &context);
    assert!(
        tp_i18n::tr(reason.unwrap()).contains("main textures"),
        "the tooltip says it copies between main textures"
    );
    ws_mut(&mut h).set_active_surface(0);
    h.run();
    assert_eq!(h.state().edit_context().cabin_sources, 1);
}

/// The reported flow: a text is colored, sized and stroked, saved as a
/// text style, then applied to a new text while it is still being typed.
#[test]
fn a_text_style_applies_the_whole_lettering_while_typing() {
    let mut h = open();
    let blue = Rgba::rgb(0x5B, 0x8D, 0xEF);
    let source = {
        let ws = ws_mut(&mut h);
        let id = ws.start_new_text(Point::new(500.0, 400.0), 0.0);
        ws.text_insert("Nice", 0.1);
        ws.end_text_session(0.2);
        ws.selection = vec![id];
        ws.apply_color(ColorTarget::Fill, blue);
        ws.commit_pending(1.0);
        ws.set_char_style("undo-change-font", |s| {
            s.weight = 900;
            s.size = 300.0;
        });
        ws.commit_pending(2.0);
        ws.apply_paint(
            ColorTarget::Stroke,
            Paint::Solid(Rgba::rgb(255, 0, 0)),
            "undo-change-stroke",
        );
        ws.commit_pending(3.0);
        id
    };
    h.run();
    // The accessibility action: the pointer click doesn't reach this
    // button in the harness after the setup above (egui sees no layer
    // over it); the button's handler is the same.
    h.get_by_label("New Text Style from Selection")
        .click_accesskit();
    h.run();
    let style = ws(&h).project.text_styles[0].id;
    assert_eq!(get(&h, 0, source).text.unwrap().style_id, Some(style));
    // A new text, still being typed, gets the style.
    let left = {
        let ws = ws_mut(&mut h);
        let id = ws.start_new_text(Point::new(500.0, 1200.0), 4.0);
        ws.text_insert("Left", 4.1);
        id
    };
    h.run();
    h.get_by_label("Text style Text style 1").click();
    h.run();
    let o = get(&h, 0, left);
    assert_eq!(o.fill, Paint::Solid(blue));
    assert_eq!(
        o.stroke.map(|s| s.paint),
        Some(Paint::Solid(Rgba::rgb(255, 0, 0)))
    );
    let t = o.text.unwrap();
    assert_eq!(
        (t.content.as_str(), t.style.weight, t.style.size),
        ("Left", 900, 300.0)
    );
    assert_eq!(t.style_id, Some(style));
    assert!(toggled(&h, "Text style Text style 1"));
    // One Undo removes the style, not the text.
    ws_mut(&mut h).undo();
    h.run();
    let t = get(&h, 0, left).text.unwrap();
    assert_eq!((t.content.as_str(), t.style_id), ("Left", None));
}
