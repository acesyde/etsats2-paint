//! Canvas drawing: artboard, objects, selection overlay, gesture feedback.

use egui::{
    Align2, Color32, CornerRadius, FontId, Modifiers, Painter, Pos2, Rect, Shadow, Shape, Stroke,
    StrokeKind, Ui, Vec2,
};
use std::sync::Arc;

use tp_core::document::{
    Frame, Node, Object, PathData, Rgba, ShapeKind, Subpath, flatten_subpaths, selection_frame,
};
use tp_core::kurbo;
use tp_ui::icons;
use tp_ui::tokens::{canvas as tokens, color, radius, space};

use crate::geometry_cache::{ShapeMesh, shape_mesh, tolerance, uses_mesh, zoom_bucket};
use crate::gesture::{Gesture, screen_handles};
use crate::gradient_textures::MeshPaint;
use crate::path_edit::line_points;
use crate::text_engine::layout_to_doc;
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
    drawn_cursor: super::DrawnCursor,
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
    let draw_list = tp_core::document::tree::draw_list(&surface.objects);
    for (object, opacity) in &draw_list {
        let bounds = screen_rect(map, object.frame.bounding_box());
        if !bounds.intersects(area) {
            continue;
        }
        match object.kind {
            ShapeKind::Text => draw_text(&painter, ws, object, *opacity, map, bucket),
            ShapeKind::Image { asset } => {
                draw_image(ui.ctx(), &painter, ws, object, asset, *opacity, map);
            }
            _ => {
                let geometry = ws.geometry.geometry(object, bucket);
                match &geometry.mesh {
                    Some(mesh) => draw_mesh_object(&painter, ws, object, *opacity, mesh, map),
                    None => {
                        let points: Vec<Pos2> = geometry
                            .outline()
                            .iter()
                            .map(|p| map.to_screen(*p))
                            .collect();
                        draw_object(&painter, object, *opacity, points, map.scale);
                    }
                }
            }
        }
    }

    draw_template(ui.ctx(), &painter, ws, map);
    painter.rect_stroke(
        artboard,
        0,
        Stroke::new(1.0, color::BORDER_STRONG),
        StrokeKind::Outside,
    );
    draw_grid_and_guides(&painter, ws, map, area);
    let side = ws.project.surface().size;
    painter.text(
        artboard.left_top() - Vec2::new(0.0, space::SM),
        Align2::LEFT_BOTTOM,
        format!("{} · {side} × {side} px", ws.project.surface().name),
        egui::TextStyle::Small.resolve(ui.style()),
        color::TEXT_SECONDARY,
    );

    // Hover outline (idle, selection tools): what a click would select.
    if matches!(ws.gesture, Gesture::Idle)
        && matches!(ws.tool, crate::tool::Tool::Select | crate::tool::Tool::Move)
        && let Some(p) = pointer
        && let Some(id) = super::click_target(ws, map.to_doc(p), map, modifiers.command)
        && !ws.selection.contains(&id)
        && let Some(object) = ws.project.surface().get(id).cloned()
    {
        let geometry = ws.geometry.geometry(&object, bucket);
        for (points, closed) in &geometry.lines {
            halo_polyline(
                &painter,
                points.iter().map(|p| map.to_screen(*p)).collect(),
                *closed,
            );
        }
    }

    if ws.is_editing_text() {
        draw_text_session(ui, &painter, ws, map, now);
    } else {
        draw_selection(&painter, ws, map, bucket);
    }
    draw_snap_hits(&painter, ws, map, area);
    draw_gesture_feedback(ui, &painter, ws, map, pointer, modifiers);
    draw_pen_session(&painter, ws, map, pointer);

    if let (Some((glyph, offset)), Some(p)) = (drawn_cursor, pointer) {
        let font = icons::font(18.0);
        let at = p + offset;
        painter.text(
            at + Vec2::splat(1.0),
            Align2::CENTER_CENTER,
            glyph,
            font.clone(),
            Color32::WHITE,
        );
        painter.text(at, Align2::CENTER_CENTER, glyph, font, Color32::BLACK);
    }

    draw_hint(ui, &painter, ws, area, now);
}

