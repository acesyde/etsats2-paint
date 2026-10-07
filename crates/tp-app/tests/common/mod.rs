//! Shared helpers for UI tests.

#![allow(dead_code)]

use egui::Vec2;
use egui_kittest::Harness;
use tp_app::AppState;
use tp_app::prefs::Prefs;
use tp_app::vehicles::VehicleLibrary;
use tp_vehicles::Package;

pub const SIZE: Vec2 = Vec2::new(1440.0, 900.0);
/// Name of the built-in sample vehicle every test project is made for.
pub const SAMPLE: &str = "TruckPaint Sample Truck";

/// A vehicle library in a temporary folder (kept for the whole test run)
/// with the built-in sample vehicle installed.
pub fn sample_library() -> VehicleLibrary {
    let dir = tempfile::tempdir().expect("temporary folder").keep();
    let mut library = VehicleLibrary::open(&dir);
    library.install_sample().expect("sample vehicle");
    library
}

/// The application with in-memory preferences and the sample vehicle
/// installed.
fn state(prefs: Prefs) -> AppState {
    let mut state = AppState::with_prefs(prefs, None);
    state.vehicles = sample_library();
    state
}

/// Harness running the whole application with in-memory preferences.
pub fn harness_with(prefs: Prefs) -> Harness<'static, AppState> {
    Harness::builder()
        .with_size(SIZE)
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state(prefs))
}

pub fn harness() -> Harness<'static, AppState> {
    harness_with(Prefs::default())
}

/// Same, rendering with wgpu (for screenshots).
pub fn wgpu_harness_with(prefs: Prefs, size: Vec2) -> Harness<'static, AppState> {
    Harness::builder()
        .with_size(size)
        .wgpu()
        .build_ui_state(|ui, state: &mut AppState| state.show(ui), state(prefs))
}

/// Opens a project for the sample vehicle's Standard cab, as New Project
/// would make it: its 4096 px Cabin is active. Its templates are hidden:
/// the sample's SVG templates render in a background thread whose repaints
/// would make editing tests depend on timing (tests/vehicles.rs covers
/// templates and the wizard).
pub fn create_project(harness: &mut Harness<'static, AppState>) {
    let has_sample = harness
        .state()
        .vehicles
        .get(tp_app::vehicles::SAMPLE_ID)
        .is_some();
    if !has_sample {
        harness.state_mut().vehicles = sample_library();
    }
    let package = Package::read(tp_app::vehicles::SAMPLE).expect("sample package");
    let mut project = tp_app::vehicle_project::vehicle_project(SAMPLE, &package, "standard")
        .expect("sample variant");
    for surface in &mut project.surfaces {
        if let Some(t) = surface.template.as_mut() {
            t.visible = false;
        }
    }
    harness.state_mut().open_project(project);
    harness.run();
    assert!(harness.state().has_project(), "project should be open");
}
