//! The shadow popover of the inspector's Shadow row: the shadow's color
//! (picker, hex and palette), opacity, offset and blur; and the row's
//! summary ("8 / 8 · 8", with "Offset 8 / 8, blur 8" on hover).

use egui::{RichText, Ui};
use tp_core::document::{MAX_SHADOW_BLUR, MAX_SHADOW_OFFSET, Shadow};
use tp_i18n::{format_number, tr};
use tp_ui::tokens::color;
use tp_ui::widgets::NumericField;

use super::properties::common;
use super::{PanelEnv, apply_field, colors};

/// The Shadow row's summary for `shadows` (one per selected object that
/// can have one): offset and blur, or "Mixed" when they differ.
pub fn summary(shadows: &[Option<Shadow>]) -> String {
    let same = common(shadows.iter().map(|s| s.map(|s| (s.offset, s.blur))));
    match same {
        Some(Some((offset, blur))) => tr!(
            "inspector-shadow-summary",
            x = format_number(offset.x, 1),
            y = format_number(offset.y, 1),
            blur = format_number(blur, 1)
        ),
        _ => tr("mixed"),
    }
}

/// The full text of the Shadow row's summary ("Offset 8 / 8, blur 8"),
/// shown on hover; `None` when the shadows differ.
pub fn summary_full(shadows: &[Option<Shadow>]) -> Option<String> {
    match common(shadows.iter().map(|s| s.map(|s| (s.offset, s.blur)))) {
        Some(Some((offset, blur))) => Some(tr!(
            "inspector-shadow-summary-full",
            x = format_number(offset.x, 1),
            y = format_number(offset.y, 1),
            blur = format_number(blur, 1)
        )),
        _ => None,
    }
}

/// The shadow popover's content.
pub fn popover(ui: &mut Ui, env: &mut PanelEnv<'_>) {
    ui.horizontal(|ui| {
        ui.set_min_height(tp_ui::tokens::size::HIT_MIN);
        ui.label(
            RichText::new(tr("inspector-shadow"))
                .text_style(tp_ui::theme::label_strong_style())
                .color(color::TEXT_PRIMARY),
        );
    });
    let shadows: Vec<Shadow> = env.ws.selected_shadows().into_iter().flatten().collect();
    if shadows.is_empty() {
        return;
    }
    let shown = |f: fn(&Shadow) -> f64| common(shadows.iter().map(f));
    let color = common(shadows.iter().map(|s| s.color));
    let linked = common(shadows.iter().map(|s| s.swatch)).flatten();
    colors::shadow_color(ui, env, color, linked);

    let opacity = shown(|s| f64::from(s.opacity) * 100.0);
    let e = NumericField::new(&tr("props-opacity"), &tr("shadow-opacity"), opacity)
        .suffix("%")
        .decimals(0)
        .range(0.0..=100.0)
        .width(48.0)
        .show(ui);
    apply_field(env, e, |ws, v| {
        ws.set_shadow(|s| s.opacity = (v / 100.0) as f32);
    });
    ui.horizontal(|ui| {
        let offset = MAX_SHADOW_OFFSET;
        let e = NumericField::new("X", &tr("shadow-offset-x"), shown(|s| s.offset.x))
            .suffix("px")
            .decimals(1)
            .range(-offset..=offset)
            .width(48.0)
            .show(ui);
        apply_field(env, e, |ws, v| ws.set_shadow(|s| s.offset.x = v));
        let e = NumericField::new("Y", &tr("shadow-offset-y"), shown(|s| s.offset.y))
            .suffix("px")
            .decimals(1)
            .range(-offset..=offset)
            .width(48.0)
            .show(ui);
        apply_field(env, e, |ws, v| ws.set_shadow(|s| s.offset.y = v));
    });
    let e = NumericField::new(&tr("shadow-blur"), &tr("shadow-blur"), shown(|s| s.blur))
        .suffix("px")
        .decimals(1)
        .speed(0.5)
        .range(0.0..=MAX_SHADOW_BLUR)
        .width(48.0)
        .show(ui);
    apply_field(env, e, |ws, v| ws.set_shadow(|s| s.blur = v));
}
