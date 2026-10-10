use egui::{
    Align2, CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, TextStyle, TextWrapMode, Ui,
    Vec2, WidgetInfo, WidgetText, WidgetType,
};

use super::IconButton;
use super::color_widgets::paint_swatch;
use super::{SwatchColor, paint_focus_ring};
use crate::icons;
use crate::tokens::{color, radius, size, space, stroke, typography};

/// Height of a paint row.
pub const PAINT_ROW_HEIGHT: f32 = 32.0;

/// One row of the inspector's Appearance section (Fill, Stroke): the
/// paint's swatch, the row's name and its value ("#5B8DEF", a linked
/// swatch's name, "Linear", "Mixed"). The swatch side opens the color
/// popover; an optional summary at the right end ("6 px · Outside") opens
/// the row's own settings, and an optional icon button at the very end
/// (the Shadow row's remove button) acts on the row.
///
/// The row the color editor targets is outlined, and reported as selected
/// to assistive technology, so the target does not rely on color alone. A
/// value linked to the brand is drawn in the link color after a link icon.
pub struct PaintRow<'a> {
    swatch: SwatchColor,
    label: &'a str,
    name: &'a str,
    value: &'a str,
    linked: bool,
    target: bool,
    summary: Option<(&'a str, &'a str)>,
    action: Option<(&'a str, &'a str)>,
    hover: Option<&'a str>,
}

/// What a [`PaintRow`] reports.
pub struct PaintRowResponse {
    /// The whole row (the anchor of its popovers).
    pub row: Rect,
    /// The swatch side: swatch, name and value.
    pub swatch: Response,
    /// The summary at the right end, when the row has one.
    pub summary: Option<Response>,
    /// The icon button at the right end, when the row has one.
    pub action: Option<Response>,
}

impl<'a> PaintRow<'a> {
    /// `label` is the visible name ("Fill"), `name` the accessible one
    /// ("Fill color").
    pub fn new(swatch: SwatchColor, label: &'a str, name: &'a str) -> Self {
        Self {
            swatch,
            label,
            name,
            value: "",
            linked: false,
            target: false,
            summary: None,
            action: None,
            hover: None,
        }
    }

    /// The paint's value, in the link color with a link icon when `linked`.
    pub fn value(mut self, value: &'a str, linked: bool) -> Self {
        self.value = value;
        self.linked = linked;
        self
    }

    /// Outlines the row: the color editor edits it.
    pub fn target(mut self, target: bool) -> Self {
        self.target = target;
        self
    }

    /// A clickable summary at the right end (`text`, accessible `name`).
    pub fn summary(mut self, text: &'a str, name: &'a str) -> Self {
        self.summary = Some((text, name));
        self
    }

    /// An icon button at the right end, inside the row (`icon`, accessible
    /// name and tooltip `name`).
    pub fn action(mut self, icon: &'a str, name: &'a str) -> Self {
        self.action = Some((icon, name));
        self
    }

    /// The tooltip of the swatch side, instead of the accessible name (the
    /// full text of a shortened value).
    pub fn hover_text(mut self, text: &'a str) -> Self {
        self.hover = Some(text);
        self
    }

