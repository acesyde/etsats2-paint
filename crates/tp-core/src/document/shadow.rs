//! The drop shadow of a shape, a path, a text or an image.

use kurbo::Vec2;

use super::color::Rgba;
use super::object::SwatchId;

/// Largest blur radius, in texture pixels.
pub const MAX_SHADOW_BLUR: f64 = 200.0;
/// Largest offset along each axis, in texture pixels.
pub const MAX_SHADOW_OFFSET: f64 = 1000.0;

/// The object's silhouette in `color`, moved by `offset` and blurred,
/// drawn under it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub color: Rgba,
    /// The swatch the color is linked to.
    pub swatch: Option<SwatchId>,
    /// 0.0..=1.0.
    pub opacity: f32,
    /// Texture pixels, each in -1000..=1000.
    pub offset: Vec2,
    /// Blur radius in texture pixels, 0 (a hard edge) to 200.
    pub blur: f64,
}

impl Shadow {
    /// What + Add a shadow gives: black, 50%, offset 8 / 8, blur 8.
    pub const DEFAULT: Shadow = Shadow {
        color: Rgba::rgb(0, 0, 0),
        swatch: None,
        opacity: 0.5,
        offset: Vec2::new(8.0, 8.0),
        blur: 8.0,
    };

    /// The same shadow with every value in its range (non-finite values
    /// become the default's).
    pub fn clamped(self) -> Self {
        let finite = |v: f64, default: f64| if v.is_finite() { v } else { default };
        let offset =
            |v: f64, default: f64| finite(v, default).clamp(-MAX_SHADOW_OFFSET, MAX_SHADOW_OFFSET);
        let opacity = if self.opacity.is_finite() {
            self.opacity
        } else {
            Self::DEFAULT.opacity
        };
        Self {
            opacity: opacity.clamp(0.0, 1.0),
            offset: Vec2::new(
                offset(self.offset.x, Self::DEFAULT.offset.x),
                offset(self.offset.y, Self::DEFAULT.offset.y),
            ),
            blur: finite(self.blur, Self::DEFAULT.blur).clamp(0.0, MAX_SHADOW_BLUR),
            ..self
        }
    }
}
