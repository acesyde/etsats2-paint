//! Shared helpers for UI tests.

#![allow(dead_code)]

use egui::Vec2;
use egui_kittest::Harness;
use tp_app::AppState;
use tp_app::prefs::Prefs;

pub const SIZE: Vec2 = Vec2::new(1440.0, 900.0);

/// Harness running the whole application with in-memory preferences.
pub fn harness_with(prefs: Prefs) -> Harness<'static, AppState> {
    Harness::builder().with_size(SIZE).build_ui_state(
        |ui, state: &mut AppState| state.show(ui),
        AppState::with_prefs(prefs, None),
    )
}

pub fn harness() -> Harness<'static, AppState> {
    harness_with(Prefs::default())
}

/// Same, rendering with wgpu (for screenshots).
pub fn wgpu_harness_with(prefs: Prefs, size: Vec2) -> Harness<'static, AppState> {
    Harness::builder().with_size(size).wgpu().build_ui_state(
        |ui, state: &mut AppState| state.show(ui),
        AppState::with_prefs(prefs, None),
    )
}

/// Creates a default project through the New Project dialog.
pub fn create_project(harness: &mut Harness<'static, AppState>) {
    use egui_kittest::kittest::Queryable;
    harness.get_by_label("New Project").click();
    harness.run();
    harness.get_by_label("Next").click();
    harness.run();
    harness.get_by_label("Create").click();
    harness.run();
    assert!(harness.state().has_project(), "project should be open");
}
