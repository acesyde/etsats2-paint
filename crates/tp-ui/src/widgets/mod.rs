//! Professional widgets built from the design tokens.

mod buttons;
mod checkbox;
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
pub use checkbox::{CHECK_BOX_SIZE, SquareCheckbox, paint_check_box};
pub use color_widgets::{
    ColorSwatch, FillOrStroke, FillStrokeSwatches, GradientPreview, Hsv, PREVIEW_STOPS,
    SwatchColor, alpha_slider, hue_slider, paint_checkerboard, sv_square,
};
pub use empty_state::EmptyState;
pub use gradient_bar::{GradientBar, GradientBarEvent, keyboard_claimed};
pub use menu_row::MenuRow;
pub use numeric_field::{
    FieldEvent, NumericField, label_column, parse_number, remember_escape, take_escape,
};
pub use paint_row::{PAINT_ROW_HEIGHT, PaintRow, PaintRowResponse};
pub use popover::Popover;
pub use segmented::{Segment, SegmentedControl};
pub use thin_slider::ThinSlider;
pub use tool_button::ToolButton;

use egui::{CornerRadius, Rect, Response, Stroke, StrokeKind, Ui, WidgetText};

use crate::tokens::{color, stroke};

/// The largest rect of `size`'s shape that fits in `outer`, centered
/// (aspect-fit: a picture drawn there is never stretched).
pub fn fit_rect(outer: Rect, size: egui::Vec2) -> Rect {
    if size.x <= 0.0 || size.y <= 0.0 {
        return outer;
    }
    let k = (outer.width() / size.x).min(outer.height() / size.y);
    Rect::from_center_size(outer.center(), size * k)
}

/// Where a picture of `size`'s shape is drawn in `tile`, and the part of it
/// shown there (uv, 0..1), never stretched. A picture at least as wide as
/// tall fills the tile's width: centered with room above and below when it
/// is wider than the tile, its middle shown when it is taller (a square in
/// a landscape tile). A taller picture is fitted whole.
pub fn thumbnail_place(tile: Rect, size: egui::Vec2) -> (Rect, Rect) {
    let full = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    if size.x <= 0.0 || size.y <= 0.0 || size.x < size.y {
        return (fit_rect(tile, size), full);
    }
    let height = tile.width() * size.y / size.x;
    if height <= tile.height() {
        let rect = Rect::from_center_size(tile.center(), egui::vec2(tile.width(), height));
        (rect, full)
    } else {
        let shown = tile.height() / height;
        let uv = Rect::from_center_size(egui::pos2(0.5, 0.5), egui::vec2(1.0, shown));
        (tile, uv)
    }
}

/// A single-line label at most `width` wide (capped at the room left): a
/// longer text ends with "…" and shows in full in a tooltip, and is never
/// cut mid-letter or wrapped. Its accessible name is the full text.
pub fn truncated_label(ui: &mut Ui, text: impl Into<WidgetText>, width: f32) -> Response {
    let width = width.min(ui.available_width()).max(0.0);
    // A scope in the same layout, narrowed: the label sits where any other
    // widget would (centered in a row, on the right in a right-to-left one).
    ui.scope(|ui| {
        ui.set_max_width(width);
        // egui's truncation shows the full text on hover when elided.
        ui.add(
            egui::Label::new(text)
                .truncate()
                .show_tooltip_when_elided(true)
                .selectable(false),
        )
    })
    .inner
}

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

#[cfg(test)]
mod tests {
    use egui::{Rect, pos2, vec2};

    use super::{fit_rect, thumbnail_place};

    #[test]
    fn fit_keeps_the_shape_centered() {
        let tile = Rect::from_min_size(pos2(0.0, 0.0), vec2(26.0, 26.0));
        let r = fit_rect(tile, vec2(8192.0, 1024.0));
        assert_eq!(r.size(), vec2(26.0, 3.25));
        assert_eq!(r.center(), tile.center());
    }

    #[test]
    fn thumbnails_are_never_stretched() {
        let tile = Rect::from_min_size(pos2(0.0, 0.0), vec2(150.0, 108.0));
        // A square fills the width, its middle shown.
        let (rect, uv) = thumbnail_place(tile, vec2(4096.0, 4096.0));
        assert_eq!(rect, tile);
        assert!((uv.height() - 108.0 / 150.0).abs() < 1e-6 && uv.width() == 1.0);
        // A wide texture is shown whole, centered.
        let (rect, uv) = thumbnail_place(tile, vec2(8192.0, 1024.0));
        assert_eq!(rect.size(), vec2(150.0, 18.75));
        assert_eq!(rect.center(), tile.center());
        assert_eq!(uv.size(), vec2(1.0, 1.0));
        // A tall one is fitted whole.
        let (rect, _) = thumbnail_place(tile, vec2(1024.0, 2048.0));
        assert_eq!(rect.size(), vec2(54.0, 108.0));
    }
}
