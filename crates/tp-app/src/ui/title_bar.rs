//! The title bar drawn by the app on Windows and Linux (the window has no
//! system frame), 40 px high: the TruckPaint mark, the menus, the space
//! switcher centered when it fits, Export… and the window controls. Its free
//! area moves the window and a double-click maximizes or restores it. The
//! window's edges get resize grips.

use egui::viewport::ResizeDirection;
use egui::{
    Align, Context, CursorIcon, Id, Layout, Order, PointerButton, Rect, Response, Sense, Stroke,
    Ui, UiBuilder, Vec2, ViewportCommand, WidgetInfo, WidgetType, pos2, vec2,
};
use tp_i18n::tr;
use tp_ui::icons;
use tp_ui::tokens::{color, radius, space, stroke};

use super::CommandUi;
use super::menu_bar;
use super::workspace::top_bar;
use crate::commands::CommandId;
use crate::layout::{Space, WorkspaceLayout};
use crate::title_bar::DoubleClickAction;

/// Side of the TruckPaint mark.
const MARK_SIZE: f32 = 22.0;
/// Width of each window control.
pub const CONTROL_WIDTH: f32 = 44.0;
/// Side of a window control's glyph.
const GLYPH: f32 = 10.0;
/// Padding of a menu's title (horizontal, vertical).
const MENU_PADDING: Vec2 = vec2(8.0, 5.0);
/// Width of the resize grips along the window's edges, and side of those
/// at its corners.
pub const GRIP_WIDTH: f32 = 5.0;
pub const GRIP_CORNER: f32 = 10.0;

/// Draws the bar in the panel's `ui`. `space` is the shown space of the open
/// project, `None` on the home screen (no switcher nor Export…); `layout`
/// is `None` on the home screen too.
pub fn show(
    ui: &mut Ui,
    cmds: &mut CommandUi<'_>,
    space: Option<Space>,
    layout: Option<&WorkspaceLayout>,
    aids: crate::prefs::ViewAids,
) {
    let bar = ui.max_rect();
    // Under every widget, so they keep their clicks.
    top_bar::drag_area(ui, DoubleClickAction::Maximize);

    // The mark and the menus, from the left.
    let left = Rect::from_min_max(pos2(bar.left() + space::MD, bar.top()), bar.max);
    let menus_right = ui
        .scope_builder(
            UiBuilder::new()
                .max_rect(left)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                mark(ui);
                ui.add_space(space::SM);
                // The menu bar's row takes the bar's height, so the titles
                // are centered in it; the titles keep the usual height.
                ui.spacing_mut().interact_size.y = ui.available_height();
                egui::MenuBar::new()
                    .style(|style: &mut egui::Style| {
                        egui::containers::menu::menu_style(style);
                        style.spacing.button_padding = MENU_PADDING;
                        style.spacing.item_spacing.x = 0.0;
                        style.spacing.interact_size.y = tp_ui::tokens::size::HIT_MIN;
                    })
                    .ui(ui, |ui| {
                        menu_bar::menu_buttons(ui, cmds, layout, aids);
                        ui.cursor().left()
                    })
                    .inner
            },
        )
        .inner;

    // The window controls and Export…, from the right.
    let export_left = ui
        .scope_builder(
            UiBuilder::new()
                .max_rect(bar)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                window_controls(ui);
                if space.is_some() {
                    ui.add_space(space::SM);
                    top_bar::export_button(ui, cmds);
                }
                ui.min_rect().left()
            },
        )
        .inner;

    // The switcher, centered in the bar when there is room, else just after
    // the menus.
    if let Some(current) = space {
        let labels = top_bar::space_labels();
        let control = top_bar::switcher(cmds, &labels);
        let width = control.width(ui);
        let x = switcher_left(bar, width, menus_right + space::LG, export_left - space::LG);
        let rect = Rect::from_min_size(pos2(x, bar.top()), vec2(width, bar.height()));
        let picked = ui
            .scope_builder(
                UiBuilder::new()
                    .max_rect(rect)
                    .layout(Layout::left_to_right(Align::Center)),
                |ui| control.show(ui, current),
            )
            .inner;
        if let Some(space) = picked {
            cmds.push(CommandId::ShowSpace(space));
        }
    }
}

/// Left edge of a switcher `width` wide: centered in `bar` when it fits
/// between `from` and `to`, else at `from`.
fn switcher_left(bar: Rect, width: f32, from: f32, to: f32) -> f32 {
    let centered = bar.center().x - width / 2.0;
    if centered >= from && centered + width <= to {
        centered
    } else {
        from
    }
}

/// The TruckPaint mark: the app's glyph on a dark square.
fn mark(ui: &mut Ui) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(MARK_SIZE), Sense::hover());
    let painter = ui.painter();
    painter.rect(
        rect,
        radius::SM + 1,
        color::SURFACE_0,
        Stroke::new(stroke::HAIRLINE, color::BORDER_STRONG),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icons::VEHICLE,
        icons::font(14.0),
        color::TEXT_PRIMARY,
    );
    response
        .on_hover_text(crate::paths::APP_NAME)
        .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, crate::paths::APP_NAME));
}

#[derive(Clone, Copy, PartialEq)]
enum Control {
    Minimize,
    Maximize,
    Restore,
    Close,
}

/// Minimize, Maximize or Restore, and Close, laid out right to left.
fn window_controls(ui: &mut Ui) {
    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    let ctx = ui.ctx().clone();
    if window_control(ui, Control::Close).clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
    let middle = if maximized {
        Control::Restore
    } else {
        Control::Maximize
    };
    if window_control(ui, middle).clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    }
    if window_control(ui, Control::Minimize).clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
    }
}

