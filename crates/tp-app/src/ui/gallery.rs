//! Developer window showing every design-system widget in every state.

use egui::{RichText, Ui};
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{
    EmptyState, IconButton, MenuRow, PanelHeader, SegmentedControl, StepIndicator, ToolButton,
    primary_button, secondary_button,
};

fn section(ui: &mut Ui, title: &str) {
    ui.add_space(space::MD);
    ui.label(RichText::new(title).text_style(label_strong_style()));
    ui.add_space(space::XS);
}

pub fn show(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Design System Gallery")
        .open(open)
        .default_size([520.0, 640.0])
        .vscroll(true)
        .show(ctx, |ui| {
            section(ui, "Tool buttons — idle / active / disabled");
            ui.horizontal(|ui| {
                ui.add(ToolButton::new(icons::RECTANGLE, "Idle").shortcut(Some("R")));
                ui.add(ToolButton::new(icons::ELLIPSE, "Active").active(true));
                ui.add_enabled(false, ToolButton::new(icons::PEN, "Disabled"));
            });

            section(ui, "Icon buttons — idle / selected / disabled");
            ui.horizontal(|ui| {
                ui.add(IconButton::new(icons::SETTINGS, "Idle"));
                ui.add(IconButton::new(icons::LAYERS, "Selected").selected(true));
                ui.add_enabled(
                    false,
                    IconButton::new(icons::REMOVE, "Disabled").disabled_reason("Explains why."),
                );
            });

            section(ui, "Buttons");
            ui.horizontal(|ui| {
                ui.add(primary_button("Primary"));
                ui.add(secondary_button("Secondary"));
                ui.add_enabled(false, secondary_button("Disabled"));
            });

            section(ui, "Segmented control");
            let id = ui.id().with("gallery_segment");
            let mut current = ui.data(|d| d.get_temp::<u8>(id).unwrap_or(0));
            if let Some(v) = SegmentedControl::new()
                .segment(0u8, icons::VIEW_2D, "2D", None)
                .segment(1, icons::VIEW_3D, "3D", None)
                .segment(2, icons::VIEW_SPLIT, "Split", None)
                .show(ui, current)
            {
                current = v;
                ui.data_mut(|d| d.insert_temp(id, current));
            }

            section(ui, "Menu rows — normal / checked / disabled");
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(260.0);
                ui.add(
                    MenuRow::new("Save")
                        .icon(Some(icons::SAVE))
                        .shortcut(Some("⌘S")),
                );
                ui.add(
                    MenuRow::new("Layers")
                        .checked(Some(true))
                        .shortcut(Some("F7")),
                );
                ui.add_enabled(false, MenuRow::new("Undo").shortcut(Some("⌘Z")));
            });

            section(ui, "Panel header — expanded / collapsed");
            ui.set_width(320.0);
            PanelHeader::new(icons::LAYERS, "Layers", false).show(ui);
            PanelHeader::new(icons::COLORS, "Colors", true).show(ui);

            section(ui, "Step indicator");
            StepIndicator::new(1, &["Game", "Vehicle", "Resolution"]).show(ui);

            section(ui, "Empty state");
            EmptyState::new(icons::ASSETS, "No assets", "Imported images appear here.").show(ui);

            section(ui, "Text levels");
            ui.label(RichText::new("Primary text").color(color::TEXT_PRIMARY));
            ui.label(RichText::new("Secondary text").color(color::TEXT_SECONDARY));
            ui.label(RichText::new("Disabled text").color(color::TEXT_DISABLED));
            ui.horizontal(|ui| {
                ui.label(icons::rich(icons::WARNING).color(color::WARNING));
                ui.label(RichText::new("Warning").color(color::WARNING));
            });
        });
}
