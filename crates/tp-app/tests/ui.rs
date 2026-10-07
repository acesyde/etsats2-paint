//! Headless UI tests for the app-shell spec scenarios.

mod common;

use std::path::PathBuf;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tp_app::AppState;
use tp_app::layout::{PanelKind, ViewMode};
use tp_app::prefs::{Prefs, RecentProject};
use tp_app::state::{Modal, Screen};
use tp_app::tool::Tool;
use tp_ui::tokens::size;

use common::{create_project, harness, harness_with};

fn tool(h: &Harness<'static, AppState>) -> Tool {
    h.state().workspace().expect("workspace").tool
}

fn open_menu(h: &mut Harness<'static, AppState>, title: &str) {
    h.get_by_label(title).click();
    h.run();
}

// --- start-screen ---------------------------------------------------------

#[test]
fn home_screen_on_launch_with_empty_recent_state() {
    let h = harness();
    assert!(matches!(h.state().screen, Screen::Home));
    h.get_by_label("New Project");
    h.get_by_label("Open Project…");
    h.get_by_label("No recent projects");
}

#[test]
fn closing_project_returns_home() {
    let mut h = harness();
    create_project(&mut h);
    open_menu(&mut h, "File");
    h.get_by_label("Close").click();
    h.run();
    // A new project has unsaved changes: the prompt comes first.
    h.get_by_label("Don't Save").click();
    h.run();
    assert!(matches!(h.state().screen, Screen::Home));
}

#[test]
fn missing_recent_file_can_be_removed() {
    let prefs = Prefs {
        recent: vec![RecentProject {
            name: "Gone".into(),
            path: PathBuf::from("/definitely/not/here.truckpaint"),
            last_opened: 0,
        }],
        ..Prefs::default()
    };
    let mut h = harness_with(prefs);
    h.get_by_label("Gone (file not found)");
    h.get_by_label("Remove from list").click();
    h.run();
    assert!(h.state().prefs.recent.is_empty());
    h.get_by_label("No recent projects");
}

#[test]
fn cancelled_open_dialog_does_nothing() {
    let mut h = harness();
    h.get_by_label("Open Project…").click();
    h.run();
    assert!(matches!(h.state().screen, Screen::Home));
    assert!(h.state().modal.is_none());
}

#[test]
fn new_project_defaults_to_4096_and_untitled() {
    let mut h = harness();
    create_project(&mut h);
    let ws = h.state().workspace().unwrap();
    assert_eq!(ws.project.name, "Untitled");
    assert_eq!(ws.project.resolution.side(), 4096);
    assert_eq!(h.state().title(), "Untitled — TruckPaint");
}

#[test]
fn new_project_with_name_and_resolution_via_keyboard() {
    let mut h = harness();
    h.get_by_label("New Project").click();
    h.run();
    // Enter on the Vehicle step (Blank texture) goes to the next step.
    h.key_press(Key::Enter);
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Project name")
        .type_text("ACE Logistics");
    h.get_by_label("2048 × 2048").click();
    h.run();
    h.key_press(Key::Enter);
    h.run();
    let ws = h.state().workspace().expect("created with Enter");
    assert_eq!(ws.project.name, "ACE Logistics");
    assert_eq!(ws.project.resolution.side(), 2048);
}

#[test]
fn escape_cancels_new_project() {
    let mut h = harness();
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_label("Step 1 of 2 — Vehicle");
    h.key_press(Key::Escape);
    h.run();
    assert!(h.state().modal.is_none());
    assert!(matches!(h.state().screen, Screen::Home));
}

#[test]
fn new_project_dialog_tab_moves_focus() {
    let mut h = harness();
    h.get_by_label("New Project").click();
    h.run();
    h.get_by_label("Next").click();
    h.run();
    assert!(
        h.get_by_role_and_label(Role::TextInput, "Project name")
            .is_focused(),
        "name field focused on open"
    );
    h.key_press(Key::Tab);
    h.run();
    assert!(
        !h.get_by_role_and_label(Role::TextInput, "Project name")
            .is_focused()
    );
}

// --- workspace-layout -----------------------------------------------------

#[test]
fn menus_are_in_order() {
    let mut h = harness();
    create_project(&mut h);
    let menus: Vec<f32> = tp_app::ui::menu_bar::MENUS
        .iter()
        .map(|title| {
            // Menu titles sit in the top bar (a panel may share the label).
            h.get_all_by_label(&tp_i18n::tr(title))
                .map(|n| n.rect())
                .min_by(|a, b| a.top().total_cmp(&b.top()))
                .unwrap()
                .left()
        })
        .collect();
    assert!(menus.windows(2).all(|w| w[0] < w[1]), "{menus:?}");
}

#[test]
fn disabled_menu_item_does_nothing() {
    let mut h = harness();
    create_project(&mut h);
    open_menu(&mut h, "Edit");
    h.get_by_label("Undo").click();
    h.run();
    assert!(h.state().has_project());
    assert!(h.state().modal.is_none());
}

