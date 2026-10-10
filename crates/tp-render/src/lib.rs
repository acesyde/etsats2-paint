//! Full-resolution rendering of a livery surface, and texture encoders
//! (PNG, DDS with mipmaps).

mod cover;
mod encode;
mod render;
mod shadow;

pub use cover::{cover_image, render_cover};
pub use encode::{DdsEncoding, dds_size, encode_dds, encode_jpeg, encode_png, mip_chain};

pub use render::{Cancelled, DrawCache, RenderOptions, render, svg_options, to_rgba};
pub use resvg::tiny_skia::Pixmap;
pub use shadow::{ShadowLayer, shadow_layer};
