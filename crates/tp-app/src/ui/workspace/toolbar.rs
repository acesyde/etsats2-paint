//! Vertical tool bar.

use egui::{Align, Layout, Ui};
use tp_ui::tokens::{color, space};

use crate::commands::CommandId;
use crate::tool::Tool;
use crate::ui::CommandUi;

/// Tools grouped as separated clusters, in tool bar order.
const GROUPS: [&[Tool]; 5] = [
    &[Tool::Select, Tool::DirectSelect, Tool::Move],
    &[
        Tool::Rectangle,
        Tool::Ellipse,
        Tool::Polygon,
        Tool::Pen,
        Tool::Line,
    ],
    &[Tool::Text, Tool::Image],
    &[Tool::Eyedropper],
    &[Tool::Zoom, Tool::Hand],
];

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, active: Tool) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.y = space::XXS;
        for (index, group) in GROUPS.iter().enumerate() {
            if index > 0 {
                ui.add_space(space::XS);
                let rect = ui.available_rect_before_wrap();
                ui.painter().hline(
                    rect.x_range().shrink(6.0),
                    rect.top(),
                    egui::Stroke::new(1.0, color::BORDER),
                );
                ui.add_space(space::XS + 1.0);
            }
            for &tool in *group {
                let id = CommandId::SelectTool(tool);
                let response = cmds.tool_button(ui, id, tool == active);
                response.context_menu(|ui| {
                    cmds.menu_toggle(ui, id, tool == active);
                    ui.separator();
                    cmds.menu_item(ui, CommandId::KeyboardShortcuts);
                });
            }
        }
    });
}
