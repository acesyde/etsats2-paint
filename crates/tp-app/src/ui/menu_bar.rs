//! Application menu bar (File, Edit, Object, Layer, View, Vehicle, Export, Help).

use egui::{Align, Layout, Ui};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, size, space};
use tp_ui::widgets::SegmentedControl;

use super::CommandUi;
use crate::commands::CommandId;
use crate::layout::{PanelKind, ViewMode, WorkspaceLayout};

/// Menu titles, in order.
pub const MENUS: [&str; 8] = [
    "menu-file",
    "menu-edit",
    "menu-object",
    "menu-layer",
    "menu-view",
    "menu-vehicle",
    "menu-export",
    "menu-help",
];

/// Draws the menu bar row. `layout` is `None` on the home screen.
pub fn show(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    layout: Option<&WorkspaceLayout>,
    aids: crate::prefs::ViewAids,
) {
    ui.set_height(size::MENU_BAR_HEIGHT);
    egui::MenuBar::new().ui(ui, |ui| {
        ui.label(
            icons::rich(icons::VEHICLE)
                .color(color::ACCENT)
                .size(size::ICON_LG),
        )
        .on_hover_text(crate::paths::APP_NAME);
        ui.add_space(space::XS);

        for title in MENUS {
            ui.menu_button(tr(title), |ui| {
                ui.set_min_width(220.0);
                menu_contents(ui, cmds, title, layout, aids);
            });
        }

        if let Some(layout) = layout {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(space::XS);
                let picked = SegmentedControl::new()
                    .segment(
                        ViewMode::TwoD,
                        icons::VIEW_2D,
                        &tr("view-2d"),
                        cmds.shortcuts
                            .command(CommandId::SetViewMode(ViewMode::TwoD)),
                    )
                    .segment(
                        ViewMode::ThreeD,
                        icons::VIEW_3D,
                        &tr("view-3d"),
                        cmds.shortcuts
                            .command(CommandId::SetViewMode(ViewMode::ThreeD)),
                    )
                    .segment(
                        ViewMode::Split,
                        icons::VIEW_SPLIT,
                        &tr("view-split"),
                        cmds.shortcuts
                            .command(CommandId::SetViewMode(ViewMode::Split)),
                    )
                    .show(ui, layout.view_mode);
                if let Some(mode) = picked {
                    cmds.push(CommandId::SetViewMode(mode));
                }
            });
        }
    });
}

fn menu_contents(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    title: &str,
    layout: Option<&WorkspaceLayout>,
    aids: crate::prefs::ViewAids,
) {
    use CommandId::*;
    let item = |ui: &mut Ui, cmds: &mut CommandUi<'_>, id| {
        cmds.menu_item(ui, id);
    };
    match title {
        "menu-file" => {
            item(ui, cmds, NewProject);
            item(ui, cmds, OpenProject);
            ui.separator();
            item(ui, cmds, Save);
            item(ui, cmds, SaveAs);
            ui.separator();
            item(ui, cmds, Place);
            ui.separator();
            item(ui, cmds, CloseProject);
            ui.separator();
            item(ui, cmds, Quit);
        }
        "menu-edit" => {
            item(ui, cmds, Undo);
            item(ui, cmds, Redo);
            ui.separator();
            item(ui, cmds, Cut);
            item(ui, cmds, Copy);
            item(ui, cmds, Paste);
            item(ui, cmds, Duplicate);
            item(ui, cmds, Delete);
            ui.separator();
            item(ui, cmds, SelectAll);
            item(ui, cmds, Deselect);
            ui.separator();
            item(ui, cmds, Preferences);
        }
        "menu-object" => {
            item(ui, cmds, EditText);
            ui.separator();
            item(ui, cmds, Group);
            item(ui, cmds, Ungroup);
            ui.separator();
            item(ui, cmds, ConvertToSymbol);
            item(ui, cmds, EditSymbol);
            item(ui, cmds, DetachInstance);
            ui.separator();
            item(ui, cmds, ConvertToPath);
            item(ui, cmds, CreateOutlines);
            ui.menu_button(tr("menu-combine"), |ui| {
                ui.set_min_width(220.0);
                for op in tp_core::document::BooleanOp::ALL {
                    item(ui, cmds, Combine(op));
                }
            });
            ui.menu_button(tr("menu-align"), |ui| {
                ui.set_min_width(260.0);
                for edge in tp_core::document::Edge::ALL {
                    item(ui, cmds, Align(edge));
                }
                ui.separator();
                use tp_core::document::{DistributeAxis, DistributeMode};
                for (axis, mode) in [
                    (DistributeAxis::Horizontal, DistributeMode::Centers),
                    (DistributeAxis::Vertical, DistributeMode::Centers),
                    (DistributeAxis::Horizontal, DistributeMode::Spacing),
                    (DistributeAxis::Vertical, DistributeMode::Spacing),
                ] {
                    item(ui, cmds, Distribute(axis, mode));
                }
            });
            ui.separator();
            for axis in tp_core::document::FlipAxis::ALL {
                item(ui, cmds, Flip(axis));
            }
            ui.separator();
            item(ui, cmds, BringForward);
            item(ui, cmds, SendBackward);
        }
        "menu-layer" => {
            item(ui, cmds, NewLayer);
            item(ui, cmds, DuplicateLayer);
            item(ui, cmds, DeleteLayer);
        }
        "menu-view" => {
            item(ui, cmds, ZoomIn);
            item(ui, cmds, ZoomOut);
            item(ui, cmds, FitToScreen);
            item(ui, cmds, ActualSize);
            ui.separator();
            let mode = layout.map(|l| l.view_mode);
            for m in [ViewMode::TwoD, ViewMode::ThreeD, ViewMode::Split] {
                cmds.menu_toggle(ui, SetViewMode(m), mode == Some(m));
            }
            cmds.menu_toggle(ui, TogglePreview, mode.is_some_and(ViewMode::shows_preview));
            ui.separator();
            cmds.menu_toggle(ui, ShowGrid, layout.is_some() && aids.grid);
            cmds.menu_toggle(ui, ShowGuides, layout.is_some() && aids.guides);
            let template_shown = cmds.edit.template_visible;
            cmds.menu_toggle(ui, ShowTemplate, template_shown);
            item(ui, cmds, ClearGuides);
            cmds.menu_toggle(ui, Snapping, layout.is_some() && aids.snapping);
            ui.separator();
            cmds.menu_toggle(ui, ToggleVehicles, layout.is_some_and(|l| l.vehicles_open));
            for kind in PanelKind::ALL {
                cmds.menu_toggle(
                    ui,
                    TogglePanel(kind),
                    layout.is_some_and(|l| l.is_open(kind)),
                );
            }
            ui.separator();
            item(ui, cmds, ResetWorkspace);
            if cfg!(debug_assertions) {
                ui.separator();
                item(ui, cmds, DesignGallery);
            }
        }
        "menu-vehicle" => {
            item(ui, cmds, VehicleLibrary);
            item(ui, cmds, AddVehicle);
            item(ui, cmds, VehicleInfo);
            ui.separator();
            item(ui, cmds, NextTexture);
            item(ui, cmds, PreviousTexture);
            item(ui, cmds, CopyFromCabin);
            ui.separator();
            item(ui, cmds, UpdateTemplate);
        }
        "menu-export" => {
            item(ui, cmds, ExportTexture);
            item(ui, cmds, ExportMod);
        }
        "menu-help" => {
            item(ui, cmds, KeyboardShortcuts);
            ui.separator();
            item(ui, cmds, About);
        }
        _ => {}
    }
}
