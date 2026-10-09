//! The top bar of an open project, in every space: the project name and its
//! game, the space switcher, the breadcrumb (in the Workshop) and Export….

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

/// Room kept for the breadcrumb before the project name is shortened.
const BREADCRUMB_MIN: f32 = 120.0;

pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, ws: &Workspace) {
    let labels = Space::ALL.map(|space| tr(space.label()));
    let control = switcher(cmds, &labels);
    let (picked, ()) = Sides::new()
        .height(ui.available_height())
        .spacing(space::LG)
        .shrink_left()
        .truncate()
        .show(
            ui,
            |ui| identity(ui, ws, control),
            |ui| {
                cmds.primary_button(ui, CommandId::ExportMod, &tr("top-bar-export"));
            },
        );
    if let Some(space) = picked {
        cmds.push(CommandId::ShowSpace(space));
    }
}

/// Project name, game badge, the space switcher and, in the Workshop, the
/// breadcrumb. The name is shortened with an ellipsis when the bar is too
/// narrow (the switcher stays whole), then the breadcrumb.
/// Returns the space picked in the switcher.
fn identity(ui: &mut Ui, ws: &Workspace, control: SegmentedControl<'_, Space>) -> Option<Space> {
    let name = ws.project.name.clone();
    let game = ws.project.game().map(|g| g.to_uppercase());
    let workshop = ws.space == Space::Workshop;
    // What the name leaves: the badge, the switcher and some breadcrumb.
    let badge_width = game.as_ref().map_or(0.0, |g| badge_size(ui, g).x);
    let reserved = badge_width
        + control.width(ui)
        + space::MD
        + 2.0 * ui.spacing().item_spacing.x
        + if workshop { BREADCRUMB_MIN } else { 0.0 };
    let budget = (ui.available_width() - reserved).max(40.0);
    ui.scope(|ui| {
        ui.set_max_width(budget);
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
    if let Some(game) = &game {
        badge(ui, game);
    }
    ui.add_space(space::MD - ui.spacing().item_spacing.x);
    let picked = control.show(ui, ws.space);
    if workshop {
        ui.add_space(space::MD);
        breadcrumb::label(ui, &Crumbs::of(ws));
    }
    picked
}

/// Padding of the game's badge around its text.
const BADGE_PAD: egui::Vec2 = egui::vec2(space::XS + 2.0, space::XXS);

fn badge_galley(ui: &Ui, text: &str) -> std::sync::Arc<egui::Galley> {
    ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::monospace(typography::CAPTION),
        color::TEXT_SECONDARY,
    )
}

fn badge_size(ui: &Ui, text: &str) -> egui::Vec2 {
    badge_galley(ui, text).size() + 2.0 * BADGE_PAD
}

/// The game's badge ("ETS2", "ATS"): mono caption on a chip.
fn badge(ui: &mut Ui, text: &str) {
    let galley = badge_galley(ui, text);
    let (rect, response) =
        ui.allocate_exact_size(galley.size() + 2.0 * BADGE_PAD, egui::Sense::hover());
    ui.painter().rect_filled(rect, radius::SM, color::CHIP);
    ui.painter()
        .galley(rect.min + BADGE_PAD, galley, color::TEXT_SECONDARY);
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
}

/// Project / Workshop / Brand as ghost tabs; the shown space is the
/// active one.
fn switcher<'a>(cmds: &CommandUi<'_>, labels: &'a [String; 3]) -> SegmentedControl<'a, Space> {
    let mut control = SegmentedControl::new().tabs();
    for (space, label) in Space::ALL.into_iter().zip(labels) {
        let shortcut = cmds.shortcuts.command(CommandId::ShowSpace(space));
        control = control.segment(space, "", label, shortcut);
    }
    control
}
