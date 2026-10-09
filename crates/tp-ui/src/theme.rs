//! Maps design tokens onto egui's style and applies user scaling.

use egui::{
    Color32, CornerRadius, FontFamily, FontId, Margin, Shadow, Stroke, Style, TextStyle,
    ThemePreference, Vec2, Visuals, epaint::FontColorTransferFunction, style::WidgetVisuals,
};

use crate::fonts;
use crate::tokens::{color, radius, size, space, stroke, typography};

/// Bounds of the user-adjustable UI scale.
pub const UI_SCALE_RANGE: std::ops::RangeInclusive<f32> = 0.75..=2.0;
/// Bounds of the user-adjustable text size.
pub const TEXT_SCALE_RANGE: std::ops::RangeInclusive<f32> = 0.85..=1.5;

/// Custom text style for large screen titles.
pub fn title_style() -> TextStyle {
    TextStyle::Name("title".into())
}

/// Custom text style for the home screen hero text.
pub fn display_style() -> TextStyle {
    TextStyle::Name("display".into())
}

/// Custom text style for small uppercase-ish section labels.
pub fn label_strong_style() -> TextStyle {
    TextStyle::Name("label-strong".into())
}

/// User-controlled scaling, on top of the OS display scale factor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeSettings {
    /// Scales the whole interface (widgets, spacing, text).
    pub ui_scale: f32,
    /// Scales text only.
    pub text_scale: f32,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            text_scale: 1.0,
        }
    }
}

impl ThemeSettings {
    /// Returns a copy with both values clamped to their allowed range.
    pub fn clamped(self) -> Self {
        Self {
            ui_scale: self
                .ui_scale
                .clamp(*UI_SCALE_RANGE.start(), *UI_SCALE_RANGE.end()),
            text_scale: self
                .text_scale
                .clamp(*TEXT_SCALE_RANGE.start(), *TEXT_SCALE_RANGE.end()),
        }
    }
}

/// One-time setup: fonts and dark theme preference.
pub fn install(ctx: &egui::Context, settings: ThemeSettings) {
    fonts::install(ctx);
    ctx.set_theme(ThemePreference::Dark);
    apply(ctx, settings);
}

/// Applies (or re-applies) the theme with the given scaling. Cheap enough to
/// call whenever preferences change; takes effect on the next frame.
pub fn apply(ctx: &egui::Context, settings: ThemeSettings) {
    let settings = settings.clamped();
    // zoom_factor multiplies the native pixels-per-point, so the OS scale
    // factor (HiDPI) stays the 100% baseline.
    if (ctx.zoom_factor() - settings.ui_scale).abs() > f32::EPSILON {
        ctx.set_zoom_factor(settings.ui_scale);
    }
    ctx.all_styles_mut(|style| configure_style(style, settings.text_scale));
}

/// How glyph coverage turns into alpha on a display with
/// `pixels_per_point` physical pixels per point. egui's dark-mode curve
/// thickens edges, which suits HiDPI screens; on 1× screens it leaves hard,
/// stepped edges on dark-on-light text (the white pill, primary buttons),
/// so a softer gamma is used there.
pub fn text_smoothing(pixels_per_point: f32) -> FontColorTransferFunction {
    if pixels_per_point < 1.5 {
        FontColorTransferFunction::Gamma(0.7)
    } else {
        FontColorTransferFunction::DARK_MODE_DEFAULT
    }
}

/// Keeps the text smoothing matched to the current display; call every
/// frame (the window can move between a HiDPI and a 1× screen).
pub fn follow_display(ctx: &egui::Context) {
    let wanted = text_smoothing(ctx.pixels_per_point());
    if ctx
        .global_style()
        .visuals
        .text_options
        .color_transfer_function
        != wanted
    {
        ctx.all_styles_mut(|style| {
            style.visuals.text_options.color_transfer_function = wanted;
        });
    }
}

