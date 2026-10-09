//! Headless tests for the frame of an open project: the spaces, the top bar
//! with its breadcrumb and Export…, the tool rail, the tool options bar, the
//! left panel and the inspector, focus mode and the status bar.

mod common;

use egui::accesskit::{Role, Toggled};
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::layout::{LeftTab, Space};
use tp_app::state::Modal;
use tp_app::tool::Tool;
use tp_app::workspace::Workspace;
use tp_core::document::{Frame, ObjectId, ShapeKind};
use tp_core::kurbo::{Point, Size};
use tp_ui::tokens::size;

type H = Harness<'static, AppState>;

/// The sample truck project in the Workshop (Standard cab active).
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

fn space(h: &H) -> Space {
    ws(h).space
}

fn key_cmd(h: &mut H, key: Key) {
    h.key_press_modifiers(Modifiers::COMMAND, key);
    h.run();
}

fn add_shape(h: &mut H, kind: ShapeKind, x: f64, y: f64) -> ObjectId {
    let frame = Frame::new(Point::new(x, y), Size::new(300.0, 300.0), 0.0);
    let id = ws_mut(h).create_shape(kind, frame, 1.0);
    h.run();
    id
}

fn hexagon(h: &mut H, x: f64) -> ObjectId {
    add_shape(
        h,
        ShapeKind::Polygon {
            sides: 6,
            star: None,
        },
        x,
        800.0,
    )
}

fn toggled(h: &H, role: Role, label: &str) -> bool {
    h.get_by_role_and_label(role, label)
        .accesskit_node()
        .toggled()
        == Some(Toggled::True)
}

/// Whether the breadcrumb (top bar and canvas) reads `text`.
fn breadcrumb(h: &H, text: &str) -> bool {
    h.query_all_by_label(text).count() == 2
}

/// Presses at `from`, drags to `to` over a few frames and releases.
fn drag(h: &mut H, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.step();
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    for i in 1..=4 {
        h.event(Event::PointerMoved(from + (to - from) * (i as f32 / 4.0)));
        h.step();
    }
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run();
}

/// Hovers `node` long enough for its tooltip to show.
fn hover_for_tooltip(h: &mut H, label: &str, role: Role) {
    h.get_by_role_and_label(role, label).hover();
    for _ in 0..60 {
        h.step();
    }
    h.run();
}

// --- Spaces --------------------------------------------------------------------

#[test]
fn a_new_project_opens_in_the_project_space() {
    let mut h = common::harness();
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_role_and_label(Role::RadioButton, common::SAMPLE)
        .click();
    h.run();
    h.get_by_label("Create Project").click();
    h.run();
    assert_eq!(space(&h), Space::Project);
    assert_eq!(ws(&h).project.surface().name, "Standard cab");
    assert!(toggled(&h, Role::RadioButton, "Project"));
}

#[test]
fn project_frame_has_no_workshop_parts() {
    let mut h = open();
    key_cmd(&mut h, Key::Num1);
    assert_eq!(space(&h), Space::Project);
    for (role, label) in [
        (Role::Button, "Ellipse"),
        (Role::RadioButton, "Layers"),
        (Role::TextInput, "Hex color"),
        (Role::CheckBox, "Snapping"),
    ] {
        assert!(h.query_by_role_and_label(role, label).is_none(), "{label}");
    }
    assert!(h.query_by_label_contains("Zoom ").is_none());
    assert!(
        h.query_by_label("Unsaved changes").is_some(),
        "save state only"
    );
    // The menu bar, the top bar and the status bar stay.
    h.get_by_label("File");
    h.get_by_label("Export…");
}

#[test]
fn clicking_a_texture_in_the_project_space_shows_it_in_the_workshop() {
    let mut h = open();
    key_cmd(&mut h, Key::Num1);
    h.get_by_label("Texture TruckPaint Sample Truck › Side skirts")
        .click();
    h.run();
    assert_eq!(space(&h), Space::Workshop);
    assert_eq!(ws(&h).project.surface().name, "Side skirts");
    // Highlighted in the Textures tab.
    assert_eq!(
        h.get_by_label("Texture TruckPaint Sample Truck › Side skirts")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True)
    );
}

