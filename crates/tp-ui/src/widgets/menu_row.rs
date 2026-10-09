use egui::{Align2, Response, Sense, TextStyle, Ui, Vec2, Widget, WidgetInfo, WidgetType};

use crate::icons;
use crate::tokens::{color, radius, size, space, typography};

/// Menu item with an optional leading icon / check mark, a label and a
/// right-aligned shortcut in the platform notation.
pub struct MenuRow<'a> {
    label: &'a str,
    icon: Option<&'a str>,
    shortcut: Option<&'a str>,
    checked: Option<bool>,
}

impl<'a> MenuRow<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            shortcut: None,
            checked: None,
        }
    }

    pub fn icon(mut self, icon: Option<&'a str>) -> Self {
        self.icon = icon;
        self
    }

    pub fn shortcut(mut self, shortcut: Option<&'a str>) -> Self {
        self.shortcut = shortcut;
        self
    }

    /// Makes the row a toggle; a check mark is shown when `true`.
    pub fn checked(mut self, checked: Option<bool>) -> Self {
        self.checked = checked;
        self
    }
}

const ROW_HEIGHT: f32 = 28.0;
/// Padding before the label, and the column of check marks (and icons)
/// when the menu has one.
const PAD: f32 = space::SM + 2.0;
const LEADING: f32 = 20.0;

impl Widget for MenuRow<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let body = TextStyle::Body.resolve(ui.style());
        // Shortcuts in the mono face, as values.
        let small = egui::FontId::monospace(body.size * typography::CAPTION / typography::BODY);
        // The leading column is kept by every row of a menu once one of its
        // rows has a check mark or an icon, so labels stay aligned.
        let column_id = ui.id().with("menu_row_leading");
        if self.checked.is_some() || self.icon.is_some() {
            ui.data_mut(|d| d.insert_temp(column_id, true));
        }
        let leading_width = if ui.data(|d| d.get_temp::<bool>(column_id).unwrap_or(false)) {
            LEADING
        } else {
            0.0
        };
        let label_width = ui
            .painter()
            .layout_no_wrap(self.label.to_owned(), body.clone(), color::TEXT_PRIMARY)
            .size()
            .x;
        let shortcut_width = self.shortcut.map_or(0.0, |s| {
            ui.painter()
                .layout_no_wrap(s.to_owned(), small.clone(), color::TEXT_MUTED)
                .size()
                .x
                + space::XL
        });
        let desired = Vec2::new(
            PAD + leading_width + label_width + shortcut_width + PAD,
            ROW_HEIGHT.max(size::HIT_MIN),
        );
        // Menus use a justified layout: rows stretch to the widest one.
        let (rect, response) = ui.allocate_at_least(desired, Sense::click());
        let enabled = ui.is_enabled();
        response.widget_info(|| match self.checked {
            Some(checked) => {
                WidgetInfo::selected(WidgetType::Checkbox, enabled, checked, self.label)
            }
            None => WidgetInfo::labeled(WidgetType::Button, enabled, self.label),
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let highlighted = enabled && (response.hovered() || response.has_focus());
            if highlighted {
                painter.rect_filled(rect, radius::MD, color::SURFACE_3);
            }
            // Muted shortcuts lose contrast on the hover fill: secondary
            // there.
            let (fg, fg_weak) = if !enabled {
                (color::TEXT_DISABLED, color::TEXT_DISABLED)
            } else if highlighted {
                (color::TEXT_PRIMARY, color::TEXT_SECONDARY)
            } else {
                (color::TEXT_PRIMARY, color::TEXT_MUTED)
            };
            let leading = match (self.checked, self.icon) {
                (Some(true), _) => Some(icons::CHECK),
                (Some(false), _) => None,
                (None, icon) => icon,
            };
            if let Some(glyph) = leading {
                painter.text(
                    egui::pos2(rect.left() + PAD + LEADING / 2.0 - 3.0, rect.center().y),
                    Align2::CENTER_CENTER,
                    glyph,
                    crate::icons::font(size::ICON - 2.0),
                    fg,
                );
            }
            painter.text(
                egui::pos2(rect.left() + PAD + leading_width, rect.center().y),
                Align2::LEFT_CENTER,
                self.label,
                body,
                fg,
            );
            if let Some(shortcut) = self.shortcut {
                painter.text(
                    egui::pos2(rect.right() - PAD, rect.center().y),
                    Align2::RIGHT_CENTER,
                    shortcut,
                    small,
                    fg_weak,
                );
            }
        }
        response
    }
}
