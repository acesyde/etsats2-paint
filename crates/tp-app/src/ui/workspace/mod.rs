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
            ViewMode::TwoD => canvas::show(ui, &mut cmds, ws),
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
                    .show(ui, |ui| canvas::show(ui, &mut cmds, ws));
            }
        });
}
