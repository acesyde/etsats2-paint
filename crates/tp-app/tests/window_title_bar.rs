//! Headless tests for the top of the window: the title bar mode of each
//! platform, the macOS title strip holding the top bar, and the menu row.

mod common;

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton, Pos2, ViewportCommand};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::layout::Space;
use tp_app::title_bar::TitleBarMode;
use tp_ui::tokens::size;

type H = Harness<'static, AppState>;

/// The sample project in the Workshop, named "ACE Logistics", on macOS
/// (`mac`) or another system.
fn open(mac: bool) -> H {
    let mut h = common::harness();
    h.state_mut().macos = mac;
    common::create_project(&mut h);
    h.state_mut().workspace_mut().unwrap().project.name = "ACE Logistics".into();
    h.run();
    h
}

/// Presses at `from`, drags to `to` and releases, one frame per event;
/// returns the window commands sent meanwhile.
fn drag(h: &mut H, from: Pos2, to: Pos2) -> Vec<ViewportCommand> {
    let mut sent = Vec::new();
    let mut step = |h: &mut H, event| {
        h.event(event);
        h.step();
        sent.extend(common::viewport_commands(h));
    };
    step(h, Event::PointerMoved(from));
    step(
        h,
        Event::PointerButton {
            pos: from,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
    );
    for i in 1..=4 {
        step(
            h,
            Event::PointerMoved(from + (to - from) * (i as f32 / 4.0)),
        );
    }
    step(
        h,
        Event::PointerButton {
            pos: to,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    );
    h.run();
    sent
}

#[test]
fn the_mode_follows_the_platform_the_preference_and_the_environment() {
    let mut state = AppState::with_prefs(Default::default(), None);
    state.macos = true;
    state.prefs.system_title_bar = true;
    assert_eq!(state.title_bar_mode(), TitleBarMode::MacNative);
    state.macos = false;
    assert_eq!(state.title_bar_mode(), TitleBarMode::System);
    state.prefs.system_title_bar = false;
    assert_eq!(state.title_bar_mode(), TitleBarMode::Drawn);
    state.force_system_title_bar = true;
    assert_eq!(state.title_bar_mode(), TitleBarMode::System);
}

/// window-title-bar: Moving the window from the bar.
#[test]
fn moving_the_window_from_the_bar() {
    let mut h = open(true);
    let brand = h.get_by_role_and_label(Role::RadioButton, "Brand").rect();
    let export = h.get_by_role_and_label(Role::Button, "Export…").rect();
    assert!(brand.right() < export.left() - 40.0);
    let y = size::TOP_BAR_HEIGHT / 2.0;
    let from = Pos2::new((brand.right() + export.left()) / 2.0, y);
    let sent = drag(&mut h, from, from + egui::vec2(60.0, 30.0));
    assert!(sent.contains(&ViewportCommand::StartDrag), "{sent:?}");
    assert_eq!(h.state().workspace().unwrap().space, Space::Workshop);
}

#[test]
fn the_traffic_lights_keep_their_place() {
    let h = open(true);
    let name = h.get_by_label("ACE Logistics").rect();
    assert!(name.left() >= tp_app::title_bar::TRAFFIC_LIGHTS_WIDTH);
    assert!(name.bottom() <= size::TOP_BAR_HEIGHT);
}

#[test]
fn no_menu_row_once_the_menus_are_in_the_menu_bar() {
    let mut h = open(true);
    // Until the system menu is installed, the menus are a row under the
    // title strip.
    let file = h.get_by_label("File").rect();
    assert!(file.top() >= size::TOP_BAR_HEIGHT, "{file:?}");
    h.state_mut().native_menu_installed = true;
    h.run();
    assert!(h.query_by_label("File").is_none());
    assert!(h.query_by_label("ACE Logistics").is_some());
    // Elsewhere the flag changes nothing.
    h.state_mut().macos = false;
    h.run();
    assert!(h.query_by_label("File").is_some());
}

#[test]
fn the_name_is_shown_with_the_system_title_bar_only() {
    let mut h = open(false);
    // The drawn title bar: the name is the window's title.
    assert!(h.query_by_label("ACE Logistics").is_none());
    h.state_mut().prefs.system_title_bar = true;
    h.run();
    let file = h.get_by_label("File").rect();
    let name = h.get_by_label("ACE Logistics").rect();
    assert!(file.bottom() <= name.top(), "menu row, then the top bar");
}

#[test]
fn home_screen_title_strip_on_macos() {
    let mut h = common::harness();
    h.state_mut().macos = true;
    h.run();
    let file = h.get_by_label("File").rect();
    assert!(file.top() >= size::TOP_BAR_HEIGHT, "{file:?}");
    let sent = drag(
        &mut h,
        Pos2::new(600.0, size::TOP_BAR_HEIGHT / 2.0),
        Pos2::new(700.0, 60.0),
    );
    assert!(sent.contains(&ViewportCommand::StartDrag), "{sent:?}");
}

/// Double-clicks at `at` (press and release twice, 0.1 s apart); returns
/// the window commands sent meanwhile.
fn double_click_at(h: &mut H, at: Pos2, start: f64) -> Vec<ViewportCommand> {
    let mut sent = Vec::new();
    h.event(Event::PointerMoved(at));
    for k in 0..2 {
        h.input_mut().time = Some(start + 0.1 * f64::from(k));
        for pressed in [true, false] {
            h.event(Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        h.step();
        sent.extend(common::viewport_commands(h));
    }
    h.input_mut().time = None;
    h.run();
    sent
}

#[test]
fn double_click_on_the_bar_does_the_system_action() {
    let mut h = open(true);
    let export = h.get_by_role_and_label(Role::Button, "Export…").rect();
    let at = Pos2::new(export.left() - 30.0, size::TOP_BAR_HEIGHT / 2.0);
    let sent = double_click_at(&mut h, at, 100.0);
    assert!(sent.contains(&ViewportCommand::Maximized(true)), "{sent:?}");
    // Set to do nothing.
    h.state_mut().double_click = tp_app::title_bar::DoubleClickAction::None;
    let sent = double_click_at(&mut h, at, 200.0);
    assert!(
        !sent
            .iter()
            .any(|c| matches!(c, ViewportCommand::Maximized(_))),
        "{sent:?}"
    );
    assert_eq!(h.state().workspace().unwrap().space, Space::Workshop);
}

// --- The drawn title bar (Windows, Linux) -----------------------------------------

/// The sample project in the Workshop, named "ACE Logistics", with the
/// drawn title bar, in `language`.
fn open_drawn(language: tp_i18n::Language) -> H {
    let mut prefs = tp_app::prefs::Prefs::default();
    prefs.set_language(Some(language));
    let mut h = common::harness_with(prefs);
    common::create_project(&mut h);
    h.state_mut().workspace_mut().unwrap().project.name = "ACE Logistics".into();
    h.run();
    assert_eq!(h.state().title_bar_mode(), TitleBarMode::Drawn);
    h
}

/// Where the harness draws the window's top left corner (it frames the
/// app with an 8 px margin).
const ORIGIN: f32 = 8.0;

/// The node named `label` lies in the 40 px bar (the topmost one when
/// several have that name).
fn in_bar(h: &H, label: &str) -> egui::Rect {
    let rect = h
        .get_all_by_label(label)
        .map(|n| n.rect())
        .min_by(|a, b| a.top().total_cmp(&b.top()))
        .unwrap();
    assert!(
        rect.top() >= ORIGIN && rect.bottom() <= ORIGIN + size::TOP_BAR_HEIGHT,
        "{label}: {rect:?}"
    );
    rect
}

/// Sets whether the window reports itself maximized.
fn set_maximized(h: &mut H, maximized: bool) {
    h.input_mut()
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .maximized = Some(maximized);
    h.run();
}

/// window-title-bar: Bar on Windows.
#[test]
fn bar_on_windows() {
    let h = open_drawn(tp_i18n::Language::French);
    let mark = in_bar(&h, "TruckPaint");
    let mut right = mark.right();
    for menu in [
        "Fichier",
        "Édition",
        "Objet",
        "Calque",
        "Affichage",
        "Véhicule",
        "Aide",
    ] {
        let rect = in_bar(&h, menu);
        assert!(rect.left() >= right, "{menu} after the previous one");
        right = rect.right();
    }
    let project = in_bar(&h, "Projet");
    in_bar(&h, "Atelier");
    let brand = in_bar(&h, "Marque");
    assert!(project.left() >= right);
    // Centered in the bar.
    let center = (project.left() + brand.right()) / 2.0;
    assert!((center - common::SIZE.x / 2.0).abs() <= 1.0, "{center}");
    let export = in_bar(&h, "Exporter…");
    assert!(export.left() >= brand.right());
    let minimize = in_bar(&h, "Réduire");
    let maximize = in_bar(&h, "Agrandir");
    let close = in_bar(&h, "Fermer");
    assert!(minimize.left() >= export.right());
    for control in [minimize, maximize, close] {
        assert_eq!(control.width(), tp_app::ui::title_bar::CONTROL_WIDTH);
        // The bar's bottom border aside.
        assert!(
            control.height() >= size::TOP_BAR_HEIGHT - 1.0,
            "{control:?}"
        );
    }
    assert!(minimize.right() <= maximize.left() && maximize.right() <= close.left());
    assert_eq!(close.right(), common::SIZE.x - ORIGIN);
    // No name in the bar, no menu row.
    assert!(h.query_by_label("ACE Logistics").is_none());
    assert_eq!(h.get_all_by_label("Fichier").count(), 1);
    tp_i18n::set_language(tp_i18n::Language::English);
}

/// The free part of the bar, between the switcher and Export….
fn free_spot(h: &H) -> Pos2 {
    let brand = h.get_by_role_and_label(Role::RadioButton, "Brand").rect();
    let export = h.get_by_role_and_label(Role::Button, "Export…").rect();
    Pos2::new(
        (brand.right() + export.left()) / 2.0,
        size::TOP_BAR_HEIGHT / 2.0,
    )
}

/// window-title-bar: Maximize by double-click.
#[test]
fn maximize_by_double_click() {
    let mut h = open_drawn(tp_i18n::Language::English);
    let at = free_spot(&h);
    let sent = double_click_at(&mut h, at, 100.0);
    assert!(sent.contains(&ViewportCommand::Maximized(true)), "{sent:?}");
    h.get_by_role_and_label(Role::Button, "Maximize");
    set_maximized(&mut h, true);
    // The middle control reads Restore.
    assert!(h.query_by_label("Maximize").is_none());
    in_bar(&h, "Restore");
    let sent = double_click_at(&mut h, at, 200.0);
    assert!(
        sent.contains(&ViewportCommand::Maximized(false)),
        "{sent:?}"
    );
    assert_eq!(h.state().workspace().unwrap().space, Space::Workshop);
    // The controls do the same, and Minimize minimizes.
    h.get_by_label("Restore").click();
    h.step();
    let sent = common::viewport_commands(&h);
    assert!(
        sent.contains(&ViewportCommand::Maximized(false)),
        "{sent:?}"
    );
    h.run();
    h.get_by_label("Minimize").click();
    h.step();
    let sent = common::viewport_commands(&h);
    assert!(sent.contains(&ViewportCommand::Minimized(true)), "{sent:?}");
}

#[test]
fn dragging_the_drawn_bar_moves_the_window() {
    let mut h = open_drawn(tp_i18n::Language::English);
    let from = free_spot(&h);
    let sent = drag(&mut h, from, from + egui::vec2(60.0, 30.0));
    assert!(sent.contains(&ViewportCommand::StartDrag), "{sent:?}");
    assert_eq!(h.state().workspace().unwrap().space, Space::Workshop);
}

#[test]
fn control_tooltips_name_them() {
    let mut h = open_drawn(tp_i18n::Language::German);
    let close = h.get_by_label("Schließen").rect();
    h.event(Event::PointerMoved(close.center()));
    for _ in 0..30 {
        h.step();
    }
    assert!(h.get_all_by_label("Schließen").count() >= 2, "the tooltip");
    tp_i18n::set_language(tp_i18n::Language::English);
}

/// window-title-bar: Close with unsaved changes.
#[test]
fn close_with_unsaved_changes() {
    let mut h = open_drawn(tp_i18n::Language::English);
    assert!(h.state().workspace().unwrap().has_unsaved_changes());
    h.get_by_role_and_label(Role::Button, "Close").click();
    h.step();
    let sent = common::viewport_commands(&h);
    assert!(sent.contains(&ViewportCommand::Close), "{sent:?}");
    // The window system answers with a close request.
    h.input_mut()
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .events
        .push(egui::ViewportEvent::Close);
    h.step();
    assert!(common::viewport_commands(&h).contains(&ViewportCommand::CancelClose));
    assert!(matches!(
        h.state().modal,
        Some(tp_app::state::Modal::UnsavedChanges(_))
    ));
    h.run();
    h.get_by_label("Cancel").click();
    h.run();
    assert!(h.state().modal.is_none());
    assert!(h.state().has_project());
    assert!(h.state().workspace().unwrap().has_unsaved_changes());
}

/// window-title-bar: Home screen bar.
#[test]
fn home_screen_bar() {
    let mut h = common::harness();
    h.run();
    in_bar(&h, "TruckPaint");
    for menu in ["File", "Edit", "Object", "Layer", "View", "Vehicle", "Help"] {
        in_bar(&h, menu);
    }
    for control in ["Minimize", "Maximize", "Close"] {
        in_bar(&h, control);
    }
    assert!(h.query_by_label("Export…").is_none());
    assert!(
        h.query_by_role_and_label(Role::RadioButton, "Workshop")
            .is_none()
    );
    let sent = drag(
        &mut h,
        Pos2::new(900.0, size::TOP_BAR_HEIGHT / 2.0),
        Pos2::new(1000.0, 60.0),
    );
    assert!(sent.contains(&ViewportCommand::StartDrag), "{sent:?}");
}

/// The menus open from the drawn bar, and their commands run.
#[test]
fn menus_open_from_the_drawn_bar() {
    let mut h = open_drawn(tp_i18n::Language::English);
    h.get_by_label("File").click();
    h.run();
    let export = h.get_by_label("Export Mod…").rect();
    assert!(export.top() >= size::TOP_BAR_HEIGHT - 4.0);
    h.get_by_label("Export Mod…").click();
    h.run();
    assert!(matches!(
        h.state().modal,
        Some(tp_app::state::Modal::ExportMod(_))
    ));
}

// --- Resize grips -----------------------------------------------------------------

#[test]
fn resizing_from_the_bottom_right_corner() {
    let mut h = open_drawn(tp_i18n::Language::English);
    let corner = Pos2::new(common::SIZE.x - 3.0, common::SIZE.y - 3.0);
    let sent = drag(&mut h, corner, corner + egui::vec2(-80.0, -60.0));
    assert!(
        sent.contains(&ViewportCommand::BeginResize(
            egui::viewport::ResizeDirection::SouthEast
        )),
        "{sent:?}"
    );
}

/// The pointer's shape over `at`.
fn cursor_at(h: &mut H, at: Pos2) -> egui::CursorIcon {
    h.event(Event::PointerMoved(at));
    h.step();
    h.output().platform_output.cursor_icon
}

#[test]
fn the_edges_show_resize_cursors() {
    let mut h = open_drawn(tp_i18n::Language::English);
    let mid = common::SIZE.y / 2.0;
    assert_eq!(
        cursor_at(&mut h, Pos2::new(2.0, mid)),
        egui::CursorIcon::ResizeHorizontal
    );
    assert_eq!(
        cursor_at(&mut h, Pos2::new(700.0, common::SIZE.y - 2.0)),
        egui::CursorIcon::ResizeVertical
    );
    assert_eq!(
        cursor_at(&mut h, Pos2::new(2.0, 2.0)),
        egui::CursorIcon::ResizeNwSe
    );
}

#[test]
fn no_grip_when_maximized() {
    let mut h = open_drawn(tp_i18n::Language::English);
    set_maximized(&mut h, true);
    let mid = common::SIZE.y / 2.0;
    assert_ne!(
        cursor_at(&mut h, Pos2::new(2.0, mid)),
        egui::CursorIcon::ResizeHorizontal
    );
    let corner = Pos2::new(common::SIZE.x - 3.0, common::SIZE.y - 3.0);
    let sent = drag(&mut h, corner, corner + egui::vec2(-80.0, -60.0));
    assert!(
        !sent
            .iter()
            .any(|c| matches!(c, ViewportCommand::BeginResize(_))),
        "{sent:?}"
    );
}

#[test]
fn no_grip_with_the_system_title_bar() {
    let mut h = open_drawn(tp_i18n::Language::English);
    h.state_mut().prefs.system_title_bar = true;
    h.run();
    let mid = common::SIZE.y / 2.0;
    assert_ne!(
        cursor_at(&mut h, Pos2::new(2.0, mid)),
        egui::CursorIcon::ResizeHorizontal
    );
}

// --- System title bar option ------------------------------------------------------

/// Clicks Use the system title bar in Preferences, then closes them;
/// returns the window commands sent meanwhile.
fn toggle_system_title_bar(h: &mut H) -> Vec<ViewportCommand> {
    h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Comma);
    h.run();
    h.get_by_label("Use the system title bar").click();
    let mut sent = Vec::new();
    for _ in 0..4 {
        h.step();
        sent.extend(common::viewport_commands(h));
    }
    h.get_by_label("Done").click();
    for _ in 0..4 {
        h.step();
        sent.extend(common::viewport_commands(h));
    }
    h.run();
    sent
}

fn decorations(sent: &[ViewportCommand]) -> Vec<bool> {
    sent.iter()
        .filter_map(|c| match c {
            ViewportCommand::Decorations(on) => Some(*on),
            _ => None,
        })
        .collect()
}

/// window-title-bar: Turning the system title bar on, then Back to the
/// drawn bar.
#[test]
fn turning_the_system_title_bar_on_and_off() {
    let mut h = open_drawn(tp_i18n::Language::English);
    assert!(h.state().workspace().unwrap().has_unsaved_changes());
    let sent = toggle_system_title_bar(&mut h);
    assert_eq!(decorations(&sent), [true], "{sent:?}");
    assert!(h.state().prefs.system_title_bar);
    assert_eq!(h.state().title_bar_mode(), TitleBarMode::System);
    // The menu row, then the top bar with the name.
    let file = h.get_by_label("File").rect();
    let name = h.get_by_label("ACE Logistics").rect();
    assert!(file.bottom() <= name.top(), "{file:?} {name:?}");
    assert!(h.query_by_label("Minimize").is_none());
    h.get_by_role_and_label(Role::Button, "Export…");
    assert!(h.state().workspace().unwrap().has_unsaved_changes());
    // Nothing more is sent while the mode stays.
    h.run();
    assert!(decorations(&common::viewport_commands(&h)).is_empty());

    let sent = toggle_system_title_bar(&mut h);
    assert_eq!(decorations(&sent), [false], "{sent:?}");
    assert_eq!(h.state().title_bar_mode(), TitleBarMode::Drawn);
    in_bar(&h, "File");
    in_bar(&h, "Minimize");
    in_bar(&h, "Export…");
    h.get_by_role_and_label(Role::RadioButton, "Workshop");
    assert!(h.query_by_label("ACE Logistics").is_none());
    assert!(h.state().workspace().unwrap().has_unsaved_changes());
}

/// window-title-bar: Forced by the environment.
#[test]
fn forced_by_the_environment() {
    let mut state = AppState::with_prefs(Default::default(), None);
    state.force_system_title_bar = true;
    let mut h = common::builder()
        .with_size(common::SIZE)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state);
    h.run();
    assert_eq!(h.state().title_bar_mode(), TitleBarMode::System);
    // The menus are a row, and the window keeps the system's frame.
    let file = h.get_by_label("File").rect();
    assert!(
        file.bottom() <= ORIGIN + size::MENU_BAR_HEIGHT + 1.0,
        "{file:?}"
    );
    assert!(h.query_by_label("Minimize").is_none());
    assert!(decorations(&common::viewport_commands(&h)).is_empty());
}

/// window-title-bar: Not on macOS; elsewhere the option is offered.
#[test]
fn the_option_is_not_offered_on_macos() {
    for mac in [true, false] {
        let mut h = open(mac);
        h.key_press_modifiers(Modifiers::COMMAND, egui::Key::Comma);
        h.run();
        assert_eq!(
            h.query_by_label("Use the system title bar").is_some(),
            !mac,
            "mac: {mac}"
        );
    }
}

/// Every menu shows the same entries with the system title bar as with
/// the drawn one.
#[test]
fn every_menu_works_with_the_system_title_bar() {
    let entries = |system: bool| {
        let mut h = open_drawn(tp_i18n::Language::English);
        h.state_mut().prefs.system_title_bar = system;
        h.run();
        let mut all = Vec::new();
        for title in ["File", "Edit", "Object", "Layer", "View", "Vehicle", "Help"] {
            h.get_by_label(title).click();
            h.run();
            let mut labels: Vec<String> = h
                .query_all(egui_kittest::kittest::by().predicate(|n| {
                    n.role() == Role::Button && n.label().is_some_and(|l| !l.is_empty())
                }))
                .filter_map(|n| n.accesskit_node().label())
                .collect();
            labels.sort();
            all.push((title, labels));
            h.key_press(egui::Key::Escape);
            h.run();
        }
        all
    };
    let drawn = entries(false);
    let system = entries(true);
    for ((title, d), (_, s)) in drawn.iter().zip(&system) {
        // The window controls are drawn only in the drawn bar.
        let mut d = d.clone();
        for control in ["Minimize", "Maximize", "Close"] {
            let at = d.iter().position(|l| l == control).unwrap();
            d.remove(at);
        }
        assert_eq!(&d, s, "{title}");
    }
    // A command runs from the menu row.
    let mut h = open_drawn(tp_i18n::Language::English);
    h.state_mut().prefs.system_title_bar = true;
    h.run();
    h.get_by_label("File").click();
    h.run();
    h.get_by_label("Export Mod…").click();
    h.run();
    assert!(matches!(
        h.state().modal,
        Some(tp_app::state::Modal::ExportMod(_))
    ));
}

/// workspace-spaces: German at the minimum width.
#[test]
fn german_at_the_minimum_width() {
    let mut h = open_drawn(tp_i18n::Language::German);
    h.set_size(egui::vec2(960.0, 600.0));
    h.run();
    let tr = tp_i18n::tr;
    let mut labels: Vec<String> = tp_app::ui::menu_bar::MENUS.map(tr).into();
    labels.extend(tp_app::ui::workspace::top_bar::space_labels());
    assert_eq!(tr("top-bar-export"), "Exportieren…");
    labels.push(tr("top-bar-export"));
    labels.extend(["window-minimize", "window-maximize", "window-close"].map(tr));
    // Each whole, in the bar, left to right without overlap.
    let mut right = in_bar(&h, "TruckPaint").right();
    for label in &labels {
        let rect = in_bar(&h, label);
        assert!(rect.left() >= right - 0.5, "{label} overlaps: {rect:?}");
        right = rect.right();
    }
    assert!(right <= 960.0 - ORIGIN + 0.5);
    tp_i18n::set_language(tp_i18n::Language::English);
}
