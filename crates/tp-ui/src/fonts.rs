//! Embedded fonts: Geist for interface text, JetBrains Mono for values
//! (numbers, sizes, positions, hex colors, paths), Phosphor for icons.
//!
//! Inter is no longer an interface font; it stays bundled as a document font
//! for text objects (`tp-text`).

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

macro_rules! font {
    ($file:literal) => {
        include_bytes!(concat!("../../../assets/fonts/", $file))
    };
}

const GEIST_REGULAR: &[u8] = font!("Geist-Regular.ttf");
const GEIST_MEDIUM: &[u8] = font!("Geist-Medium.ttf");
const GEIST_SEMIBOLD: &[u8] = font!("Geist-SemiBold.ttf");
const MONO_REGULAR: &[u8] = font!("JetBrainsMono-Regular.ttf");
const MONO_MEDIUM: &[u8] = font!("JetBrainsMono-Medium.ttf");

/// Font data name of Geist Regular, first in the Proportional family.
pub const GEIST: &str = "geist";
/// Font data name of JetBrains Mono Regular, first in the Monospace family.
pub const MONO: &str = "jetbrains-mono";
/// Family name for medium interface text (Geist Medium).
pub const MEDIUM: &str = "geist-medium";
/// Family name for semibold interface text: titles, headings, emphasized
/// labels (Geist SemiBold).
pub const SEMIBOLD: &str = "geist-semibold";
/// Family name for emphasized values (JetBrains Mono Medium).
pub const MONO_MEDIUM_FAMILY: &str = "jetbrains-mono-medium";

/// The medium font family.
pub fn medium_family() -> FontFamily {
    FontFamily::Name(MEDIUM.into())
}

/// The semibold font family.
pub fn semibold_family() -> FontFamily {
    FontFamily::Name(SEMIBOLD.into())
}

/// The medium monospace font family.
pub fn mono_medium_family() -> FontFamily {
    FontFamily::Name(MONO_MEDIUM_FAMILY.into())
}

/// Builds the application's font definitions: Geist first in Proportional
/// and JetBrains Mono first in Monospace. Interface families fall back to
/// Phosphor icons, then JetBrains Mono (for symbols Geist lacks, such as the
/// macOS modifier keys `⌃⌥⌘`), then egui's default fonts (emoji, symbols).
pub fn font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    for (name, data) in [
        (GEIST, GEIST_REGULAR),
        (MEDIUM, GEIST_MEDIUM),
        (SEMIBOLD, GEIST_SEMIBOLD),
        (MONO, MONO_REGULAR),
        (MONO_MEDIUM_FAMILY, MONO_MEDIUM),
    ] {
        fonts
            .font_data
            .insert(name.into(), Arc::new(FontData::from_static(data)));
    }
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, MONO.into());

    // Registers the Phosphor data (and adds it to Proportional, which is
    // rebuilt below: Phosphor maps a-z to blank glyphs, so it always comes
    // after a text face).
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    let defaults: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .into_iter()
        .flatten()
        .filter(|name| *name != "phosphor")
        .cloned()
        .collect();

    // Icons first, then egui's emoji/symbol fallbacks (none of our faces,
    // whose Private Use Area glyphs could shadow the icons).
    let mut icon_family = vec!["phosphor".to_owned()];
    icon_family.extend(defaults.iter().cloned());
    fonts.families.insert(crate::icons::family(), icon_family);

    let mut fallbacks = vec!["phosphor".to_owned(), MONO.to_owned()];
    fallbacks.extend(defaults);
    for (family, first) in [
        (FontFamily::Proportional, GEIST),
        (medium_family(), MEDIUM),
        (semibold_family(), SEMIBOLD),
        (mono_medium_family(), MONO_MEDIUM_FAMILY),
    ] {
        let mut list = vec![first.to_owned()];
        list.extend(fallbacks.iter().filter(|name| *name != first).cloned());
        fonts.families.insert(family, list);
    }

    fonts
}

/// Installs the embedded fonts on the context.
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(font_definitions());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geist_and_jetbrains_mono_lead_their_families() {
        let fonts = font_definitions();
        assert_eq!(fonts.families[&FontFamily::Proportional][0], GEIST);
        assert_eq!(fonts.families[&FontFamily::Monospace][0], MONO);
        assert_eq!(fonts.families[&semibold_family()][0], SEMIBOLD);
        assert_eq!(fonts.families[&medium_family()][0], MEDIUM);
        assert_eq!(fonts.families[&mono_medium_family()][0], MONO_MEDIUM_FAMILY);
        // Icons resolve from every interface family.
        for family in [FontFamily::Proportional, semibold_family()] {
            assert!(fonts.families[&family].iter().any(|n| n == "phosphor"));
        }
        // Inter is a document font only.
        assert!(!fonts.font_data.keys().any(|name| name.contains("inter")));
    }

    #[test]
    fn font_licenses_ship_next_to_the_fonts() {
        for license in [
            include_str!("../../../assets/fonts/Geist-LICENSE.txt"),
            include_str!("../../../assets/fonts/JetBrainsMono-LICENSE.txt"),
        ] {
            assert!(license.contains("SIL Open Font License, Version 1.1"));
        }
    }
}
