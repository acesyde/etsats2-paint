//! The vector document: objects, geometry, transforms and undo history.

mod color;
mod history;
mod object;
mod transform;

pub use color::{DEFAULT_FILL, Rgba};
pub use history::{COALESCE_WINDOW, DEFAULT_MAX_STEPS, History};
pub use object::{
    Frame, MIN_SIZE, Object, ObjectId, ShapeKind, StrokeStyle, convex_polygons_overlap,
    flatten_closed, normalize_degrees,
};
pub use transform::{
    Handle, ResizeOptions, angle_around, resize, rotate, selection_frame, snap_direction, translate,
};