/// A document-space mesh on screen, colored by `paint` (a solid color, or
/// a gradient ramp whose texture coordinates follow the document).
fn mesh_to_screen(mesh: &tp_text::mesh::Mesh, map: &ScreenMap, paint: &MeshPaint) -> egui::Mesh {
    let mut out = egui::Mesh::with_texture(paint.texture_id());
    out.indices.clone_from(&mesh.indices);
    out.vertices = mesh
        .vertices
        .iter()
        .map(|[x, y]| {
            let (color, uv) = paint.vertex(kurbo::Point::new(f64::from(*x), f64::from(*y)));
            egui::epaint::Vertex {
                pos: Pos2::new(x * map.scale, y * map.scale) + map.offset,
                uv,
                color,
            }
        })
        .collect();
    out
}

/// How to color meshes painted with `paint` on `object`.
fn mesh_paint(
    ctx: &egui::Context,
    ws: &Workspace,
    paint: &tp_core::document::Paint,
    object: &Object,
    opacity: f32,
) -> MeshPaint {
    ws.gradients.mesh_paint(ctx, paint, &object.frame, opacity)
}

/// Alignment lines and crosses of what the gesture in progress snapped to.
fn draw_snap_hits(painter: &Painter, ws: &Workspace, map: &ScreenMap, area: Rect) {
    use crate::snap::SnapHit;
    use tp_core::Axis;
    let stroke = Stroke::new(1.0, tokens::SNAP);
    for hit in &ws.snap_hits {
        match *hit {
            SnapHit::Line {
                axis,
                value,
                extent,
                ..
            } => {
                let at = value as f32 * map.scale;
                match axis {
                    Axis::Vertical => {
                        let x = at + map.offset.x;
                        let range = match extent {
                            Some((a, b)) => {
                                (a as f32 * map.scale + map.offset.y)
                                    ..=(b as f32 * map.scale + map.offset.y)
                            }
                            None => area.y_range().into(),
                        };
                        painter.vline(x, range, stroke);
                    }
                    Axis::Horizontal => {
                        let y = at + map.offset.y;
                        let range = match extent {
                            Some((a, b)) => {
                                (a as f32 * map.scale + map.offset.x)
                                    ..=(b as f32 * map.scale + map.offset.x)
                            }
                            None => area.x_range().into(),
                        };
                        painter.hline(range, y, stroke);
                    }
                }
            }
            SnapHit::Point(p) => {
                let c = map.to_screen(p);
                let r = 4.0;
                painter.line_segment([c - Vec2::splat(r), c + Vec2::splat(r)], stroke);
                painter.line_segment([c + Vec2::new(-r, r), c + Vec2::new(r, -r)], stroke);
            }
        }
    }
}

/// The grid over the artboard, then the guides (and the one being created).
fn draw_grid_and_guides(painter: &Painter, ws: &Workspace, map: &ScreenMap, area: Rect) {
    if ws.aids.grid {
        let side = ws.project.surface().size;
        super::aids::paint_grid(painter, map, area, side, ws.aids.grid_spacing);
    }
    if ws.aids.guides {
        for guide in &ws.project.surface().guides {
            super::aids::paint_guide(painter, map, area, guide, tokens::GUIDE);
        }
    }
    if let Gesture::Guide {
        axis,
        index: None,
        position,
        ..
    } = &ws.gesture
    {
        let guide = tp_core::Guide::new(*axis, *position);
        super::aids::paint_guide(painter, map, area, &guide, tokens::GUIDE);
    }
}

/// Draws a shape from its triangles (fill, then stroke).
fn draw_mesh_object(
    painter: &Painter,
    ws: &Workspace,
    object: &Object,
    opacity: f32,
    mesh: &ShapeMesh,
    map: &ScreenMap,
) {
    let ctx = painter.ctx().clone();
    let fill = mesh_paint(&ctx, ws, &object.fill, object, opacity);
    let stroke = object
        .stroke
        .map(|s| mesh_paint(&ctx, ws, &s.paint, object, opacity));
    let layers = [
        (&mesh.fill, Some(fill)),
        (&mesh.casing, stroke),
        (&mesh.line, Some(fill)),
        (&mesh.stroke, stroke),
    ];
    for (part, paint) in layers {
        if let (Some(part), Some(paint)) = (part, paint)
            && !paint.is_invisible()
        {
            painter.add(Shape::mesh(mesh_to_screen(part, map, &paint)));
        }
    }
}

