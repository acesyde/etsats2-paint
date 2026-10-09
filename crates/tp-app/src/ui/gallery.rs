//! Developer window showing every design-system widget in every state.

use egui::Color32;
use egui::{RichText, Ui};
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, space};
use tp_ui::widgets::{
    ColorSwatch, FillOrStroke, FillStrokeSwatches, Hsv, NumericField, SwatchColor, alpha_slider,
    hue_slider, sv_square, toggle_icon_button,
};
use tp_ui::widgets::{
    EmptyState, IconButton, MenuRow, SegmentedControl, ToolButton, primary_button, secondary_button,
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
                .segment(0u8, icons::RECTANGLE, "Rectangle", None)
                .segment(1, icons::ELLIPSE, "Ellipse", None)
                .segment(2, icons::POLYGON, "Polygon", None)
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
                        .shortcut(Some("2")),
                );
                ui.add_enabled(false, MenuRow::new("Undo").shortcut(Some("⌘Z")));
            });

            section(ui, "Empty state");
            EmptyState::new(icons::ASSETS, "No assets", "Imported images appear here.").show(ui);

            section(ui, "Numeric fields — value / mixed / disabled");
            ui.horizontal(|ui| {
                NumericField::new("W", "Gallery width", Some(400.0))
                    .suffix("px")
                    .show(ui);
                NumericField::new("R", "Gallery rotation", None)
                    .suffix("°")
                    .show(ui);
                ui.add_enabled_ui(false, |ui| {
                    NumericField::new("H", "Gallery height", Some(200.0))
                        .suffix("px")
                        .show(ui);
                });
            });

            section(
                ui,
                "Swatches — solid / translucent / none / mixed / selected",
            );
            ui.horizontal(|ui| {
                ColorSwatch::new(
                    SwatchColor::Solid(Color32::from_rgb(0x7A, 0x1F, 0x2B)),
                    "Solid",
                )
                .show(ui);
                ColorSwatch::new(
                    SwatchColor::Solid(Color32::from_rgba_unmultiplied(0x5B, 0x8D, 0xEF, 120)),
                    "Translucent",
                )
                .show(ui);
                ColorSwatch::new(SwatchColor::None, "None").show(ui);
                ColorSwatch::new(SwatchColor::Mixed, "Mixed").show(ui);
                ColorSwatch::new(SwatchColor::Solid(Color32::WHITE), "Selected")
                    .selected(true)
                    .show(ui);
                ui.add_space(space::LG);
                FillStrokeSwatches {
                    fill: SwatchColor::Solid(Color32::from_rgb(0xF0, 0xB4, 0x4C)),
                    stroke: SwatchColor::Solid(Color32::BLACK),
                    active: FillOrStroke::Fill,
                }
                .show(ui);
            });

            section(ui, "Picker");
            let picker_id = ui.id().with("gallery_picker");
            let mut hsv = ui
                .data(|d| d.get_temp::<Hsv>(picker_id))
                .unwrap_or(Hsv::new(0.6, 0.6, 0.9, 1.0));
            ui.allocate_ui(egui::Vec2::new(240.0, 180.0), |ui| {
                sv_square(ui, &mut hsv, 110.0);
                hue_slider(ui, &mut hsv);
                alpha_slider(ui, &mut hsv);
            });
            ui.data_mut(|d| d.insert_temp(picker_id, hsv));

            section(ui, "Toggle icon buttons — on / off");
            ui.horizontal(|ui| {
                toggle_icon_button(ui, true, icons::VISIBLE, icons::HIDDEN, "Hide", "Show");
                toggle_icon_button(ui, false, icons::VISIBLE, icons::HIDDEN, "Hide", "Show");
                toggle_icon_button(ui, true, icons::UNLOCKED, icons::LOCKED, "Lock", "Unlock");
                toggle_icon_button(ui, false, icons::UNLOCKED, icons::LOCKED, "Lock", "Unlock");
            });

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