/// Fills `style` from the design tokens.
pub fn configure_style(style: &mut Style, text_scale: f32) {
    let t = |pt: f32| pt * text_scale;
    let semibold = fonts::semibold_family();
    style.text_styles = [
        (
            TextStyle::Small,
            FontId::new(t(typography::CAPTION), FontFamily::Proportional),
        ),
        (
            TextStyle::Body,
            FontId::new(t(typography::BODY), FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(t(typography::BODY), FontFamily::Proportional),
        ),
        (
            TextStyle::Heading,
            FontId::new(t(typography::HEADING), semibold.clone()),
        ),
        (
            TextStyle::Monospace,
            FontId::new(t(typography::MONO), FontFamily::Monospace),
        ),
        (
            title_style(),
            FontId::new(t(typography::TITLE), semibold.clone()),
        ),
        (
            display_style(),
            FontId::new(t(typography::DISPLAY), semibold.clone()),
        ),
        (
            label_strong_style(),
            FontId::new(t(typography::LABEL), semibold),
        ),
    ]
    .into();

    let spacing = &mut style.spacing;
    spacing.item_spacing = Vec2::new(space::SM, space::XS + 2.0);
    spacing.button_padding = Vec2::new(space::SM + 2.0, space::XS + 1.0);
    spacing.interact_size = Vec2::new(size::HIT_MIN * 2.0, size::HIT_MIN);
    spacing.window_margin = Margin::same(space::LG as i8);
    spacing.menu_margin = Margin::same(space::XS as i8 + 2);
    spacing.indent = space::LG;
    spacing.icon_width = 14.0;
    spacing.icon_width_inner = 8.0;
    spacing.icon_spacing = space::SM - 2.0;
    spacing.menu_width = 240.0;
    spacing.tooltip_width = 320.0;
    spacing.combo_height = 240.0;

    style.interaction.tooltip_delay = 0.45;
    style.interaction.show_tooltips_only_when_still = true;
    style.interaction.selectable_labels = false;
    style.animation_time = 0.1;
    style.explanation_tooltips = false;
    style.compact_menu_style = true;

    style.visuals = visuals();
}

fn widget(bg: Color32, outline: Color32, fg: Color32, corner: u8, expansion: f32) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: bg,
        bg_stroke: Stroke::new(stroke::HAIRLINE, outline),
        corner_radius: CornerRadius::same(corner),
        fg_stroke: Stroke::new(1.5, fg),
        expansion,
    }
}

