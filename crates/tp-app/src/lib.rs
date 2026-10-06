//! TruckPaint desktop application.

pub mod app;
pub mod commands;
pub mod layout;
pub mod logging;
pub mod paths;
pub mod prefs;
pub mod state;
pub mod tool;
pub mod ui;

pub use app::TruckPaintApp;
pub use state::AppState;
