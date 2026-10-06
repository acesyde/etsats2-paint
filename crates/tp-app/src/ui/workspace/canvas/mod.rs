//! Interactive canvas: navigation, tools, selection and transforms.

mod paint;

use egui::{
    CursorIcon, Event, Key, Modifiers, MouseWheelUnit, PointerButton, Pos2, Rect, Response, Sense,
    Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_core::document::{
    Handle, ResizeOptions, ShapeKind, angle_around, resize, rotate, selection_frame, translate,
};
use tp_core::kurbo;
use tp_ui::tokens::canvas as tokens;

use crate::commands::CommandId;
use crate::gesture::{Gesture, OverlayTarget, Transforming, drawing_frame, overlay_target};
use crate::tool::Tool;
use crate::ui::CommandUi;
use crate::viewport::{ScreenMap, Viewport};
use crate::workspace::Workspace;

/// Points one wheel "line" is worth.
const POINTS_PER_LINE: f32 = 40.0;
/// Zoom multiplier per point of wheel travel (≈ 15% per mouse-wheel notch).
const WHEEL_ZOOM_BASE: f64 = 1.0035;

/// Draws the canvas and handles its input.
pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, ws: &mut Workspace) {
    let area = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(area, Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, "Canvas"));
    let ctx = ui.ctx().clone();
    let ppp = ctx.pixels_per_point();
    let now = ctx.input(|i| i.time);

    // Fit on first display, and keep fitting until the user navigates.
    let size = ws.project.surface().size;
    if ws.viewport.is_none_or(|v| v.fitted) {
        ws.viewport = Some(Viewport::fit(size, area, ppp));
    }
    ws.canvas_rect = Some(area);

    if response.hovered() || response.contains_pointer() {
        handle_wheel(&ctx, ws, area, ppp);
    }
    handle_pointer(ui, &response, ws, area, ppp, now);

    if response.secondary_clicked() {
        select_under_pointer_for_menu(ws, &response, area, ppp);
    }
    response.context_menu(|ui| context_menu(ui, cmds));

    let map = ws.viewport.expect("viewport set").map(area, ppp);
    let pointer = ctx.pointer_hover_pos().filter(|p| area.contains(*p));
    let modifiers = ctx.input(|i| i.modifiers);
    let rotate_cursor = set_cursor(&ctx, ws, &map, pointer, modifiers, response.hovered());
    paint::paint(ui, ws, area, &map, pointer, modifiers, now, rotate_cursor);
    ws.geometry.prune();
}

fn handle_wheel(ctx: &egui::Context, ws: &mut Workspace, area: Rect, ppp: f32) {
    let Some(view) = ws.viewport.as_mut() else {
        return;
    };
    let anchor = ctx.pointer_hover_pos().unwrap_or(area.center());
    let events = ctx.input(|i| i.events.clone());
    for event in events {
        match event {
            Event::MouseWheel {
                unit,
                delta,
                modifiers,
                ..
            } => {
                let points = match unit {
                    MouseWheelUnit::Point => delta,
                    MouseWheelUnit::Line => delta * POINTS_PER_LINE,
                    MouseWheelUnit::Page => delta * area.height(),
                };
                let is_mouse_wheel = !matches!(unit, MouseWheelUnit::Point);
                if modifiers.command || (is_mouse_wheel && !modifiers.shift) {
                    let travel = if points.y.abs() >= points.x.abs() {
                        points.y
                    } else {
                        points.x
                    };
                    let factor = WHEEL_ZOOM_BASE.powf(f64::from(travel));
                    view.zoom_by(area, ppp, anchor, factor);
                } else if is_mouse_wheel {
                    // Shift + mouse wheel: horizontal pan.
                    let travel = if points.y.abs() >= points.x.abs() {
                        points.y
                    } else {
                        points.x
                    };
                    view.pan(ppp, Vec2::new(travel, 0.0));
                } else {
                    view.pan(ppp, points);
                }
            }
            Event::Zoom(factor) => view.zoom_by(area, ppp, anchor, f64::from(factor)),
            _ => {}
        }
    }
}

