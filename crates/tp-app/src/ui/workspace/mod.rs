//! Editor workspace: menu bar, tool bar, canvas area, panels, 3D preview and
//! status bar.

pub mod canvas;
pub mod panels;
pub mod preview;
pub mod status_bar;
pub mod toolbar;

use egui::{CentralPanel, Frame, Margin, Panel, Ui};
use tp_ui::tokens::{color, size, space};

use super::home::bar_frame;
use super::{CommandUi, menu_bar};
use crate::layout::{SPLIT_FRACTION_RANGE, ViewMode};
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
        .show(ui, |ui| status_bar::show(ui, ws, layout.view_mode));

    Panel::left("tool_bar")
        .exact_size(size::TOOL_BAR_WIDTH)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(color::SURFACE_1)
                .inner_margin(Margin::symmetric(6, 8)),
        )
        .show(ui, |ui| toolbar::show(ui, &mut cmds, ws.tool));

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
        .show(ui, |ui| match layout.view_mode {
            ViewMode::TwoD => canvas_with_tabs(ui, &mut cmds, ws),
            ViewMode::ThreeD => preview::show(ui, &mut cmds, false),
            ViewMode::Split => {
                let total = ui.available_width();
                let (min, max) = (*SPLIT_FRACTION_RANGE.start(), *SPLIT_FRACTION_RANGE.end());
                let split = Panel::right(egui::Id::new(("preview_split", generation)))
                    .resizable(true)
                    .default_size(total * layout.split_fraction)
                    .size_range((total * min)..=(total * max))
                    .frame(Frame::new().fill(color::SURFACE_0))
                    .show(ui, |ui| preview::show(ui, &mut cmds, true));
                if total > 0.0 {
                    let fraction = (split.response.rect.width() / total).clamp(min, max);
                    if (fraction - layout.split_fraction).abs() > 0.005 {
                        layout.split_fraction = fraction;
                    }
                }
                CentralPanel::no_frame()
                    .frame(Frame::new().fill(color::SURFACE_0))
                    .show(ui, |ui| canvas_with_tabs(ui, &mut cmds, ws));
            }
        });
}

/// The canvas, with tabs to switch between the textures of the active
/// variant when it has several.
fn canvas_with_tabs(
    ui: &mut Ui,
    cmds: &mut crate::ui::CommandUi<'_>,
    ws: &mut crate::workspace::Workspace,
) {
    let project = &ws.project;
    let active = project.active_surface;
    let range = project
        .surface()
        .template
        .as_ref()
        .map_or(active..active + 1, |t| {
            project.variant_range(&t.package_id, &t.variant_id)
        });
    if range.len() > 1 {
        let tabs: Vec<(usize, String, bool)> = range
            .map(|i| {
                let s = &project.surfaces[i];
                let flagged = s
                    .template
                    .as_ref()
                    .is_some_and(|t| t.status == tp_core::TemplateStatus::LayoutChanged);
                (i, s.name.clone(), flagged)
            })
            .collect();
        let picked = Frame::new()
            .fill(color::SURFACE_1)
            .inner_margin(Margin::symmetric(tp_ui::tokens::space::SM as i8, 2))
            .show(ui, |ui| {
                ui.push_id("texture_tabs", |ui| {
                    let mut control = tp_ui::widgets::SegmentedControl::new();
                    for (i, name, flagged) in &tabs {
                        let icon = if *flagged {
                            tp_ui::icons::WARNING
                        } else {
                            tp_ui::icons::VEHICLE
                        };
                        control = control.segment(*i, icon, name, None);
                    }
                    control.show(ui, active)
                })
                .inner
            })
            .inner;
        if let Some(index) = picked {
            ws.set_active_surface(index);
        }
    }
    canvas::show(ui, cmds, ws);
}
