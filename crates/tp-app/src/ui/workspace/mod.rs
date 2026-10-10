//! Editor workspace: the menu bar, the top bar and the status bar shared by
//! the three spaces, and each space's content. The Workshop is the tool
//! options bar, the tool rail, the left panel, the canvas area and the
//! inspector.

pub mod breadcrumb;
pub mod canvas;
pub mod inspector;
pub mod left_panel;
pub mod panels;
pub mod spaces;
pub mod status_bar;
pub mod tool_options;
pub mod tool_rail;
pub mod top_bar;

use egui::{CentralPanel, Frame, Margin, Panel, Ui};
use tp_ui::tokens::{color, size, space};

use super::home::bar_frame;
use super::{CommandUi, menu_bar};
use crate::layout::Space;
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
    let now = ctx.input(|i| i.time);
    let mut cmds = CommandUi::new(&ctx, queue, edit);
    let space = ws.space;
    // The Project space sets where its pictures take dropped files; a mod
    // setting typed in when another space was shown is committed.
    ws.picture_drop_zones = [None; 2];
    if space != Space::Project {
        ws.commit_mod_draft(now);
    }

    Panel::top("menu_bar")
        .frame(bar_frame())
        .show(ui, |ui| menu_bar::show(ui, &mut cmds, Some(layout), aids));

    Panel::top("top_bar")
        .exact_size(size::TOP_BAR_HEIGHT)
        .frame(bar_frame().inner_margin(Margin::symmetric(space::MD as i8, 0)))
        .show(ui, |ui| top_bar::show(ui, &mut cmds, ws));

    Panel::bottom("status_bar")
        .exact_size(size::STATUS_BAR_HEIGHT)
        .frame(bar_frame().inner_margin(Margin::symmetric(space::MD as i8, 0)))
        .show(ui, |ui| status_bar::show(ui, &mut cmds, ws, aids));

    macro_rules! env {
        () => {
            panels::PanelEnv {
                ws: &mut *ws,
                recent_colors: recent_colors.as_slice(),
                vehicles: &*vehicles,
                vehicle_request: &mut *vehicle_request,
                library: &mut *library,
                now,
            }
        };
    }

    // Edit Swatch…, opened from any list of the palette.
    panels::colors::edit_swatch_popup(&ctx, &mut env!());

    match space {
        Space::Project => spaces::project::show(ui, &mut cmds, &mut env!()),
        Space::Brand => spaces::brand::show(ui, &mut cmds, &mut env!()),
        Space::Workshop => {
            Panel::top("tool_options")
                .exact_size(size::TOOL_OPTIONS_HEIGHT)
                .frame(bar_frame().inner_margin(Margin::symmetric(space::MD as i8, 0)))
                .show(ui, |ui| tool_options::show(ui, &mut env!()));

            Panel::left("tool_rail")
                .exact_size(size::TOOL_RAIL_WIDTH)
                .resizable(false)
                .frame(
                    Frame::new()
                        .fill(color::SURFACE_1)
                        .inner_margin(Margin::symmetric(0, space::SM as i8)),
                )
                .show(ui, |ui| tool_rail::show(ui, &mut cmds, ws.tool));

            // Hide Panels (Tab) hides the left panel and the inspector; the
            // rail and the bars stay.
            if !layout.panels_hidden {
                left_panel::show(ui, &mut cmds, layout, &mut env!());
                inspector::show(ui, &mut cmds, layout, &mut env!());
            }

            CentralPanel::no_frame()
                .frame(Frame::new().fill(color::SURFACE_0))
                .show(ui, |ui| {
                    canvas::show(ui, &mut cmds, ws);
                    breadcrumb::inlaid(ui, ws);
                });
        }
    }
}
