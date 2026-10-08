//! Interactive canvas: navigation, tools, selection and transforms.

pub mod aids;
mod gradient_tool;
mod paint;

use egui::{
    CursorIcon, Event, Key, Modifiers, MouseWheelUnit, PointerButton, Pos2, Rect, Response, Sense,
    Ui, Vec2, WidgetInfo, WidgetType,
};
use tp_core::document::{
    Frame, Handle, ObjectId, Paint, ResizeOptions, Rgba, ShapeKind, angle_around, resize, rotate,
    selection_frame, translate,
};
use tp_core::kurbo;
use tp_core::{Axis, Guide};
use tp_i18n::tr;
use tp_ui::tokens::canvas as tokens;

use crate::commands::CommandId;
use crate::gesture::{Gesture, OverlayTarget, Transforming, drawing_frame_aspect, overlay_target};
use crate::path_edit::{PointTarget, line_points};
use crate::snap::Snapper;
use crate::tool::Tool;
use crate::ui::CommandUi;
use crate::viewport::{ScreenMap, Viewport};
use crate::workspace::{ColorTarget, Workspace};
use tp_core::document::snap_direction;
use tp_core::kurbo::Vec2 as Vec2Doc;

/// Points one wheel "line" is worth.
const POINTS_PER_LINE: f32 = 40.0;
/// Zoom multiplier per point of wheel travel (≈ 15% per mouse-wheel notch).
const WHEEL_ZOOM_BASE: f64 = 1.0035;

/// Draws the canvas and handles its input.
pub fn show(ui: &mut Ui, cmds: &mut CommandUi<'_>, ws: &mut Workspace) {
    let full = ui.available_rect_before_wrap();
    ui.allocate_rect(full, Sense::hover());
    let (top_ruler, left_ruler, area) = aids::split(full);
    let response = ui.interact(area, ui.id().with("canvas"), Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, tr("canvas")));
    let rulers = [
        (
            ui.interact(top_ruler, ui.id().with("ruler_top"), Sense::drag()),
            Axis::Horizontal,
        ),
        (
            ui.interact(left_ruler, ui.id().with("ruler_left"), Sense::drag()),
            Axis::Vertical,
        ),
    ];
    for (ruler, axis) in &rulers {
        let label = match axis {
            Axis::Horizontal => "canvas-ruler-horizontal",
            Axis::Vertical => "canvas-ruler-vertical",
        };
        ruler.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, tr(label)));
    }
    let ctx = ui.ctx().clone();
    let ppp = ctx.pixels_per_point();
    let now = ctx.input(|i| i.time);
    ws.images.poll(&ctx);

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
    for (ruler, axis) in &rulers {
        handle_ruler(ui, ruler, *axis, ws, area, ppp, now);
    }

    if response.secondary_clicked() {
        select_under_pointer_for_menu(ws, &response, area, ppp);
    }
    response.context_menu(|ui| context_menu(ui, cmds));

    let map = ws.viewport.expect("viewport set").map(area, ppp);
    let pointer = ctx.pointer_hover_pos().filter(|p| area.contains(*p));
    let modifiers = ctx.input(|i| i.modifiers);
    let drawn_cursor = set_cursor(&ctx, ws, &map, pointer, modifiers, response.hovered());
    paint::paint(ui, ws, area, &map, pointer, modifiers, now, drawn_cursor);
    let painter = ui.painter();
    let corner = Rect::from_min_max(full.min, area.min);
    painter.rect_filled(corner, 0, tokens::RULER_BG);
    aids::paint_ruler(painter, top_ruler, Axis::Horizontal, &map, pointer);
    aids::paint_ruler(painter, left_ruler, Axis::Vertical, &map, pointer);
    ws.geometry.prune();
    ws.text.prune();
    ws.images.prune();
    ws.gradients.prune();

    // An asset dragged from the Assets panel and dropped here.
    if let Some(asset) = response.dnd_release_payload::<tp_core::document::AssetId>() {
        let at = ctx.pointer_interact_pos().map(|p| map.to_doc(p));
        ws.place_asset(*asset, at, now);
    }
    // A symbol dragged from the Symbols panel.
    if let Some(symbol) = response.dnd_release_payload::<tp_core::document::SymbolId>() {
        let at = ctx.pointer_interact_pos().map(|p| map.to_doc(p));
        ws.place_symbol(*symbol, at, now);
    }
    symbol_bar(ui, cmds, ws, area);
}

