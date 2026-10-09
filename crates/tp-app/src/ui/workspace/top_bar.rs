//! The top bar of an open project, in every space: the project name and its
//! game, the breadcrumb (in the Workshop), the space switcher and Export….

use egui::{Label, RichText, Sides, Ui, WidgetInfo, WidgetType};
use tp_i18n::tr;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, radius, space, typography};
use tp_ui::widgets::SegmentedControl;

use super::breadcrumb::{self, Crumbs};
use crate::commands::CommandId;
use crate::layout::Space;
use crate::ui::CommandUi;
use crate::workspace::Workspace;

/// Share of the left side the project name may take before it is
/// shortened, so the breadcrumb keeps some room.
const NAME_SHARE: f32 = 0.45;

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, ws: &Workspace) {
    let space = ws.space;
    Sides::new()
        .height(ui.available_height())
        .spacing(space::LG)
        .shrink_left()
        .truncate()
        .show(
            ui,
            |ui| identity(ui, ws, space == Space::Workshop),
            |ui| {
                cmds.primary_button(ui, CommandId::ExportMod, &tr("top-bar-export"));
                ui.add_space(space::SM);
                switcher(ui, cmds, space);
            },
        );
}

/// Project name, game badge and, in the Workshop, the breadcrumb. Texts are
/// shortened with an ellipsis when the bar is too narrow, the name first.
fn identity(ui: &mut Ui, ws: &Workspace, workshop: bool) {
    let name = ws.project.name.clone();
    let budget = ui.available_width();
    ui.scope(|ui| {
        ui.set_max_width(budget * NAME_SHARE);
        ui.add(
            Label::new(
                RichText::new(&name)
                    .text_style(label_strong_style())
                    .color(color::TEXT_PRIMARY),
            )
            .truncate(),
        )
        .on_hover_text(&name);
    });
    if let Some(game) = ws.project.game() {
        badge(ui, &game.to_uppercase());
    }
    if workshop {
        ui.add_space(space::MD);
        breadcrumb::label(ui, &Crumbs::of(ws));
    }
}

/// The game's badge ("ETS2", "ATS"): mono caption on a raised chip.
fn badge(ui: &mut Ui, text: &str) {
    let pad = egui::vec2(space::XS + 2.0, space::XXS);
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::monospace(typography::CAPTION),
        color::TEXT_SECONDARY,
    );
    let (rect, response) = ui.allocate_exact_size(galley.size() + 2.0 * pad, egui::Sense::hover());
    ui.painter().rect_filled(rect, radius::SM, color::SURFACE_3);
    ui.painter()
        .galley(rect.min + pad, galley, color::TEXT_SECONDARY);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
}

/// Project / Workshop / Brand; the shown space is the active option.
fn switcher(ui: &mut Ui, cmds: &mut CommandUi<'_>, shown: Space) {
    let labels = Space::ALL.map(|space| tr(space.label()));
    let mut control = SegmentedControl::new();
    for (space, label) in Space::ALL.into_iter().zip(&labels) {
        let shortcut = cmds.shortcuts.command(CommandId::ShowSpace(space));
        control = control.segment(space, "", label, shortcut);
    }
    if let Some(space) = control.show(ui, shown) {
        cmds.push(CommandId::ShowSpace(space));
    }
}
