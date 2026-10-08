//! Editor workspace: menu bar, tool bar, sidebar, canvas area, panels and
//! status bar.

pub mod canvas;
pub mod panels;
pub mod status_bar;
pub mod toolbar;

use egui::{CentralPanel, Frame, Margin, Panel, Ui};
use tp_ui::tokens::{color, size, space};

use super::home::bar_frame;
use super::{CommandUi, menu_bar};
use crate::state::{AppState, Screen};

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let ctx = ui.ctx().clone();
    let edit = state.edit_context();
    let AppState {
        prefs,
        queue,
        screen,
        vehicles,
        vehicle_request,
        library,
        ..
    } = state;
    let Screen::Workspace(ws) = screen else {
        return;
    };
    let crate::prefs::Prefs {
        layout,
        recent_colors,
        view_aids,
        ..
    } = prefs;
    let aids = *view_aids;
    let generation = layout.generation;
    let mut cmds = CommandUi::new(&ctx, queue, edit);

    Panel::top("menu_bar")
        .frame(bar_frame())
        .show(ui, |ui| menu_bar::show(ui, &mut cmds, Some(layout), aids));

    Panel::bottom("status_bar")
        .exact_size(size::STATUS_BAR_HEIGHT)
        .frame(bar_frame().inner_margin(Margin::symmetric(space::MD as i8, 0)))
        .show(ui, |ui| status_bar::show(ui, ws));

    Panel::left("tool_bar")
        .exact_size(size::TOOL_BAR_WIDTH)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(color::SURFACE_1)
                .inner_margin(Margin::symmetric(6, 8)),
        )
        .show(ui, |ui| toolbar::show(ui, &mut cmds, ws.tool));

    vehicles_sidebar(
        ui,
        &mut cmds,
        layout,
        ws,
        vehicles,
        vehicle_request,
        library,
        generation,
    );

    let column = Panel::right(egui::Id::new(("panel_column", generation)))
        .resizable(true)
        .default_size(layout.column_width)
        .size_range(size::PANEL_COLUMN_MIN..=size::PANEL_COLUMN_MAX)
        .frame(Frame::new().fill(color::SURFACE_1))
        .show(ui, |ui| {
            let mut env = panels::PanelEnv {
                ws,
                recent_colors,
                vehicles,
                vehicle_request,
                library,
                now: ctx.input(|i| i.time),
            };
            panels::show(ui, &mut cmds, layout, &mut env);
        });
    let width = column.response.rect.width().round();
    if (width - layout.column_width).abs() >= 1.0 {
        layout.column_width = width.clamp(size::PANEL_COLUMN_MIN, size::PANEL_COLUMN_MAX);
    }

    CentralPanel::no_frame()
        .frame(Frame::new().fill(color::SURFACE_0))
        .show(ui, |ui| canvas_area(ui, &mut cmds, ws));
}

/// The sidebar: the project and its fleet tree on the left of the canvas,
/// or a strip with a button to show it again.
#[allow(clippy::too_many_arguments)]
fn vehicles_sidebar(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    layout: &mut crate::layout::WorkspaceLayout,
    ws: &mut crate::workspace::Workspace,
    vehicles: &crate::vehicles::VehicleLibrary,
    vehicle_request: &mut Option<crate::state::VehicleRequest>,
    library: &mut crate::library::LibraryStore,
    generation: u32,
) {
    use egui::ScrollArea;
    use tp_i18n::tr;
    use tp_ui::widgets::IconButton;

    let frame = Frame::new()
        .fill(color::SURFACE_1)
        .inner_margin(Margin::symmetric(space::SM as i8, space::SM as i8));
    if !layout.vehicles_open {
        Panel::left("vehicles_strip")
            .exact_size(36.0)
            .resizable(false)
            .frame(frame)
            .show(ui, |ui| {
                let show = tr("sidebar-show");
                if ui
                    .add(IconButton::new(tp_ui::icons::VEHICLE, &show))
                    .on_hover_text(&show)
                    .clicked()
                {
                    cmds.push(crate::commands::CommandId::ToggleVehicles);
                }
            });
        return;
    }
    let range = crate::layout::VEHICLES_WIDTH_RANGE;
    let sidebar = Panel::left(egui::Id::new(("vehicles_sidebar", generation)))
        .resizable(true)
        .default_size(layout.vehicles_width)
        .size_range(range.clone())
        .frame(frame)
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("vehicles_sidebar_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = space::XS + 2.0;
                    let mut env = panels::PanelEnv {
                        ws,
                        recent_colors: &[],
                        vehicles,
                        vehicle_request,
                        library,
                        now: ui.input(|i| i.time),
                    };
                    panels::vehicle::show(ui, cmds, &mut env);
                });
        });
    let width = sidebar.response.rect.width().round();
    if (width - layout.vehicles_width).abs() >= 1.0 {
        layout.vehicles_width = width.clamp(*range.start(), *range.end());
    }
}

/// The canvas (textures are switched from the sidebar).
fn canvas_area(
    ui: &mut Ui,
    cmds: &mut crate::ui::CommandUi<'_>,
    ws: &mut crate::workspace::Workspace,
) {
    canvas::show(ui, cmds, ws);
}
