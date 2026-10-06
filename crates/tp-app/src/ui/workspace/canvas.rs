//! Canvas area placeholder: pasteboard with the project's artboard fitted and
//! centered. Navigation and drawing arrive with `canvas-core`.

use egui::{
    Align2, Color32, CornerRadius, Pos2, Rect, Sense, Shadow, Stroke, StrokeKind, Ui, Vec2,
    WidgetInfo, WidgetType,
};
use tp_ui::tokens::{color, space};

use crate::commands::CommandId;
use crate::state::Workspace;
use crate::ui::CommandUi;

/// Margin between the canvas edges and the fitted artboard.
pub const FIT_MARGIN: f32 = 40.0;
/// Room above the artboard for its label.
const LABEL_SPACE: f32 = 18.0;
const ARTBOARD_FILL: Color32 = Color32::from_rgb(0xE6, 0xE7, 0xEA);
const ARTBOARD_GRID: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 14);

/// Largest square fitting in `area` with margins, centered.
pub fn fit_artboard(area: Rect) -> Rect {
    let inner = Rect::from_min_max(
        area.min + Vec2::new(FIT_MARGIN, FIT_MARGIN + LABEL_SPACE),
        area.max - Vec2::splat(FIT_MARGIN),
    );
    let side = inner.width().min(inner.height()).max(1.0);
    Rect::from_center_size(inner.center(), Vec2::splat(side))
}

/// Maps a screen position to texture pixel coordinates, if over the artboard.
pub fn screen_to_texture(artboard: Rect, texture_side: u32, pos: Pos2) -> Option<(u32, u32)> {
    if !artboard.contains(pos) || artboard.width() <= 0.0 {
        return None;
    }
    let side = texture_side as f32;
    let to_px = |offset: f32, extent: f32| -> u32 {
        ((offset / extent) * side).floor().clamp(0.0, side - 1.0) as u32
    };
    Some((
        to_px(pos.x - artboard.left(), artboard.width()),
        to_px(pos.y - artboard.top(), artboard.height()),
    ))
}

/// Zoom level (in percent) at which a texture is displayed.
pub fn zoom_percent(artboard: Rect, texture_side: u32, pixels_per_point: f32) -> f32 {
    artboard.width() * pixels_per_point / texture_side as f32 * 100.0
}

/// Draws the canvas area and returns the artboard rectangle.
pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, ws: &Workspace) -> Rect {
    let area = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(area, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, "Canvas"));
    let artboard = fit_artboard(area);
    let painter = ui.painter_at(area);

    let shadow = Shadow {
        offset: [0, 6],
        blur: 24,
        spread: 0,
        color: color::SHADOW,
    };
    painter.add(shadow.as_shape(artboard, CornerRadius::ZERO));
    painter.rect_filled(artboard, CornerRadius::ZERO, ARTBOARD_FILL);

    // Faint 8×8 grid hinting at UV space.
    let grid = Stroke::new(1.0, ARTBOARD_GRID);
    for i in 1..8 {
        let t = i as f32 / 8.0;
        painter.vline(
            artboard.left() + artboard.width() * t,
            artboard.y_range(),
            grid,
        );
        painter.hline(
            artboard.x_range(),
            artboard.top() + artboard.height() * t,
            grid,
        );
    }
    painter.rect_stroke(
        artboard,
        CornerRadius::ZERO,
        Stroke::new(1.0, color::BORDER_STRONG),
        StrokeKind::Outside,
    );

    let side = ws.project.resolution.side();
    painter.text(
        artboard.left_top() - Vec2::new(0.0, space::SM),
        Align2::LEFT_BOTTOM,
        format!("{} · {side} × {side} px", ws.active_surface),
        egui::TextStyle::Small.resolve(ui.style()),
        color::TEXT_SECONDARY,
    );

    response.context_menu(|ui| {
        cmds.menu_item(ui, CommandId::Paste);
        cmds.menu_item(ui, CommandId::SelectAll);
        ui.separator();
        cmds.menu_item(ui, CommandId::FitToScreen);
        cmds.menu_item(ui, CommandId::ActualSize);
        ui.separator();
        cmds.menu_item(ui, CommandId::ShowGrid);
        cmds.menu_item(ui, CommandId::ShowGuides);
    });
    artboard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artboard_is_square_centered_and_inside() {
        let area = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(1000.0, 600.0));
        let a = fit_artboard(area);
        assert!((a.width() - a.height()).abs() < 1e-3);
        assert!(area.contains_rect(a));
        assert!((a.center().x - area.center().x).abs() < 1e-3);
    }

    #[test]
    fn corners_map_to_texture_bounds() {
        let a = Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::splat(400.0));
        assert_eq!(screen_to_texture(a, 4096, a.min), Some((0, 0)));
        assert_eq!(
            screen_to_texture(a, 4096, a.max - Vec2::splat(0.01)),
            Some((4095, 4095))
        );
        assert_eq!(screen_to_texture(a, 4096, a.center()), Some((2048, 2048)));
        assert_eq!(screen_to_texture(a, 4096, Pos2::new(0.0, 0.0)), None);
    }

    #[test]
    fn zoom_accounts_for_pixels_per_point() {
        let a = Rect::from_min_size(Pos2::ZERO, Vec2::splat(512.0));
        assert!((zoom_percent(a, 4096, 1.0) - 12.5).abs() < 1e-3);
        assert!((zoom_percent(a, 4096, 2.0) - 25.0).abs() < 1e-3);
    }
}
