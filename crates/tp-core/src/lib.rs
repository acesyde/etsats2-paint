//! Domain types for the TruckPaint livery editor.
//!
//! This crate has no UI, filesystem or threading dependencies so it can be
//! unit-tested in isolation (and compiled to other targets later).

mod brand;
pub mod document;
mod project;

pub use brand::{BrandKit, GraphicStyle, Look, Swatch, TextStyle};
pub use kurbo;
pub use project::{
    Asset, AssetKind, Axis, DEFAULT_PROJECT_NAME, DEFAULT_SWATCH_PREFIX, Guide, MAIN_SURFACE_NAME,
    Project, ProjectVehicle, Snapshot, Surface, SurfaceTemplate, TemplateStatus, TextureKey,
    TexturePart, TextureResolution,
};
