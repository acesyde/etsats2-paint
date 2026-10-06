//! Right-hand stack of collapsible, closable panels.

use egui::{Frame, Margin, ScrollArea, Ui};
use tp_ui::icons;
use tp_ui::tokens::space;
use tp_ui::widgets::{EmptyState, PanelHeader};

use crate::commands::CommandId;
use crate::layout::WorkspaceLayout;
use crate::ui::CommandUi;

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, layout: &mut WorkspaceLayout) {
    let open: Vec<_> = layout.panels.iter().filter(|s| s.open).copied().collect();
    if open.is_empty() {
        EmptyState::new(
            icons::LAYERS,
            "All panels are closed",
            "Reopen panels from the View menu, or reset the workspace.",
        )
        .show(ui);
        ui.vertical_centered(|ui| {
            if ui.button("Reset Workspace").clicked() {
                cmds.push(CommandId::ResetWorkspace);
            }
        });
        return;
    }

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for slot in open {
                let header =
                    PanelHeader::new(slot.kind.icon(), slot.kind.title(), slot.collapsed).show(ui);
                if header.toggle {
                    layout.toggle_collapsed(slot.kind);
                }
                if header.close {
                    cmds.push(CommandId::TogglePanel(slot.kind));
                }
                header.response.context_menu(|ui| {
                    let collapse_label = if slot.collapsed { "Expand" } else { "Collapse" };
                    if ui
                        .add(tp_ui::widgets::MenuRow::new(collapse_label))
                        .clicked()
                    {
                        layout.toggle_collapsed(slot.kind);
                        ui.close();
                    }
                    if ui
                        .add(tp_ui::widgets::MenuRow::new("Close Panel"))
                        .clicked()
                    {
                        cmds.push(CommandId::TogglePanel(slot.kind));
                        ui.close();
                    }
                    ui.separator();
                    cmds.menu_item(ui, CommandId::ResetWorkspace);
                });

                if !slot.collapsed {
                    Frame::new()
                        .inner_margin(Margin::symmetric(space::MD as i8, space::SM as i8))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let (title, message) = slot.kind.empty_state();
                            EmptyState::new(slot.kind.icon(), title, message).show(ui);
                        });
                }
            }
        });
}
