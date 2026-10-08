//! Domain types for the TruckPaint livery editor.
//!
//! This crate has no UI, filesystem or threading dependencies so it can be
//! unit-tested in isolation (and compiled to other targets later).

mod brand;
pub mod document;
pub mod import;
pub mod mod_settings;
mod project;
mod symbols;

pub use brand::{BrandKit, GraphicStyle, Look, Swatch, TextStyle};
pub use import::LibraryKey;
pub use kurbo;
pub use mod_settings::ModSettings;
pub use project::{
    Asset, AssetKind, Axis, DEFAULT_PROJECT_NAME, DEFAULT_SWATCH_PREFIX, GameData, Guide,
    MAIN_SURFACE_NAME, Project, ProjectVehicle, RequiredMod, Snapshot, Surface, SurfaceTemplate,
    TemplateStatus, TextureKey, TexturePart, TextureResolution,
};
pub use symbols::Symbol;
