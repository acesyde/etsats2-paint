use egui::{
    Align2, CornerRadius, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, WidgetType,
};

use super::{name_and_shortcut_tooltip, paint_focus_ring};
use crate::theme::label_strong_style;
use crate::tokens::{color, radius, size, space, stroke};

/// One choice of a [`SegmentedControl`].
pub struct Segment<'a, T> {
    pub value: T,
    pub icon: &'a str,
    pub label: &'a str,
    pub shortcut: Option<String>,
}

/// Row of mutually exclusive choices (e.g. 2D / 3D / Split).
///
/// The selected segment uses a raised fill, semibold text and an accent
/// underline, so selection does not depend on color alone.
pub struct SegmentedControl<'a, T> {
    segments: Vec<Segment<'a, T>>,
}

impl<'a, T: Copy + PartialEq> SegmentedControl<'a, T> {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn segment(
        mut self,
        value: T,
        icon: &'a str,
        label: &'a str,
        shortcut: Option<String>,
    ) -> Self {
        self.segments.push(Segment {
            value,
            icon,
            label,
            shortcut,
        });
        self
    }

    /// Draws the control; returns the newly picked value, if any.
    pub fn show(self, ui: &mut Ui, current: T) -> Option<T> {
        let mut picked = None;
        let height = size::HIT_MIN + 2.0;
        let body = egui::TextStyle::Button.resolve(ui.style());
        let strong = label_strong_style().resolve(ui.style());
        let icon_font = crate::icons::font(size::ICON - 2.0);

        let widths: Vec<f32> = self
            .segments
            .iter()
            .map(|s| {
                let text = ui
                    .painter()
                    .layout_no_wrap(s.label.to_owned(), strong.clone(), color::TEXT_PRIMARY)
                    .size()
                    .x;
                text + size::ICON + space::SM * 2.0 + space::XS
            })
            .collect();
        let total = Vec2::new(widths.iter().sum::<f32>() + 4.0, height);
        let (outer, _) = ui.allocate_exact_size(total, Sense::hover());
        ui.painter().rect(
            outer,
            CornerRadius::same(radius::MD),
            color::SURFACE_0,
            Stroke::new(1.0, color::BORDER),
            StrokeKind::Inside,
        );

        let mut x = outer.left() + 2.0;
        for (segment, width) in self.segments.into_iter().zip(widths) {
            let rect = Rect::from_min_size(
                egui::pos2(x, outer.top() + 2.0),
                Vec2::new(width, height - 4.0),
            );
            x += width;
            let id = ui.id().with(("segment", segment.label));
            let response = ui.interact(rect, id, Sense::click());
            let selected = segment.value == current;
            let enabled = ui.is_enabled();
            response.widget_info(|| {
                WidgetInfo::selected(WidgetType::RadioButton, enabled, selected, segment.label)
            });
            let painter = ui.painter();
            if selected {
                painter.rect_filled(rect, CornerRadius::same(radius::SM), color::SURFACE_4);
                painter.hline(
                    rect.x_range().shrink(space::SM),
                    rect.bottom() - 1.5,
                    Stroke::new(stroke::FOCUS + 0.5, color::ACCENT),
                );
            } else if response.hovered() {
                painter.rect_filled(rect, CornerRadius::same(radius::SM), color::SURFACE_3);
            }
            let fg = if selected || response.hovered() {
                color::TEXT_PRIMARY
            } else {
                color::TEXT_SECONDARY
            };
            let icon_x = rect.left() + space::SM + size::ICON / 2.0 - 2.0;
            painter.text(
                egui::pos2(icon_x, rect.center().y),
                Align2::CENTER_CENTER,
                segment.icon,
                icon_font.clone(),
                fg,
            );
            painter.text(
                egui::pos2(icon_x + size::ICON / 2.0 + space::XS, rect.center().y),
                Align2::LEFT_CENTER,
                segment.label,
                if selected {
                    strong.clone()
                } else {
                    body.clone()
                },
                fg,
            );
            paint_focus_ring(ui, rect, &response, radius::SM);
            let response = name_and_shortcut_tooltip(
                response,
                segment.label,
                segment.shortcut.as_deref(),
                None,
            );
            if response.clicked() && !selected {
                picked = Some(segment.value);
            }
        }
        picked
    }
}

impl<T: Copy + PartialEq> Default for SegmentedControl<'_, T> {
    fn default() -> Self {
        Self::new()
    }
}
