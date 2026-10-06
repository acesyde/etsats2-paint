use egui::{Align, Layout, RichText, Ui};

use crate::theme::label_strong_style;
use crate::tokens::{color, space};

/// Centered icon + title + explanation, shown instead of a blank area.
pub struct EmptyState<'a> {
    icon: &'a str,
    title: &'a str,
    message: &'a str,
}

impl<'a> EmptyState<'a> {
    pub fn new(icon: &'a str, title: &'a str, message: &'a str) -> Self {
        Self {
            icon,
            title,
            message,
        }
    }

    pub fn show(self, ui: &mut Ui) {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(space::LG);
            ui.label(
                crate::icons::rich(self.icon)
                    .size(28.0)
                    .color(color::TEXT_SECONDARY),
            );
            ui.add_space(space::XS);
            ui.label(
                RichText::new(self.title)
                    .text_style(label_strong_style())
                    .color(color::TEXT_PRIMARY),
            );
            ui.add(
                egui::Label::new(RichText::new(self.message).color(color::TEXT_SECONDARY)).wrap(),
            );
            ui.add_space(space::LG);
        });
    }
}