/// While a symbol is edited: "Editing symbol <name>" and Done, over the
/// top of the canvas.
fn symbol_bar(ui: &mut Ui, cmds: &mut CommandUi<'_>, ws: &Workspace, area: Rect) {
    let Some(symbol) = ws.project.edited_symbol() else {
        return;
    };
    let bar = Rect::from_min_size(area.min, egui::vec2(area.width(), 36.0));
    ui.painter()
        .rect_filled(bar, 0, tp_ui::tokens::color::ACCENT_SUBTLE);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(bar.shrink2(egui::vec2(12.0, 4.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let text = tr!("symbol-bar-editing", name = symbol.name.as_str());
    child
        .label(
            egui::RichText::new(format!("{} {text}", tp_ui::icons::SYMBOL))
                .color(tp_ui::tokens::color::TEXT_PRIMARY),
        )
        .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &text));
    child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let done = ui.add(tp_ui::widgets::primary_button(&tr("symbol-bar-done")));
        if done.clicked() {
            cmds.push(crate::commands::CommandId::FinishSymbol);
        }
    });
}

/// Whether positions snap now: Snapping on and Cmd/Ctrl not held.
fn snapping(ws: &Workspace, m: Modifiers) -> bool {
    ws.aids.snapping && !m.command
}

/// Builds the snap targets of a gesture, leaving out `exclude`.
fn prepare_snapper(ws: &mut Workspace, exclude: &[ObjectId], skip_guide: Option<usize>) {
    ws.snapper = Some(Snapper::new(ws, exclude, skip_guide));
    ws.snap_hits.clear();
}

/// Snaps a free point (records the hits). Uses the gesture's targets, or
/// targets built now (clicks).
fn snap_point(
    ws: &mut Workspace,
    map: &ScreenMap,
    doc: kurbo::Point,
    m: Modifiers,
) -> kurbo::Point {
    if !snapping(ws, m) {
        ws.snap_hits.clear();
        return doc;
    }
    if ws.snapper.is_none() {
        prepare_snapper(ws, &[], None);
    }
    let tolerance = map.doc_len(tokens::SNAP_DISTANCE);
    let (p, hits) = ws
        .snapper
        .as_ref()
        .expect("built")
        .snap_point(doc, tolerance);
    ws.snap_hits = hits;
    p
}

/// The pointer of a shape or line being drawn: snapped unless a Shift
/// constraint couples the axes.
fn drawing_point(
    ws: &mut Workspace,
    map: &ScreenMap,
    doc: kurbo::Point,
    m: Modifiers,
) -> kurbo::Point {
    if m.shift {
        ws.snap_hits.clear();
        doc
    } else {
        snap_point(ws, map, doc, m)
    }
}

/// Snapped move delta: the selection bounds' edges and center snap on the
/// axes the Shift constraint leaves free.
fn snapped_move(
    ws: &mut Workspace,
    map: &ScreenMap,
    originals: &[tp_core::document::Object],
    delta: Vec2Doc,
    m: Modifiers,
) -> Vec2Doc {
    let delta = if m.shift {
        snap_direction(delta)
    } else {
        delta
    };
    ws.snap_hits.clear();
    if !snapping(ws, m) {
        return delta;
    }
    let (free_x, free_y) = if !m.shift {
        (true, true)
    } else {
        (delta.y == 0.0, delta.x == 0.0)
    };
    let Some(bounds) = originals
        .iter()
        .map(|o| o.bounding_box())
        .reduce(|a, b| a.union(b))
    else {
        return delta;
    };
    let b = bounds + delta;
    let tolerance = map.doc_len(tokens::SNAP_DISTANCE);
    let Some(snapper) = ws.snapper.as_ref() else {
        return delta;
    };
    let mut out = delta;
    let mut hits = Vec::new();
    if free_x
        && let Some((d, hit)) = snapper.snap_axis(
            Axis::Vertical,
            &[b.x0, b.center().x, b.x1],
            (b.y0, b.y1),
            tolerance,
        )
    {
        out.x += d;
        hits.push(hit);
    }
    if free_y
        && let Some((d, hit)) = snapper.snap_axis(
            Axis::Horizontal,
            &[b.y0, b.center().y, b.y1],
            (b.x0, b.x1),
            tolerance,
        )
    {
        out.y += d;
        hits.push(hit);
    }
    ws.snap_hits = hits;
    out
}

/// Coordinate of `doc` along the guide axis (y for horizontal guides).
fn along(axis: Axis, doc: kurbo::Point) -> f64 {
    match axis {
        Axis::Horizontal => doc.y,
        Axis::Vertical => doc.x,
    }
}

