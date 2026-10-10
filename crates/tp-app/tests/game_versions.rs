//! Headless tests for the project's game versions: the chips of the
//! Project space, the versions the fleet supports, and their checks in
//! Export Mod.

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

/// The sample truck project, in the Project space.
fn open() -> H {
    let mut h = common::harness();
    common::create_project(&mut h);
    common::show_space(&mut h, tp_app::layout::Space::Project);
    common::settle_renders(&mut h);
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

/// Clicks + Add, types `text` in its field, then presses `key` (Tab
/// leaves the field).
fn add(h: &mut H, text: &str, key: Key) {
    h.get_by_role_and_label(Role::Button, "Add").click();
    h.run();
    let field = h.get_by_role_and_label(Role::TextInput, FIELD);
    assert!(field.is_focused(), "+ Add's field has the focus");
    field.type_text(text);
    h.run();
    h.key_press(key);
    h.run();
}

/// The chips, in order.
fn chips(h: &H) -> Vec<String> {
    ws(h)
        .project
        .game_versions
        .iter()
        .filter(|v| h.query_by_label(v).is_some())
        .cloned()
        .collect()
}

fn steps(h: &H) -> usize {
    ws(h).history.len()
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
fn editing_the_game_versions() {
    let mut h = open();
    assert!(ws(&h).project.game_versions.is_empty());
    add(&mut h, "1.56.*, 1.57.*", Key::Enter);
    assert_eq!(ws(&h).project.game_versions, ["1.56.*", "1.57.*"]);
    assert_eq!(chips(&h), ["1.56.*", "1.57.*"]);
    let left = |v: &str| h.get_by_label(v).rect().left();
    assert!(left("1.56.*") < left("1.57.*"));
    // + Add is back.
    assert!(h.query_by_role_and_label(Role::Button, "Add").is_some());
    assert_eq!(ws(&h).save_state(), SaveState::Unsaved);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-edit-game-versions"));
    ws_mut(&mut h).undo();
    h.run();
    assert!(ws(&h).project.game_versions.is_empty());
    assert!(h.query_by_label("1.56.*").is_none());
}

#[test]
fn blanks_and_empty_entries_dropped() {
    let mut h = open();
    add(&mut h, " 1.57.* ,, 1.56.*", Key::Tab);
    assert_eq!(ws(&h).project.game_versions, ["1.57.*", "1.56.*"]);
    let left = |v: &str| h.get_by_label(v).rect().left();
    assert!(left("1.57.*") < left("1.56.*"));
}

#[test]
fn removing_a_game_version() {
    let mut h = open();
    add(&mut h, "1.56.*, 1.57.*", Key::Enter);
    let before = steps(&h);
    h.get_by_label("Remove 1.56.*").click();
    h.run();
    assert_eq!(ws(&h).project.game_versions, ["1.57.*"]);
    assert_eq!(steps(&h), before + 1);
    assert_eq!(ws(&h).history.undo_label(), Some("undo-edit-game-versions"));
    assert!(h.query_by_label("1.56.*").is_none());
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).project.game_versions, ["1.56.*", "1.57.*"]);
}

#[test]
fn escape_restores() {
    let mut h = open();
    add(&mut h, "1.56.*", Key::Enter);
    let before = steps(&h);
    add(&mut h, "1.57.*", Key::Escape);
    assert_eq!(ws(&h).project.game_versions, ["1.56.*"]);
    assert!(h.query_by_label("1.57.*").is_none());
    assert_eq!(steps(&h), before);
    assert!(
        h.query_by_role_and_label(Role::TextInput, FIELD).is_none(),
        "the field closed"
    );
}

#[test]
fn a_version_already_listed() {
    let mut h = open();
    add(&mut h, "1.56.*", Key::Enter);
    let before = steps(&h);
    add(&mut h, "1.56.*", Key::Enter);
    assert_eq!(ws(&h).project.game_versions, ["1.56.*"]);
    assert_eq!(steps(&h), before);
    // Nothing typed adds nothing.
    add(&mut h, "", Key::Enter);
    assert_eq!(steps(&h), before);
    // Saved before any of it: still saved once it was all undone.
    ws_mut(&mut h).undo();
    h.run();
    assert_eq!(ws(&h).save_state(), SaveState::Saved);
}

