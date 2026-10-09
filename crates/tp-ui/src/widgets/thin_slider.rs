use std::ops::RangeInclusive;

use egui::{CornerRadius, Key, Response, Sense, Ui, Vec2, Widget, WidgetInfo};

use super::paint_focus_ring;
use crate::tokens::{color, radius, size};

/// Height of the track, in points.
const TRACK: f32 = 3.0;
/// Radius of the knob shown while the slider is hovered, dragged or
/// focused.
const KNOB: f32 = 4.0;

/// A thin slider: a 3-point track filled up to the value, with no knob at
/// rest (the status bar's template opacity, the inspector's Opacity, the
/// preferences' scales). The hit area keeps the minimum height; a press or
/// a drag anywhere on it sets the value, and the arrow keys step it when it
/// has the keyboard focus (Shift: ten steps).
pub struct ThinSlider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    name: &'a str,
    width: f32,
    step: f32,
}

impl<'a> ThinSlider<'a> {
    /// `name` is the accessible name.
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>, name: &'a str) -> Self {
        let step = (range.end() - range.start()) / 100.0;
        Self {
            value,
            range,
            name,
            width: 72.0,
            step,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Values are rounded to multiples of `step` (also the arrow keys'
    /// step).
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    fn snap(&self, v: f32) -> f32 {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let v = if self.step > 0.0 {
            lo + ((v - lo) / self.step).round() * self.step
        } else {
            v
        };
        v.clamp(lo, hi)
    }
}

impl Widget for ThinSlider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(self.width, size::HIT_MIN),
            Sense::click_and_drag(),
        );
        let (lo, hi) = (*self.range.start(), *self.range.end());
        let span = (hi - lo).max(f32::EPSILON);
        let old = *self.value;

        if let Some(pos) = response.interact_pointer_pos()
            && (response.dragged() || response.clicked() || response.drag_started())
        {
            let t = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            *self.value = self.snap(lo + t * span);
        }
        if response.has_focus() {
            let (left, right, shift) = ui.input(|i| {
                (
                    i.num_presses(Key::ArrowLeft) + i.num_presses(Key::ArrowDown),
                    i.num_presses(Key::ArrowRight) + i.num_presses(Key::ArrowUp),
                    i.modifiers.shift,
                )
            });
            let steps = right as f32 - left as f32;
            if steps != 0.0 {
                let factor = if shift { 10.0 } else { 1.0 };
                *self.value = self.snap(*self.value + steps * factor * self.step);
            }
        }
        if (*self.value - old).abs() > f32::EPSILON {
            response.mark_changed();
        }
        let enabled = ui.is_enabled();
        let value = f64::from(*self.value);
        response.widget_info(|| WidgetInfo::slider(enabled, value, self.name));

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let track = egui::Rect::from_center_size(rect.center(), Vec2::new(rect.width(), TRACK));
            let corner = CornerRadius::same(2);
            painter.rect_filled(track, corner, color::BORDER);
            let t = ((*self.value - lo) / span).clamp(0.0, 1.0);
            let mut filled = track;
            filled.set_right(track.left() + t * track.width());
            let active =
                enabled && (response.hovered() || response.dragged() || response.has_focus());
            let ink = if !enabled {
                color::TEXT_DISABLED
            } else if active {
                color::TEXT_PRIMARY
            } else {
                color::TEXT_SECONDARY
            };
            painter.rect_filled(filled, corner, ink);
            if active {
                painter.circle_filled(egui::pos2(filled.right(), track.center().y), KNOB, ink);
            }
            paint_focus_ring(ui, rect, &response, radius::SM);
        }
        response
    }
}
