//! Status bar: zoom, pointer position, active surface, save state.

use egui::{Align, Layout, RichText, Ui};
use tp_ui::icons;
use tp_ui::tokens::{color, space};

use super::canvas::{screen_to_texture, zoom_percent};
use crate::layout::ViewMode;
use crate::state::{SaveState, Workspace};

fn item(ui: &mut Ui, text: impl Into<String>) {
    ui.label(
        RichText::new(text.into())
            .small()
            .color(color::TEXT_SECONDARY),
    );
}

fn divider(ui: &mut Ui) {
    ui.add_space(space::XS);
    ui.separator();
    ui.add_space(space::XS);
}

/// Formats a zoom percentage compactly (`12.5%`, `67%`).
pub fn format_zoom(percent: f32) -> String {
    if percent < 10.0 {
        format!("{percent:.1}%")
    } else {
        format!("{percent:.0}%")
    }
}

pub fn show(ui: &mut Ui, ws: &Workspace, view_mode: ViewMode) {
    let side = ws.project.resolution.side();
    let ppp = ui.ctx().pixels_per_point();
    ui.horizontal_centered(|ui| {
        let zoom = match (view_mode.shows_canvas(), ws.artboard_rect) {
            (true, Some(rect)) => format_zoom(zoom_percent(rect, side, ppp)),
            _ => "—".to_owned(),
        };
        item(ui, format!("Zoom {zoom}"));
        divider(ui);

        let pointer = ui.ctx().pointer_hover_pos();
        let position = match (ws.artboard_rect, pointer) {
            (Some(rect), Some(pos)) if view_mode.shows_canvas() => {
                screen_to_texture(rect, side, pos)
            }
            _ => None,
        };
        let position = position.map_or_else(
            || "X —  Y —".to_owned(),
            |(x, y)| format!("X {x}  Y {y} px"),
        );
        item(ui, position);
        divider(ui);
        item(ui, &ws.active_surface);

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (icon, text, tint) = match ws.save_state {
                SaveState::Saved => (icons::SAVED, "Saved", color::SUCCESS),
                SaveState::Unsaved => (icons::UNSAVED, "Unsaved changes", color::WARNING),
            };
            ui.label(RichText::new(text).small().color(color::TEXT_PRIMARY));
            ui.label(icons::rich(icon).color(tint));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::format_zoom;

    #[test]
    fn zoom_formatting() {
        assert_eq!(format_zoom(8.0), "8.0%");
        assert_eq!(format_zoom(66.66), "67%");
    }
}
