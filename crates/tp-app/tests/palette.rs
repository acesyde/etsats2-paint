//! Headless tests for the command palette: opening and closing it, its
//! rows (commands with shortcut, path, check mark and disabled reason; the
//! project's textures with their state), the keys, choosing a command or a
//! texture, recent commands, and the Textures tab's Search textures field.

mod common;

use egui::accesskit::{Role, Toggled};
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::commands::{CommandId, ShortcutFormatter, format_shortcut};
use tp_app::layout::{LeftTab, Space};
use tp_app::prefs::Prefs;
use tp_app::state::Modal;
use tp_app::tool::Tool;
use tp_app::ui::palette::Palette;
use tp_app::workspace::Workspace;
use tp_core::TemplateStatus;
use tp_core::document::{FlipAxis, Frame, ObjectId, ShapeKind};
use tp_core::kurbo::{Point, Size};
use tp_vehicles::Package;

type H = Harness<'static, AppState>;

const PLACEHOLDER: &str = "Search commands and textures";
const TRUCK: &str = "TruckPaint Sample Truck";
const TRAILER: &str = "TruckPaint Sample Trailer";

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

/// The sample truck project in the Workshop, snapping off.
fn open() -> H {
    let mut prefs = Prefs::default();
    prefs.view_aids.snapping = false;
    let mut h = common::harness_with(prefs);
    common::create_project(&mut h);
    h
}

/// The sample truck with its default textures and the sample trailer with
/// all of its, in the Workshop.
fn fleet() -> H {
    let mut h = open();
    let trailer = Package::read(tp_app::vehicles::SAMPLES[1].bytes).unwrap();
    let textures = tp_app::vehicle_project::default_textures(&trailer.manifest);
    let ws = ws_mut(&mut h);
    ws.add_vehicle(&trailer, &textures, 0.0).unwrap();
    for surface in &mut ws.project.surfaces {
        if let Some(t) = surface.template.as_mut() {
            t.visible = false;
        }
    }
    common::settle_renders(&mut h);
    h
}

fn surface(h: &H, name: &str) -> usize {
    ws(h)
        .project
        .surfaces
        .iter()
        .position(|s| s.name == name)
        .unwrap_or_else(|| panic!("no texture {name}"))
}

/// Runs a few frames (a caret blinking keeps `Harness::run` from
/// settling).
fn settle(h: &mut H) {
    for _ in 0..4 {
        h.step();
    }
}

/// A rectangle turned by 30°, so that flipping it changes it.
fn rect(h: &mut H) -> ObjectId {
    let now = h.ctx.input(|i| i.time);
    let frame = Frame::new(Point::new(600.0, 500.0), Size::new(300.0, 100.0), 30.0);
    let id = ws_mut(h).create_shape(ShapeKind::rectangle(), frame, now);
    h.run();
    id
}

fn select(h: &mut H, ids: &[ObjectId]) {
    ws_mut(h).selection = ids.to_vec();
    h.run();
}

fn shortcut(h: &mut H) {
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    settle(h);
}

fn palette(h: &H) -> Option<&Palette> {
    match &h.state().modal {
        Some(Modal::CommandPalette(p)) => Some(p),
        _ => None,
    }
}

fn is_open(h: &H) -> bool {
    palette(h).is_some()
}

/// Opens the palette with its shortcut.
fn open_palette(h: &mut H) {
    shortcut(h);
    assert!(is_open(h), "the palette should be open");
}

fn field_focused(h: &H) -> bool {
    h.ctx
        .memory(|m| m.focused() == Some(egui::Id::new("command_palette_field")))
}

/// Types `text` like a keyboard (key presses and the text they produce)
/// into the focused field.
fn type_keys(h: &mut H, text: &str) {
    for c in text.chars() {
        let name = if c == ' ' {
            "Space".to_owned()
        } else {
            c.to_ascii_uppercase().to_string()
        };
        if let Some(key) = Key::from_name(&name) {
            h.event(Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            });
        }
        h.event(Event::Text(c.to_string()));
        h.step();
    }
    settle(h);
}

/// Opens the palette and types `query`.
fn search(h: &mut H, query: &str) {
    open_palette(h);
    type_keys(h, query);
    assert_eq!(palette(h).unwrap().query, query);
}

fn press(h: &mut H, key: Key) {
    h.key_press(key);
    settle(h);
}

