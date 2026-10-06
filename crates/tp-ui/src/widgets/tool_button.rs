use egui::{
    Align2, CornerRadius, Pos2, Rect, Response, Sense, Ui, Vec2, Widget, WidgetInfo, WidgetType,
};

use super::{name_and_shortcut_tooltip, paint_focus_ring};
use crate::tokens::{color, radius, size, stroke};

/// Square icon button for the tool bar.
///
/// The active state is shown by a filled background *and* an indicator bar
/// on the leading edge, so it remains visible without color perception.
pub struct ToolButton<'a> {
    icon: &'a str,
    name: &'a str,
    shortcut: Option<&'a str>,
    active: bool,
}

impl<'a> ToolButton<'a> {
    pub fn new(icon: &'a str, name: &'a str) -> Self {
        Self {
            icon,
            name,
            shortcut: None,
            active: false,
        }
    }

    /// Formatted shortcut shown in the tooltip.
    pub fn shortcut(mut self, shortcut: Option<&'a str>) -> Self {
        self.shortcut = shortcut;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl Widget for ToolButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(size::TOOL_BUTTON), Sense::click());
        let enabled = ui.is_enabled();
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::Button, enabled, self.active, self.name)
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let corner = CornerRadius::same(radius::MD);
            let pressed = response.is_pointer_button_down_on();
            let hovered = response.hovered() && enabled;

            let (bg, fg) = if !enabled {
                (None, color::TEXT_DISABLED)
            } else if self.active {
                (Some(color::ACCENT_SUBTLE), color::TEXT_PRIMARY)
            } else if pressed {
                (Some(color::SURFACE_4), color::TEXT_PRIMARY)
            } else if hovered {
                (Some(color::SURFACE_3), color::TEXT_PRIMARY)
            } else {
                (None, color::TEXT_SECONDARY)
            };
            if let Some(bg) = bg {
                painter.rect_filled(rect, corner, bg);
            }
            if self.active {
                let bar = Rect::from_min_max(
                    Pos2::new(rect.left(), rect.top() + 6.0),
                    Pos2::new(rect.left() + stroke::INDICATOR, rect.bottom() - 6.0),
                );
                painter.rect_filled(bar, CornerRadius::same(2), color::ACCENT);
            }
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                self.icon,
                crate::icons::font(size::ICON_LG),
                fg,
            );
            paint_focus_ring(ui, rect, &response, radius::MD);
        }

        name_and_shortcut_tooltip(response, self.name, self.shortcut, None)
    }
}
