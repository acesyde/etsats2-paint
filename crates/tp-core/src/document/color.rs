/// 8-bit sRGB color with straight (non-premultiplied) alpha.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Same color with its alpha multiplied by `opacity` (0..=1).
    pub fn with_opacity(self, opacity: f32) -> Self {
        let a = (f32::from(self.a) * opacity.clamp(0.0, 1.0)).round() as u8;
        Self { a, ..self }
    }
}

/// Fill of newly created shapes until colors can be edited.
pub const DEFAULT_FILL: Rgba = Rgba::rgb(0x5B, 0x8D, 0xEF);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_scales_alpha() {
        assert_eq!(Rgba::rgb(1, 2, 3).with_opacity(0.5).a, 128);
        assert_eq!(Rgba::with_alpha(1, 2, 3, 100).with_opacity(2.0).a, 100);
    }
}