/// Draws a text from its glyph outlines (fill, then stroke on top).
fn draw_text(
    painter: &Painter,
    ws: &mut Workspace,
    object: &Arc<Object>,
    opacity: f32,
    map: &ScreenMap,
    bucket: i32,
) {
    let mesh = ws.text.mesh(object, bucket, tolerance(bucket));
    let ctx = painter.ctx().clone();
    if object.fill.is_visible() {
        let fill = mesh_paint(&ctx, ws, &object.fill, object, opacity);
        painter.add(Shape::mesh(mesh_to_screen(&mesh.fill, map, &fill)));
    }
    if let (Some(stroke_mesh), Some(stroke)) = (&mesh.stroke, object.stroke) {
        let paint = mesh_paint(&ctx, ws, &stroke.paint, object, opacity);
        painter.add(Shape::mesh(mesh_to_screen(stroke_mesh, map, &paint)));
    }
}

/// Draws an image as a textured quad on its (possibly rotated) frame, or a
/// placeholder while its texture is not ready.
/// Draws the active surface's template over the artwork, stretched to the
/// artboard (it is never an object: no selection, export or eyedropper).
fn draw_template(ctx: &egui::Context, painter: &Painter, ws: &mut Workspace, map: &ScreenMap) {
    let surface = ws.project.surface();
    let Some(template) = surface.template.clone().filter(|t| t.is_drawn()) else {
        return;
    };
    let side = surface.size;
    let Some(asset) = ws.project.assets.get(&template.asset).cloned() else {
        return;
    };
    let Some(texture) = ws.images.texture(
        ctx,
        &asset,
        side as f32 * map.scale * ctx.pixels_per_point(),
    ) else {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
        return;
    };
    let rect = screen_rect(map, kurbo::Rect::new(0.0, 0.0, side, side));
    let tint = Color32::WHITE.gamma_multiply(template.opacity.clamp(0.0, 1.0));
    painter.image(
        texture.id(),
        rect,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        tint,
    );
}

fn draw_image(
    ctx: &egui::Context,
    painter: &Painter,
    ws: &mut Workspace,
    object: &Object,
    asset: tp_core::document::AssetId,
    opacity: f32,
    map: &ScreenMap,
) {
    let corners = object.frame.corners().map(|c| map.to_screen(c));
    let texture = ws.project.assets.get(&asset).cloned().and_then(|a| {
        let longest = object.frame.size.width.max(object.frame.size.height) as f32;
        ws.images
            .texture(ctx, &a, longest * map.scale * ctx.pixels_per_point())
    });
    match texture {
        Some(texture) => {
            let mut mesh = egui::Mesh::with_texture(texture.id());
            let tint = Color32::WHITE.gamma_multiply(opacity);
            // Corners go top-left, top-right, bottom-right, bottom-left; a
            // mirrored image swaps left and right.
            let uvs = if object.mirrored {
                [(1.0, 0.0), (0.0, 0.0), (0.0, 1.0), (1.0, 1.0)]
            } else {
                [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
            };
            for (pos, (u, v)) in corners.iter().zip(uvs) {
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: *pos,
                    uv: Pos2::new(u, v),
                    color: tint,
                });
            }
            mesh.add_triangle(0, 1, 2);
            mesh.add_triangle(0, 2, 3);
            painter.add(Shape::mesh(mesh));
        }
        None => {
            let failed = ws.images.failed(asset);
            painter.add(Shape::convex_polygon(
                corners.to_vec(),
                tokens::IMAGE_PLACEHOLDER.gamma_multiply(opacity),
                Stroke::new(1.0, color::BORDER_STRONG),
            ));
            let center = map.to_screen(object.frame.center);
            let glyph = if failed { icons::WARNING } else { icons::IMAGE };
            painter.text(
                center,
                Align2::CENTER_CENTER,
                glyph,
                icons::font(18.0),
                color::SURFACE_3,
            );
            if !failed {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
        }
    }
}

/// Caret blink period, in seconds (on for half of it).
const CARET_BLINK: f64 = 1.0;

