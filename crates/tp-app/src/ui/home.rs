//! Home screen: New Project, Open Project and Recent Projects.

use egui::{
    Align, Align2, CentralPanel, CornerRadius, Frame, Layout, Margin, Panel, RichText, ScrollArea,
    Sense, Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_ui::icons;
use tp_ui::theme::{display_style, label_strong_style, title_style};
use tp_ui::tokens::{color, radius, size, space};
use tp_ui::widgets::{EmptyState, IconButton, primary_button, secondary_button};

use super::{CommandUi, menu_bar};
use crate::commands::{CommandId, EditContext};
use crate::prefs::{RecentProject, now_unix};
use crate::recovery::Recovered;
use crate::state::{AppState, PendingAction, disabled_reason, is_enabled};

const SIDEBAR_WIDTH: f32 = 280.0;
const ROW_HEIGHT: f32 = 52.0;

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let ctx = ui.ctx().clone();
    let mut cmds = CommandUi::new(&ctx, &mut state.queue, EditContext::default());

    Panel::top("home_menu_bar")
        .frame(bar_frame())
        .show(ui, |ui| {
            menu_bar::show(ui, &mut cmds, None, Default::default())
        });

    Panel::left("home_sidebar")
        .exact_size(SIDEBAR_WIDTH)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(color::SURFACE_1)
                .inner_margin(Margin::same(space::XL as i8)),
        )
        .show(ui, |ui| sidebar(ui, &mut cmds));

    let mut action = None;
    let mut recovery_action = None;
    CentralPanel::no_frame()
        .frame(
            Frame::new()
                .fill(color::SURFACE_0)
                .inner_margin(Margin::symmetric(40, 32)),
        )
        .show(ui, |ui| {
            if !state.recovered.is_empty() {
                recovery_action = recovered_list(ui, &state.recovered);
                ui.add_space(space::XXL);
            }
            action = recent_list(ui, &state.prefs.recent, &state.recent_available);
        });
    match action {
        Some((index, RowAction::Remove)) => {
            state.prefs.recent.remove(index);
            state.recent_available.remove(index);
        }
        Some((index, RowAction::Open)) => {
            let path = state.prefs.recent[index].path.clone();
            state.guard(&ctx, PendingAction::Open(path));
        }
        None => {}
    }
    match recovery_action {
        Some((session, true)) => state.guard(&ctx, PendingAction::Restore(session)),
        Some((session, false)) => state.discard_recovered(&session),
        None => {}
    }
}

/// What a click on a recent entry asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RowAction {
    Open,
    Remove,
}

/// Recovered projects; returns a session to restore (true) or discard.
fn recovered_list(ui: &mut Ui, recovered: &[Recovered]) -> Option<(String, bool)> {
    ui.horizontal(|ui| {
        ui.label(icons::rich(icons::WARNING).size(20.0).color(color::WARNING));
        ui.label(
            RichText::new("Recovered projects")
                .text_style(title_style())
                .color(color::TEXT_PRIMARY),
        );
    });
    ui.label(
        RichText::new("TruckPaint closed unexpectedly. These projects had unsaved changes.")
            .color(color::TEXT_SECONDARY),
    );
    ui.add_space(space::SM);
    let now = now_unix();
    let mut chosen = None;
    for r in recovered {
        let width = ui.available_width().min(760.0);
        Frame::new()
            .fill(color::SURFACE_1)
            .corner_radius(CornerRadius::same(radius::MD))
            .inner_margin(Margin::symmetric(space::LG as i8, space::SM as i8))
            .show(ui, |ui| {
                ui.set_width(width - 2.0 * space::LG);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        let (title, detail) = match &r.meta {
                            Some(meta) => (
                                meta.name.clone(),
                                format!(
                                    "{} · {}",
                                    meta.original
                                        .as_ref()
                                        .map_or("Never saved".to_owned(), |p| p
                                            .display()
                                            .to_string()),
                                    relative_time(now, meta.saved_at)
                                ),
                            ),
                            None => (
                                "Damaged recovery copy".to_owned(),
                                "This copy cannot be restored.".to_owned(),
                            ),
                        };
                        let label = ui.label(
                            RichText::new(&title)
                                .text_style(label_strong_style())
                                .color(color::TEXT_PRIMARY),
                        );
                        label.widget_info(|| {
                            WidgetInfo::labeled(
                                WidgetType::Label,
                                true,
                                format!("Recovered {title}"),
                            )
                        });
                        ui.label(RichText::new(detail).small().color(color::TEXT_SECONDARY));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if r.meta.is_some() && ui.add(primary_button("Restore")).clicked() {
                            chosen = Some((r.session.clone(), true));
                        }
                        if ui.add(secondary_button("Discard")).clicked() {
                            chosen = Some((r.session.clone(), false));
                        }
                    });
                });
            });
        ui.add_space(space::XS);
    }
    chosen
}