#[test]
fn switching_spaces_keeps_selection_zoom_and_view() {
    let mut h = open();
    let id = add_shape(&mut h, ShapeKind::rectangle(), 500.0, 500.0);
    let view = {
        let ws = ws_mut(&mut h);
        let mut view = ws.viewport.unwrap();
        view.zoom = 2.0;
        view.fitted = false;
        view.center = Point::new(900.0, 700.0);
        ws.viewport = Some(view);
        view
    };
    h.run();
    key_cmd(&mut h, Key::Num1);
    key_cmd(&mut h, Key::Num2);
    assert_eq!(space(&h), Space::Workshop);
    assert_eq!(ws(&h).selection, vec![id]);
    assert_eq!(ws(&h).viewport, Some(view));
}

#[test]
fn undo_from_the_brand_space_stays_in_brand() {
    let mut h = open();
    let id = add_shape(&mut h, ShapeKind::rectangle(), 500.0, 500.0);
    key_cmd(&mut h, Key::Num3);
    assert_eq!(space(&h), Space::Brand);
    key_cmd(&mut h, Key::Z);
    assert!(ws(&h).project.surface().get(id).is_none());
    assert_eq!(space(&h), Space::Brand);
}

#[test]
fn the_switcher_shows_a_space() {
    let mut h = open();
    key_cmd(&mut h, Key::Num3);
    assert!(toggled(&h, Role::RadioButton, "Brand"));
    h.get_by_role_and_label(Role::RadioButton, "Workshop")
        .click();
    h.run();
    assert_eq!(space(&h), Space::Workshop);
    assert!(toggled(&h, Role::RadioButton, "Workshop"));
    assert!(!toggled(&h, Role::RadioButton, "Brand"));
}

// --- Top bar, Export…, breadcrumb ---------------------------------------------

#[test]
fn top_bar_names_the_project_and_its_game() {
    let mut h = open();
    ws_mut(&mut h).project.name = "ACE Logistics".into();
    h.run();
    let name = h.get_by_label("ACE Logistics").rect();
    let badge = h.get_by_label("ETS2").rect();
    assert!(name.right() <= badge.left());
    assert!(name.bottom() <= size::MENU_BAR_HEIGHT + size::TOP_BAR_HEIGHT + 1.0);
}

#[test]
fn a_long_name_is_shortened_and_the_controls_stay_whole() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(900.0, 700.0))
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(tp_app::prefs::Prefs::default(), None),
        );
    common::create_project(&mut h);
    let long = "ACE Logistics International Heavy Haulage and Refrigerated Transport";
    ws_mut(&mut h).project.name = long.into();
    h.run();
    let export = h.get_by_role_and_label(Role::Button, "Export…").rect();
    assert!(export.right() <= 900.0, "{export:?}");
    let project = h.get_by_role_and_label(Role::RadioButton, "Project").rect();
    let name = h.get_by_label(long).rect();
    assert!(name.right() <= project.left(), "{name:?} {project:?}");
    // Shown in full on hover.
    h.get_by_label(long).hover();
    for _ in 0..60 {
        h.step();
    }
    h.run();
    assert!(h.query_all_by_label(long).count() >= 2, "tooltip");
}

#[test]
fn export_from_the_project_space_and_with_the_keyboard() {
    let mut h = open();
    key_cmd(&mut h, Key::Num1);
    // The dialog's summary shows a spinner: step rather than run.
    h.get_by_role_and_label(Role::Button, "Export…").click();
    h.step();
    assert!(matches!(h.state().modal, Some(Modal::ExportMod(_))));
    h.state_mut().modal = None;
    key_cmd(&mut h, Key::Num2);
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    h.step();
    assert!(matches!(h.state().modal, Some(Modal::ExportMod(_))));
}

/// The sample project with "Logo" being edited (made from a rectangle).
fn editing_logo(h: &mut H) {
    let id = add_shape(h, ShapeKind::rectangle(), 500.0, 500.0);
    let ws = ws_mut(h);
    ws.selection = vec![id];
    let symbol = ws.convert_to_symbol(1.0).unwrap();
    ws.project.rename_symbol(symbol, "Logo");
    ws.edit_symbol(symbol, 2.0);
    h.run();
}