/// The dark visuals derived from tokens.
pub fn visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.dark_mode = true;
    v.override_text_color = None;
    v.weak_text_color = Some(color::TEXT_SECONDARY);

    // Labels and frames that don't react.
    v.widgets.noninteractive = widget(
        color::SURFACE_1,
        color::BORDER,
        color::TEXT_SECONDARY,
        radius::MD,
        0.0,
    );
    // Idle controls sit on the control surface, outlined by a hairline
    // (text fields keep this outline over their sunken fill).
    v.widgets.inactive = widget(
        color::CONTROL,
        color::BORDER,
        color::TEXT_PRIMARY,
        radius::MD,
        0.0,
    );
    v.widgets.hovered = widget(
        color::SURFACE_3,
        color::BORDER_STRONG,
        color::TEXT_PRIMARY,
        radius::MD,
        0.0,
    );
    // Pressed: a darker fill change and a light outline, readable in grayscale.
    v.widgets.active = widget(
        color::SURFACE_4,
        color::TEXT_SECONDARY,
        color::TEXT_PRIMARY,
        radius::MD,
        0.0,
    );
    // An open menu (of the menu bar) or combo box sits on a chip.
    v.widgets.open = widget(
        color::CHIP,
        color::BORDER_STRONG,
        color::TEXT_PRIMARY,
        radius::MD,
        0.0,
    );

    // Selected items and text selections: a neutral fill (no accent) with ink
    // text; the focus ring and the outlines of focused fields are ink too.
    v.selection.bg_fill = color::SELECTED;
    v.selection.stroke = Stroke::new(stroke::FOCUS, color::FOCUS);

    v.hyperlink_color = color::TEXT_PRIMARY;
    v.faint_bg_color = color::SURFACE_2;
    v.extreme_bg_color = color::SURFACE_0;
    // Fields are sunken.
    v.text_edit_bg_color = Some(color::FIELD);
    v.code_bg_color = color::SURFACE_0;
    v.warn_fg_color = color::WARNING;
    v.error_fg_color = color::ERROR;

    v.window_corner_radius = CornerRadius::same(radius::LG);
    v.menu_corner_radius = CornerRadius::same(radius::LG + 2);
    // egui draws popups, menus and tooltips with the window fill: the
    // control surface. Modal dialogs use their own frame on the panel
    // surface.
    v.window_fill = color::CONTROL;
    v.window_stroke = Stroke::new(stroke::HAIRLINE, color::BORDER_STRONG);
    v.window_shadow = Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: color::SHADOW,
    };
    v.popup_shadow = Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: color::SHADOW,
    };
    v.window_highlight_topmost = false;
    v.panel_fill = color::SURFACE_1;
    v.button_frame = true;
    v.collapsing_header_frame = false;
    v.indent_has_left_vline = false;
    v.striped = false;
    v.slider_trailing_fill = true;
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn softer_text_smoothing_on_1x_displays() {
        assert_eq!(text_smoothing(1.0), FontColorTransferFunction::Gamma(0.7));
        assert_eq!(
            text_smoothing(2.0),
            FontColorTransferFunction::DARK_MODE_DEFAULT
        );
    }

    #[test]
    fn settings_are_clamped() {
        let s = ThemeSettings {
            ui_scale: 5.0,
            text_scale: 0.1,
        }
        .clamped();
        assert_eq!(s.ui_scale, 2.0);
        assert_eq!(s.text_scale, 0.85);
    }

    #[test]
    fn text_scale_affects_font_sizes() {
        let mut style = Style::default();
        configure_style(&mut style, 1.5);
        let body = &style.text_styles[&TextStyle::Body];
        assert!((body.size - typography::BODY * 1.5).abs() < 1e-4);
    }

    #[test]
    fn visuals_use_tokens() {
        let v = visuals();
        assert_eq!(v.panel_fill, color::SURFACE_1);
        assert_eq!(v.window_fill, color::CONTROL);
        assert_eq!(v.text_edit_bg_color, Some(color::FIELD));
        assert_eq!(v.widgets.inactive.bg_fill, color::CONTROL);
        assert_eq!(v.widgets.hovered.bg_fill, color::SURFACE_3);
        assert_eq!(v.selection.bg_fill, color::SELECTED);
    }

    #[test]
    fn widget_states_are_distinct() {
        let w = visuals().widgets;
        let fills = [
            w.inactive.bg_fill,
            w.hovered.bg_fill,
            w.active.bg_fill,
            visuals().selection.bg_fill,
        ];
        for (i, a) in fills.iter().enumerate() {
            for b in &fills[i + 1..] {
                assert_ne!(a, b, "two widget states share a fill");
            }
        }
    }

    #[test]
    fn type_scale() {
        let mut style = Style::default();
        configure_style(&mut style, 1.0);
        let size = |s: TextStyle| style.text_styles[&s].size;
        assert_eq!(size(title_style()), 22.0);
        assert_eq!(size(TextStyle::Heading), 15.0);
        assert_eq!(size(TextStyle::Body), 13.0);
        assert_eq!(size(TextStyle::Monospace), 12.0);
        assert_eq!(size(TextStyle::Small), 11.0);
        assert_eq!(
            style.text_styles[&TextStyle::Monospace].family,
            FontFamily::Monospace
        );
        assert_eq!(
            style.text_styles[&title_style()].family,
            fonts::semibold_family()
        );
    }
}
