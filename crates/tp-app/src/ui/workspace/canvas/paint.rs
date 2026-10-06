//! Canvas drawing: artboard, objects, selection overlay, gesture feedback.

use egui::{
    Align2, Color32, CornerRadius, FontId, Modifiers, Painter, Pos2, Rect, Shadow, Shape, Stroke,
    StrokeKind, Ui, Vec2,
};
use tp_core::document::{Frame, Object, Rgba, selection_frame};
use tp_core::kurbo;
use tp_ui::icons;
use tp_ui::tokens::{canvas as tokens, color, radius, space};

use crate::geometry_cache::zoom_bucket;
use crate::gesture::{Gesture, drawing_frame, screen_handles};
use crate::viewport::ScreenMap;
use crate::workspace::Workspace;

fn color32(c: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

fn screen_rect(map: &ScreenMap, r: kurbo::Rect) -> Rect {
    Rect::from_two_pos(
        map.to_screen(kurbo::Point::new(r.x0, r.y0)),
        map.to_screen(kurbo::Point::new(r.x1, r.y1)),
    )
}

/// Draws a closed overlay line with a contrasting halo underneath.
fn halo_polyline(painter: &Painter, points: Vec<Pos2>, closed: bool) {
    let halo = Stroke::new(tokens::LINE + 2.0, tokens::HALO);
    let line = Stroke::new(tokens::LINE, tokens::SELECTION);
    if closed {
        painter.add(Shape::closed_line(points.clone(), halo));
        painter.add(Shape::closed_line(points, line));
    } else {
        painter.add(Shape::line(points.clone(), halo));
        painter.add(Shape::line(points, line));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn paint(
    ui: &Ui,
    ws: &mut Workspace,
    area: Rect,
    map: &ScreenMap,
    pointer: Option<Pos2>,
    modifiers: Modifiers,
    now: f64,
    rotate_cursor: bool,
) {
    let painter = ui.painter_at(area);
    painter.rect_filled(area, 0, tokens::PASTEBOARD);

    // Artboard.
    let surface = ws.project.surface();
    let artboard = screen_rect(map, surface.bounds());
    let shadow = Shadow {
        offset: [0, 6],
        blur: 24,
        spread: 0,
        color: color::SHADOW,
    };
    painter.add(shadow.as_shape(artboard, CornerRadius::ZERO));
    painter.rect_filled(artboard, 0, tokens::ARTBOARD);

    // Objects, bottom to top, culled to the visible area.
    let bucket = zoom_bucket(f64::from(map.scale));
    let objects = surface.objects.clone();
    for object in &objects {
        let bounds = screen_rect(map, object.frame.bounding_box());
        if !bounds.intersects(area) {
            continue;
        }
        let outline = ws.geometry.outline(object, bucket);
        let points: Vec<Pos2> = outline.iter().map(|p| map.to_screen(*p)).collect();
        draw_object(&painter, object, points, map.scale);
    }

    painter.rect_stroke(
        artboard,
        0,
        Stroke::new(1.0, color::BORDER_STRONG),
        StrokeKind::Outside,
    );
    let side = ws.project.surface().size;
    painter.text(
        artboard.left_top() - Vec2::new(0.0, space::SM),
        Align2::LEFT_BOTTOM,
        format!("{} · {side} × {side} px", ws.project.surface().name),
        egui::TextStyle::Small.resolve(ui.style()),
        color::TEXT_SECONDARY,
    );

    // Hover outline (idle, selection tools).
    if matches!(ws.gesture, Gesture::Idle)
        && matches!(ws.tool, crate::tool::Tool::Select | crate::tool::Tool::Move)
        && let Some(p) = pointer
        && let Some(id) = ws
            .project
            .surface()
            .hit_test(map.to_doc(p), map.doc_len(tokens::HIT_TOLERANCE))
        && !ws.selection.contains(&id)
        && let Some(object) = ws.project.surface().get(id).cloned()
    {
        let outline = ws.geometry.outline(&object, bucket);
        halo_polyline(
            &painter,
            outline.iter().map(|p| map.to_screen(*p)).collect(),
            true,
        );
    }

    draw_selection(&painter, ws, map, bucket);
    draw_gesture_feedback(ui, &painter, ws, map, pointer, modifiers);

    if rotate_cursor && let Some(p) = pointer {
        let font = icons::font(18.0);
        painter.text(
            p + Vec2::splat(1.0),
            Align2::CENTER_CENTER,
            icons::ROTATE,
            font.clone(),
            Color32::WHITE,
        );
        painter.text(
            p,
            Align2::CENTER_CENTER,
            icons::ROTATE,
            font,
            Color32::BLACK,
        );
    }

    draw_hint(ui, &painter, ws, area, now);
}

fn draw_object(painter: &Painter, object: &Object, points: Vec<Pos2>, scale: f32) {
    if points.len() < 3 {
        return;
    }
    let fill = color32(object.fill.with_opacity(object.opacity));
    let stroke = object.stroke.map(|s| {
        Stroke::new(
            (s.width as f32 * scale).max(0.5),
            color32(s.color.with_opacity(object.opacity)),
        )
    });
    match stroke {
        Some(stroke) => {
            painter.add(Shape::convex_polygon(points.clone(), fill, Stroke::NONE));
            painter.add(Shape::closed_line(points, stroke));
        }
        None => {
            painter.add(Shape::convex_polygon(points, fill, Stroke::NONE));
        }
    }
}

fn draw_selection(painter: &Painter, ws: &mut Workspace, map: &ScreenMap, bucket: i32) {
    if ws.selection.is_empty() {
        return;
    }
    let selected = ws.selected_objects();
    let Some(bounds) = selection_frame(&selected) else {
        return;
    };
    // Each object's own outline, thin and without halo so it never hides
    // the object's stroke.
    for id in ws.selection.clone() {
        if let Some(object) = ws.project.surface().get(id).cloned() {
            let outline = ws.geometry.outline(&object, bucket);
            let points = outline.iter().map(|p| map.to_screen(*p)).collect();
            painter.add(Shape::closed_line(
                points,
                Stroke::new(tokens::LINE, tokens::SELECTION),
            ));
        }
    }
    if ws.gesture.edits_document() && !matches!(ws.gesture, Gesture::Moving(_)) {
        // Keep the overlay light while resizing/rotating: bounds only.
        halo_polyline(
            painter,
            bounds.corners().map(|c| map.to_screen(c)).to_vec(),
            true,
        );
        return;
    }
    halo_polyline(
        painter,
        bounds.corners().map(|c| map.to_screen(c)).to_vec(),
        true,
    );

    for (_, pos) in screen_handles(&bounds, map) {
        let r = Rect::from_center_size(pos, Vec2::splat(tokens::HANDLE_SIZE));
        painter.rect_filled(r.expand(1.0), 1, tokens::HALO);
        painter.rect(
            r,
            1,
            tokens::HANDLE_FILL,
            Stroke::new(tokens::LINE, tokens::SELECTION),
            StrokeKind::Inside,
        );
    }
    let c = map.to_screen(bounds.center);
    let m = tokens::CENTER_MARK / 2.0;
    for (a, b) in [
        (c - Vec2::new(m, 0.0), c + Vec2::new(m, 0.0)),
        (c - Vec2::new(0.0, m), c + Vec2::new(0.0, m)),
    ] {
        halo_polyline(painter, vec![a, b], false);
    }
}

fn label_pill(ui: &Ui, painter: &Painter, at: Pos2, text: String) {
    let font = egui::TextStyle::Small.resolve(ui.style());
    let galley = painter.layout_no_wrap(text, font, color::TEXT_PRIMARY);
    let rect = Rect::from_min_size(
        at + Vec2::new(14.0, 14.0),
        galley.size() + Vec2::new(12.0, 6.0),
    );
    painter.rect_filled(rect, radius::SM, color::SURFACE_4);
    painter.galley(rect.min + Vec2::new(6.0, 3.0), galley, color::TEXT_PRIMARY);
}

fn draw_gesture_feedback(
    ui: &Ui,
    painter: &Painter,
    ws: &Workspace,
    map: &ScreenMap,
    pointer: Option<Pos2>,
    modifiers: Modifiers,
) {
    let Some(pointer) = pointer.or_else(|| ui.ctx().pointer_latest_pos()) else {
        return;
    };
    let doc = map.to_doc(pointer);
    match &ws.gesture {
        Gesture::Drawing { kind, start } => {
            let frame = drawing_frame(*start, doc, modifiers.shift, modifiers.alt);
            let preview = Object::new(tp_core::document::ObjectId(0), *kind, frame);
            let points: Vec<Pos2> = preview
                .flattened(map.doc_len(0.25))
                .iter()
                .map(|p| map.to_screen(*p))
                .collect();
            let mut ghost = preview.clone();
            ghost.opacity = 0.6;
            draw_object(painter, &ghost, points.clone(), map.scale);
            halo_polyline(painter, points, true);
            label_pill(ui, painter, pointer, size_text(&frame));
        }
        Gesture::Marquee { start, .. } | Gesture::ZoomRect { start } => {
            let r = Rect::from_two_pos(map.to_screen(*start), pointer);
            painter.rect_filled(r, 0, tokens::MARQUEE_FILL);
            halo_polyline(
                painter,
                vec![
                    r.left_top(),
                    r.right_top(),
                    r.right_bottom(),
                    r.left_bottom(),
                ],
                true,
            );
        }
        Gesture::Moving(base) => {
            if let (Some(first), Some(now)) =
                (base.originals.first(), ws.selected_objects().first())
            {
                let d = now.frame.center - first.frame.center;
                label_pill(ui, painter, pointer, format!("Δ {:.0}, {:.0} px", d.x, d.y));
            }
        }
        Gesture::Resizing { .. } => {
            if let Some(bounds) = selection_frame(&ws.selected_objects()) {
                label_pill(ui, painter, pointer, size_text(&bounds));
            }
        }
        Gesture::Rotating { base, .. } => {
            let selected = ws.selected_objects();
            let text = match (selected.as_slice(), base.originals.first()) {
                ([single], _) => format!("{:.0}°", single.frame.rotation_deg),
                (many, Some(first)) => {
                    let now = many.iter().find(|o| o.id == first.id);
                    let delta =
                        now.map_or(0.0, |o| o.frame.rotation_deg - first.frame.rotation_deg);
                    format!("{:.0}°", tp_core::document::normalize_degrees(delta))
                }
                _ => String::new(),
            };
            if !text.is_empty() {
                label_pill(ui, painter, pointer, text);
            }
        }
        Gesture::Idle | Gesture::Panning => {}
    }
}

fn size_text(frame: &Frame) -> String {
    format!("{:.0} × {:.0} px", frame.size.width, frame.size.height)
}

fn draw_hint(ui: &Ui, painter: &Painter, ws: &mut Workspace, area: Rect, now: f64) {
    let Some(hint) = &ws.hint else {
        return;
    };
    if now > hint.until {
        ws.hint = None;
        return;
    }
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_secs_f64(hint.until - now));
    let font = FontId::new(
        egui::TextStyle::Body.resolve(ui.style()).size,
        egui::FontFamily::Proportional,
    );
    let galley = painter.layout_no_wrap(hint.text.clone(), font, color::TEXT_PRIMARY);
    let size = galley.size() + Vec2::new(2.0 * space::MD, 2.0 * space::SM);
    let rect = Rect::from_center_size(
        Pos2::new(area.center().x, area.top() + space::LG + size.y / 2.0),
        size,
    );
    painter.rect(
        rect,
        radius::LG,
        color::SURFACE_4,
        Stroke::new(1.0, color::BORDER_STRONG),
        StrokeKind::Inside,
    );
    painter.galley(
        rect.min + Vec2::new(space::MD, space::SM),
        galley,
        color::TEXT_PRIMARY,
    );
}
