//! TruckPaint desktop application.

pub mod app;
pub mod arrange;
pub mod commands;
pub mod export;
pub mod file_dialogs;
pub mod geometry_cache;
pub mod gesture;
pub mod image_cache;
pub mod import;
pub mod layout;
pub mod logging;
pub mod outline_text;
pub mod path_edit;
pub mod paths;
pub mod placement;
pub mod prefs;
pub mod project_io;
pub mod recovery;
pub mod saver;
pub mod snap;
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