#[test]
fn export_is_disabled_while_a_symbol_is_edited() {
    let mut h = open();
    editing_logo(&mut h);
    assert!(
        h.get_by_role_and_label(Role::Button, "Export…")
            .accesskit_node()
            .is_disabled()
    );
    hover_for_tooltip(&mut h, "Export…", Role::Button);
    let reason = tp_i18n::tr("reason-editing-symbol");
    assert!(h.query_by_label(&reason).is_some(), "{reason}");
}

#[test]
fn breadcrumb_follows_the_active_texture() {
    let mut h = open();
    assert!(breadcrumb(
        &h,
        "TruckPaint Sample Truck › Main textures › Standard cab 4096²"
    ));
    key_cmd(&mut h, Key::CloseBracket);
    let chassis = ws(&h).project.surface();
    assert_eq!(chassis.name, "Chassis");
    let text = format!(
        "TruckPaint Sample Truck › Accessories › Chassis {}²",
        chassis.size
    );
    assert!(breadcrumb(&h, &text), "{text}");
    let skirts = ws(&h)
        .project
        .surfaces
        .iter()
        .position(|s| s.name == "Side skirts")
        .unwrap();
    ws_mut(&mut h).set_active_surface(skirts);
    h.run();
    assert!(breadcrumb(
        &h,
        "TruckPaint Sample Truck › Accessories › Side skirts 1024²"
    ));
    // The canvas's breadcrumb stays when the panels are hidden.
    h.key_press(Key::Tab);
    h.run();
    assert!(breadcrumb(
        &h,
        "TruckPaint Sample Truck › Accessories › Side skirts 1024²"
    ));
    // Not in the other spaces.
    key_cmd(&mut h, Key::Num1);
    assert!(
        h.query_by_label_contains("Side skirts 1024²").is_none(),
        "Workshop only"
    );
}

#[test]
fn breadcrumb_while_editing_a_symbol() {
    let mut h = open();
    editing_logo(&mut h);
    assert!(breadcrumb(&h, "Symbol › Logo"));
    // The bar above the canvas with Done stays.
    assert!(h.query_by_label("Editing symbol Logo").is_some());
    h.get_by_label("Done").click();
    h.run();
    assert!(!ws(&h).is_editing_symbol());
}

#[test]
fn the_canvas_breadcrumb_lets_the_canvas_take_presses() {
    let mut h = open();
    ws_mut(&mut h).tool = Tool::Rectangle;
    h.run();
    let crumb = h
        .query_all_by_label("TruckPaint Sample Truck › Main textures › Standard cab 4096²")
        .last()
        .unwrap()
        .rect();
    let before = ws(&h).project.surface().objects.len();
    drag(
        &mut h,
        crumb.center(),
        crumb.center() + Vec2::new(200.0, 200.0),
    );
    assert_eq!(ws(&h).project.surface().objects.len(), before + 1);
}

// --- Tool rail ------------------------------------------------------------------

#[test]
fn tool_rail_is_48_px_with_one_active_tool() {
    let mut h = open();
    let select = h.get_by_role_and_label(Role::Button, "Selection").rect();
    // The left panel starts at the rail's right edge (its first tab after
    // the panel's margin and the tab strip's inset), half a rail from the
    // centered tool buttons.
    let tabs = h
        .get_by_role_and_label(Role::RadioButton, "Textures")
        .rect();
    let margin = tp_ui::tokens::space::SM + 2.0 + 4.0;
    let rail_half = tabs.left() - margin - select.center().x;
    assert!(
        (rail_half - size::TOOL_RAIL_WIDTH / 2.0).abs() < 1.0,
        "{select:?} {tabs:?}"
    );
    assert!(toggled(&h, Role::Button, "Selection"));
    h.get_by_role_and_label(Role::Button, "Rectangle").click();
    h.run();
    assert_eq!(ws(&h).tool, Tool::Rectangle);
    assert!(toggled(&h, Role::Button, "Rectangle"));
    assert!(!toggled(&h, Role::Button, "Selection"));
    // The rail stays when the panels are hidden.
    h.key_press(Key::Tab);
    h.run();
    h.get_by_role_and_label(Role::Button, "Ellipse");
}