fn handle_pointer(
    ui: &Ui,
    response: &Response,
    ws: &mut Workspace,
    area: Rect,
    ppp: f32,
    now: f64,
) {
    let ctx = ui.ctx();
    let modifiers = ctx.input(|i| i.modifiers);
    let pointer = ctx.pointer_latest_pos().unwrap_or(area.center());
    let map = ws.viewport.expect("viewport set").map(area, ppp);

    // Escape cancels a gesture, restoring the document as it was.
    if ws.gesture.is_active() && ctx.input(|i| i.key_pressed(Key::Escape)) {
        cancel_gesture(ws);
        return;
    }

    // Middle button always pans.
    if response.drag_started_by(PointerButton::Middle) {
        ws.gesture = Gesture::Panning;
    }
    if response.dragged_by(PointerButton::Middle)
        && let Some(view) = ws.viewport.as_mut()
    {
        view.pan(ppp, response.drag_delta());
    }
    if response.drag_stopped_by(PointerButton::Middle) {
        ws.gesture = Gesture::Idle;
    }

    if response.drag_started_by(PointerButton::Primary) {
        let origin = ctx.input(|i| i.pointer.press_origin()).unwrap_or(pointer);
        start_gesture(ws, &map, origin, modifiers, now);
    }
    if response.dragged_by(PointerButton::Primary) {
        update_gesture(ws, &map, pointer, response.drag_delta(), modifiers, ppp);
    }
    if response.drag_stopped_by(PointerButton::Primary) {
        finish_gesture(ws, &map, pointer, modifiers, area, ppp, now);
    }
    if response.clicked_by(PointerButton::Primary) {
        click(ws, &map, pointer, modifiers, area, ppp, now);
    }
}

fn hit_tolerance(map: &ScreenMap) -> f64 {
    map.doc_len(tokens::HIT_TOLERANCE)
}

fn transforming(ws: &Workspace, start: kurbo::Point) -> Transforming {
    Transforming {
        start,
        originals: ws.selected_objects(),
        before: ws.snapshot(),
    }
}

fn unavailable_hint(tool: Tool) -> String {
    format!("The {} tool is not available yet", tool.name())
}

fn start_gesture(
    ws: &mut Workspace,
    map: &ScreenMap,
    origin: Pos2,
    modifiers: Modifiers,
    now: f64,
) {
    let doc = map.to_doc(origin);
    ws.gesture = match ws.tool {
        Tool::Hand => Gesture::Panning,
        Tool::Zoom => Gesture::ZoomRect { start: doc },
        Tool::Rectangle => Gesture::Drawing {
            kind: ShapeKind::rectangle(),
            start: doc,
        },
        Tool::Ellipse => Gesture::Drawing {
            kind: ShapeKind::Ellipse,
            start: doc,
        },
        Tool::Select | Tool::Move => {
            let bounds = selection_frame(&ws.selected_objects());
            match overlay_target(bounds.as_ref(), map, origin) {
                Some(OverlayTarget::Handle(handle)) => Gesture::Resizing {
                    handle,
                    bounds: bounds.expect("handle implies selection"),
                    base: transforming(ws, doc),
                },
                Some(OverlayTarget::Rotate) => {
                    let pivot = bounds.expect("rotation implies selection").center;
                    Gesture::Rotating {
                        pivot,
                        start_angle: angle_around(pivot, doc),
                        base: transforming(ws, doc),
                    }
                }
                None => match ws.project.surface().hit_test(doc, hit_tolerance(map)) {
                    Some(id) => {
                        if !ws.selection.contains(&id) {
                            if !modifiers.shift {
                                ws.selection.clear();
                            }
                            ws.selection.push(id);
                        }
                        Gesture::Moving(transforming(ws, doc))
                    }
                    None => Gesture::Marquee {
                        start: doc,
                        base: if modifiers.shift {
                            ws.selection.clone()
                        } else {
                            Vec::new()
                        },
                    },
                },
            }
        }
        other => {
            ws.show_hint(unavailable_hint(other), now);
            Gesture::Idle
        }
    };
}

