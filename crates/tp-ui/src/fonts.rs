//! Embedded fonts: Inter for UI text, Phosphor for icons.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

const INTER_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf");

/// Family name for semibold UI text (headings, emphasized labels).
pub const SEMIBOLD: &str = "inter-semibold";

/// The semibold font family.
pub fn semibold_family() -> FontFamily {
    FontFamily::Name(SEMIBOLD.into())
}

/// Builds the application's font definitions: Inter first, Phosphor icons as
/// the first fallback, then egui's default fonts (emoji, symbols, monospace).
pub fn font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "inter".into(),
        Arc::new(FontData::from_static(INTER_REGULAR)),
    );
    fonts.font_data.insert(
        SEMIBOLD.into(),
        Arc::new(FontData::from_static(INTER_SEMIBOLD)),
    );

    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    proportional.insert(0, "inter".into());

    // Inserted at index 1 of Proportional and as its own family.
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    // Icons first, then egui's emoji/symbol fallbacks (but not Inter, whose
    // Private Use Area glyphs would shadow the icons).
    let mut icon_family: Vec<String> = vec!["phosphor".into()];
    icon_family.extend(
        fonts
            .families
            .get(&FontFamily::Proportional)
            .into_iter()
            .flatten()
            .filter(|name| *name != "inter" && *name != "phosphor")
            .cloned(),
    );
    fonts.families.insert(crate::icons::family(), icon_family);

    let mut semibold = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    semibold.retain(|name| name != "inter");
    semibold.insert(0, SEMIBOLD.into());
    fonts.families.insert(semibold_family(), semibold);

    fonts
}

/// Installs the embedded fonts on the context.
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(font_definitions());
}