/// The accessible names of the palette's rows, top to bottom.
fn rows(h: &H) -> Vec<String> {
    let mut rows: Vec<(f32, String)> = h
        .query_all_by(|n| {
            n.toggled().is_some()
                && n.label()
                    .is_some_and(|l| l.contains(" › ") && !l.starts_with("Texture "))
        })
        .map(|n| (n.rect().top(), n.accesskit_node().label().unwrap()))
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    rows.into_iter().map(|(_, l)| l).collect()
}

/// The row whose name starts with `start`.
fn row<'h>(h: &'h H, start: &str) -> egui_kittest::Node<'h> {
    let prefix = start.to_owned();
    h.query_all_by(move |n| n.label().is_some_and(|l| l.starts_with(&prefix)))
        .find(|n| n.accesskit_node().toggled().is_some())
        .unwrap_or_else(|| panic!("no row {start:?}"))
}

fn has_row(h: &H, start: &str) -> bool {
    h.query_all_by(|n| n.label().is_some_and(|l| l.starts_with(start)))
        .any(|n| n.accesskit_node().toggled().is_some())
}

/// The top of the palette's heading `label` (lined up with Commands).
fn heading(h: &H, label: &str) -> f32 {
    let left = h.get_by_label("Commands").rect().left();
    h.get_all_by_label(label)
        .map(|n| n.rect())
        .find(|r| (r.left() - left).abs() < 1.0)
        .unwrap_or_else(|| panic!("no heading {label}"))
        .top()
}

fn highlighted(node: &egui_kittest::Node<'_>) -> bool {
    node.accesskit_node().toggled() == Some(Toggled::True)
}

fn shortcut_text(h: &H, id: CommandId) -> String {
    ShortcutFormatter::new(&h.ctx).command(id).unwrap()
}

/// A press and release at `at`, as a click on the canvas.
fn click_at(h: &mut H, at: Pos2) {
    h.event(Event::PointerMoved(at));
    h.step();
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
    settle(h);
}

/// Drags on the canvas from `from` to `to`.
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

fn screen(h: &H, x: f64, y: f64) -> Pos2 {
    ws(h)
        .screen_map(1.0)
        .expect("canvas")
        .to_screen(Point::new(x, y))
}

// ------------------------------------------------------ opening and closing

#[test]
fn opening_from_the_workshop() {
    let mut h = open();
    open_palette(&mut h);
    let p = palette(&h).unwrap();
    assert!(p.query.is_empty() && !p.textures_only);
    assert!(field_focused(&h));
    assert!(
        h.query_by_role_and_label(Role::TextInput, PLACEHOLDER)
            .is_some()
    );
}

#[test]
fn opening_while_typing_in_a_field() {
    let mut h = open();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    h.get_by_role_and_label(Role::TextInput, "X position")
        .focus();
    h.run();
    let before = ws(&h).project.surface().get(id).unwrap().frame;
    shortcut(&mut h);
    assert!(is_open(&h));
    h.run();
    assert!(field_focused(&h));
    assert_eq!(ws(&h).project.surface().get(id).unwrap().frame, before);
}

/// command-system: Command palette by key.
#[test]
fn opening_while_the_opacity_field_has_focus() {
    let mut h = open();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    h.get_by_role_and_label(Role::TextInput, "Opacity").focus();
    h.run();
    shortcut(&mut h);
    assert!(is_open(&h));
}

#[test]
fn opening_on_the_home_screen() {
    let mut h = common::harness();
    h.run();
    open_palette(&mut h);
    assert!(h.query_by_label("Commands").is_some());
    assert!(h.query_by_label("Textures").is_none());
    assert!(has_row(&h, "File › New Project…"));
}

#[test]
fn not_over_a_dialog() {
    let mut h = open();
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::ExportMod(_))));
    shortcut(&mut h);
    assert!(matches!(h.state().modal, Some(Modal::ExportMod(_))));
}

#[test]
fn toggling_with_the_shortcut() {
    let mut h = open();
    open_palette(&mut h);
    shortcut(&mut h);
    assert!(!is_open(&h));
    open_palette(&mut h);
    press(&mut h, Key::Escape);
    assert!(!is_open(&h));
}

#[test]
fn closing_by_a_press_outside() {
    let mut h = open();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    ws_mut(&mut h).tool = Tool::Rectangle;
    let objects = ws(&h).project.surface().objects.len();
    open_palette(&mut h);
    // A press and drag on the canvas, far from the palette.
    let from = screen(&h, 200.0, 3000.0);
    drag(&mut h, from, from + Vec2::new(80.0, 60.0));
    assert!(!is_open(&h));
    assert_eq!(ws(&h).project.surface().objects.len(), objects);
    assert_eq!(ws(&h).selection, [id]);
    // A click outside closes it too, and selects nothing.
    open_palette(&mut h);
    ws_mut(&mut h).tool = Tool::Select;
    let at = screen(&h, 200.0, 3000.0);
    click_at(&mut h, at);
    assert!(!is_open(&h));
    assert_eq!(ws(&h).selection, [id]);
}