/// Drags from a ruler create a guide, added when released over the canvas.
fn handle_ruler(
    ui: &Ui,
    ruler: &Response,
    axis: Axis,
    ws: &mut Workspace,
    area: Rect,
    ppp: f32,
    now: f64,
) {
    let ctx = ui.ctx();
    let map = ws.viewport.expect("viewport set").map(area, ppp);
    let Some(pointer) = ctx.pointer_latest_pos() else {
        return;
    };
    let doc = map.to_doc(pointer);
    if ruler.drag_started_by(PointerButton::Primary) {
        ws.commit_pending(now);
        if ws.is_editing_text() {
            ws.end_text_session(now);
        }
        ws.gesture = Gesture::Guide {
            axis,
            index: None,
            position: along(axis, doc),
            before: ws.snapshot(),
        };
        prepare_snapper(ws, &[], None);
    }
    if ruler.dragged_by(PointerButton::Primary)
        && matches!(ws.gesture, Gesture::Guide { index: None, axis: a, .. } if a == axis)
    {
        let modifiers = ctx.input(|i| i.modifiers);
        let at = snap_guide(ws, &map, axis, along(axis, doc), modifiers);
        if let Gesture::Guide { position, .. } = &mut ws.gesture {
            *position = at;
        }
    }
    if ruler.drag_stopped_by(PointerButton::Primary)
        && let Gesture::Guide {
            index: None,
            position,
            before,
            ..
        } = std::mem::take(&mut ws.gesture)
        && area.contains(pointer)
        && {
            ws.snapper = None;
            ws.snap_hits.clear();
            true
        }
    {
        ws.project.add_guide(Guide::new(axis, position));
        ws.record("undo-add-guide", before, now, false);
        if !ws.aids.guides {
            ws.request_show_guides = true;
        }
    }
}

/// A press on a shown guide, unless it lands inside an object's filled
/// shape (objects win there).
fn guide_press(
    ws: &Workspace,
    map: &ScreenMap,
    origin: Pos2,
    doc: kurbo::Point,
) -> Option<Gesture> {
    if !ws.aids.guides {
        return None;
    }
    let guides = &ws.project.surface().guides;
    let index = aids::guide_at(guides, map, origin)?;
    if ws.project.surface().hit_test(doc, 0.0).is_some() {
        return None;
    }
    let guide = guides[index];
    Some(Gesture::Guide {
        axis: guide.axis,
        index: Some(index),
        position: guide.position,
        before: ws.snapshot(),
    })
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
        // Targets for gestures that edit existing things leave them out.
        match &ws.gesture {
            Gesture::Moving(_)
            | Gesture::Resizing { .. }
            | Gesture::MovingPoints(_)
            | Gesture::MovingHandle { .. } => {
                let exclude = ws.selection.clone();
                prepare_snapper(ws, &exclude, None);
            }
            Gesture::Guide { index, .. } => {
                let skip = *index;
                prepare_snapper(ws, &[], skip);
            }
            _ => {}
        }
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
    if response.double_clicked_by(PointerButton::Primary) {
        double_click(ws, &map, pointer, now);
    }
    // Targets only live for the gesture (objects may change afterwards).
    if !ws.gesture.is_active() {
        ws.snapper = None;
        ws.snap_hits.clear();
    }
}

/// Whether `doc` is over the text being edited.
fn in_edited_text(ws: &Workspace, doc: kurbo::Point, map: &ScreenMap) -> bool {
    ws.editing_text()
        .and_then(|id| ws.project.surface().get(id))
        .is_some_and(|o| o.contains(doc, hit_tolerance(map)))
}

/// The editable text under `doc`, looking inside groups.
fn text_under(ws: &Workspace, doc: kurbo::Point, map: &ScreenMap) -> Option<ObjectId> {
    let id = click_target(ws, doc, map, true)?;
    ws.can_edit_text(id).then_some(id)
}

fn double_click(ws: &mut Workspace, map: &ScreenMap, pointer: Pos2, now: f64) {
    if ws.tool == Tool::DirectSelect {
        match ws.point_target(map.to_doc(pointer), point_radius(map)) {
            Some(PointTarget::Point(r)) => ws.toggle_point(r, now),
            Some(PointTarget::Segment(id, subpath, segment, t)) => {
                ws.insert_point(id, subpath, segment, t, now);
            }
            _ => {}
        }
        return;
    }
    if !matches!(ws.tool, Tool::Select | Tool::Move | Tool::Text) {
        return;
    }
    let doc = map.to_doc(pointer);
    // An instance: edit its symbol.
    if !ws.is_editing_text()
        && let Some(id) = click_target(ws, doc, map, true)
        && let Some(o) = ws.project.surface().get(id)
        && let ShapeKind::Instance { symbol, .. } = o.kind
    {
        ws.edit_symbol(symbol, now);
        return;
    }
    if in_edited_text(ws, doc, map) {
        ws.text_select_word(doc, now);
    } else if let Some(id) = text_under(ws, doc, map) {
        ws.start_editing(id, Some(doc), now);
    }
}

fn hit_tolerance(map: &ScreenMap) -> f64 {
    map.doc_len(tokens::HIT_TOLERANCE)
}

