use egui::{Align2, Rect, Response, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType};

use super::{IconButton, paint_focus_ring};
use crate::icons;
use crate::theme::label_strong_style;
use crate::tokens::{color, radius, size, space};

/// Header row of a collapsible, closable panel.
pub struct PanelHeader<'a> {
    icon: &'a str,
    title: &'a str,
    collapsed: bool,
}

/// What happened on a [`PanelHeader`] this frame.
pub struct PanelHeaderResponse {
    /// Response of the clickable header area (use it for context menus).
    pub response: Response,
    /// The header was clicked: toggle collapsed state.
    pub toggle: bool,
    /// The close button was clicked.
    pub close: bool,
}

impl<'a> PanelHeader<'a> {
    pub fn new(icon: &'a str, title: &'a str, collapsed: bool) -> Self {
        Self {
            icon,
            title,
            collapsed,
        }
    }

    pub fn show(self, ui: &mut Ui) -> PanelHeaderResponse {
        let width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, size::PANEL_HEADER_HEIGHT), Sense::click());
        let expanded = !self.collapsed;
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::CollapsingHeader, true, expanded, self.title)
        });

        let painter = ui.painter();
        let bg = if response.is_pointer_button_down_on() {
            color::SURFACE_4
        } else if response.hovered() {
            color::SURFACE_3
        } else {
            color::SURFACE_2
        };
        painter.rect_filled(rect, 0, bg);
        painter.hline(
            rect.x_range(),
            rect.bottom() - 0.5,
            Stroke::new(1.0, color::BORDER),
        );

        let mut x = rect.left() + space::SM;
        let caret = if self.collapsed {
            icons::COLLAPSED
        } else {
            icons::EXPANDED
        };
        let icon_font = crate::icons::font(size::ICON - 4.0);
        painter.text(
            egui::pos2(x + 5.0, rect.center().y),
            Align2::CENTER_CENTER,
            caret,
            icon_font,
            color::TEXT_SECONDARY,
        );
        x += 14.0;
        painter.text(
            egui::pos2(x + 7.0, rect.center().y),
            Align2::CENTER_CENTER,
            self.icon,
            crate::icons::font(size::ICON - 2.0),
            color::TEXT_SECONDARY,
        );
        x += 14.0 + space::SM - 2.0;
        let title_font = label_strong_style().resolve(ui.style());
        painter.text(
            egui::pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            self.title,
            title_font,
            color::TEXT_PRIMARY,
        );
        paint_focus_ring(ui, rect.shrink(1.0), &response, radius::SM);

        // Close button, right-aligned, on top of the header.
        let close_rect = Rect::from_center_size(
            egui::pos2(
                rect.right() - space::XS - size::HIT_MIN / 2.0,
                rect.center().y,
            ),
            Vec2::splat(size::HIT_MIN),
        );
        let close_label = format!("Close {}", self.title);
        let close = ui
            .put(close_rect, IconButton::new(icons::CLOSE, &close_label))
            .clicked();

        PanelHeaderResponse {
            toggle: response.clicked() && !close,
            close,
            response,
        }
    }
}