fn update_gesture(
    ws: &mut Workspace,
    map: &ScreenMap,
    pointer: Pos2,
    drag_delta: Vec2,
    modifiers: Modifiers,
    ppp: f32,
) {
    let doc = map.to_doc(pointer);
    match &ws.gesture {
        Gesture::Panning => {
            if let Some(view) = ws.viewport.as_mut() {
                view.pan(ppp, drag_delta);
            }
        }
        Gesture::Moving(base) => {
            let moved = translate(&base.originals, doc - base.start, modifiers.shift);
            ws.project.surface_mut().replace(&moved);
        }
        Gesture::Resizing {
            handle,
            bounds,
            base,
        } => {
            let options = ResizeOptions {
                proportional: modifiers.shift,
                from_center: modifiers.alt,
            };
            let resized = resize(&base.originals, *bounds, *handle, doc, options);
            ws.project.surface_mut().replace(&resized);
        }
        Gesture::Rotating {
            pivot,
            start_angle,
            base,
        } => {
            let angle = angle_around(*pivot, doc) - start_angle;
            let rotated = rotate(&base.originals, *pivot, angle, modifiers.shift);
            ws.project.surface_mut().replace(&rotated);
        }
        Gesture::Marquee { start, base } => {
            let rect = kurbo::Rect::from_points(*start, doc);
            let mut selection = base.clone();
            for id in ws.project.surface().objects_in_rect(rect) {
                if !selection.contains(&id) {
                    selection.push(id);
                }
            }
            ws.selection = selection;
        }
        Gesture::Idle | Gesture::Drawing { .. } | Gesture::ZoomRect { .. } => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn finish_gesture(
    ws: &mut Workspace,
    map: &ScreenMap,
    pointer: Pos2,
    modifiers: Modifiers,
    area: Rect,
    ppp: f32,
    now: f64,
) {
    let gesture = std::mem::take(&mut ws.gesture);
    let doc = map.to_doc(pointer);
    match gesture {
        Gesture::Moving(base) => ws.record("Move", base.before, now, false),
        Gesture::Resizing { base, .. } => ws.record("Resize", base.before, now, false),
        Gesture::Rotating { base, .. } => ws.record("Rotate", base.before, now, false),
        Gesture::Drawing { kind, start } => {
            if map.to_screen(start).distance(pointer) >= 2.0 {
                let frame = drawing_frame(start, doc, modifiers.shift, modifiers.alt);
                ws.create_shape(kind, frame, now);
            }
        }
        Gesture::ZoomRect { start } => {
            if map.to_screen(start).distance(pointer) >= 4.0
                && let Some(view) = ws.viewport.as_mut()
            {
                view.zoom_to_rect(area, ppp, kurbo::Rect::from_points(start, doc));
            }
        }
        Gesture::Idle | Gesture::Panning | Gesture::Marquee { .. } => {}
    }
}

fn cancel_gesture(ws: &mut Workspace) {
    let gesture = std::mem::take(&mut ws.gesture);
    match gesture {
        Gesture::Moving(base) | Gesture::Resizing { base, .. } | Gesture::Rotating { base, .. } => {
            ws.selection = ws.project.restore(&base.before);
        }
        Gesture::Marquee { base, .. } => ws.selection = base,
        _ => {}
    }
}

fn click(
    ws: &mut Workspace,
    map: &ScreenMap,
    pointer: Pos2,
    modifiers: Modifiers,
    area: Rect,
    ppp: f32,
    now: f64,
) {
    let doc = map.to_doc(pointer);
    match ws.tool {
        Tool::Select | Tool::Move => match ws.project.surface().hit_test(doc, hit_tolerance(map)) {
            Some(id) if modifiers.shift => {
                if let Some(i) = ws.selection.iter().position(|s| *s == id) {
                    ws.selection.remove(i);
                } else {
                    ws.selection.push(id);
                }
            }
            Some(id) => ws.selection = vec![id],
            None if !modifiers.shift => ws.selection.clear(),
            None => {}
        },
        Tool::Zoom => {
            if let Some(view) = ws.viewport.as_mut() {
                let direction = if modifiers.alt { -1 } else { 1 };
                let zoom = Viewport::step_zoom(view.zoom, direction);
                view.zoom_at(area, ppp, pointer, zoom);
            }
        }
        Tool::Rectangle | Tool::Ellipse | Tool::Hand => {}
        other => ws.show_hint(unavailable_hint(other), now),
    }
}

fn select_under_pointer_for_menu(ws: &mut Workspace, response: &Response, area: Rect, ppp: f32) {
    let Some(pos) = response.interact_pointer_pos() else {
        return;
    };
    let map = ws.viewport.expect("viewport set").map(area, ppp);
    if let Some(id) = ws
        .project
        .surface()
        .hit_test(map.to_doc(pos), hit_tolerance(&map))
        && !ws.selection.contains(&id)
    {
        ws.selection = vec![id];
    }
}

fn context_menu(ui: &mut Ui, cmds: &mut CommandUi<'_>) {
    use CommandId::*;
    for id in [Cut, Copy, Paste, Duplicate, Delete] {
        cmds.menu_item(ui, id);
    }
    ui.separator();
    cmds.menu_item(ui, BringForward);
    cmds.menu_item(ui, SendBackward);
    ui.separator();
    cmds.menu_item(ui, SelectAll);
    cmds.menu_item(ui, Deselect);
    ui.separator();
    cmds.menu_item(ui, FitToScreen);
    cmds.menu_item(ui, ActualSize);
}

/// Resize cursor matching a handle direction rotated by `rotation_deg`.
pub fn resize_cursor(handle: Handle, rotation_deg: f64) -> CursorIcon {
    let base = f64::from(handle.y).atan2(f64::from(handle.x)).to_degrees();
    let angle = (base + rotation_deg).rem_euclid(180.0);
    match ((angle + 22.5) / 45.0).floor() as i32 % 4 {
        0 => CursorIcon::ResizeHorizontal,
        1 => CursorIcon::ResizeNwSe,
        2 => CursorIcon::ResizeVertical,
        _ => CursorIcon::ResizeNeSw,
    }
}

/// Sets the cursor; returns true when the canvas must draw a rotate cursor.
fn set_cursor(
    ctx: &egui::Context,
    ws: &Workspace,
    map: &ScreenMap,
    pointer: Option<Pos2>,
    modifiers: Modifiers,
    hovered: bool,
) -> bool {
    let icon = match &ws.gesture {
        Gesture::Panning => Some(CursorIcon::Grabbing),
        Gesture::Moving(_) => Some(CursorIcon::Move),
        Gesture::Resizing { handle, bounds, .. } => {
            Some(resize_cursor(*handle, bounds.rotation_deg))
        }
        Gesture::Rotating { .. } => None,
        Gesture::Drawing { .. } => Some(CursorIcon::Crosshair),
        Gesture::Marquee { .. } => Some(CursorIcon::Default),
        Gesture::ZoomRect { .. } => Some(CursorIcon::ZoomIn),
        Gesture::Idle => {
            if !hovered {
                return false;
            }
            match ws.tool {
                Tool::Hand => Some(CursorIcon::Grab),
                Tool::Zoom if modifiers.alt => Some(CursorIcon::ZoomOut),
                Tool::Zoom => Some(CursorIcon::ZoomIn),
                Tool::Rectangle | Tool::Ellipse => Some(CursorIcon::Crosshair),
                Tool::Select | Tool::Move => {
                    let bounds = selection_frame(&ws.selected_objects());
                    match pointer.and_then(|p| overlay_target(bounds.as_ref(), map, p)) {
                        Some(OverlayTarget::Handle(h)) => {
                            Some(resize_cursor(h, bounds.map_or(0.0, |b| b.rotation_deg)))
                        }
                        Some(OverlayTarget::Rotate) => None,
                        None if ws.tool == Tool::Move => Some(CursorIcon::Move),
                        None => Some(CursorIcon::Default),
                    }
                }
                _ => Some(CursorIcon::NotAllowed),
            }
        }
    };
    match icon {
        Some(icon) => {
            ctx.set_cursor_icon(icon);
            false
        }
        None => {
            ctx.set_cursor_icon(CursorIcon::None);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_cursor_follows_rotation() {
        let right = Handle { x: 1, y: 0 };
        assert_eq!(resize_cursor(right, 0.0), CursorIcon::ResizeHorizontal);
        assert_eq!(resize_cursor(right, 90.0), CursorIcon::ResizeVertical);
        assert_eq!(
            resize_cursor(Handle { x: 1, y: 1 }, 0.0),
            CursorIcon::ResizeNwSe
        );
        assert_eq!(
            resize_cursor(Handle { x: 1, y: -1 }, 0.0),
            CursorIcon::ResizeNeSw
        );
    }
}
