//! 3D preview area. Rendering arrives with `export-and-preview`; until then
//! it shows an explicit placeholder.

use egui::{Align, Frame, Layout, Margin, RichText, Ui};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, size, space};
use tp_ui::widgets::{EmptyState, IconButton};

use crate::commands::CommandId;
use crate::ui::CommandUi;

/// `closable`: show a hide button (split view).
pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, closable: bool) {
    Frame::new()
        .fill(color::SURFACE_1)
        .inner_margin(Margin::symmetric(space::SM as i8, 0))
        .show(ui, |ui| {
            ui.set_height(size::PANEL_HEADER_HEIGHT);
            ui.horizontal_centered(|ui| {
                ui.label(icons::rich(icons::PREVIEW_3D).color(color::TEXT_SECONDARY));
                ui.label(RichText::new(tr("cmd-3d-preview")).text_style(label_strong_style()));
                if closable {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let hide =
                            ui.add(IconButton::new(icons::CLOSE, &tr("preview-hide")).shortcut(
                                cmds.shortcuts.command(CommandId::TogglePreview).as_deref(),
                            ));
                        if hide.clicked() {
                            cmds.push(CommandId::TogglePreview);
                        }
                    });
                }
            });
        });

    let rest = ui.available_rect_before_wrap();
    ui.painter().rect_filled(rest, 0, color::SURFACE_0);
    ui.allocate_ui_with_layout(rest.size(), Layout::top_down(Align::Center), |ui| {
        ui.add_space((rest.height() / 2.0 - 70.0).max(space::LG));
        ui.set_max_width(320.0);
        EmptyState::new(
            icons::PREVIEW_3D,
            &tr("preview-soon"),
            &tr("preview-soon-hint"),
        )
        .show(ui);
    });
}
