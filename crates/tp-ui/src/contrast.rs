//! WCAG 2.x contrast computation and the list of text/background pairs the
//! theme promises to keep readable.

use egui::Color32;

use crate::tokens::color;

/// Minimum contrast ratio required for a pair, by text emphasis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emphasis {
    /// Primary text: at least 7:1.
    Primary,
    /// Secondary text and informative icons: at least 4.5:1.
    Secondary,
    /// Disabled content: at least 3:1.
    Disabled,
}

impl Emphasis {
    pub fn min_ratio(self) -> f32 {
        match self {
            Self::Primary => 7.0,
            Self::Secondary => 4.5,
            Self::Disabled => 3.0,
        }
    }
}

fn channel_to_linear(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.039_28 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Relative luminance as defined by WCAG.
pub fn relative_luminance(c: Color32) -> f32 {
    0.2126 * channel_to_linear(c.r())
        + 0.7152 * channel_to_linear(c.g())
        + 0.0722 * channel_to_linear(c.b())
}

/// Contrast ratio between two opaque colors, in `1.0..=21.0`.
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// A foreground/background combination used by the theme.
#[derive(Clone, Debug)]
pub struct ContrastPair {
    pub name: String,
    pub fg: Color32,
    pub bg: Color32,
    pub emphasis: Emphasis,
}

/// Every text/background pair the theme uses.
pub fn theme_pairs() -> Vec<ContrastPair> {
    use Emphasis::{Disabled, Primary, Secondary};
    let backgrounds = [
        ("surface0", color::SURFACE_0),
        ("surface1", color::SURFACE_1),
        ("surface2", color::SURFACE_2),
        ("surface3", color::SURFACE_3),
        ("accent_subtle", color::ACCENT_SUBTLE),
    ];
    let mut pairs = Vec::new();
    for (bg_name, bg) in backgrounds {
        for (fg_name, fg, emphasis) in [
            ("text_primary", color::TEXT_PRIMARY, Primary),
            ("text_secondary", color::TEXT_SECONDARY, Secondary),
            ("text_disabled", color::TEXT_DISABLED, Disabled),
            ("accent", color::ACCENT, Secondary),
            ("success", color::SUCCESS, Secondary),
            ("warning", color::WARNING, Secondary),
            ("error", color::ERROR, Secondary),
        ] {
            pairs.push(ContrastPair {
                name: format!("{fg_name} on {bg_name}"),
                fg,
                bg,
                emphasis,
            });
        }
    }
    // Pressed state only ever shows primary text.
    pairs.push(ContrastPair {
        name: "text_primary on surface4".to_owned(),
        fg: color::TEXT_PRIMARY,
        bg: color::SURFACE_4,
        emphasis: Primary,
    });
    pairs.push(ContrastPair {
        name: "text_on_accent on accent_fill".to_owned(),
        fg: color::TEXT_ON_ACCENT,
        bg: color::ACCENT_FILL,
        emphasis: Primary,
    });
    pairs.push(ContrastPair {
        name: "text_on_accent on accent_fill_hover".to_owned(),
        fg: color::TEXT_ON_ACCENT,
        bg: color::ACCENT_FILL_HOVER,
        emphasis: Secondary,
    });
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_on_white_is_21() {
        let ratio = contrast_ratio(Color32::BLACK, Color32::WHITE);
        assert!((ratio - 21.0).abs() < 0.01, "{ratio}");
    }

    #[test]
    fn ratio_is_symmetric() {
        let a = contrast_ratio(color::TEXT_PRIMARY, color::SURFACE_1);
        let b = contrast_ratio(color::SURFACE_1, color::TEXT_PRIMARY);
        assert!((a - b).abs() < f32::EPSILON);
    }

    #[test]
    fn theme_pairs_meet_minimums() {
        let failures: Vec<String> = theme_pairs()
            .into_iter()
            .filter_map(|p| {
                let ratio = contrast_ratio(p.fg, p.bg);
                (ratio < p.emphasis.min_ratio())
                    .then(|| format!("{}: {ratio:.2} < {:.1}", p.name, p.emphasis.min_ratio()))
            })
            .collect();
        assert!(
            failures.is_empty(),
            "contrast failures:\n{}",
            failures.join("\n")
        );
    }
}