#[test]
fn tool_tooltip_in_french_with_its_shortcut() {
    let mut prefs = tp_app::prefs::Prefs::default();
    prefs.set_language(Some(tp_i18n::Language::French));
    let mut h = common::harness_with(prefs);
    common::create_project(&mut h);
    let name = tp_i18n::tr("tool-gradient");
    assert_eq!(name, "Dégradé");
    hover_for_tooltip(&mut h, &name, Role::Button);
    let shortcut = tp_app::commands::ShortcutFormatter::new(&h.ctx)
        .command(tp_app::commands::CommandId::SelectTool(Tool::Gradient))
        .unwrap();
    assert!(shortcut.contains('G'), "{shortcut}");
    assert!(h.query_by_label(&shortcut).is_some(), "{shortcut}");
    tp_i18n::set_language(tp_i18n::Language::English);
}

// --- Tool options bar -------------------------------------------------------------

/// The Sides field of the tool options bar (the topmost one).
fn options_sides(h: &H) -> egui_kittest::Node<'_> {
    h.get_all_by_role_and_label(Role::TextInput, "Sides")
        .min_by(|a, b| a.rect().top().total_cmp(&b.rect().top()))
        .unwrap()
}

fn type_in_options_sides(h: &mut H, text: &str) {
    options_sides(h).focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    options_sides(h).type_text(text);
    h.run();
    h.key_press(Key::Enter);
    h.run();
}

#[test]
fn polygon_settings_for_new_polygons() {
    let mut h = open();
    key_cmd(&mut h, Key::Num2);
    ws_mut(&mut h).tool = Tool::Polygon;
    h.run();
    let steps = ws(&h).history.len();
    type_in_options_sides(&mut h, "5");
    h.get_by_role_and_label(Role::CheckBox, "Star").click();
    h.run();
    assert_eq!(ws(&h).history.len(), steps, "settings are not recorded");
    let map = ws(&h).screen_map(1.0).unwrap();
    drag(
        &mut h,
        map.to_screen(Point::new(1000.0, 1000.0)),
        map.to_screen(Point::new(1800.0, 1800.0)),
    );
    let star = ws(&h).selected_objects()[0].clone();
    assert_eq!(
        star.kind,
        ShapeKind::Polygon {
            sides: 5,
            star: Some(0.5)
        }
    );
    assert_eq!(star.flattened(0.1).len(), 10);
}

#[test]
fn polygon_settings_edit_the_selected_polygons() {
    let mut h = open();
    let a = hexagon(&mut h, 800.0);
    let b = hexagon(&mut h, 1600.0);
    ws_mut(&mut h).selection = vec![a, b];
    ws_mut(&mut h).tool = Tool::Polygon;
    h.run();
    let frames = [a, b].map(|id| ws(&h).project.surface().get(id).unwrap().frame);
    type_in_options_sides(&mut h, "8");
    for (id, frame) in [a, b].into_iter().zip(frames) {
        let o = ws(&h).project.surface().get(id).unwrap().clone();
        assert_eq!(
            o.kind,
            ShapeKind::Polygon {
                sides: 8,
                star: None
            }
        );
        assert_eq!(o.frame, frame, "in their bounds");
    }
    // The inspector's Polygon section shows the same value.
    let values: Vec<_> = h
        .get_all_by_role_and_label(Role::TextInput, "Sides")
        .map(|n| n.value())
        .collect();
    assert_eq!(values, [Some("8".into()), Some("8".into())]);
    key_cmd(&mut h, Key::Z);
    for id in [a, b] {
        assert_eq!(
            ws(&h).project.surface().get(id).unwrap().kind,
            ShapeKind::Polygon {
                sides: 6,
                star: None
            }
        );
    }
}

#[test]
fn other_tools_show_their_name_only() {
    let mut h = open();
    ws_mut(&mut h).tool = Tool::Hand;
    h.run();
    assert!(h.query_by_role_and_label(Role::Label, "Hand").is_some());
    assert!(h.query_by_label("Sides").is_none());
    assert!(h.query_by_label("Star").is_none());
}

// --- Left panel, inspector, focus mode, canvas ----------------------------------------

