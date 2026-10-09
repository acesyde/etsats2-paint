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
    /// Accessible name and tooltip (the label unless set).
    pub name: &'a str,
    pub shortcut: Option<String>,
}

/// Row of mutually exclusive choices (e.g. Solid / Linear / Radial).
///
/// The options sit side by side in one raised track; the active one is a
/// white pill (primary accent) with dark semibold text, the others plain
/// text on the track. The pill is a fill change, so the active option stays
/// visible in grayscale.
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
            name: label,
            shortcut,
        });
        self
    }

    /// A segment whose accessible name and tooltip differ from its label
    /// (e.g. "Round cap" and "Round join" both labelled "Round").
    pub fn named_segment(mut self, value: T, icon: &'a str, label: &'a str, name: &'a str) -> Self {
        self.segments.push(Segment {
            value,
            icon,
            label,
            name,
            shortcut: None,
        });
        self
    }

    /// Draws the control; returns the newly picked value, if any.
    pub fn show(self, ui: &mut Ui, current: T) -> Option<T> {
        let mut picked = None;
        // Inner padding of the track, gap between options, horizontal
        // padding of an option.
        const INSET: f32 = 2.0;
        const GAP: f32 = 1.0;
        const PAD_X: f32 = space::SM;
        let body = egui::TextStyle::Button.resolve(ui.style());
        let strong = label_strong_style().resolve(ui.style());
        let icon_font = crate::icons::font(size::ICON - 2.0);
        let icon_width = |icon: &str| {
            if icon.is_empty() {
                0.0
            } else {
                size::ICON + space::XS
            }
        };

        // Widths follow the (wider) semibold text so options don't move when
        // the active one changes.
        let widths: Vec<f32> = self
            .segments
            .iter()
            .map(|s| {
                let text = ui
                    .painter()
                    .layout_no_wrap(s.label.to_owned(), strong.clone(), color::TEXT_PRIMARY)
                    .size()
                    .x;
                text + icon_width(s.icon) + PAD_X * 2.0
            })
            .collect();
        let gaps = GAP * self.segments.len().saturating_sub(1) as f32;
        let total = Vec2::new(
            widths.iter().sum::<f32>() + gaps + INSET * 2.0,
            size::HIT_MIN + INSET * 2.0,
        );
        let (outer, _) = ui.allocate_exact_size(total, Sense::hover());
        ui.painter().rect(
            outer,
            CornerRadius::same(radius::LG),
            color::RAISED,
            Stroke::new(stroke::HAIRLINE, color::BORDER),
            StrokeKind::Inside,
        );

        let enabled = ui.is_enabled();
        let mut x = outer.left() + INSET;
        for (segment, width) in self.segments.into_iter().zip(widths) {
            let rect = Rect::from_min_size(
                egui::pos2(x, outer.top() + INSET),
                Vec2::new(width, size::HIT_MIN),
            );
            x += width + GAP;
            let id = ui.id().with(("segment", segment.name));
            let response = ui.interact(rect, id, Sense::click());
            let selected = segment.value == current;
            response.widget_info(|| {
                WidgetInfo::selected(WidgetType::RadioButton, enabled, selected, segment.name)
            });
            let painter = ui.painter();
            let hovered = enabled && response.hovered();
            if selected {
                painter.rect_filled(rect, CornerRadius::same(radius::MD), color::ACCENT_PRIMARY);
            } else if hovered {
                painter.rect_filled(rect, CornerRadius::same(radius::MD), color::SURFACE_3);
            }
            let fg = if !enabled {
                color::TEXT_DISABLED
            } else if selected {
                color::TEXT_ON_PRIMARY
            } else if hovered {
                color::TEXT_PRIMARY
            } else {
                color::TEXT_SECONDARY
            };
            let font = if selected {
                strong.clone()
            } else {
                body.clone()
            };
            // Icon and label, centered together in the option.
            let label_width = painter
                .layout_no_wrap(segment.label.to_owned(), font.clone(), fg)
                .size()
                .x;
            let content = icon_width(segment.icon) + label_width;
            let mut cx = rect.center().x - content / 2.0;
            if !segment.icon.is_empty() {
                painter.text(
                    egui::pos2(cx + size::ICON / 2.0, rect.center().y),
                    Align2::CENTER_CENTER,
                    segment.icon,
                    icon_font.clone(),
                    fg,
                );
                cx += icon_width(segment.icon);
            }
            painter.text(
                egui::pos2(cx, rect.center().y),
                Align2::LEFT_CENTER,
                segment.label,
                font,
                fg,
            );
            paint_focus_ring(ui, rect, &response, radius::MD);
            let response = name_and_shortcut_tooltip(
                response,
                segment.name,
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
