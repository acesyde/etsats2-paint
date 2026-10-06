//! TruckPaint desktop application.

pub mod app;
pub mod commands;
pub mod geometry_cache;
pub mod gesture;
pub mod image_cache;
pub mod import;
pub mod layout;
pub mod logging;
pub mod paths;
pub mod placement;
pub mod prefs;
pub mod state;
pub mod text_engine;
pub mod text_input;
pub mod text_session;
pub mod tool;
pub mod ui;
pub mod viewport;
pub mod workspace;

pub use app::TruckPaintApp;
pub use state::AppState;