/// Highlighted characters and blinking caret of the text being edited.
fn draw_text_session(ui: &Ui, painter: &Painter, ws: &mut Workspace, map: &ScreenMap, now: f64) {
    let Some((object, layout)) = ws.session_layout() else {
        return;
    };
    let Some(session) = &ws.text_session else {
        return;
    };
    let to_doc = layout_to_doc(&object);
    let to_screen = |p: kurbo::Point| map.to_screen(to_doc * p);
    // Thin frame so the edited text stays visible on any background.
    painter.add(Shape::closed_line(
        object.frame.corners().map(|c| map.to_screen(c)).to_vec(),
        Stroke::new(tokens::LINE, tokens::SELECTION.gamma_multiply(0.5)),
    ));
    for r in session.edit.selection_rects(&layout) {
        let quad = [
            kurbo::Point::new(r.x0, r.y0),
            kurbo::Point::new(r.x1, r.y0),
            kurbo::Point::new(r.x1, r.y1),
            kurbo::Point::new(r.x0, r.y1),
        ]
        .map(to_screen);
        painter.add(Shape::convex_polygon(
            quad.to_vec(),
            tokens::TEXT_SELECTION,
            Stroke::NONE,
        ));
    }
    let elapsed = (now - session.caret_since).max(0.0);
    let phase = elapsed % CARET_BLINK;
    if !session.edit.has_selection() && phase < CARET_BLINK / 2.0 {
        let c = session.edit.caret_rect(&layout);
        let top = to_screen(kurbo::Point::new(c.x0, c.y0));
        let bottom = to_screen(kurbo::Point::new(c.x0, c.y1));
        painter.line_segment([top, bottom], Stroke::new(1.5, tokens::CARET));
    }
    let next = if phase < CARET_BLINK / 2.0 {
        CARET_BLINK / 2.0 - phase
    } else {
        CARET_BLINK - phase
    };
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_secs_f64(next.max(0.01)));
}

