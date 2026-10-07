//! Status bar: zoom, pointer position, active surface, save state.

use egui::{Align, Layout, RichText, Ui};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, space};

use crate::layout::ViewMode;
use crate::workspace::{SaveState, Workspace};

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
        format!("{}%", tp_i18n::localize_number(&format!("{percent:.1}")))
    } else {
        format!("{percent:.0}%")
    }
}

pub fn show(ui: &mut Ui, ws: &Workspace, view_mode: ViewMode) {
    ui.horizontal_centered(|ui| {
        let zoom = match (view_mode.shows_canvas(), ws.viewport) {
            (true, Some(view)) => format_zoom((view.zoom * 100.0) as f32),
            _ => "—".to_owned(),
        };
        item(ui, tr!("status-zoom", zoom = zoom));
        divider(ui);

        let ppp = ui.ctx().pixels_per_point();
        let pointer = ui.ctx().pointer_hover_pos();
        let side = ws.project.surface().size;
        let position = match (ws.viewport, ws.canvas_rect, pointer) {
            (Some(view), Some(canvas), Some(pos))
                if view_mode.shows_canvas() && canvas.contains(pos) =>
            {
                let p = view.map(canvas, ppp).to_doc(pos);
                (p.x >= 0.0 && p.y >= 0.0 && p.x < side && p.y < side)
                    .then(|| (p.x.floor() as i64, p.y.floor() as i64))
            }
            _ => None,
        };
        let position = position.map_or_else(
            || "X —  Y —".to_owned(),
            |(x, y)| format!("X {x}  Y {y} px"),
        );
        item(ui, position);
        divider(ui);
        // "Vehicle › Texture", shortened when too long.
        let active = ws.project.active_surface;
        let name = ws.project.surface_names(active).map_or_else(
            || ws.project.surface().name.clone(),
            |(v, t)| format!("{v} › {t}"),
        );
        let label =
            egui::Label::new(RichText::new(&name).small().color(color::TEXT_SECONDARY)).truncate();
        ui.add(label).on_hover_text(&name);

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (icon, text, tint) = match ws.save_state() {
                SaveState::Saved => (icons::SAVED, tr("status-saved"), color::SUCCESS),
                SaveState::Unsaved => (icons::UNSAVED, tr("status-unsaved"), color::WARNING),
                SaveState::Saving => (icons::SAVE, tr("status-saving"), color::TEXT_SECONDARY),
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
