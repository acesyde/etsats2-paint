//! Full-resolution rendering of a livery surface, and texture encoders
//! (PNG, DDS with mipmaps).

mod encode;
mod render;

pub use encode::{DdsEncoding, dds_size, encode_dds, encode_png, mip_chain};

pub use render::{Cancelled, RenderOptions, render, svg_options, to_rgba};
pub use resvg::tiny_skia::Pixmap;
