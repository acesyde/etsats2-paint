//! Domain types for the TruckPaint livery editor.
//!
//! This crate has no UI, filesystem or threading dependencies so it can be
//! unit-tested in isolation (and compiled to other targets later).

mod project;

pub use project::{DEFAULT_PROJECT_NAME, ProjectStub, TextureResolution};
