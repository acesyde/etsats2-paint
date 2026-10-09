use egui::{
    Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo,
    WidgetType,
};

use super::{name_and_shortcut_tooltip, paint_focus_ring};
use crate::theme::label_strong_style;
use crate::tokens::{color, radius, size, space, stroke, typography};

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
/// The options sit side by side in one track on the control surface; the
/// active one is a white pill (primary accent) with dark semibold text, the
/// others muted text on the track. The pill is a fill change, so the active
/// option stays visible in grayscale.
///
/// [`SegmentedControl::tabs`] draws the same choice as ghost tabs (the
/// space switcher of the top bar): no track, the active option on the chip
/// surface.
pub struct SegmentedControl<'a, T> {
    segments: Vec<Segment<'a, T>>,
    fill: bool,
    track: Color32,
    tabs: bool,
}

/// Padding of the track around the options (its outline included), gap
/// between options, horizontal padding of an option and its height.
const INSET: f32 = 4.0;
const GAP: f32 = 2.0;
const PAD_X: f32 = space::SM;
const OPTION_HEIGHT: f32 = size::HIT_MIN + 1.0;
/// The same for ghost tabs.
const TAB_PAD_X: f32 = space::MD;
const TAB_HEIGHT: f32 = 28.0;

impl<'a, T: Copy + PartialEq> SegmentedControl<'a, T> {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            fill: false,
            track: color::CONTROL,
            tabs: false,
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

    /// Spreads the options evenly over the available width (when their
    /// labels fit).
    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    /// The track's fill, for a control on a surface other than a panel
    /// (sunken in a popover, which is on the control surface itself).
    pub fn track(mut self, track: Color32) -> Self {
        self.track = track;
        self
    }

    /// Ghost tabs instead of a track and a pill.
    pub fn tabs(mut self) -> Self {
        self.tabs = true;
        self
    }

    fn fonts(&self, ui: &Ui) -> (FontId, FontId) {
        let body = egui::TextStyle::Button.resolve(ui.style());
        if self.tabs {
            return (body.clone(), body);
        }
        // Compact text, following the user's text size.
        let size = body.size * typography::CONTROL / typography::BODY;
        let strong = label_strong_style().resolve(ui.style());
        (
            FontId::new(size, body.family),
            FontId::new(size, strong.family),
        )
    }

    fn icon_width(icon: &str) -> f32 {
        if icon.is_empty() {
            0.0
        } else {
            size::ICON + space::XS
        }
    }

    /// The width each option needs; they follow the (wider) active font so
    /// options don't move when the active one changes.
    fn natural_widths(&self, ui: &Ui) -> Vec<f32> {
        let (_, strong) = self.fonts(ui);
        let pad = if self.tabs { TAB_PAD_X } else { PAD_X };
        self.segments
            .iter()
            .map(|s| {
                let text = ui
                    .painter()
                    .layout_no_wrap(s.label.to_owned(), strong.clone(), color::TEXT_PRIMARY)
                    .size()
                    .x;
                text + Self::icon_width(s.icon) + pad * 2.0
            })
            .collect()
    }

    /// The width the control takes when not filling, for laying out a bar
    /// around it.
    pub fn width(&self, ui: &Ui) -> f32 {
        let inset = if self.tabs { 0.0 } else { INSET };
        let gaps = GAP * self.segments.len().saturating_sub(1) as f32;
        self.natural_widths(ui).iter().sum::<f32>() + gaps + inset * 2.0
    }

    /// Draws the control; returns the newly picked value, if any.
    pub fn show(self, ui: &mut Ui, current: T) -> Option<T> {
        let mut picked = None;
        let (body, strong) = self.fonts(ui);
        let icon_font = crate::icons::font(size::ICON - 2.0);
        let (inset, height) = if self.tabs {
            (0.0, TAB_HEIGHT)
        } else {
            (INSET, OPTION_HEIGHT)
        };

        let mut widths = self.natural_widths(ui);
        let gaps = GAP * self.segments.len().saturating_sub(1) as f32;
        let natural = widths.iter().sum::<f32>() + gaps + inset * 2.0;
        if self.fill && ui.available_width() > natural && !widths.is_empty() {
            let each = (ui.available_width() - gaps - inset * 2.0) / widths.len() as f32;
            widths.iter_mut().for_each(|w| *w = each.floor());
        }
        let total = Vec2::new(
            widths.iter().sum::<f32>() + gaps + inset * 2.0,
            height + inset * 2.0,
        );
        let (outer, _) = ui.allocate_exact_size(total, Sense::hover());
        if !self.tabs {
            ui.painter().rect(
                outer,
                CornerRadius::same(radius::LG),
                self.track,
                Stroke::new(stroke::HAIRLINE, color::BORDER),
                StrokeKind::Inside,
            );
        }

        let enabled = ui.is_enabled();
        let mut x = outer.left() + inset;
        for (segment, width) in self.segments.into_iter().zip(widths) {
            let rect =
                Rect::from_min_size(egui::pos2(x, outer.top() + inset), Vec2::new(width, height));
            x += width + GAP;
            let id = ui.id().with(("segment", segment.name));
            let response = ui.interact(rect, id, Sense::click());
            let selected = segment.value == current;
            response.widget_info(|| {
                WidgetInfo::selected(WidgetType::RadioButton, enabled, selected, segment.name)
            });
            let painter = ui.painter();
            let hovered = enabled && response.hovered();
            let corner = CornerRadius::same(radius::MD);
            if selected {
                let fill = if self.tabs {
                    color::CHIP
                } else {
                    color::ACCENT_PRIMARY
                };
                painter.rect_filled(rect, corner, fill);
            } else if hovered {
                painter.rect_filled(rect, corner, color::SURFACE_3);
            }
            let fg = if !enabled {
                color::TEXT_DISABLED
            } else if selected && !self.tabs {
                color::TEXT_ON_PRIMARY
            } else if selected || hovered {
                color::TEXT_PRIMARY
            } else {
                color::TEXT_MUTED
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
            let content = Self::icon_width(segment.icon) + label_width;
            let mut cx = rect.center().x - content / 2.0;
            if !segment.icon.is_empty() {
                painter.text(
                    egui::pos2(cx + size::ICON / 2.0, rect.center().y),
                    Align2::CENTER_CENTER,
                    segment.icon,
                    icon_font.clone(),
                    fg,
                );
                cx += Self::icon_width(segment.icon);
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
