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
    let mut pairs = Vec::new();
    let mut add = |fg_name: &str, fg, bg_name: &str, bg, emphasis| {
        pairs.push(ContrastPair {
            name: format!("{fg_name} on {bg_name}"),
            fg,
            bg,
            emphasis,
        });
    };

    // The surfaces carry every kind of text, the accents used as text or
    // informative icons, and the semantic colors: the three elevation
    // levels, the sunken fields and the control surface (buttons, tracks,
    // menus, popovers, pills).
    for (bg_name, bg) in [
        ("canvas", color::SURFACE_0),
        ("panel", color::SURFACE_1),
        ("raised", color::SURFACE_2),
        ("field", color::FIELD),
        ("control", color::CONTROL),
    ] {
        for (fg_name, fg, emphasis) in [
            ("ink", color::TEXT_PRIMARY, Primary),
            ("text_secondary", color::TEXT_SECONDARY, Secondary),
            ("text_muted", color::TEXT_MUTED, Secondary),
            ("text_disabled", color::TEXT_DISABLED, Disabled),
            ("signal", color::SIGNAL, Secondary),
            ("link", color::LINK, Secondary),
            ("success", color::SUCCESS, Secondary),
            ("warning", color::WARNING, Secondary),
            ("error", color::ERROR, Secondary),
        ] {
            add(fg_name, fg, bg_name, bg, emphasis);
        }
    }

    // Hovered controls and selected rows show plain text; disabled controls
    // are never hovered or selected but may sit on a hover-colored row.
    for (bg_name, bg) in [("hover", color::SURFACE_3), ("selected", color::SELECTED)] {
        add("ink", color::TEXT_PRIMARY, bg_name, bg, Primary);
        add(
            "text_secondary",
            color::TEXT_SECONDARY,
            bg_name,
            bg,
            Secondary,
        );
        add("link", color::LINK, bg_name, bg, Secondary);
    }
    add(
        "text_disabled",
        color::TEXT_DISABLED,
        "hover",
        color::SURFACE_3,
        Disabled,
    );
    // Chips (the game badge, the active space) show ink or secondary text.
    add("ink", color::TEXT_PRIMARY, "chip", color::CHIP, Primary);
    add(
        "text_secondary",
        color::TEXT_SECONDARY,
        "chip",
        color::CHIP,
        Secondary,
    );
    // Pressed state only ever shows primary text.
    add(
        "ink",
        color::TEXT_PRIMARY,
        "pressed",
        color::SURFACE_4,
        Primary,
    );

    // Dark text on the white pill of a segmented control and on primary
    // buttons.
    add(
        "text_on_primary",
        color::TEXT_ON_PRIMARY,
        "primary pill",
        color::ACCENT_PRIMARY,
        Primary,
    );
    pairs
}

/// The pairs below their minimum ratio, described for a test failure.
pub fn failures(pairs: &[ContrastPair]) -> Vec<String> {
    pairs
        .iter()
        .filter_map(|p| {
            let ratio = contrast_ratio(p.fg, p.bg);
            (ratio < p.emphasis.min_ratio())
                .then(|| format!("{}: {ratio:.2} < {:.1}", p.name, p.emphasis.min_ratio()))
        })
        .collect()
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
        let failures = failures(&theme_pairs());
        assert!(
            failures.is_empty(),
            "contrast failures:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    fn required_pairs_are_checked() {
        let names: Vec<String> = theme_pairs().into_iter().map(|p| p.name).collect();
        for surface in ["canvas", "panel", "raised", "field", "control"] {
            for fg in ["ink", "text_muted", "signal", "link"] {
                let name = format!("{fg} on {surface}");
                assert!(names.contains(&name), "missing pair {name}");
            }
        }
        assert!(names.contains(&"text_on_primary on primary pill".to_owned()));
        assert!(names.contains(&"ink on chip".to_owned()));
    }

    /// The check is not vacuous: darkening a token below its minimum is
    /// caught on the right pairs.
    #[test]
    fn darkened_tokens_fail() {
        let darken = |c: Color32| Color32::from_rgb(c.r() / 2, c.g() / 2, c.b() / 2);
        for token in [color::SIGNAL, color::LINK, color::TEXT_PRIMARY] {
            let pairs: Vec<ContrastPair> = theme_pairs()
                .into_iter()
                .map(|mut p| {
                    if p.fg == token {
                        p.fg = darken(p.fg);
                    }
                    p
                })
                .collect();
            let failures = failures(&pairs);
            assert!(
                failures.iter().any(|f| f.contains("on raised")),
                "darkened {token:?} not caught: {failures:?}"
            );
        }
        // A lighter pill loses its dark text.
        let pairs: Vec<ContrastPair> = theme_pairs()
            .into_iter()
            .map(|mut p| {
                if p.bg == color::ACCENT_PRIMARY {
                    p.bg = darken(p.bg);
                }
                p
            })
            .collect();
        assert!(
            failures(&pairs)
                .iter()
                .any(|f| f.starts_with("text_on_primary"))
        );
    }
}