#[test]
fn selection_is_default_and_clicking_rectangle_activates_it() {
    let mut h = harness();
    create_project(&mut h);
    assert_eq!(tool(&h), Tool::Select);
    h.get_by_label("Rectangle").click();
    h.run();
    assert_eq!(tool(&h), Tool::Rectangle);
}

#[test]
fn tool_buttons_have_minimum_hit_area() {
    let mut h = harness();
    create_project(&mut h);
    let rect = h.get_by_label("Ellipse").rect();
    assert!(rect.width() >= size::HIT_MIN && rect.height() >= size::HIT_MIN);
    let close = h.get_by_label("Close Layers").rect();
    assert!(close.width() >= size::HIT_MIN && close.height() >= size::HIT_MIN);
}

#[test]
fn artboard_stays_visible_when_resizing() {
    let mut h = harness();
    create_project(&mut h);
    for size in [
        Vec2::new(1440.0, 900.0),
        Vec2::new(1000.0, 640.0),
        Vec2::new(1800.0, 700.0),
    ] {
        h.set_size(size);
        h.run();
        let canvas = h.get_by_label("Canvas").rect();
        let artboard = h.state().workspace().unwrap().artboard_rect(1.0).unwrap();
        assert!(canvas.contains_rect(artboard), "{size:?}");
        assert!((artboard.width() - artboard.height()).abs() < 0.5);
    }
}

#[test]
fn collapsing_a_panel() {
    let mut h = harness();
    create_project(&mut h);
    h.get_by_label("No layers yet");
    h.get_by_label("Layers").click();
    h.run();
    assert!(h.state().prefs.layout.slot(PanelKind::Layers).collapsed);
    assert!(h.query_by_label("No layers yet").is_none());
}

#[test]
fn closing_and_reopening_panel_keeps_position() {
    let mut h = harness();
    create_project(&mut h);
    let index = |h: &Harness<'static, AppState>| {
        h.state()
            .prefs
            .layout
            .panels
            .iter()
            .position(|s| s.kind == PanelKind::Colors)
    };
    let before = index(&h);
    h.get_by_label("Close Colors").click();
    h.run();
    assert!(!h.state().prefs.layout.is_open(PanelKind::Colors));
    assert!(h.query_by_label("Close Colors").is_none());

    open_menu(&mut h, "View");
    h.get_by_label("Colors").click();
    h.run();
    assert!(h.state().prefs.layout.is_open(PanelKind::Colors));
    assert_eq!(index(&h), before);
}

#[test]
fn panel_column_width_is_clamped() {
    let mut prefs = Prefs::default();
    prefs.layout.column_width = 5_000.0;
    let mut h = harness_with(prefs);
    create_project(&mut h);
    let width = h.state().prefs.layout.column_width;
    assert!(width <= size::PANEL_COLUMN_MAX, "{width}");
}

#[test]
fn panel_header_context_menu() {
    let mut h = harness();
    create_project(&mut h);
    h.get_by_label("Layers").click_secondary();
    h.run();
    h.get_by_label("Collapse");
    h.get_by_label("Close Panel").click();
    h.run();
    assert!(!h.state().prefs.layout.is_open(PanelKind::Layers));
}

#[test]
fn view_modes_show_expected_regions() {
    let mut h = harness();
    create_project(&mut h);
    h.get_by_label("Canvas");
    assert!(h.query_by_label("3D preview coming soon").is_none());

    h.get_by_label("Split").click();
    h.run();
    assert_eq!(h.state().prefs.layout.view_mode, ViewMode::Split);
    let canvas = h.get_by_label("Canvas").rect();
    let preview = h.get_by_label("3D preview coming soon").rect();
    assert!(
        canvas.center().x < preview.center().x,
        "canvas left of preview"
    );

    h.get_by_label("3D").click();
    h.run();
    assert!(h.query_by_label("Canvas").is_none());
    h.get_by_label("3D preview coming soon");

    // Hiding the preview returns to the canvas only.
    h.state_mut().prefs.layout.view_mode = ViewMode::Split;
    h.run();
    h.get_by_label("Hide 3D Preview").click();
    h.run();
    assert_eq!(h.state().prefs.layout.view_mode, ViewMode::TwoD);
}

#[test]
fn status_bar_shows_unsaved_for_new_project() {
    let mut h = harness();
    create_project(&mut h);
    h.get_by_label("Unsaved changes");
    h.get_by_label_contains("Zoom ");
}

#[test]
fn reset_workspace_restores_panels() {
    let mut h = harness();
    create_project(&mut h);
    h.get_by_label("Close Assets").click();
    h.run();
    h.get_by_label("Layers").click();
    h.run();
    open_menu(&mut h, "View");
    h.get_by_label("Reset Workspace").click();
    h.run();
    let layout = &h.state().prefs.layout;
    assert!(layout.panels.iter().all(|s| s.open && !s.collapsed));
}

// --- command-system -------------------------------------------------------