#[test]
fn text_editing_kept() {
    let mut h = open();
    ws_mut(&mut h).tool = Tool::Text;
    let at = screen(&h, 800.0, 900.0);
    click_at(&mut h, at);
    type_keys(&mut h, "ACE");
    let id = ws(&h).editing_text().expect("editing");
    let caret = ws(&h).text_session.as_ref().unwrap().edit.cursor();
    open_palette(&mut h);
    assert_eq!(ws(&h).editing_text(), Some(id));
    press(&mut h, Key::Escape);
    assert!(!is_open(&h));
    assert_eq!(ws(&h).editing_text(), Some(id));
    assert_eq!(ws(&h).text_session.as_ref().unwrap().edit.cursor(), caret);
}

// --------------------------------------------------------------------- rows

#[test]
fn a_command_with_its_shortcut_and_path() {
    let mut h = open();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    search(&mut h, "flip hor");
    let key = shortcut_text(&h, CommandId::Flip(FlipAxis::Horizontal));
    if h.ctx.os().is_mac() {
        assert_eq!(key, "⇧H");
    }
    let name = format!("Object › Flip Horizontal, {key}");
    assert_eq!(rows(&h)[0], name);
    assert!(highlighted(&row(&h, &name)));
}

#[test]
fn a_toggle_shows_its_state() {
    let mut h = open();
    h.state_mut().prefs.view_aids.grid = true;
    search(&mut h, "grid");
    assert!(has_row(&h, "View › Show Grid"));
    let check = h.get_by_role_and_label(Role::CheckBox, "Show Grid");
    assert_eq!(check.accesskit_node().toggled(), Some(Toggled::True));
    // Off: no check mark.
    press(&mut h, Key::Escape);
    h.state_mut().prefs.view_aids.grid = false;
    search(&mut h, "grid");
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Show Grid")
            .is_none()
    );
}

#[test]
fn a_tool_outside_the_menus() {
    let mut h = open();
    search(&mut h, "ellipse");
    let key = shortcut_text(&h, CommandId::SelectTool(Tool::Ellipse));
    assert_eq!(key, "E");
    assert!(has_row(&h, "Tools › Ellipse, E"));
}

#[test]
fn texture_rows_of_a_fleet() {
    let mut h = fleet();
    search(&mut h, "mudfl");
    let node = row(&h, &format!("{TRAILER} › Accessories › Mudflaps"));
    let name = node.accesskit_node().label().unwrap();
    assert!(name.ends_with("², Empty"), "{name}");
}

#[test]
fn flagged_texture() {
    let mut h = fleet();
    let i = surface(&h, "Standard cab");
    ws_mut(&mut h).project.surfaces[i]
        .template
        .as_mut()
        .unwrap()
        .status = TemplateStatus::LayoutChanged;
    h.run();
    search(&mut h, "standard");
    let name = row(&h, &format!("{TRUCK} › Main textures › Standard cab"))
        .accesskit_node()
        .label()
        .unwrap();
    assert!(name.ends_with("To check"), "{name}");
}

#[test]
fn nothing_matches() {
    let mut h = open();
    search(&mut h, "zzzz");
    assert!(rows(&h).is_empty());
    assert!(
        h.query_by_label("No command or texture matches “zzzz”.")
            .is_some()
    );
}

#[test]
fn gesture_commands_left_out() {
    let mut h = open();
    search(&mut h, "nudge");
    assert!(
        !rows(&h).iter().any(|r| r.contains("Nudge")),
        "{:?}",
        rows(&h)
    );
    assert!(h.query_by_label_contains("Command Palette").is_none());
}

/// workspace-layout: Palette path follows the menu.
#[test]
fn palette_path_follows_the_menu() {
    let mut h = open();
    search(&mut h, "align left");
    assert!(has_row(&h, "Object › Align › Align Left"));
}

// --------------------------------------------------------------------- keys