pub fn bar_frame() -> Frame {
    Frame::new()
        .fill(color::SURFACE_1)
        .inner_margin(Margin::symmetric(space::SM as i8, 0))
}

fn sidebar(ui: &mut Ui, cmds: &mut CommandUi<'_>) {
    ui.horizontal(|ui| {
        ui.label(icons::rich(icons::VEHICLE).size(30.0).color(color::ACCENT));
        ui.label(
            RichText::new(crate::paths::APP_NAME)
                .text_style(display_style())
                .color(color::TEXT_PRIMARY),
        );
    });
    ui.label(
        RichText::new("Livery editor for Euro Truck Simulator 2 and American Truck Simulator")
            .color(color::TEXT_SECONDARY),
    );
    ui.add_space(space::XXL);

    let full = Vec2::new(ui.available_width(), 34.0);
    let new_tip = cmds.shortcuts.command(CommandId::NewProject);
    let response = ui
        .add(primary_button("New Project").min_size(full))
        .on_hover_text(format!(
            "Create a new livery project ({})",
            new_tip.unwrap_or_default()
        ));
    if response.clicked() {
        cmds.push(CommandId::NewProject);
    }
    ui.add_space(space::SM);
    let open_enabled = cmds.enabled(CommandId::OpenProject);
    let response = ui
        .add_enabled(
            open_enabled,
            secondary_button("Open Project…").min_size(full),
        )
        .on_disabled_hover_text(disabled_reason(CommandId::OpenProject).unwrap_or_default());
    if response.clicked() {
        cmds.push(CommandId::OpenProject);
    }

    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        ui.horizontal(|ui| {
            cmds.icon_button(ui, CommandId::Preferences, false);
            ui.label(RichText::new("Preferences").color(color::TEXT_SECONDARY));
        });
    });
}

/// Draws the recent projects list; returns the entry clicked and what for.
fn recent_list(
    ui: &mut Ui,
    recent: &[RecentProject],
    available: &[bool],
) -> Option<(usize, RowAction)> {
    ui.label(
        RichText::new("Recent projects")
            .text_style(title_style())
            .color(color::TEXT_PRIMARY),
    );
    ui.add_space(space::LG);

    if recent.is_empty() {
        ui.allocate_ui(Vec2::new(ui.available_width().min(520.0), 200.0), |ui| {
            EmptyState::new(
                icons::RECENT,
                "No recent projects",
                "Projects you open will appear here. Start by creating a new project.",
            )
            .show(ui);
        });
        return None;
    }

    let now = now_unix();
    let mut action = None;
    ScrollArea::vertical().show(ui, |ui| {
        for (index, project) in recent.iter().enumerate() {
            let exists = available.get(index).copied().unwrap_or(false);
            if let Some(a) = recent_row(ui, project, exists, now) {
                action = Some((index, a));
            }
            ui.add_space(space::XS);
        }
    });
    action
}

