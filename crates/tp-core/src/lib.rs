//! Domain types for the TruckPaint livery editor.
//!
//! This crate has no UI, filesystem or threading dependencies so it can be
//! unit-tested in isolation (and compiled to other targets later).

pub mod document;
mod project;

pub use kurbo;
pub use project::{
    DEFAULT_PROJECT_NAME, MAIN_SURFACE_NAME, Project, Snapshot, Surface, TextureResolution,
};
