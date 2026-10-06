//! TruckPaint desktop application.

pub mod app;
pub mod commands;
pub mod geometry_cache;
pub mod gesture;
pub mod layout;
pub mod logging;
pub mod paths;
pub mod prefs;
pub mod state;
pub mod tool;
pub mod ui;
pub mod viewport;
pub mod workspace;

pub use app::TruckPaintApp;
pub use state::AppState;
