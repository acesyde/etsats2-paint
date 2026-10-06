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
use crate::state::{AppState, disabled_reason, is_enabled};

const SIDEBAR_WIDTH: f32 = 280.0;
const ROW_HEIGHT: f32 = 52.0;

pub fn show(ui: &mut Ui, state: &mut AppState) {
    let ctx = ui.ctx().clone();
    let mut cmds = CommandUi::new(&ctx, &mut state.queue, EditContext::default());

    Panel::top("home_menu_bar")
        .frame(bar_frame())
        .show(ui, |ui| menu_bar::show(ui, &mut cmds, None));

    Panel::left("home_sidebar")
        .exact_size(SIDEBAR_WIDTH)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(color::SURFACE_1)
                .inner_margin(Margin::same(space::XL as i8)),
        )
        .show(ui, |ui| sidebar(ui, &mut cmds));

    let mut remove = None;
    CentralPanel::no_frame()
        .frame(
            Frame::new()
                .fill(color::SURFACE_0)
                .inner_margin(Margin::symmetric(40, 32)),
        )
        .show(ui, |ui| {
            remove = recent_list(ui, &state.prefs.recent, &state.recent_available);
        });
    if let Some(index) = remove {
        state.prefs.recent.remove(index);
        state.recent_available.remove(index);
    }
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

/// Draws the recent projects list; returns the index of an entry to remove.
fn recent_list(ui: &mut Ui, recent: &[RecentProject], available: &[bool]) -> Option<usize> {
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
    let mut remove = None;
    ScrollArea::vertical().show(ui, |ui| {
        for (index, project) in recent.iter().enumerate() {
            let exists = available.get(index).copied().unwrap_or(false);
            if recent_row(ui, project, exists, now) {
                remove = Some(index);
            }
            ui.add_space(space::XS);
        }
    });
    remove
}

/// One recent entry. Returns true when "Remove from list" was clicked.
fn recent_row(ui: &mut Ui, project: &RecentProject, exists: bool, now: u64) -> bool {
    let width = ui.available_width().min(760.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::hover());
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

    if !removed {
        let tip = if !exists {
            "This project's file was moved or deleted."
        } else {
            disabled_reason(CommandId::OpenProject).unwrap_or_default()
        };
        response.on_hover_text(tip);
    }
    removed
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