#[test]
fn moving_with_the_arrows() {
    let mut h = fleet();
    open_palette(&mut h);
    // Empty query: the textures first, the first one highlighted.
    assert!(highlighted(&row(
        &h,
        &format!("{TRUCK} › Main textures › Standard cab")
    )));
    press(&mut h, Key::ArrowDown);
    press(&mut h, Key::ArrowDown);
    press(&mut h, Key::Enter);
    assert!(!is_open(&h));
    assert_eq!(ws(&h).project.active_surface, 2);
    // Up stops at the first result.
    open_palette(&mut h);
    press(&mut h, Key::ArrowUp);
    press(&mut h, Key::Enter);
    assert_eq!(ws(&h).project.active_surface, 0);
    // Page Down moves by a page.
    open_palette(&mut h);
    press(&mut h, Key::PageDown);
    let first = row(&h, &format!("{TRUCK} › Main textures › Standard cab"));
    assert!(!highlighted(&first));
}

#[test]
fn arrows_dont_nudge() {
    let mut h = open();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    let before = ws(&h).project.surface().get(id).unwrap().frame;
    open_palette(&mut h);
    press(&mut h, Key::ArrowDown);
    press(&mut h, Key::ArrowRight);
    assert_eq!(ws(&h).project.surface().get(id).unwrap().frame, before);
    assert!(is_open(&h));
}

#[test]
fn letters_type_in_the_field() {
    let mut h = open();
    open_palette(&mut h);
    type_keys(&mut h, "r");
    assert_eq!(palette(&h).unwrap().query, "r");
    assert_eq!(ws(&h).tool, Tool::Select);
    // Space types too, Tab keeps the focus.
    type_keys(&mut h, " ");
    assert_eq!(ws(&h).tool, Tool::Select);
    press(&mut h, Key::Tab);
    assert!(field_focused(&h));
    assert!(!h.state().prefs.layout.panels_hidden);
}

#[test]
fn escape_keeps_the_selection() {
    let mut h = fleet();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    ws_mut(&mut h).tool = Tool::Rectangle;
    let active = ws(&h).project.active_surface;
    search(&mut h, "chassis");
    press(&mut h, Key::Escape);
    assert!(!is_open(&h));
    assert_eq!(ws(&h).selection, [id]);
    assert_eq!(ws(&h).tool, Tool::Rectangle);
    assert_eq!(ws(&h).project.active_surface, active);
}

#[test]
fn save_while_the_palette_is_open() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = open();
    rect(&mut h);
    h.state_mut().dialogs = Box::new(tp_app::file_dialogs::ScriptedDialogs {
        save: [dir.path().join("ace")].into(),
        ..Default::default()
    });
    open_palette(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::S);
    h.run();
    assert!(!is_open(&h));
    for _ in 0..400 {
        h.step();
        if ws(&h).saving.is_none() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(dir.path().join("ace.truckpaint").is_file());
}

// ----------------------------------------------------------------- choosing

#[test]
fn flip_from_the_palette() {
    let mut h = open();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    search(&mut h, "flip hor");
    press(&mut h, Key::Enter);
    assert!(!is_open(&h));
    assert_eq!(ws(&h).history.undo_label(), Some("op-flip-horizontal"));
    h.get_by_label("Edit").click();
    h.run();
    assert!(h.query_by_label("Undo Flip Horizontal").is_some());
}

/// command-system: Same command from the palette.
#[test]
fn same_command_from_the_palette_and_the_menu() {
    let mut h = open();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    search(&mut h, "flip horizontal");
    press(&mut h, Key::Enter);
    let from_palette = ws(&h).project.surface().get(id).unwrap().frame;
    let label = ws(&h).history.undo_label();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    h.get_by_label("Object").click();
    h.run();
    common::last(&h, "Flip Horizontal").click();
    h.run();
    assert_eq!(
        ws(&h).project.surface().get(id).unwrap().frame,
        from_palette
    );
    assert_eq!(ws(&h).history.undo_label(), label);
}

/// command-system: Disabled command in the palette.
#[test]
fn disabled_command_in_the_palette() {
    let mut h = open();
    rect(&mut h);
    select(&mut h, &[]);
    let objects = ws(&h).project.surface().objects.len();
    search(&mut h, "delete");
    let name = format!("Edit › Delete, {}", shortcut_text(&h, CommandId::Delete));
    row(&h, &name).hover();
    settle(&mut h);
    assert!(highlighted(&row(&h, &name)));
    press(&mut h, Key::Enter);
    assert!(is_open(&h));
    assert_eq!(ws(&h).project.surface().objects.len(), objects);
}

