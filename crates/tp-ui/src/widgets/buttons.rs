use egui::{
    Align2, Button, CornerRadius, Response, RichText, Sense, Stroke, Ui, Vec2, Widget, WidgetInfo,
    WidgetType,
};

use super::{name_and_shortcut_tooltip, paint_focus_ring};
use crate::tokens::{color, radius, size};

/// Small icon-only button. A tooltip name is mandatory (spec: icon-only
/// controls always have a tooltip).
pub struct IconButton<'a> {
    icon: &'a str,
    name: &'a str,
    shortcut: Option<&'a str>,
    selected: bool,
    disabled_reason: Option<&'a str>,
}

impl<'a> IconButton<'a> {
    pub fn new(icon: &'a str, name: &'a str) -> Self {
        Self {
            icon,
            name,
            shortcut: None,
            selected: false,
            disabled_reason: None,
        }
    }

    pub fn shortcut(mut self, shortcut: Option<&'a str>) -> Self {
        self.shortcut = shortcut;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Explanation shown in the tooltip when the button is disabled.
    pub fn disabled_reason(mut self, reason: &'a str) -> Self {
        self.disabled_reason = Some(reason);
        self
    }
}

impl Widget for IconButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size::HIT_MIN), Sense::click());
        let enabled = ui.is_enabled();
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Button, enabled, self.selected, self.name)
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let corner = CornerRadius::same(radius::SM);
            let fg = if !enabled {
                color::TEXT_DISABLED
            } else if response.hovered() || self.selected {
                color::TEXT_PRIMARY
            } else {
                color::TEXT_SECONDARY
            };
            if enabled && response.is_pointer_button_down_on() {
                painter.rect_filled(rect, corner, color::SURFACE_4);
            } else if enabled && response.hovered() {
                painter.rect_filled(rect, corner, color::SURFACE_3);
            } else if self.selected {
                painter.rect_filled(rect, corner, color::SELECTED);
            }
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                self.icon,
                crate::icons::font(size::ICON),
                fg,
            );
            paint_focus_ring(ui, rect, &response, radius::SM);
        }

        name_and_shortcut_tooltip(response, self.name, self.shortcut, self.disabled_reason)
    }
}

/// Icon button with two states (e.g. eye open/closed, lock open/closed): the
/// state is shown by the glyph shape, and the name says what a click does.
/// Returns true when clicked.
pub fn toggle_icon_button(
    ui: &mut Ui,
    on: bool,
    on_icon: &str,
    off_icon: &str,
    name_when_on: &str,
    name_when_off: &str,
) -> bool {
    let (icon, name) = if on {
        (on_icon, name_when_on)
    } else {
        (off_icon, name_when_off)
    };
    ui.add(IconButton::new(icon, name).selected(!on)).clicked()
}

/// Call-to-action button: the primary accent (white) with dark text.
pub fn primary_button(text: &str) -> Button<'static> {
    Button::new(RichText::new(text.to_owned()).color(color::TEXT_ON_PRIMARY))
        .fill(color::ACCENT_PRIMARY)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(radius::MD))
        .min_size(Vec2::new(88.0, 30.0))
}

/// Neutral button.
pub fn secondary_button(text: &str) -> Button<'static> {
    Button::new(text.to_owned())
        .corner_radius(CornerRadius::same(radius::MD))
        .min_size(Vec2::new(88.0, 30.0))
}