/// One window control, 44 px wide and as high as the bar, named by its
/// tooltip; Close turns red on hover.
fn window_control(ui: &mut Ui, control: Control) -> Response {
    let name = tr(match control {
        Control::Minimize => "window-minimize",
        Control::Maximize => "window-maximize",
        Control::Restore => "window-restore",
        Control::Close => "window-close",
    });
    let size = vec2(CONTROL_WIDTH, ui.available_height());
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &name));
    let hovered = response.hovered() || response.has_focus();
    let ink = if hovered && control == Control::Close {
        ui.painter().rect_filled(rect, 0, color::SIGNAL);
        egui::Color32::WHITE
    } else if hovered {
        ui.painter().rect_filled(rect, 0, color::CONTROL);
        color::TEXT_PRIMARY
    } else {
        color::TEXT_SECONDARY
    };
    paint_glyph(ui, control, rect.center(), ink);
    response.on_hover_text(name)
}

/// The control's glyph, drawn with hairlines on the pixel grid.
fn paint_glyph(ui: &Ui, control: Control, center: egui::Pos2, ink: egui::Color32) {
    let painter = ui.painter();
    let line = Stroke::new(stroke::HAIRLINE, ink);
    let center = pos2(
        painter.round_to_pixel_center(center.x),
        painter.round_to_pixel_center(center.y),
    );
    let half = GLYPH / 2.0;
    let square = Rect::from_center_size(center, Vec2::splat(GLYPH));
    match control {
        Control::Minimize => {
            painter.hline(square.x_range(), center.y, line);
        }
        Control::Maximize => {
            painter.rect_stroke(square, 0, line, egui::StrokeKind::Middle);
        }
        Control::Restore => {
            let back = square.translate(vec2(2.0, -2.0)).shrink(1.0);
            let front = square.translate(vec2(-1.0, 1.0)).shrink(1.0);
            painter.rect_stroke(front, 0, line, egui::StrokeKind::Middle);
            // The back square shows its top and right edges only.
            painter.hline(back.x_range(), back.top(), line);
            painter.vline(back.right(), back.y_range(), line);
        }
        Control::Close => {
            painter.line_segment(
                [center - Vec2::splat(half), center + Vec2::splat(half)],
                line,
            );
            painter.line_segment(
                [center + vec2(-half, half), center + vec2(half, -half)],
                line,
            );
        }
    }
}

/// The grips resizing the window from its edges and corners, over
/// everything else; not shown while the window is maximized or full
/// screen. Call last in the frame.
pub fn resize_grips(ctx: &Context) {
    let (maximized, fullscreen) = ctx.input(|i| {
        let v = i.viewport();
        (v.maximized.unwrap_or(false), v.fullscreen.unwrap_or(false))
    });
    if maximized || fullscreen {
        return;
    }
    for (k, (dir, rect)) in grips(ctx.viewport_rect()).into_iter().enumerate() {
        egui::Area::new(Id::new("resize_grip").with(k))
            .order(Order::Foreground)
            .fixed_pos(rect.min)
            .constrain(false)
            .show(ctx, |ui| {
                let (_, response) = ui.allocate_exact_size(rect.size(), Sense::drag());
                let response = response.on_hover_cursor(cursor(dir));
                if response.drag_started_by(PointerButton::Primary) {
                    ui.ctx()
                        .send_viewport_cmd(ViewportCommand::BeginResize(dir));
                }
            });
    }
}

/// The grips of a window covering `screen`: the edges between the
/// corners, then the corners.
fn grips(screen: Rect) -> [(ResizeDirection, Rect); 8] {
    use ResizeDirection::*;
    let (w, c) = (GRIP_WIDTH, GRIP_CORNER);
    let (l, r, t, b) = (screen.left(), screen.right(), screen.top(), screen.bottom());
    let rect = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(pos2(x0, y0), pos2(x1, y1));
    [
        (North, rect(l + c, t, r - c, t + w)),
        (South, rect(l + c, b - w, r - c, b)),
        (West, rect(l, t + c, l + w, b - c)),
        (East, rect(r - w, t + c, r, b - c)),
        (NorthWest, rect(l, t, l + c, t + c)),
        (NorthEast, rect(r - c, t, r, t + c)),
        (SouthWest, rect(l, b - c, l + c, b)),
        (SouthEast, rect(r - c, b - c, r, b)),
    ]
}

/// The pointer over a grip resizing towards `dir`.
fn cursor(dir: ResizeDirection) -> CursorIcon {
    use ResizeDirection::*;
    match dir {
        North | South => CursorIcon::ResizeVertical,
        East | West => CursorIcon::ResizeHorizontal,
        NorthEast | SouthWest => CursorIcon::ResizeNeSw,
        NorthWest | SouthEast => CursorIcon::ResizeNwSe,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switcher_centered_when_it_fits() {
        let bar = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 40.0));
        assert_eq!(switcher_left(bar, 200.0, 300.0, 800.0), 400.0);
        // The menus reach past the centered position: after the menus.
        assert_eq!(switcher_left(bar, 200.0, 450.0, 800.0), 450.0);
        // Export… would overlap it: after the menus.
        assert_eq!(switcher_left(bar, 200.0, 300.0, 550.0), 300.0);
    }

    #[test]
    fn grips_cover_edges_and_corners() {
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
        let grips = grips(screen);
        for (i, (_, a)) in grips.iter().enumerate() {
            for (_, b) in &grips[i + 1..] {
                assert!(!a.intersects(*b) || a.intersect(*b).area() == 0.0);
            }
        }
        let (dir, corner) = grips[7];
        assert_eq!(dir, ResizeDirection::SouthEast);
        assert!(corner.contains(pos2(795.0, 595.0)));
        assert_eq!(corner.width(), GRIP_CORNER);
        assert_eq!(grips[2].1.width(), GRIP_WIDTH);
    }
}