#[test]
fn badly_written_version_marked() {
    let mut h = open();
    add(&mut h, "1.56.x", Key::Enter);
    assert_eq!(ws(&h).project.game_versions, ["1.56.x"]);
    let message = "1.56.x isn't a game version: write it like 1.56.* or 1.56.2.";
    h.get_by_label("1.56.x").hover();
    h.run();
    // Said on hover, and under the field.
    assert!(h.query_all_by_label(message).count() >= 2);
}

#[test]
fn the_fleet_supported_versions_are_shown_under_the_field() {
    let mut h = open();
    let hint = h.get_by_label("Supported by every vehicle: >=1.56").rect();
    assert!(hint.top() > h.get_by_role_and_label(Role::Button, "Add").rect().bottom());
    add_old_vehicle(&mut h);
    assert!(
        h.query_by_label("No game version is supported by every vehicle")
            .is_some()
    );
}

#[test]
fn no_vehicle_limits_the_versions() {
    let mut h = open();
    for v in &mut ws_mut(&mut h).project.vehicles {
        v.game_data.as_mut().unwrap().versions = String::new();
    }
    h.run();
    assert!(
        h.query_by_label("Supported by every vehicle: any version")
            .is_some()
    );
}

#[test]
fn versions_supported_by_the_fleet() {
    let mut h = open();
    ws_mut(&mut h).project.vehicles[0]
        .game_data
        .as_mut()
        .unwrap()
        .versions = ">=1.53, <1.58".into();
    let ws = ws_mut(&mut h);
    let mut other = ws.project.vehicles[0].clone();
    other.name = "Other".into();
    other.package_id = "custom.other".into();
    let data = other.game_data.as_mut().unwrap();
    data.versions = "^1.56".into();
    data.path = "other".into();
    ws.project.vehicles.push(other);
    h.run();
    assert!(
        h.query_by_label("Supported by every vehicle: >=1.56, <1.58")
            .is_some()
    );
}

/// Opens Export Mod and returns the problems it lists.
fn export_problems(h: &mut H) -> Vec<String> {
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    for _ in 0..4 {
        h.step();
    }
    assert!(matches!(h.state().modal, Some(Modal::ExportMod(_))));
    let project = &ws(h).project;
    tp_app::mod_export::problems(project)
        .iter()
        .map(tp_app::mod_export::Problem::message)
        .collect()
}

#[test]
fn export_mod_checks_the_game_versions() {
    let mut h = open();
    add(&mut h, "1.55.*, 1.56.*", Key::Enter);
    let problems = export_problems(&mut h);
    assert_eq!(
        problems,
        ["TruckPaint Sample Truck (>=1.56) doesn't support game version 1.55.*."]
    );
    // In the dialog, and under the field and in Before exporting behind it.
    assert_eq!(h.query_all_by_label(&problems[0]).count(), 3);
    assert!(common::last(&h, "Export").accesskit_node().is_disabled());
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

#[test]
fn show_leads_to_add() {
    let mut h = open();
    ws_mut(&mut h).project.game_versions = vec!["1.56.x".into()];
    h.run();
    // Before exporting is below the window.
    h.get_by_label("Show Game versions").click_accesskit();
    h.run();
    assert!(h.get_by_role_and_label(Role::TextInput, FIELD).is_focused());
    // From the Export Mod dialog too.
    h.key_press(Key::Escape);
    h.run();
    assert!(h.query_by_role_and_label(Role::TextInput, FIELD).is_none());
    h.key_press_modifiers(Modifiers::COMMAND, Key::E);
    for _ in 0..4 {
        h.step();
    }
    common::last(&h, "Show Game versions").click();
    h.run();
    assert!(h.state().modal.is_none());
    assert!(h.get_by_role_and_label(Role::TextInput, FIELD).is_focused());
}
