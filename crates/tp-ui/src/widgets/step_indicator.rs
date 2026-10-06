use egui::{RichText, Stroke, Ui, Vec2, WidgetInfo, WidgetType};

use crate::tokens::{color, space};

/// "Step N of M — Title" with one marker per step.
///
/// Completed and current steps are filled, the current one is also larger,
/// so progress is readable without color.
pub struct StepIndicator<'a> {
    current: usize,
    titles: &'a [&'a str],
}

impl<'a> StepIndicator<'a> {
    /// `current` is zero-based.
    pub fn new(current: usize, titles: &'a [&'a str]) -> Self {
        Self { current, titles }
    }

    pub fn show(self, ui: &mut Ui) {
        let total = self.titles.len();
        let title = self.titles.get(self.current).copied().unwrap_or_default();
        let text = format!("Step {} of {total} — {title}", self.current + 1);
        ui.horizontal(|ui| {
            for index in 0..total {
                let radius = if index == self.current { 5.0 } else { 3.5 };
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), egui::Sense::hover());
                if index <= self.current {
                    ui.painter()
                        .circle_filled(rect.center(), radius, color::ACCENT);
                } else {
                    ui.painter().circle_stroke(
                        rect.center(),
                        radius,
                        Stroke::new(1.0, color::TEXT_SECONDARY),
                    );
                }
            }
            ui.add_space(space::XS);
            let response = ui.label(RichText::new(&text).color(color::TEXT_SECONDARY));
            response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
        });
    }
}