/// Draws a solid-painted shape (gradients go through meshes); `opacity`
/// already includes the object's and its groups'.
fn draw_object(painter: &Painter, object: &Object, opacity: f32, points: Vec<Pos2>, scale: f32) {
    if points.len() < 3 {
        return;
    }
    let fill = color32(object.fill.first_color().with_opacity(opacity));
    let stroke = object.stroke.map(|s| {
        Stroke::new(
            (s.width as f32 * scale).max(0.5),
            color32(s.paint.first_color().with_opacity(opacity)),
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
            let geometry = ws.geometry.geometry(&object, bucket);
            for (points, closed) in &geometry.lines {
                let points: Vec<Pos2> = points.iter().map(|p| map.to_screen(*p)).collect();
                let stroke = Stroke::new(tokens::LINE, tokens::SELECTION);
                painter.add(if *closed {
                    Shape::closed_line(points, stroke)
                } else {
                    Shape::line(points, stroke)
                });
            }
        }
    }
    // Align to: Key object — the object the others align to.
    if let Some(key) = ws.key_object()
        && let Some(object) = ws.project.surface().get(key).cloned()
    {
        let geometry = ws.geometry.geometry(&object, bucket);
        for (points, closed) in &geometry.lines {
            let points: Vec<Pos2> = points.iter().map(|p| map.to_screen(*p)).collect();
            let stroke = Stroke::new(tokens::KEY_OBJECT_LINE, tokens::SELECTION);
            painter.add(if *closed {
                Shape::closed_line(points, stroke)
            } else {
                Shape::line(points, stroke)
            });
        }
    }
    if ws.tool == crate::tool::Tool::DirectSelect {
        draw_points(painter, ws, map);
        return;
    }
    if ws.tool == crate::tool::Tool::Gradient {
        // Bounds without transform handles, then the gradient's handles.
        halo_polyline(
            painter,
            bounds.corners().map(|c| map.to_screen(c)).to_vec(),
            true,
        );
        super::gradient_tool::draw(painter, ws, map);
        return;
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
    // Sizes and angles are values: monospace, at caption size.
    let font = FontId::monospace(egui::TextStyle::Small.resolve(ui.style()).size);
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
    match &ws.gesture {
        Gesture::Drawing {
            kind: ShapeKind::Path,
            start,
            current,
        } => {
            let (a, b) = line_points(*start, *current, modifiers.shift, modifiers.alt);
            let line = ws.styled_path(
                PathData::new(vec![Subpath::new(
                    vec![Node::corner(a), Node::corner(b)],
                    false,
                )]),
                "Line",
            );
            draw_preview(painter, ws, &line, map);
            // Same value the inspector's Layout section will show as the rotation.
            let angle = crate::path_edit::line_angle(a, b);
            label_pill(
                ui,
                painter,
                pointer,
                format!(
                    "{:.0} px · {}°",
                    a.distance(b),
                    tp_i18n::localize_number(&format!("{angle:.1}"))
                ),
            );
        }
        Gesture::Drawing {
            kind,
            start,
            current,
        } => {
            let frame = super::shape_frame(*kind, *start, *current, modifiers);
            draw_preview(painter, ws, &ws.styled_shape(*kind, frame), map);
            label_pill(ui, painter, pointer, size_text(&frame));
        }
        Gesture::Marquee { start, .. }
        | Gesture::ZoomRect { start }
        | Gesture::PointMarquee { start, .. } => {
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
        Gesture::Guide { axis, position, .. } => {
            let text = match axis {
                tp_core::Axis::Horizontal => format!("y {position:.0} px"),
                tp_core::Axis::Vertical => format!("x {position:.0} px"),
            };
            label_pill(ui, painter, pointer, text);
        }
        Gesture::Idle
        | Gesture::Panning
        | Gesture::TextSelect
        | Gesture::PenHandle
        | Gesture::MovingPoints(_)
        | Gesture::MovingHandle { .. }
        | Gesture::Gradient(_) => {}
    }
}

/// Draws a shape being created, translucent, with its outline.
fn draw_preview(painter: &Painter, ws: &Workspace, object: &Object, map: &ScreenMap) {
    let tol = map.doc_len(0.25);
    let opacity = 0.6;
    if uses_mesh(object) {
        draw_mesh_object(painter, ws, object, opacity, &shape_mesh(object, tol), map);
    } else {
        let points = object
            .flattened(tol)
            .iter()
            .map(|p| map.to_screen(*p))
            .collect();
        draw_object(painter, object, opacity, points, map.scale);
    }
    for (points, closed) in flatten_subpaths(object.path(), tol) {
        halo_polyline(
            painter,
            points.iter().map(|p| map.to_screen(*p)).collect(),
            closed,
        );
    }
}

/// An anchor point square (filled when selected).
fn draw_point(painter: &Painter, at: Pos2, selected: bool) {
    let r = Rect::from_center_size(at, Vec2::splat(tokens::POINT_SIZE));
    painter.rect_filled(r.expand(1.0), 0, tokens::HALO);
    let fill = if selected {
        tokens::SELECTION
    } else {
        tokens::HANDLE_FILL
    };
    painter.rect(
        r,
        0,
        fill,
        Stroke::new(tokens::LINE, tokens::SELECTION),
        StrokeKind::Inside,
    );
}

/// A handle: a line from its point to a dot.
fn draw_handle(painter: &Painter, point: Pos2, handle: Pos2) {
    halo_polyline(painter, vec![point, handle], false);
    painter.circle(
        handle,
        tokens::HANDLE_DOT / 2.0,
        tokens::SELECTION,
        Stroke::new(1.0, tokens::HALO),
    );
}

/// Anchor points and handles of the selected paths (Direct Selection).
fn draw_points(painter: &Painter, ws: &Workspace, map: &ScreenMap) {
    for o in ws.selected_paths() {
        let affine = o.frame.affine();
        let handles = ws.shown_handles(&o);
        let Some(path) = o.path_data() else {
            continue;
        };
        for (r, _, at) in &handles {
            if let Some(node) = path.node(r.node) {
                draw_handle(
                    painter,
                    map.to_screen(affine * node.point),
                    map.to_screen(*at),
                );
            }
        }
        for r in path.node_refs() {
            let at = map.to_screen(affine * path.node(r).expect("ref").point);
            let selected = ws
                .points
                .contains(&tp_core::document::PointRef::new(o.id, r));
            draw_point(painter, at, selected);
        }
    }
}

/// The path being drawn with the Pen tool, the next segment under the
/// pointer, and its points.
fn draw_pen_session(painter: &Painter, ws: &Workspace, map: &ScreenMap, pointer: Option<Pos2>) {
    let Some(session) = &ws.pen else {
        return;
    };
    let mut nodes = session.nodes.clone();
    let closing = pointer
        .is_some_and(|p| ws.pen_over_first(map.to_doc(p), map.doc_len(tokens::POINT_HIT_RADIUS)));
    if !matches!(ws.gesture, Gesture::PenHandle)
        && let Some(p) = pointer
    {
        // Preview of the next segment.
        let target = if closing {
            nodes[0].point
        } else {
            map.to_doc(p)
        };
        nodes.push(Node::corner(target));
    }
    if nodes.len() >= 2 {
        let preview = ws.styled_path(PathData::new(vec![Subpath::new(nodes, false)]), "Path");
        draw_preview(painter, ws, &preview, map);
    }
    let to_screen = |p: kurbo::Point| map.to_screen(p);
    if let Some(last) = session.nodes.last() {
        for h in [last.handle_in, last.handle_out].into_iter().flatten() {
            draw_handle(painter, to_screen(last.point), to_screen(h));
        }
    }
    for (i, n) in session.nodes.iter().enumerate() {
        let highlighted = i == 0 && closing;
        draw_point(
            painter,
            to_screen(n.point),
            highlighted || i + 1 == session.nodes.len(),
        );
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
