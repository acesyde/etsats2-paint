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

impl Rgba {
    /// `#RRGGBB`, or `#RRGGBBAA` when not opaque.
    pub fn to_hex(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// Parses `#RGB`, `#RRGGBB` or `#RRGGBBAA` (the `#` is optional,
    /// case-insensitive, surrounding spaces ignored).
    pub fn from_hex(text: &str) -> Option<Self> {
        let hex = text.trim().trim_start_matches('#');
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        match hex.len() {
            3 => {
                let nibble = |i: usize| u8::from_str_radix(&hex[i..=i], 16).ok().map(|v| v * 17);
                Some(Self::rgb(nibble(0)?, nibble(1)?, nibble(2)?))
            }
            6 => Some(Self::rgb(byte(0)?, byte(2)?, byte(4)?)),
            8 => Some(Self::with_alpha(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
            _ => None,
        }
    }

    fn unit(self) -> (f32, f32, f32, f32) {
        let f = |c: u8| f32::from(c) / 255.0;
        (f(self.r), f(self.g), f(self.b), f(self.a))
    }

    fn from_unit(r: f32, g: f32, b: f32, a: f32) -> Self {
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        Self::with_alpha(q(r), q(g), q(b), q(a))
    }
}

/// Hue (degrees, 0..360), saturation, value and alpha (0..1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsva {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
}

/// Hue (degrees, 0..360), saturation, lightness and alpha (0..1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsla {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: f32,
}

fn hue_of(r: f32, g: f32, b: f32, max: f32, delta: f32) -> f32 {
    if delta <= f32::EPSILON {
        return 0.0;
    }
    let h = if max == r {
        ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    h * 60.0
}

/// RGB from hue (degrees) and chroma/offset, the shared HSV/HSL formula.
fn rgb_from_hue(h: f32, chroma: f32, m: f32) -> (f32, f32, f32) {
    let h = h.rem_euclid(360.0) / 60.0;
    let x = chroma * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    (r + m, g + m, b + m)
}

impl From<Rgba> for Hsva {
    fn from(c: Rgba) -> Self {
        let (r, g, b, a) = c.unit();
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        Self {
            h: hue_of(r, g, b, max, delta),
            s: if max <= f32::EPSILON {
                0.0
            } else {
                delta / max
            },
            v: max,
            a,
        }
    }
}

impl From<Hsva> for Rgba {
    fn from(c: Hsva) -> Self {
        let chroma = c.v * c.s;
        let (r, g, b) = rgb_from_hue(c.h, chroma, c.v - chroma);
        Rgba::from_unit(r, g, b, c.a)
    }
}

impl From<Rgba> for Hsla {
    fn from(c: Rgba) -> Self {
        let (r, g, b, a) = c.unit();
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let l = (max + min) / 2.0;
        let s = if delta <= f32::EPSILON {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };
        Self {
            h: hue_of(r, g, b, max, delta),
            s,
            l,
            a,
        }
    }
}

impl From<Hsla> for Rgba {
    fn from(c: Hsla) -> Self {
        let chroma = (1.0 - (2.0 * c.l - 1.0).abs()) * c.s;
        let (r, g, b) = rgb_from_hue(c.h, chroma, c.l - chroma / 2.0);
        Rgba::from_unit(r, g, b, c.a)
    }
}

/// Fill of newly created shapes until colors can be edited.
pub const DEFAULT_FILL: Rgba = Rgba::rgb(0x5B, 0x8D, 0xEF);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parsing() {
        assert_eq!(Rgba::from_hex("#f80"), Some(Rgba::rgb(0xFF, 0x88, 0x00)));
        assert_eq!(Rgba::from_hex("7A1F2B"), Some(Rgba::rgb(0x7A, 0x1F, 0x2B)));
        assert_eq!(
            Rgba::from_hex(" #7a1f2b80 "),
            Some(Rgba::with_alpha(0x7A, 0x1F, 0x2B, 0x80))
        );
        assert_eq!(Rgba::from_hex("zz12"), None);
        assert_eq!(Rgba::from_hex("#12345"), None);
        assert_eq!(Rgba::from_hex(""), None);
    }

    #[test]
    fn hex_formatting() {
        assert_eq!(Rgba::rgb(255, 136, 0).to_hex(), "#FF8800");
        assert_eq!(Rgba::with_alpha(1, 2, 3, 4).to_hex(), "#01020304");
    }

    #[test]
    fn hsv_known_values_and_round_trip() {
        let red = Hsva::from(Rgba::rgb(255, 0, 0));
        assert_eq!((red.h, red.s, red.v), (0.0, 1.0, 1.0));
        let blue = Hsva::from(Rgba::rgb(0, 0, 255));
        assert!((blue.h - 240.0).abs() < 1e-3);
        for c in [
            Rgba::rgb(0x7A, 0x1F, 0x2B),
            Rgba::rgb(12, 200, 99),
            Rgba::with_alpha(5, 5, 5, 128),
            Rgba::rgb(255, 255, 255),
        ] {
            assert_eq!(Rgba::from(Hsva::from(c)), c);
            assert_eq!(Rgba::from(Hsla::from(c)), c);
        }
    }

    #[test]
    fn hsl_known_values() {
        let gray = Hsla::from(Rgba::rgb(128, 128, 128));
        assert_eq!(gray.s, 0.0);
        assert!((gray.l - 128.0 / 255.0).abs() < 1e-6);
        assert_eq!(
            Rgba::from(Hsla {
                h: 120.0,
                s: 1.0,
                l: 0.5,
                a: 1.0
            }),
            Rgba::rgb(0, 255, 0)
        );
    }

    #[test]
    fn opacity_scales_alpha() {
        assert_eq!(Rgba::rgb(1, 2, 3).with_opacity(0.5).a, 128);
        assert_eq!(Rgba::with_alpha(1, 2, 3, 100).with_opacity(2.0).a, 100);
    }
}