#[test]
fn menu_and_shortcut_trigger_same_command() {
    let mut h = harness();
    create_project(&mut h);
    open_menu(&mut h, "View");
    h.get_by_role_and_label(Role::CheckBox, "Layers").click();
    h.run();
    assert!(!h.state().prefs.layout.is_open(PanelKind::Layers));
    h.key_press(Key::F7);
    h.run();
    assert!(h.state().prefs.layout.is_open(PanelKind::Layers));
}

#[test]
fn tool_shortcuts_and_temporary_hand() {
    let mut h = harness();
    create_project(&mut h);
    h.key_press(Key::E);
    h.run();
    assert_eq!(tool(&h), Tool::Ellipse);
    h.key_press_modifiers(Modifiers::SHIFT, Key::I);
    h.run();
    assert_eq!(tool(&h), Tool::Image);
    h.key_press(Key::I);
    h.run();
    assert_eq!(tool(&h), Tool::Eyedropper);

    h.key_press(Key::R);
    h.run();
    h.key_down(Key::Space);
    h.run();
    assert_eq!(tool(&h), Tool::Hand);
    h.key_up(Key::Space);
    h.run();
    assert_eq!(tool(&h), Tool::Rectangle);
}

#[test]
fn plain_tool_key_does_not_fire_with_command_modifier() {
    let mut h = harness();
    create_project(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::R);
    h.run();
    assert_eq!(tool(&h), Tool::Select);
}

#[test]
fn typing_in_text_field_does_not_switch_tools() {
    let mut h = Harness::builder().with_size(common::SIZE).build_ui_state(
        |ui, (state, text, frames): &mut (AppState, String, u32)| {
            state.show(ui);
            // The theme's fonts are usable from the second frame on.
            if *frames > 0 {
                egui::Window::new("Scratch").show(ui.ctx(), |ui| {
                    ui.add(egui::TextEdit::singleline(text).hint_text("scratch field"))
                        .widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Scratch")
                        });
                });
            }
            *frames += 1;
        },
        (
            AppState::with_prefs(Prefs::default(), None),
            String::new(),
            0,
        ),
    );
    h.run();
    h.state_mut().0.open_project(tp_core::Project::new(
        "Test",
        tp_core::TextureResolution::R2048,
    ));
    h.run();
    h.get_by_role_and_label(Role::TextInput, "Scratch").focus();
    h.run();
    // A real keyboard sends both the key event and the text event.
    h.key_press(Key::R);
    h.get_by_role_and_label(Role::TextInput, "Scratch")
        .type_text("R");
    h.run();
    assert_eq!(h.state().1, "R");
    assert_eq!(h.state().0.workspace().unwrap().tool, Tool::Select);
}

#[test]
fn disabled_command_shortcut_does_nothing() {
    let mut h = harness();
    create_project(&mut h);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert!(h.state().has_project());
    assert!(h.state().modal.is_none());
}

// --- app-preferences / ui scaling -----------------------------------------

#[test]
fn preferences_reset_to_defaults() {
    let prefs = Prefs {
        ui_scale: 1.5,
        text_scale: 1.25,
        ..Prefs::default()
    };
    let mut h = harness_with(prefs);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    h.run();
    assert!(matches!(h.state().modal, Some(Modal::Preferences)));
    h.get_by_label("Reset to defaults").click();
    h.run();
    assert_eq!(h.state().prefs.ui_scale, 1.0);
    assert_eq!(h.state().prefs.text_scale, 1.0);
}

#[test]
fn ui_scale_applies_without_restart() {
    let mut h = harness();
    h.run();
    let ctx = h.ctx.clone();
    assert!((ctx.zoom_factor() - 1.0).abs() < 1e-4);
    let body_before = ctx.global_style().text_styles[&egui::TextStyle::Body].size;

    h.state_mut().prefs.ui_scale = 1.5;
    h.state_mut().prefs.text_scale = 1.2;
    h.run();
    assert!((ctx.zoom_factor() - 1.5).abs() < 1e-4);
    let body_after = ctx.global_style().text_styles[&egui::TextStyle::Body].size;
    assert!((body_after - body_before * 1.2).abs() < 1e-3);
}

#[test]
fn macos_menus_show_symbol_shortcuts() {
    let mut h = Harness::builder()
        .with_size(common::SIZE)
        .with_os(egui::os::OperatingSystem::Mac)
        .build_ui_state(
            |ui, state: &mut AppState| state.show(ui),
            AppState::with_prefs(Prefs::default(), None),
        );
    create_project(&mut h);
    open_menu(&mut h, "Edit");
    let formatter = tp_app::commands::ShortcutFormatter::new(&h.ctx);
    let undo = tp_app::commands::CommandId::Undo;
    let redo = tp_app::commands::CommandId::Redo;
    assert_eq!(formatter.command(undo).as_deref(), Some("⌘Z"));
    assert_eq!(formatter.command(redo).as_deref(), Some("⇧⌘Z"));
}
