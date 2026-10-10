//! The top bar of an open project on macOS and with the system title bar,
//! in every space, 40 px high: the project name and its game, the space
//! switcher and Export…. On macOS it sits in the window's title strip, after
//! the traffic lights, and its free area moves the window. The drawn title
//! bar (`ui::title_bar`) reuses its switcher and Export….

use egui::{
    Label, PointerButton, RichText, Sense, Sides, Ui, ViewportCommand, WidgetInfo, WidgetType,
};
use tp_i18n::tr;
use tp_ui::theme::label_strong_style;
use tp_ui::tokens::{color, radius, space, typography};
use tp_ui::widgets::SegmentedControl;

use crate::commands::CommandId;
use crate::layout::Space;
use crate::title_bar::{DoubleClickAction, TitleBarMode};
use crate::ui::CommandUi;
use crate::workspace::Workspace;

pub fn show(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    ws: &Workspace,
    mode: TitleBarMode,
    double_click: DoubleClickAction,
) {
    if mode == TitleBarMode::MacNative {
        drag_area(ui, double_click);
    }
    let labels = space_labels();
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
                export_button(ui, cmds);
            },
        );
    if let Some(space) = picked {
        cmds.push(CommandId::ShowSpace(space));
    }
}

/// The whole bar, under its widgets, as a title bar: dragging it moves the
/// window, and a double-click does the system's action. Call it before
/// drawing the bar's widgets, which keep their own clicks.
pub fn drag_area(ui: &mut Ui, double_click: DoubleClickAction) {
    let response = ui.interact(
        ui.clip_rect(),
        ui.id().with("title_bar_drag"),
        Sense::click_and_drag(),
    );
    if response.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }
    if response.double_clicked() {
        double_click.apply(ui.ctx());
    }
}

/// Export…, the primary action, running Export Mod….
pub fn export_button(ui: &mut Ui, cmds: &mut CommandUi<'_>) -> egui::Response {
    cmds.primary_button(ui, CommandId::ExportMod, &tr("top-bar-export"))
}

/// The switcher's labels: Project, Workshop, Brand.
pub fn space_labels() -> [String; 3] {
    Space::ALL.map(|space| tr(space.label()))
}

/// Project name, game badge and the space switcher. The name is shortened
/// with an ellipsis when the bar is too narrow (the switcher stays whole).
/// Returns the space picked in the switcher.
fn identity(ui: &mut Ui, ws: &Workspace, control: SegmentedControl<'_, Space>) -> Option<Space> {
    let name = ws.project.name.clone();
    let game = ws.project.game().map(|g| g.to_uppercase());
    // What the name leaves: the badge and the switcher.
    let badge_width = game.as_ref().map_or(0.0, |g| badge_size(ui, g).x);
    let reserved = badge_width + control.width(ui) + space::MD + 2.0 * ui.spacing().item_spacing.x;
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
    control.show(ui, ws.space)
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

/// Project / Workshop / Brand as ghost tabs, from [`space_labels`]; the
/// shown space is the active one.
pub fn switcher<'a>(cmds: &CommandUi<'_>, labels: &'a [String; 3]) -> SegmentedControl<'a, Space> {
    let mut control = SegmentedControl::new().tabs();
    for (space, label) in Space::ALL.into_iter().zip(labels) {
        let shortcut = cmds.shortcuts.command(CommandId::ShowSpace(space));
        control = control.segment(space, "", label, shortcut);
    }
    control
}
