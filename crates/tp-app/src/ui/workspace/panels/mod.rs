//! Right-hand stack of collapsible, closable panels.

mod assets;
pub mod character;
mod colors;
mod layers;
pub mod line_style;
mod properties;
mod stroke;
mod styles;
mod symbols;
mod transform;
pub mod vehicle;

use egui::{Frame, Margin, ScrollArea, Ui};
use tp_i18n::tr;
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
    /// Installed vehicle packages.
    pub vehicles: &'a crate::vehicles::VehicleLibrary,
    /// An action on one of the project's vehicles, run after the frame.
    pub vehicle_request: &'a mut Option<crate::state::VehicleRequest>,
    /// The personal library (Add to Library).
    pub library: &'a mut crate::library::LibraryStore,
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

/// Add to Library or Update in Library, in an element's context menu.
pub fn library_item(ui: &mut Ui, env: &mut PanelEnv<'_>, element: crate::library::Element) {
    let label = env.library.menu_label(&env.ws.project, element);
    if ui.add(tp_ui::widgets::MenuRow::new(&label)).clicked() {
        env.ws.add_to_library(env.library, element, env.now);
        ui.close();
    }
}

/// "Import from Library…", offered by panels while they are empty.
pub fn import_from_library_button(ui: &mut Ui, cmds: &mut CommandUi<'_>) {
    ui.vertical_centered(|ui| {
        let button = ui.add_enabled(
            cmds.enabled(CommandId::ImportFromLibrary),
            tp_ui::widgets::secondary_button(&tr("cmd-import-from-library")),
        );
        if button.clicked() {
            cmds.push(CommandId::ImportFromLibrary);
        }
    });
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
            &tr("panels-all-closed"),
            &tr("panels-all-closed-hint"),
        )
        .show(ui);
        ui.vertical_centered(|ui| {
            if ui.button(tr("cmd-reset-workspace")).clicked() {
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
                    PanelHeader::new(slot.kind.icon(), &tr(slot.kind.title()), slot.collapsed)
                        .show(ui);
                if header.toggle {
                    layout.toggle_collapsed(slot.kind);
                }
                if header.close {
                    cmds.push(CommandId::TogglePanel(slot.kind));
                }
                header.response.context_menu(|ui| {
                    let collapse_label = tr(if slot.collapsed {
                        "panel-expand"
                    } else {
                        "panel-collapse"
                    });
                    if ui
                        .add(tp_ui::widgets::MenuRow::new(&collapse_label))
                        .clicked()
                    {
                        layout.toggle_collapsed(slot.kind);
                        ui.close();
                    }
                    if ui
                        .add(tp_ui::widgets::MenuRow::new(&tr("panel-close")))
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
        PanelKind::Colors => colors::show(ui, cmds, env),
        PanelKind::Styles => styles::show(ui, cmds, env),
        PanelKind::Symbols => symbols::show(ui, cmds, env),
        PanelKind::Stroke => stroke::show(ui, env),
        PanelKind::Layers => layers::show(ui, cmds, env),
        PanelKind::Assets => assets::show(ui, cmds, env),
        // Shown in the Vehicles sidebar, never in the column.
        PanelKind::Vehicle => {}
    }
}