#[test]
fn a_disabled_command() {
    let mut h = open();
    search(&mut h, "group");
    let name = format!("Object › Group, {}", shortcut_text(&h, CommandId::Group));
    let node = row(&h, &name);
    assert!(node.accesskit_node().is_disabled());
    assert!(
        node.accesskit_node()
            .label()
            .unwrap()
            .ends_with("unavailable: Select one or more objects first.")
    );
    node.hover();
    settle(&mut h);
    assert!(
        h.query_by_label("Select one or more objects first.")
            .is_some()
    );
    press(&mut h, Key::Enter);
    assert!(is_open(&h));
    assert!(ws(&h).project.surface().objects.is_empty());
}

#[test]
fn opening_a_texture_from_the_project_space() {
    let mut h = fleet();
    common::show_space(&mut h, Space::Project);
    h.run();
    search(&mut h, "mudflaps");
    press(&mut h, Key::Enter);
    assert!(!is_open(&h));
    assert_eq!(ws(&h).space, Space::Workshop);
    assert_eq!(ws(&h).project.active_surface, surface(&h, "Mudflaps"));
    common::show_tab(&mut h, LeftTab::Textures);
    let label = format!("Texture {TRAILER} › Mudflaps,");
    let tree = h.get_by_label_contains(&label);
    assert_eq!(tree.accesskit_node().toggled(), Some(Toggled::True));
}

#[test]
fn command_that_opens_a_dialog() {
    let mut h = open();
    search(&mut h, "export mod");
    // Its path names the File menu, where Export Mod… is.
    assert!(has_row(&h, "File › Export Mod…"));
    press(&mut h, Key::Enter);
    assert!(matches!(h.state().modal, Some(Modal::ExportMod(_))));
}

#[test]
fn space_command() {
    let mut h = fleet();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    let active = ws(&h).project.active_surface;
    search(&mut h, "brand");
    assert!(rows(&h)[0].starts_with("View › Brand"), "{:?}", rows(&h));
    press(&mut h, Key::Enter);
    assert_eq!(ws(&h).space, Space::Brand);
    assert_eq!(ws(&h).selection, [id]);
    assert_eq!(ws(&h).project.active_surface, active);
}

/// vehicle-projects: Switching from the palette.
#[test]
fn switching_textures_from_the_palette() {
    let mut h = fleet();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    let standard = surface(&h, "Standard cab");
    ws_mut(&mut h).set_active_surface(standard);
    select(&mut h, &[id]);
    {
        let ws = ws_mut(&mut h);
        let view = ws.viewport.as_mut().unwrap();
        view.set_zoom_centered(2.0);
        view.center.x += 40.0;
        view.center.y += 30.0;
    }
    h.run();
    let view = ws(&h).viewport.unwrap();
    search(&mut h, "chassis");
    press(&mut h, Key::Enter);
    assert_eq!(ws(&h).project.active_surface, surface(&h, "Chassis"));
    assert!(ws(&h).selection.is_empty());
    search(&mut h, "standard cab");
    press(&mut h, Key::Enter);
    assert_eq!(ws(&h).project.active_surface, standard);
    let back = ws(&h).viewport.unwrap();
    assert_eq!(back.zoom, view.zoom);
    assert_eq!(back.center, view.center);
}

#[test]
fn recent_first() {
    let mut h = fleet();
    let id = rect(&mut h);
    select(&mut h, &[id]);
    search(&mut h, "flip hor");
    press(&mut h, Key::Enter);
    search(&mut h, "show grid");
    press(&mut h, Key::Enter);
    assert!(h.state().prefs.view_aids.grid);
    open_palette(&mut h);
    let y = |h: &H, label: &str| h.get_by_label(label).rect().top();
    let grid = row(&h, "View › Show Grid").rect().top();
    let flip = row(&h, "Object › Flip Horizontal").rect().top();
    let textures = heading(&h, "Textures");
    assert!(y(&h, "Recent") < grid && grid < flip && flip < textures);
    // Not repeated under Commands.
    assert_eq!(
        h.query_all_by(|n| n.label().is_some_and(|l| l.starts_with("View › Show Grid")))
            .filter(|n| n.accesskit_node().toggled().is_some())
            .count(),
        1
    );
    assert_eq!(
        h.state().palette_recent,
        [CommandId::ShowGrid, CommandId::Flip(FlipAxis::Horizontal)]
    );
}