/// One recent entry: opens on click, or offers "Remove from list" when its
/// file is missing.
fn recent_row(ui: &mut Ui, project: &RecentProject, exists: bool, now: u64) -> Option<RowAction> {
    let width = ui.available_width().min(760.0);
    let sense = if exists {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), sense);
    let open_enabled = is_enabled(CommandId::OpenProject, &EditContext::default());
    let label = if exists {
        project.name.clone()
    } else {
        format!("{} (file not found)", project.name)
    };
    response
        .widget_info(|| WidgetInfo::labeled(WidgetType::Button, open_enabled && exists, &label));

    let painter = ui.painter();
    let bg = if response.hovered() {
        color::SURFACE_2
    } else {
        color::SURFACE_1
    };
    painter.rect_filled(rect, CornerRadius::same(radius::MD), bg);

    let icon = if exists {
        icons::FILE
    } else {
        icons::FILE_MISSING
    };
    let text_color = if exists {
        color::TEXT_PRIMARY
    } else {
        color::TEXT_SECONDARY
    };
    painter.text(
        egui::pos2(rect.left() + space::LG + 8.0, rect.center().y),
        Align2::CENTER_CENTER,
        icon,
        icons::font(20.0),
        text_color,
    );
    let text_x = rect.left() + space::LG * 2.0 + 16.0;
    painter.text(
        egui::pos2(text_x, rect.top() + 17.0),
        Align2::LEFT_CENTER,
        &project.name,
        label_strong_style().resolve(ui.style()),
        text_color,
    );
    painter.text(
        egui::pos2(text_x, rect.top() + 35.0),
        Align2::LEFT_CENTER,
        project.path.display().to_string(),
        egui::TextStyle::Small.resolve(ui.style()),
        color::TEXT_SECONDARY,
    );

    let mut removed = false;
    let right = rect.right() - space::MD;
    if exists {
        painter.text(
            egui::pos2(right, rect.center().y),
            Align2::RIGHT_CENTER,
            relative_time(now, project.last_opened),
            egui::TextStyle::Small.resolve(ui.style()),
            color::TEXT_SECONDARY,
        );
    } else {
        let button_rect = egui::Rect::from_center_size(
            egui::pos2(right - size::HIT_MIN / 2.0, rect.center().y),
            Vec2::splat(size::HIT_MIN),
        );
        removed = ui
            .put(
                button_rect,
                IconButton::new(icons::REMOVE, "Remove from list"),
            )
            .clicked();
        let text_rect = ui.painter().text(
            egui::pos2(button_rect.left() - space::SM, rect.center().y),
            Align2::RIGHT_CENTER,
            "File not found",
            egui::TextStyle::Small.resolve(ui.style()),
            color::WARNING,
        );
        ui.painter().text(
            egui::pos2(text_rect.left() - space::XS, rect.center().y),
            Align2::RIGHT_CENTER,
            icons::WARNING,
            icons::font(size::ICON - 2.0),
            color::WARNING,
        );
    }

    if removed {
        return Some(RowAction::Remove);
    }
    if exists && open_enabled {
        let response = response.on_hover_text(format!("Open {}", project.path.display()));
        if response.clicked() {
            return Some(RowAction::Open);
        }
    } else if !exists {
        response.on_hover_text("This project's file was moved or deleted.");
    }
    None
}

/// Human-friendly "time ago" text.
pub fn relative_time(now: u64, then: u64) -> String {
    let secs = now.saturating_sub(then);
    let plural = |n: u64, unit: &str| {
        if n == 1 {
            format!("1 {unit} ago")
        } else {
            format!("{n} {unit}s ago")
        }
    };
    match secs {
        0..60 => "Just now".to_owned(),
        60..3_600 => plural(secs / 60, "minute"),
        3_600..86_400 => plural(secs / 3_600, "hour"),
        86_400..172_800 => "Yesterday".to_owned(),
        172_800..2_592_000 => plural(secs / 86_400, "day"),
        2_592_000..31_536_000 => plural(secs / 2_592_000, "month"),
        _ => plural(secs / 31_536_000, "year"),
    }
}

#[cfg(test)]
mod tests {
    use super::relative_time;

    #[test]
    fn relative_times() {
        assert_eq!(relative_time(100, 90), "Just now");
        assert_eq!(relative_time(10_000, 10_000 - 120), "2 minutes ago");
        assert_eq!(relative_time(100_000, 100_000 - 3_600), "1 hour ago");
        assert_eq!(relative_time(1_000_000, 1_000_000 - 90_000), "Yesterday");
        assert_eq!(
            relative_time(10_000_000, 10_000_000 - 5 * 86_400),
            "5 days ago"
        );
        assert_eq!(relative_time(5, 10), "Just now");
    }
}
