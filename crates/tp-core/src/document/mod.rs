//! The vector document: objects, geometry, transforms and undo history.

mod color;
mod history;
mod object;
pub mod path;
mod transform;
pub mod tree;

pub use color::{DEFAULT_FILL, Hsla, Hsva, Rgba};
pub use history::{COALESCE_WINDOW, DEFAULT_MAX_STEPS, History};
pub use object::{
    AssetId, CharStyle, Frame, MIN_SIZE, Object, ObjectId, ShapeKind, StrokeStyle, TextAlign,
    TextBlock, convex_polygons_overlap, flatten_closed, flatten_subpaths, normalize_degrees,
    segments_intersect,
};
pub use path::{HandleSide, Node, NodeRef, PathData, PointRef, SegmentHit, Subpath};
pub use transform::{
    Handle, ResizeOptions, angle_around, resize, rotate, selection_frame, snap_direction, translate,
};