/// What a click at `doc` selects: the top-level object, or the innermost one
/// with Cmd/Ctrl.
pub fn click_target(
    ws: &Workspace,
    doc: kurbo::Point,
    map: &ScreenMap,
    deep: bool,
) -> Option<ObjectId> {
    ws.project
        .surface()
        .hit_test(doc, hit_tolerance(map))
        .map(|hit| if deep { hit.inner } else { hit.top })
}

fn transforming(ws: &Workspace, start: kurbo::Point) -> Transforming {
    Transforming {
        start,
        originals: ws.selected_objects(),
        before: ws.snapshot(),
    }
}

/// Radius (document units) within which a press hits a path point.
fn point_radius(map: &ScreenMap) -> f64 {
    map.doc_len(tokens::POINT_HIT_RADIUS)
}

/// Frame of a shape drawn from `start` to `doc` with the drawing modifiers:
/// Shift keeps squares, circles and regular polygons.
fn shape_frame(kind: ShapeKind, start: kurbo::Point, doc: kurbo::Point, m: Modifiers) -> Frame {
    let aspect = match kind {
        ShapeKind::Polygon { sides, star } => tp_core::document::path::regular_aspect(sides, star),
        _ => 1.0,
    };
    drawing_frame_aspect(start, doc, m.shift.then_some(aspect), m.alt)
}

/// Press with the Pen tool: closes the path on its first point, otherwise
/// adds a point (whose handle the drag then pulls).
fn pen_press(
    ws: &mut Workspace,
    doc: kurbo::Point,
    map: &ScreenMap,
    shift: bool,
    now: f64,
) -> bool {
    if ws.pen_over_first(doc, point_radius(map)) {
        ws.finish_pen(true, now);
        return false;
    }
    ws.pen_add(doc, shift);
    true
}