    pub fn show(self, ui: &mut Ui) -> PaintRowResponse {
        let (row, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), PAINT_ROW_HEIGHT),
            Sense::hover(),
        );
        let mono = egui::FontId::monospace(typography::CAPTION);

        // The action button at the very end, then the summary before it.
        let action_rect = self.action.map(|_| {
            Rect::from_min_size(
                egui::pos2(
                    row.right() - 4.0 - size::HIT_MIN,
                    row.center().y - size::HIT_MIN / 2.0,
                ),
                Vec2::splat(size::HIT_MIN),
            )
        });
        let end = action_rect.map_or(row.right() - 2.0, |r| r.left() - 2.0);

        // The summary first: the swatch side takes what is left.
        let summary = self.summary.map(|(text, name)| {
            let galley =
                ui.painter()
                    .layout_no_wrap(text.to_owned(), mono.clone(), color::TEXT_SECONDARY);
            let width = galley.size().x + 2.0 * space::SM;
            let rect = Rect::from_min_max(
                egui::pos2(end - width, row.top() + 3.0),
                egui::pos2(end, row.bottom() - 3.0),
            );
            let response = ui.interact(
                rect,
                ui.id().with(("paint_row_summary", name)),
                Sense::click(),
            );
            response.widget_info(|| {
                let mut info = WidgetInfo::labeled(WidgetType::Button, true, name);
                info.current_text_value = Some(text.to_owned());
                info
            });
            (rect, galley, response.on_hover_text(name))
        });
        let swatch_side = match &summary {
            Some((rect, ..)) => Rect::from_min_max(row.min, egui::pos2(rect.left(), row.max.y)),
            None => match action_rect {
                Some(rect) => Rect::from_min_max(row.min, egui::pos2(rect.left(), row.max.y)),
                None => row,
            },
        };
        let swatch = ui.interact(
            swatch_side,
            ui.id().with(("paint_row", self.name)),
            Sense::click(),
        );
        swatch.widget_info(|| {
            let mut info = WidgetInfo::selected(WidgetType::Button, true, self.target, self.name);
            if !self.value.is_empty() {
                info.current_text_value = Some(self.value.to_owned());
            }
            info
        });

        if ui.is_rect_visible(row) {
            let painter = ui.painter();
            let corner = CornerRadius::same(radius::MD);
            if self.target {
                painter.rect(
                    row,
                    corner,
                    color::SURFACE_3,
                    Stroke::new(stroke::HAIRLINE, color::BORDER_STRONG),
                    StrokeKind::Inside,
                );
            } else if swatch.hovered() {
                painter.rect_filled(row, corner, color::SURFACE_2);
            }
            let chip = Rect::from_center_size(
                egui::pos2(row.left() + space::SM + 10.0, row.center().y),
                Vec2::splat(20.0),
            );
            paint_swatch(painter, chip, self.swatch);
            painter.rect_stroke(
                chip,
                CornerRadius::same(radius::SM),
                Stroke::new(stroke::HAIRLINE, color::BORDER_STRONG),
                StrokeKind::Outside,
            );
            paint_focus_ring(ui, chip, &swatch, radius::SM);

            // Name, then the value at the right end of the swatch side,
            // shortened when it does not fit.
            let body = TextStyle::Body.resolve(ui.style());
            let name_x = chip.right() + space::SM + 2.0;
            let name = painter.layout_no_wrap(self.label.to_owned(), body, color::TEXT_PRIMARY);
            let name_width = name.size().x;
            painter.galley(
                egui::pos2(name_x, row.center().y - name.size().y / 2.0),
                name,
                color::TEXT_PRIMARY,
            );
            if !self.value.is_empty() {
                let right = swatch_side.right() - space::SM;
                let left = name_x + name_width + space::SM;
                let ink = if self.linked {
                    color::LINK
                } else {
                    color::TEXT_SECONDARY
                };
                let mut max = right - left;
                let mut x_end = right;
                if self.linked {
                    max -= 14.0;
                }
                if max > 8.0 {
                    let text =
                        WidgetText::from(egui::RichText::new(self.value).font(mono).color(ink));
                    let galley = text.into_galley(
                        ui,
                        Some(TextWrapMode::Truncate),
                        max,
                        TextStyle::Monospace,
                    );
                    let pos = egui::pos2(
                        x_end - galley.size().x,
                        row.center().y - galley.size().y / 2.0,
                    );
                    x_end = pos.x;
                    ui.painter().galley(pos, galley, ink);
                    if self.linked {
                        ui.painter().text(
                            egui::pos2(x_end - 3.0, row.center().y),
                            Align2::RIGHT_CENTER,
                            icons::LINKED,
                            icons::font(11.0),
                            color::LINK,
                        );
                    }
                }
            }
        }

        let summary = summary.map(|(rect, galley, response)| {
            if ui.is_rect_visible(rect) {
                if response.hovered() {
                    ui.painter().rect_filled(rect, radius::SM, color::SURFACE_4);
                }
                let pos = egui::pos2(
                    rect.right() - space::SM - galley.size().x,
                    rect.center().y - galley.size().y / 2.0,
                );
                ui.painter().galley(pos, galley, color::TEXT_SECONDARY);
                paint_focus_ring(ui, rect, &response, radius::SM);
            }
            response
        });
        let action = self
            .action
            .zip(action_rect)
            .map(|((icon, name), rect)| ui.put(rect, IconButton::new(icon, name)));
        PaintRowResponse {
            row,
            swatch: swatch.on_hover_text(self.hover.unwrap_or(self.name)),
            summary,
            action,
        }
    }
}
