//! Right-hand stack of collapsible, closable panels.

mod assets;
mod character;
mod colors;
mod layers;
pub mod line_style;
mod properties;
mod stroke;
mod transform;

use egui::{Frame, Margin, ScrollArea, Ui};
use tp_ui::icons;
use tp_ui::tokens::space;
use tp_ui::widgets::{EmptyState, FieldEvent, PanelHeader};

use crate::commands::CommandId;
use crate::layout::{PanelKind, WorkspaceLayout};
use crate::ui::CommandUi;
use crate::workspace::Workspace;

/// What panel bodies can read and edit.
pub struct PanelEnv<'a> {
    pub ws: &'a mut Workspace,
    /// Preferences' recent colors (RGBA), most recent first.
    pub recent_colors: &'a [[u8; 4]],
    pub now: f64,
}

/// Routes a field event to a live edit: `Live` applies, `Commit` applies and
/// records one undo step, `Revert` restores the document.
pub fn apply_field(
    env: &mut PanelEnv<'_>,
    event: FieldEvent,
    apply: impl FnOnce(&mut Workspace, f64),
) {
    match event {
        FieldEvent::Live(v) => apply(env.ws, v),
        FieldEvent::Commit(v) => {
            apply(env.ws, v);
            env.ws.commit_pending(env.now);
        }
        FieldEvent::Revert => env.ws.cancel_pending(),
        FieldEvent::None => {}
    }
}

/// Opens and expands a panel (e.g. Colors when a swatch is clicked).
pub fn reveal(layout: &mut WorkspaceLayout, kind: PanelKind) {
    let slot = layout.slot_mut(kind);
    slot.open = true;
    slot.collapsed = false;
}

pub fn show(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    layout: &mut WorkspaceLayout,
    env: &mut PanelEnv<'_>,
) {
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
                            ui.spacing_mut().item_spacing.y = space::XS + 2.0;
                            body(ui, cmds, layout, env, slot.kind);
                        });
                }
            }
        });
}

fn body(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    layout: &mut WorkspaceLayout,
    env: &mut PanelEnv<'_>,
    kind: PanelKind,
) {
    match kind {
        PanelKind::Properties => properties::show(ui, env, layout),
        PanelKind::Transform => transform::show(ui, cmds, env),
        PanelKind::Colors => colors::show(ui, env),
        PanelKind::Stroke => stroke::show(ui, env),
        PanelKind::Layers => layers::show(ui, cmds, env),
        PanelKind::Assets => assets::show(ui, cmds, env),
        PanelKind::Vehicle => {
            let (title, message) = kind.empty_state();
            EmptyState::new(kind.icon(), title, message).show(ui);
        }
    }
}