#[test]
fn workshop_frame_from_left_to_right() {
    let h = open();
    let rail = h.get_by_role_and_label(Role::Button, "Selection").rect();
    let left = h
        .get_by_role_and_label(Role::RadioButton, "Textures")
        .rect();
    let canvas = h.get_by_label("Canvas").rect();
    let inspector = h.get_by_label("Fill color").rect();
    assert!(rail.right() <= left.left(), "{rail:?} {left:?}");
    assert!(left.right() <= canvas.left(), "{left:?} {canvas:?}");
    assert!(
        canvas.right() <= inspector.left(),
        "{canvas:?} {inspector:?}"
    );
    // The tool options bar runs between the top bar and the canvas.
    let options = h.get_by_role_and_label(Role::Label, "Selection").rect();
    let export = h.get_by_role_and_label(Role::Button, "Export…").rect();
    assert!(export.bottom() <= options.top() && options.bottom() <= canvas.top());
    // Nothing selected: the inspector describes the active texture.
    assert!(h.query_by_label("Standard cab").is_some());
}

#[test]
fn the_left_panel_resizes_within_bounds() {
    let mut h = open();
    let edge = size::TOOL_RAIL_WIDTH + h.state().prefs.layout.left_width;
    let y = 500.0;
    drag(&mut h, Pos2::new(edge, y), Pos2::new(edge + 60.0, y));
    // Wider by the drag, less egui's drag threshold.
    let width = h.state().prefs.layout.left_width;
    assert!(
        width > size::LEFT_PANEL_DEFAULT + 40.0 && width <= size::LEFT_PANEL_DEFAULT + 60.0,
        "{width}"
    );
    let edge = size::TOOL_RAIL_WIDTH + width;
    drag(&mut h, Pos2::new(edge, y), Pos2::new(edge + 600.0, y));
    assert_eq!(h.state().prefs.layout.left_width, size::LEFT_PANEL_MAX);
}

#[test]
fn the_inspector_resizes_within_bounds() {
    let mut h = open();
    let edge = common::SIZE.x - h.state().prefs.layout.inspector_width;
    let y = 500.0;
    drag(&mut h, Pos2::new(edge, y), Pos2::new(edge - 50.0, y));
    let width = h.state().prefs.layout.inspector_width;
    assert!(
        width > size::INSPECTOR_DEFAULT + 30.0 && width <= size::INSPECTOR_DEFAULT + 50.0,
        "{width}"
    );
    let edge = common::SIZE.x - width;
    drag(&mut h, Pos2::new(edge, y), Pos2::new(edge - 600.0, y));
    assert_eq!(h.state().prefs.layout.inspector_width, size::INSPECTOR_MAX);
    // Down to its minimum: every section fits it.
    let edge = common::SIZE.x - size::INSPECTOR_MAX;
    drag(&mut h, Pos2::new(edge, y), Pos2::new(edge + 600.0, y));
    assert_eq!(h.state().prefs.layout.inspector_width, size::INSPECTOR_MIN);
}

#[test]
fn every_inspector_section_fits_the_minimum_width() {
    let mut h = open();
    h.state_mut().prefs.layout.inspector_width = size::INSPECTOR_MIN;
    h.state_mut().prefs.layout.generation += 1;
    h.run();
    let rectangle = add_shape(&mut h, ShapeKind::rectangle(), 500.0, 500.0);
    let polygon = hexagon(&mut h, 1500.0);
    let at = Point::new(1000.0, 2000.0);
    let mut text = tp_core::document::Object::text(
        ObjectId(0),
        tp_core::document::TextBlock::new("ACE", tp_core::document::CharStyle::default()),
        at,
    );
    ws_mut(&mut h).text.place_at(&mut text, at);
    let text = ws_mut(&mut h).project.add(text);
    for selection in [
        vec![],
        vec![rectangle],
        vec![polygon],
        vec![text],
        vec![rectangle, text],
    ] {
        ws_mut(&mut h).selection = selection.clone();
        h.run();
        assert_eq!(
            h.state().prefs.layout.inspector_width,
            size::INSPECTOR_MIN,
            "{selection:?}"
        );
        let fill = h.get_by_label("Fill color").rect();
        assert!(fill.right() <= common::SIZE.x, "{selection:?}");
    }
}

