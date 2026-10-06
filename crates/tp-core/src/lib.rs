//! Domain types for the TruckPaint livery editor.
//!
//! This crate has no UI, filesystem or threading dependencies so it can be
//! unit-tested in isolation (and compiled to other targets later).

pub mod document;
mod project;

pub use kurbo;
pub use project::{
    Asset, AssetKind, Axis, DEFAULT_PROJECT_NAME, Guide, MAIN_SURFACE_NAME, Project, Snapshot,
    Surface, TextureResolution,
};
