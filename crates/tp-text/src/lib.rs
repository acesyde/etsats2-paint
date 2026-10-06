//! Text for TruckPaint: font library, layout, glyph outlines and editing.

mod edit;
mod fonts;
mod layout;
pub mod mesh;

pub use edit::{EditSession, Motion};
pub use fonts::{BUNDLED_FAMILIES, FALLBACK_FAMILY, FontLibrary, bundled_font_data};
pub use layout::{GlyphCache, LineLayout, PlacedGlyph, TextLayout, layout};
