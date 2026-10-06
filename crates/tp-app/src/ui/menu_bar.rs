//! Application menu bar (File, Edit, Object, Layer, View, Vehicle, Export, Help).

use egui::{Align, Layout, Ui};
use tp_ui::icons;
use tp_ui::tokens::{color, size, space};
use tp_ui::widgets::SegmentedControl;

use super::CommandUi;
use crate::commands::CommandId;
use crate::layout::{PanelKind, ViewMode, WorkspaceLayout};

/// Menu titles, in order.
pub const MENUS: [&str; 8] = [
    "File", "Edit", "Object", "Layer", "View", "Vehicle", "Export", "Help",
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
            ui.menu_button(title, |ui| {
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
                        "2D",
                        cmds.shortcuts
                            .command(CommandId::SetViewMode(ViewMode::TwoD)),
                    )
                    .segment(
                        ViewMode::ThreeD,
                        icons::VIEW_3D,
                        "3D",
                        cmds.shortcuts
                            .command(CommandId::SetViewMode(ViewMode::ThreeD)),
                    )
                    .segment(
                        ViewMode::Split,
                        icons::VIEW_SPLIT,
                        "Split",
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
        "File" => {
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
        "Edit" => {
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
        "Object" => {
            item(ui, cmds, EditText);
            ui.separator();
            item(ui, cmds, Group);
            item(ui, cmds, Ungroup);
            ui.separator();
            item(ui, cmds, ConvertToPath);
            ui.menu_button("Align", |ui| {
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
            item(ui, cmds, BringForward);
            item(ui, cmds, SendBackward);
            ui.separator();
            item(ui, cmds, MirrorToOtherSide);
        }
        "Layer" => {
            item(ui, cmds, NewLayer);
            item(ui, cmds, DuplicateLayer);
            item(ui, cmds, DeleteLayer);
        }
        "View" => {
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
            item(ui, cmds, ClearGuides);
            cmds.menu_toggle(ui, Snapping, layout.is_some() && aids.snapping);
            ui.separator();
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
        "Vehicle" => {
            item(ui, cmds, ChooseVehicle);
            item(ui, cmds, VehicleInfo);
        }
        "Export" => {
            item(ui, cmds, ExportTexture);
            item(ui, cmds, ExportMod);
        }
        "Help" => {
            item(ui, cmds, KeyboardShortcuts);
            ui.separator();
            item(ui, cmds, About);
        }
        _ => {}
    }
}
