//! Professional widgets built from the design tokens.

mod buttons;
mod color_widgets;
mod empty_state;
mod gradient_bar;
mod menu_row;
mod numeric_field;
mod paint_row;
mod popover;
mod segmented;
mod thin_slider;
mod tool_button;

pub use buttons::{
    IconButton, SWITCH_SIZE, chip_toggle, dropdown_icon, more_menu, paint_dropdown_arrow,
    paint_switch, primary_button, secondary_button, secondary_icon_button, toggle_icon_button,
};
pub use color_widgets::{
    ColorSwatch, FillOrStroke, FillStrokeSwatches, GradientPreview, Hsv, PREVIEW_STOPS,
    SwatchColor, alpha_slider, hue_slider, paint_checkerboard, sv_square,
};
pub use empty_state::EmptyState;
pub use gradient_bar::{GradientBar, GradientBarEvent, keyboard_claimed};
pub use menu_row::MenuRow;
pub use numeric_field::{FieldEvent, NumericField, parse_number, remember_escape, take_escape};
pub use paint_row::{PAINT_ROW_HEIGHT, PaintRow, PaintRowResponse};
pub use popover::Popover;
pub use segmented::{Segment, SegmentedControl};
pub use thin_slider::ThinSlider;
pub use tool_button::ToolButton;

use egui::{CornerRadius, Rect, Response, Stroke, StrokeKind, Ui};

use crate::tokens::{color, stroke};

/// Paints the keyboard focus ring around `rect` when `response` has focus.
pub fn paint_focus_ring(ui: &Ui, rect: Rect, response: &Response, corner: u8) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(1.0),
            CornerRadius::same(corner.saturating_add(1)),
            Stroke::new(stroke::FOCUS, color::FOCUS),
            StrokeKind::Outside,
        );
    }
}

/// Shows a tooltip with a name and an optional shortcut, using the disabled
/// variant when the widget is disabled (spec: tooltips still explain disabled
/// widgets).
pub fn name_and_shortcut_tooltip(
    response: Response,
    name: &str,
    shortcut: Option<&str>,
    disabled_reason: Option<&str>,
) -> Response {
    let show = |ui: &mut Ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(name).color(color::TEXT_PRIMARY));
            if let Some(shortcut) = shortcut {
                ui.label(egui::RichText::new(shortcut).color(color::TEXT_SECONDARY));
            }
        });
        if let Some(reason) = disabled_reason {
            ui.label(egui::RichText::new(reason).color(color::TEXT_SECONDARY));
        }
    };
    if response.enabled() {
        response.on_hover_ui(show)
    } else {
        response.on_disabled_hover_ui(show)
    }
}
