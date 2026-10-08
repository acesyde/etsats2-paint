//! Headless tests for the project's game versions: the sidebar field, the
//! versions the fleet supports, and their checks in Export Mod.

mod common;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tp_app::AppState;
use tp_app::state::Modal;
use tp_app::workspace::{SaveState, Workspace};

type H = Harness<'static, AppState>;

const FIELD: &str = "Game versions";

fn open() -> H {
    let mut h = common::harness();
    common::create_project(&mut h);
    let snapshot = ws(&h).snapshot();
    ws_mut(&mut h).saved = Some(snapshot);
    h.run();
    h
}

fn ws(h: &H) -> &Workspace {
    h.state().workspace().expect("project open")
}

fn ws_mut(h: &mut H) -> &mut Workspace {
    h.state_mut().workspace_mut().expect("project open")
}

/// Focuses the field, replaces its text with `text`, then presses `key`.
fn type_into(h: &mut H, text: &str, key: Key) {
    h.get_by_role_and_label(Role::TextInput, FIELD).focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.get_by_role_and_label(Role::TextInput, FIELD)
        .type_text(text);
    h.run();
    h.key_press(key);
    h.run();
}

/// Adds a vehicle that only supports game versions before 1.55.
fn add_old_vehicle(h: &mut H) {
    let ws = ws_mut(h);
    let mut old = ws.project.vehicles[0].clone();
    old.name = "Old Hauler".into();
    old.package_id = "custom.old.hauler".into();
    old.game_data.as_mut().unwrap().versions = "<1.55".into();
    ws.project.vehicles.push(old);
    h.run();
}

#[test]
fn editing_the_game_versions_is_one_undo_step() {
    let mut h = open();
    assert!(ws(&h).project.game_versions.is_empty());
    type_into(&mut h, "1.56.*, 1.57.*", Key::Enter);
    assert_eq!(ws(&h).project.game_versions, ["1.56.*", "1.57.*"]);
    assert_eq!(ws(&h).save_state(), SaveState::Unsaved);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-edit-game-versions"));
    ws_mut(&mut h).undo();
    h.run();
    assert!(ws(&h).project.game_versions.is_empty());
}

#[test]
fn escape_keeps_the_game_versions() {
    let mut h = open();
    type_into(&mut h, "1.56.*", Key::Escape);
    assert!(ws(&h).project.game_versions.is_empty());
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
}

#[test]
fn the_fleet_supported_versions_are_shown_under_the_field() {
    let mut h = open();
    assert!(
        h.query_by_label("Supported by every vehicle: >=1.56")
            .is_some()
    );
    add_old_vehicle(&mut h);
    assert!(
        h.query_by_label("No game version is supported by every vehicle")
            .is_some()
    );
    // Name and Version stay read only.
    let disabled = h
        .query_all(
            egui_kittest::kittest::By::new()
                .predicate(|n| n.role() == Role::TextInput && n.is_disabled()),
        )
        .count();
    assert_eq!(disabled, 2);
}

/// Opens Export Mod and returns the problems it lists.
fn export_problems(h: &mut H) -> Vec<String> {
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::E);
    for _ in 0..4 {
        h.step();
    }
    assert!(matches!(h.state().modal, Some(Modal::ExportMod(_))));
    let project = &ws(h).project;
    let settings = project.mod_settings.clone();
    tp_app::mod_export::problems_with(project, &settings)
        .iter()
        .map(tp_app::ui::mod_export_dialog::problem_message)
        .collect()
}

#[test]
fn export_mod_checks_the_game_versions() {
    let mut h = open();
    type_into(&mut h, "1.55.*, 1.56.*", Key::Enter);
    let problems = export_problems(&mut h);
    assert_eq!(
        problems,
        ["TruckPaint Sample Truck (>=1.56) doesn't support game version 1.55.*."]
    );
    assert!(h.query_by_label(&problems[0]).is_some());
    assert!(h.get_by_label("Export…").accesskit_node().is_disabled());
}

#[test]
fn export_mod_refuses_vehicles_without_a_common_version() {
    let mut h = open();
    add_old_vehicle(&mut h);
    let problems = export_problems(&mut h);
    assert!(
        problems.iter().any(|p| p
            == "TruckPaint Sample Truck (>=1.56) and Old Hauler (<1.55) have no game version in common: update or remove one of them."),
        "{problems:?}"
    );
}