#[test]
fn first_use() {
    let mut h = fleet();
    // A command run from its shortcut is not recent.
    h.key_press_modifiers(Modifiers::COMMAND, Key::Quote);
    h.run();
    open_palette(&mut h);
    assert!(h.query_by_label("Recent").is_none());
    let commands = h.get_by_label("Commands").rect();
    // The heading, not the left panel's tab.
    let textures = heading(&h, "Textures");
    assert!(textures < commands.top());
}

// ------------------------------------------------------------ menu and keys

/// workspace-layout: Command Palette from the View menu on the home screen.
#[test]
fn command_palette_from_the_view_menu_on_the_home_screen() {
    let mut h = common::harness();
    h.run();
    h.get_by_label("View").click();
    h.run();
    h.get_by_label("Command Palette…").click();
    h.run();
    assert!(is_open(&h));
}

/// command-system: Command palette in the shortcuts window.
#[test]
fn command_palette_in_the_shortcuts_window() {
    let shortcut = CommandId::CommandPalette.shortcut().unwrap();
    assert_eq!(format_shortcut(&shortcut, true, true), "⌘K");
    assert_eq!(format_shortcut(&shortcut, false, false), "Ctrl+K");
    let mut h = open();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Slash);
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::KeyboardShortcuts)));
    assert!(h.query_by_label("Command Palette…").is_some());
    let key = shortcut_text(&h, CommandId::CommandPalette);
    assert!(h.query_by_label(&key).is_some());
}

// -------------------------------------------------------- Search textures

/// vehicle-projects: Search textures field.
#[test]
fn search_textures_field() {
    let mut h = fleet();
    common::show_tab(&mut h, LeftTab::Textures);
    let field = h.get_by_label("Search textures");
    let label = format!("Texture {TRUCK} › Standard cab,");
    let tree = h.get_by_label_contains(&label);
    assert!(field.rect().bottom() < tree.rect().top());
    field.click();
    h.run();
    let p = palette(&h).expect("palette open");
    assert!(p.textures_only && p.query.is_empty());
    assert!(
        h.query_by_role_and_label(Role::TextInput, "Search textures")
            .is_some()
    );
    let rows = rows(&h);
    assert!(!rows.is_empty());
    assert!(
        rows.iter()
            .all(|r| r.starts_with(TRUCK) || r.starts_with(TRAILER)),
        "{rows:?}"
    );
}

/// command-palette: Opening from the Textures tab.
#[test]
fn opening_from_the_textures_tab() {
    let mut h = fleet();
    common::show_tab(&mut h, LeftTab::Textures);
    h.get_by_label("Search textures").click();
    h.run();
    type_keys(&mut h, "ch");
    let rows = rows(&h);
    assert!(rows.iter().any(|r| r.contains("› Chassis")), "{rows:?}");
    assert!(
        rows.iter()
            .all(|r| r.starts_with(TRUCK) || r.starts_with(TRAILER)),
        "{rows:?}"
    );
}

/// command-palette: Widening to commands.
#[test]
fn widening_to_commands() {
    let mut h = fleet();
    common::show_tab(&mut h, LeftTab::Textures);
    h.get_by_label("Search textures").click();
    h.run();
    type_keys(&mut h, "grid");
    assert!(!has_row(&h, "View › Show Grid"));
    for _ in 0..4 {
        press(&mut h, Key::Backspace);
    }
    assert!(palette(&h).unwrap().textures_only);
    press(&mut h, Key::Backspace);
    assert!(!palette(&h).unwrap().textures_only);
    assert!(
        h.query_by_role_and_label(Role::TextInput, PLACEHOLDER)
            .is_some()
    );
    type_keys(&mut h, "grid");
    assert!(has_row(&h, "View › Show Grid"));
}

#[test]
fn typing_into_the_search_textures_field() {
    let mut h = fleet();
    common::show_tab(&mut h, LeftTab::Textures);
    h.get_by_label("Search textures").focus();
    h.run();
    type_keys(&mut h, "c");
    let p = palette(&h).expect("palette open");
    assert!(p.textures_only);
    assert_eq!(p.query, "c");
    // The letter typed no shortcut (C is no tool; D would be Default
    // Colors).
    assert_eq!(ws(&h).tool, Tool::Select);
    type_keys(&mut h, "h");
    assert_eq!(palette(&h).unwrap().query, "ch");
}

#[test]
fn clicking_a_row_chooses_it() {
    let mut h = open();
    search(&mut h, "brand");
    row(&h, "View › Brand").click();
    settle(&mut h);
    assert!(!is_open(&h));
    assert_eq!(ws(&h).space, Space::Brand);
}