#[test]
fn tabs_show_their_content() {
    let mut h = open();
    let id = add_shape(&mut h, ShapeKind::rectangle(), 500.0, 500.0);
    let name = ws(&h).project.surface().get(id).unwrap().name.clone();
    let rows = |h: &H| h.query_all_by_label(&name).count();
    assert!(
        h.query_by_label("Texture TruckPaint Sample Truck › Chassis")
            .is_some()
    );
    let without_layers = rows(&h);
    h.get_by_role_and_label(Role::RadioButton, "Layers").click();
    h.run();
    assert_eq!(h.state().prefs.layout.left_tab, LeftTab::Layers);
    assert_eq!(rows(&h), without_layers + 1, "the layer's row");
    assert!(
        h.query_by_label("Texture TruckPaint Sample Truck › Chassis")
            .is_none()
    );
    h.get_by_role_and_label(Role::RadioButton, "Resources")
        .click();
    h.run();
    assert_eq!(h.state().prefs.layout.left_tab, LeftTab::Resources);
    let empty = tp_i18n::tr("symbols-empty");
    assert!(h.query_by_label(&empty).is_some(), "{empty}");
}

#[test]
fn hiding_the_panels_keeps_the_zoom_and_the_center() {
    let mut h = open();
    let view = {
        let ws = ws_mut(&mut h);
        let mut view = ws.viewport.unwrap();
        view.zoom = 2.0;
        view.fitted = false;
        ws.viewport = Some(view);
        view
    };
    h.run();
    let before = ws(&h).canvas_rect.unwrap();
    h.key_press(Key::Tab);
    h.run();
    let after = ws(&h).canvas_rect.unwrap();
    assert!(after.width() > before.width());
    assert_eq!(ws(&h).viewport, Some(view));
    let map = ws(&h).screen_map(1.0).unwrap();
    let center = map.to_doc(after.center());
    assert!((center - view.center).hypot() < 1e-3);
}

// --- Status bar ---------------------------------------------------------------------

#[test]
fn status_bar_shows_the_pointer_position() {
    let mut h = open();
    let at = ws(&h)
        .screen_map(1.0)
        .unwrap()
        .to_screen(Point::new(1000.5, 2000.5));
    h.event(Event::PointerMoved(at));
    h.run();
    assert!(h.query_by_label("x 1000  y 2000 px").is_some());
}

#[test]
fn grid_toggle_in_the_status_bar() {
    let mut h = open();
    assert!(!h.state().prefs.view_aids.grid);
    assert!(!toggled(&h, Role::CheckBox, "Grid"));
    h.get_by_role_and_label(Role::CheckBox, "Grid").click();
    h.run();
    assert!(h.state().prefs.view_aids.grid);
    assert!(toggled(&h, Role::CheckBox, "Grid"));
    h.get_by_label("View").click();
    h.run();
    assert!(toggled(&h, Role::CheckBox, "Show Grid"));
}

#[test]
fn status_bar_in_the_brand_space_shows_the_save_state_only() {
    let mut h = open();
    key_cmd(&mut h, Key::Num3);
    assert!(h.query_by_label("Unsaved changes").is_some());
    assert!(h.query_by_label_contains("Zoom ").is_none());
    for label in ["Template", "Snapping", "Grid", "Guides"] {
        assert!(
            h.query_by_role_and_label(Role::CheckBox, label).is_none(),
            "{label}"
        );
    }
}

#[test]
fn template_opacity_in_the_status_bar() {
    let mut h = open();
    let opacity = |h: &H, i: usize| ws(h).project.surfaces[i].template.as_ref().unwrap().opacity;
    let others = opacity(&h, 1);
    let steps = ws(&h).history.len();
    let slider = h
        .get_by_role_and_label(Role::Slider, "Template opacity")
        .rect();
    // The thin slider's track spans its whole width.
    let x = slider.left() + slider.width() * 0.35;
    h.event(Event::PointerMoved(Pos2::new(x, slider.center().y)));
    h.step();
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: Pos2::new(x, slider.center().y),
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run();
    assert!((opacity(&h, 0) - 0.35).abs() < 0.03, "{}", opacity(&h, 0));
    assert_eq!(opacity(&h, 1), others, "other textures keep theirs");
    assert_eq!(ws(&h).history.len(), steps, "not an undo step");
    assert!(ws(&h).settings_changed);
}