fn start_gesture(
    ws: &mut Workspace,
    map: &ScreenMap,
    origin: Pos2,
    modifiers: Modifiers,
    now: f64,
) {
    let doc = map.to_doc(origin);
    ws.commit_pending(now);
    if ws.is_editing_text() {
        if matches!(ws.tool, Tool::Select | Tool::Move | Tool::Text) && in_edited_text(ws, doc, map)
        {
            ws.text_click(doc, modifiers.shift, now);
            ws.gesture = Gesture::TextSelect;
            return;
        }
        if !matches!(ws.tool, Tool::Hand | Tool::Zoom) {
            ws.end_text_session(now);
        }
    }
    // Drawing tools snap the press point (the Pen's Shift constraint
    // decides its point itself).
    let drawing = matches!(
        ws.tool,
        Tool::Rectangle | Tool::Ellipse | Tool::Polygon | Tool::Line | Tool::Pen
    );
    let doc = if drawing && !(ws.tool == Tool::Pen && modifiers.shift) {
        prepare_snapper(ws, &[], None);
        snap_point(ws, map, doc, modifiers)
    } else {
        doc
    };
    ws.gesture = match ws.tool {
        Tool::Hand => Gesture::Panning,
        Tool::Zoom => Gesture::ZoomRect { start: doc },
        Tool::Rectangle => Gesture::Drawing {
            kind: ShapeKind::rectangle(),
            start: doc,
            current: doc,
        },
        Tool::Ellipse => Gesture::Drawing {
            kind: ShapeKind::Ellipse,
            start: doc,
            current: doc,
        },
        Tool::Polygon => Gesture::Drawing {
            kind: ws.polygon_style.kind(),
            start: doc,
            current: doc,
        },
        Tool::Line => Gesture::Drawing {
            kind: ShapeKind::Path,
            start: doc,
            current: doc,
        },
        Tool::Pen => {
            if pen_press(ws, doc, map, modifiers.shift, now) {
                Gesture::PenHandle
            } else {
                Gesture::Idle
            }
        }
        Tool::Select | Tool::Move => {
            let bounds = selection_frame(&ws.selected_objects());
            let overlay = overlay_target(bounds.as_ref(), map, origin);
            if overlay.is_none()
                && let Some(g) = guide_press(ws, map, origin, doc)
            {
                ws.gesture = g;
                return;
            }
            match overlay {
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
                None => match click_target(ws, doc, map, modifiers.command) {
                    Some(id) => {
                        if !ws.selection.contains(&id) {
                            if !modifiers.shift {
                                ws.selection.clear();
                            }
                            ws.selection.push(id);
                            ws.normalize_selection();
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
        Tool::DirectSelect => {
            if ws.point_target(doc, point_radius(map)).is_none()
                && let Some(g) = guide_press(ws, map, origin, doc)
            {
                g
            } else {
                direct_press(ws, doc, map, modifiers)
            }
        }
        Tool::Gradient => gradient_tool::press(ws, map, origin, doc),
        Tool::Eyedropper | Tool::Text | Tool::Image => Gesture::Idle,
    };
}

/// Press with the Direct Selection tool: a handle, a point (moving the
/// selected points), an object (moving it) or empty canvas (point marquee).
fn direct_press(ws: &mut Workspace, doc: kurbo::Point, map: &ScreenMap, m: Modifiers) -> Gesture {
    match ws.point_target(doc, point_radius(map)) {
        Some(PointTarget::Handle(point, side)) => {
            return Gesture::MovingHandle {
                drag: ws.point_drag(doc),
                point,
                side,
            };
        }
        Some(PointTarget::Point(r)) => {
            if !ws.points.contains(&r) {
                ws.click_point(r, m.shift);
            }
            let mut drag = ws.point_drag(doc);
            drag.grabbed = ws
                .project
                .surface()
                .get(r.object)
                .and_then(|o| Some(o.frame.affine() * o.path_data()?.node(r.node)?.point));
            return Gesture::MovingPoints(drag);
        }
        _ => {}
    }
    match click_target(ws, doc, map, true) {
        Some(id) => {
            if !ws.selection.contains(&id) {
                if !m.shift {
                    ws.selection.clear();
                }
                ws.selection.push(id);
                ws.normalize_selection();
            }
            ws.points.clear();
            Gesture::Moving(transforming(ws, doc))
        }
        None => Gesture::PointMarquee {
            start: doc,
            base: if m.shift {
                ws.points.clone()
            } else {
                Default::default()
            },
            base_objects: if m.shift {
                ws.selection.clone()
            } else {
                Vec::new()
            },
        },
    }
}

/// Click with the Direct Selection tool.
fn direct_click(ws: &mut Workspace, doc: kurbo::Point, map: &ScreenMap, m: Modifiers, now: f64) {
    match ws.point_target(doc, point_radius(map)) {
        Some(PointTarget::Point(r)) => ws.click_point(r, m.shift),
        Some(PointTarget::Handle(..)) => {}
        _ => match click_target(ws, doc, map, true) {
            Some(id) => {
                if m.shift {
                    if let Some(i) = ws.selection.iter().position(|s| *s == id) {
                        ws.selection.remove(i);
                    } else {
                        ws.selection.push(id);
                        ws.normalize_selection();
                    }
                } else {
                    ws.selection = vec![id];
                    ws.points.clear();
                }
                let is_path = ws
                    .project
                    .surface()
                    .get(id)
                    .is_some_and(|o| o.kind == ShapeKind::Path || o.is_group());
                let is_instance = ws
                    .project
                    .surface()
                    .get(id)
                    .is_some_and(|o| o.is_instance());
                if is_instance {
                    ws.show_hint(tr("reason-instance-look"), now);
                } else if !is_path {
                    ws.show_hint(tr("hint-convert-to-edit"), now);
                }
            }
            None => ws.clear_points_or_selection(),
        },
    }
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
        Gesture::Gradient(drag) => {
            let drag = drag.clone();
            gradient_tool::update(ws, &drag, doc, modifiers.shift);
        }
        Gesture::Moving(base) => {
            let (originals, start) = (base.originals.clone(), base.start);
            let delta = snapped_move(ws, map, &originals, doc - start, modifiers);
            let moved = translate(&originals, delta, false);
            ws.project.surface_mut().replace(&moved);
        }
        Gesture::Resizing {
            handle,
            bounds,
            base,
        } => {
            let (handle, bounds, originals) = (*handle, *bounds, base.originals.clone());
            let options = ResizeOptions {
                proportional: modifiers.shift,
                from_center: modifiers.alt,
            };
            // Proportional resizing couples the axes: no snapping.
            let pointer = if modifiers.shift {
                ws.snap_hits.clear();
                doc
            } else {
                snap_point(ws, map, doc, modifiers)
            };
            let resized = resize(&originals, bounds, handle, pointer, options);
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
        Gesture::TextSelect => ws.text_click(doc, true, 0.0),
        Gesture::PenHandle => {
            let at = if modifiers.shift {
                doc
            } else {
                snap_point(ws, map, doc, modifiers)
            };
            ws.pen_drag_handle(at, modifiers.shift);
        }
        Gesture::Drawing { .. } => {
            let at = drawing_point(ws, map, doc, modifiers);
            if let Gesture::Drawing { current, .. } = &mut ws.gesture {
                *current = at;
            }
        }
        Gesture::Guide { axis, index, .. } => {
            let (axis, index) = (*axis, *index);
            let at = snap_guide(ws, map, axis, along(axis, doc), modifiers);
            if let Some(i) = index {
                ws.project.move_guide(i, at);
            }
            if let Gesture::Guide { position, .. } = &mut ws.gesture {
                *position = at;
            }
        }
        Gesture::MovingPoints(drag) => {
            let drag = drag.clone();
            // The grabbed point snaps; the others follow it.
            let doc = match drag.grabbed {
                Some(grabbed) if !modifiers.shift => {
                    let target = grabbed + (doc - drag.start);
                    doc + (snap_point(ws, map, target, modifiers) - target)
                }
                _ => doc,
            };
            ws.drag_points(&drag, doc, modifiers.shift);
        }
        Gesture::MovingHandle { drag, point, side } => {
            let (drag, point, side) = (drag.clone(), *point, *side);
            let doc = snap_point(ws, map, doc, modifiers);
            ws.drag_handle(&drag, point, side, doc, modifiers.alt);
        }
        Gesture::PointMarquee {
            start,
            base,
            base_objects,
        } => {
            let (rect, base, objects) = (
                kurbo::Rect::from_points(*start, doc),
                base.clone(),
                base_objects.clone(),
            );
            ws.marquee_points(rect, &base, &objects);
        }
        Gesture::Idle | Gesture::ZoomRect { .. } => {}
    }
}

/// Snaps a guide position along its axis (excluding the guide itself, left
/// out of the gesture's targets).
fn snap_guide(ws: &mut Workspace, map: &ScreenMap, axis: Axis, at: f64, m: Modifiers) -> f64 {
    ws.snap_hits.clear();
    if !snapping(ws, m) {
        return at;
    }
    let tolerance = map.doc_len(tokens::SNAP_DISTANCE);
    match ws
        .snapper
        .as_ref()
        .and_then(|s| s.snap_axis(axis, &[at], (at, at), tolerance))
    {
        Some((d, hit)) => {
            ws.snap_hits = vec![hit];
            at + d
        }
        None => at,
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
    let mut doc = map.to_doc(pointer);
    if matches!(ws.gesture, Gesture::Drawing { .. }) {
        doc = drawing_point(ws, map, doc, modifiers);
    }
    let gesture = std::mem::take(&mut ws.gesture);
    ws.snapper = None;
    ws.snap_hits.clear();
    match gesture {
        Gesture::Moving(base) => ws.record("tool-move", base.before, now, false),
        Gesture::Gradient(drag) => {
            let label = gradient_tool::label(ws.panels.color_target);
            ws.record(label, drag.before, now, false);
        }
        Gesture::Resizing { base, .. } => ws.record("undo-resize", base.before, now, false),
        Gesture::Rotating { base, .. } => ws.record("undo-rotate", base.before, now, false),
        Gesture::Drawing { kind, start, .. } => {
            if map.to_screen(start).distance(pointer) >= 2.0 {
                if kind == ShapeKind::Path {
                    let (a, b) = line_points(start, doc, modifiers.shift, modifiers.alt);
                    ws.create_line(a, b, now);
                } else {
                    ws.create_shape(kind, shape_frame(kind, start, doc, modifiers), now);
                }
            }
        }
        Gesture::ZoomRect { start } => {
            if map.to_screen(start).distance(pointer) >= 4.0
                && let Some(view) = ws.viewport.as_mut()
            {
                view.zoom_to_rect(area, ppp, kurbo::Rect::from_points(start, doc));
            }
        }
        Gesture::Idle
        | Gesture::Panning
        | Gesture::Marquee { .. }
        | Gesture::TextSelect
        | Gesture::PenHandle
        | Gesture::PointMarquee { .. } => {}
        Gesture::MovingPoints(drag) | Gesture::MovingHandle { drag, .. } => {
            ws.finish_point_drag(drag, now);
        }
        Gesture::Guide {
            index: Some(i),
            before,
            ..
        } => {
            if area.contains(pointer) {
                ws.record("undo-move-guide", before, now, false);
            } else {
                // Dropped on a ruler or outside the canvas.
                ws.project.remove_guide(i);
                ws.record("undo-delete-guide", before, now, false);
            }
        }
        Gesture::Guide { index: None, .. } => {}
    }
}

fn cancel_gesture(ws: &mut Workspace) {
    let gesture = std::mem::take(&mut ws.gesture);
    ws.snapper = None;
    ws.snap_hits.clear();
    match gesture {
        Gesture::Moving(base) | Gesture::Resizing { base, .. } | Gesture::Rotating { base, .. } => {
            ws.selection = ws.project.restore(&base.before);
        }
        Gesture::MovingPoints(drag) | Gesture::MovingHandle { drag, .. } => {
            ws.restore(&drag.before);
        }
        Gesture::Guide {
            index: Some(_),
            before,
            ..
        } => ws.restore(&before),
        Gesture::Gradient(drag) => ws.restore(&drag.before),
        Gesture::PointMarquee {
            base, base_objects, ..
        } => {
            ws.points = base;
            ws.selection = base_objects;
        }
        Gesture::Marquee { base, .. } => ws.selection = base,
        // The point stays, without the handle being dragged.
        Gesture::PenHandle => {
            if let Some(node) = ws.pen.as_mut().and_then(|s| s.nodes.last_mut()) {
                *node = tp_core::document::Node::corner(node.point);
            }
        }
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
    if ws.is_editing_text() {
        if matches!(ws.tool, Tool::Select | Tool::Move | Tool::Text) && in_edited_text(ws, doc, map)
        {
            ws.text_click(doc, modifiers.shift, now);
            return;
        }
        if !matches!(ws.tool, Tool::Hand | Tool::Zoom) {
            ws.end_text_session(now);
        }
    }
    match ws.tool {
        Tool::Text => match text_under(ws, doc, map) {
            Some(id) => {
                ws.start_editing(id, Some(doc), now);
            }
            None => {
                ws.start_new_text(doc, now);
            }
        },
        Tool::Image => ws.place_request = Some(Some(doc)),
        Tool::Select | Tool::Move | Tool::Gradient => {
            match click_target(ws, doc, map, modifiers.command) {
                Some(id) if modifiers.shift => {
                    if let Some(i) = ws.selection.iter().position(|s| *s == id) {
                        ws.selection.remove(i);
                    } else {
                        ws.selection.push(id);
                        ws.normalize_selection();
                    }
                }
                Some(id) => ws.selection = vec![id],
                None if !modifiers.shift => ws.selection.clear(),
                None => {}
            }
        }
        Tool::Zoom => {
            if let Some(view) = ws.viewport.as_mut() {
                let direction = if modifiers.alt { -1 } else { 1 };
                let zoom = Viewport::step_zoom(view.zoom, direction);
                view.zoom_at(area, ppp, pointer, zoom);
            }
        }
        Tool::Eyedropper => eyedropper(ws, doc, map, now),
        Tool::Pen => {
            let at = if modifiers.shift {
                doc
            } else {
                snap_point(ws, map, doc, modifiers)
            };
            pen_press(ws, at, map, modifiers.shift, now);
        }
        Tool::DirectSelect => direct_click(ws, doc, map, modifiers, now),
        Tool::Rectangle | Tool::Ellipse | Tool::Polygon | Tool::Line | Tool::Hand => {}
    }
}

/// Takes the fill (or stroke, per the Colors target) of the topmost visible
/// shape under `doc`, or the artboard color on empty canvas, and applies it.
fn eyedropper(ws: &mut Workspace, doc: kurbo::Point, map: &ScreenMap, now: f64) {
    let target = ws.panels.color_target;
    let artboard = tokens::ARTBOARD;
    let sampled =
        tp_core::document::tree::sample(&ws.project.surface().objects, doc, hit_tolerance(map));
    let background = Rgba::rgb(artboard.r(), artboard.g(), artboard.b());
    let paint = match sampled {
        Some(object) => match object.kind {
            ShapeKind::Image { asset } => {
                // The pixel shown under the pointer, opaque.
                let local = object.content_affine().inverse() * doc;
                let size = object.frame.size;
                let uv = [local.x / size.width + 0.5, local.y / size.height + 0.5];
                match ws.images.sample(asset, uv) {
                    Some(c) if c.a > 0 => Paint::Solid(Rgba::rgb(c.r, c.g, c.b)),
                    _ => Paint::Solid(background),
                }
            }
            // The whole paint, gradients included (placed relative to each
            // target's own frame).
            _ => match target {
                ColorTarget::Stroke => object.stroke.map_or(object.fill, |s| s.paint),
                ColorTarget::Fill => object.fill,
            },
        },
        None => Paint::Solid(background),
    };
    if let Paint::Solid(c) = paint {
        ws.recent_candidate = Some(c);
    }
    let label = match target {
        ColorTarget::Fill => "undo-change-fill",
        ColorTarget::Stroke => "undo-change-stroke",
    };
    ws.apply_paint(target, paint, label);
    ws.commit_pending(now);
}

fn select_under_pointer_for_menu(ws: &mut Workspace, response: &Response, area: Rect, ppp: f32) {
    let Some(pos) = response.interact_pointer_pos() else {
        return;
    };
    let map = ws.viewport.expect("viewport set").map(area, ppp);
    let deep = response.ctx.input(|i| i.modifiers.command);
    if let Some(id) = click_target(ws, map.to_doc(pos), &map, deep)
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
    for axis in tp_core::document::FlipAxis::ALL {
        cmds.menu_item(ui, Flip(axis));
    }
    ui.separator();
    cmds.menu_item(ui, SelectAll);
    cmds.menu_item(ui, Deselect);
    ui.separator();
    cmds.menu_item(ui, FitToScreen);
    cmds.menu_item(ui, ActualSize);
}

/// Cursor over a guide: moves across its line.
fn guide_cursor(axis: Axis) -> CursorIcon {
    match axis {
        Axis::Horizontal => CursorIcon::ResizeVertical,
        Axis::Vertical => CursorIcon::ResizeHorizontal,
    }
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

/// A cursor the canvas draws itself (no system equivalent): glyph and the
/// offset from the pointer to the glyph center (so the hotspot is right).
pub type DrawnCursor = Option<(&'static str, Vec2)>;

const ROTATE_CURSOR: (&str, Vec2) = (tp_ui::icons::ROTATE, Vec2::ZERO);
/// The eyedropper tip is at the glyph's bottom-left.
const EYEDROPPER_CURSOR: (&str, Vec2) = (tp_ui::icons::EYEDROPPER, Vec2::new(8.0, -8.0));
/// The pen nib tip is at the glyph's bottom-left.
const PEN_CURSOR: (&str, Vec2) = (tp_ui::icons::PEN, Vec2::new(8.0, -8.0));

/// Sets the system cursor, or hides it and returns the cursor to draw.
fn set_cursor(
    ctx: &egui::Context,
    ws: &Workspace,
    map: &ScreenMap,
    pointer: Option<Pos2>,
    modifiers: Modifiers,
    hovered: bool,
) -> DrawnCursor {
    let icon: Result<CursorIcon, (&'static str, Vec2)> = match &ws.gesture {
        Gesture::Panning => Ok(CursorIcon::Grabbing),
        Gesture::Moving(_) => Ok(CursorIcon::Move),
        Gesture::Resizing { handle, bounds, .. } => Ok(resize_cursor(*handle, bounds.rotation_deg)),
        Gesture::Rotating { .. } => Err(ROTATE_CURSOR),
        Gesture::Drawing { .. } | Gesture::Gradient(_) => Ok(CursorIcon::Crosshair),
        Gesture::Marquee { .. } => Ok(CursorIcon::Default),
        Gesture::ZoomRect { .. } => Ok(CursorIcon::ZoomIn),
        Gesture::TextSelect => Ok(CursorIcon::Text),
        Gesture::PenHandle => Err(PEN_CURSOR),
        Gesture::MovingPoints(_) | Gesture::MovingHandle { .. } => Ok(CursorIcon::Move),
        Gesture::Guide { axis, .. } => Ok(guide_cursor(*axis)),
        Gesture::PointMarquee { .. } => Ok(CursorIcon::Default),
        Gesture::Idle => {
            if !hovered {
                return None;
            }
            match ws.tool {
                Tool::Hand => Ok(CursorIcon::Grab),
                Tool::Zoom if modifiers.alt => Ok(CursorIcon::ZoomOut),
                Tool::Zoom => Ok(CursorIcon::ZoomIn),
                Tool::Eyedropper => Err(EYEDROPPER_CURSOR),
                Tool::Rectangle
                | Tool::Ellipse
                | Tool::Polygon
                | Tool::Line
                | Tool::Image
                | Tool::Gradient => Ok(CursorIcon::Crosshair),
                Tool::Pen => Err(PEN_CURSOR),

                Tool::Text => Ok(CursorIcon::Text),
                Tool::Select | Tool::Move
                    if pointer.is_some_and(|p| in_edited_text(ws, map.to_doc(p), map)) =>
                {
                    Ok(CursorIcon::Text)
                }
                Tool::Select | Tool::Move | Tool::DirectSelect
                    if pointer.is_some_and(|p| {
                        guide_press(ws, map, p, map.to_doc(p)).is_some()
                            && (ws.tool == Tool::DirectSelect
                                || overlay_target(
                                    selection_frame(&ws.selected_objects()).as_ref(),
                                    map,
                                    p,
                                )
                                .is_none())
                    }) =>
                {
                    let p = pointer.expect("checked");
                    let index =
                        aids::guide_at(&ws.project.surface().guides, map, p).expect("checked");
                    Ok(guide_cursor(ws.project.surface().guides[index].axis))
                }
                Tool::DirectSelect => Ok(CursorIcon::Default),
                Tool::Select | Tool::Move => {
                    let bounds = selection_frame(&ws.selected_objects());
                    match pointer.and_then(|p| overlay_target(bounds.as_ref(), map, p)) {
                        Some(OverlayTarget::Handle(h)) => {
                            Ok(resize_cursor(h, bounds.map_or(0.0, |b| b.rotation_deg)))
                        }
                        Some(OverlayTarget::Rotate) => Err(ROTATE_CURSOR),
                        None if ws.tool == Tool::Move => Ok(CursorIcon::Move),
                        None => Ok(CursorIcon::Default),
                    }
                }
            }
        }
    };
    match icon {
        Ok(icon) => {
            ctx.set_cursor_icon(icon);
            None
        }
        Err(drawn) => {
            ctx.set_cursor_icon(CursorIcon::None);
            Some(drawn)
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
