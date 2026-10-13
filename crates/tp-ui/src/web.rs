//! The color tokens as CSS custom properties, for the documentation site
//! (`website/src/styles/tokens.css`).
//!
//! The dark theme is the app's palette as is. The light theme, which the app
//! doesn't have, is derived from it here: neutral surfaces with the same low
//! saturation, text levels inverted, the primary accent turned near-black so
//! the primary action keeps its contrast, and the signal and link accents
//! keeping their hue with a lower lightness. [`light_failures`] checks it
//! against WCAG AA.

use egui::Color32;

use crate::contrast::{ContrastPair, Emphasis, failures};
use crate::tokens::color;

/// The site's tokens: the CSS name (after `--tp-`), the dark value and the
/// light value.
pub fn tokens() -> Vec<(&'static str, Color32, Color32)> {
    vec![
        ("surface-0", color::SURFACE_0, rgb(0xF6, 0xF6, 0xF7)),
        ("surface-1", color::SURFACE_1, rgb(0xFF, 0xFF, 0xFF)),
        ("surface-2", color::SURFACE_2, rgb(0xEE, 0xEE, 0xF0)),
        ("surface-3", color::SURFACE_3, rgb(0xE4, 0xE4, 0xE7)),
        ("surface-4", color::SURFACE_4, rgb(0xD9, 0xD9, 0xDD)),
        ("control", color::CONTROL, rgb(0xF0, 0xF0, 0xF2)),
        ("chip", color::CHIP, rgb(0xE6, 0xE6, 0xE9)),
        ("selected", color::SELECTED, rgb(0xE0, 0xE0, 0xE4)),
        ("border", color::BORDER, rgb(0xE2, 0xE2, 0xE5)),
        ("border-strong", color::BORDER_STRONG, rgb(0xC6, 0xC6, 0xCC)),
        ("outline", color::OUTLINE, rgb(0xD4, 0xD4, 0xD8)),
        ("text-primary", color::TEXT_PRIMARY, rgb(0x1C, 0x1C, 0x1F)),
        (
            "text-secondary",
            color::TEXT_SECONDARY,
            rgb(0x52, 0x52, 0x59),
        ),
        ("text-muted", color::TEXT_MUTED, rgb(0x63, 0x63, 0x6B)),
        ("text-disabled", color::TEXT_DISABLED, rgb(0x86, 0x86, 0x8E)),
        (
            "accent-primary",
            color::ACCENT_PRIMARY,
            rgb(0x1C, 0x1C, 0x1F),
        ),
        (
            "text-on-primary",
            color::TEXT_ON_PRIMARY,
            rgb(0xFF, 0xFF, 0xFF),
        ),
        ("signal", color::SIGNAL, rgb(0xC2, 0x32, 0x32)),
        // oklch 0.50 0.11 225: the link hue, darker.
        ("link", color::LINK, rgb(0x00, 0x6A, 0x8E)),
        ("success", color::SUCCESS, rgb(0x23, 0x7A, 0x3E)),
        ("warning", color::WARNING, rgb(0x8A, 0x58, 0x00)),
        ("error", color::ERROR, rgb(0xBF, 0x2F, 0x27)),
    ]
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

fn light(name: &str) -> Color32 {
    tokens()
        .into_iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, _, light)| light)
        .unwrap_or_else(|| panic!("no token {name}"))
}

/// The light theme's text and accent pairs below WCAG AA, on the surfaces
/// the site draws text on.
pub fn light_failures() -> Vec<String> {
    use Emphasis::{Disabled, Primary, Secondary};
    let mut pairs = Vec::new();
    for bg in ["surface-0", "surface-1", "surface-2", "control"] {
        for (fg, emphasis) in [
            ("text-primary", Primary),
            ("text-secondary", Secondary),
            ("text-muted", Secondary),
            ("text-disabled", Disabled),
            ("signal", Secondary),
            ("link", Secondary),
            ("success", Secondary),
            ("warning", Secondary),
            ("error", Secondary),
        ] {
            pairs.push(ContrastPair {
                name: format!("light {fg} on {bg}"),
                fg: light(fg),
                bg: light(bg),
                emphasis,
            });
        }
    }
    for bg in ["surface-3", "selected", "chip"] {
        pairs.push(ContrastPair {
            name: format!("light text-primary on {bg}"),
            fg: light("text-primary"),
            bg: light(bg),
            emphasis: Primary,
        });
    }
    pairs.push(ContrastPair {
        name: "light text-on-primary on accent-primary".into(),
        fg: light("text-on-primary"),
        bg: light("accent-primary"),
        emphasis: Primary,
    });
    failures(&pairs)
}

fn hex(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

/// The content of `tokens.css`.
pub fn tokens_css() -> String {
    let block = |selector: &str, dark: bool| {
        let mut out = format!("{selector} {{\n");
        for (name, d, l) in tokens() {
            let value = if dark { d } else { l };
            out.push_str(&format!("\t--tp-{name}: {};\n", hex(value)));
        }
        out.push_str("}\n");
        out
    };
    format!(
        "/* Generated from crates/tp-ui/src/tokens.rs by `mise run doc:tokens`. Do not edit. */\n\n{}\n{}",
        block(":root,\n:root[data-theme='dark']", true),
        block(":root[data-theme='light']", false),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_theme_meets_aa() {
        let failures = light_failures();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn dark_values_are_the_app_tokens() {
        let css = tokens_css();
        assert!(css.contains("--tp-surface-1: #222225;"));
        assert!(css.contains("--tp-link: #50b9df;"));
    }

    /// The committed `website/src/styles/tokens.css` matches the tokens.
    #[test]
    fn site_tokens_are_up_to_date() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../website/src/styles/tokens.css"
        );
        let committed = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            committed == tokens_css(),
            "website/src/styles/tokens.css is out of date: run `mise run doc:tokens`"
        );
    }
}
