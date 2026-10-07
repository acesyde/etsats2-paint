//! Gradient tool: handles of the selected object's gradient (start and end,
//! or center, radius and aspect) and drags that set a new gradient vector
//! on the selection.

use egui::{Color32, Painter, Pos2, Shape, Stroke};
use tp_core::document::{Gradient, GradientKind, Object, Paint, snap_direction};
use tp_core::kurbo::Point;
use tp_ui::tokens::canvas as tokens;

use crate::gesture::{Gesture, GradientDrag, GradientPoint};
use crate::viewport::ScreenMap;
use crate::workspace::{ColorTarget, Workspace, paint_of};

fn set_paint(o: &mut Object, target: ColorTarget, paint: Paint) {
    match target {
        ColorTarget::Fill => o.fill = paint,
        ColorTarget::Stroke => {
            if let Some(s) = &mut o.stroke {
                s.paint = paint;
            }
        }
    }
}

/// The only selected shape, when its target paint is a gradient: the one
/// whose handles are shown.
pub fn handle_object(ws: &Workspace) -> Option<(Object, Gradient)> {
    let target = ws.panels.color_target;
    let shapes = ws.selected_shapes();
    let [only] = shapes.as_slice() else {
        return None;
    };
    let g = *paint_of(only, target)?.gradient()?;
    Some((only.clone(), g))
}

/// Document positions of a gradient's handles: start (center), end (radius)
/// and, for radial gradients, the aspect handle.
fn handles(o: &Object, g: &Gradient) -> Vec<(GradientPoint, Point)> {
    let (s, e, m) = g.document_points(&o.frame);
    let mut out = vec![(GradientPoint::End, e), (GradientPoint::Start, s)];
    if g.kind == GradientKind::Radial {
        out.push((GradientPoint::Minor, m));
    }
    out
}

/// The gesture started by a press at `origin` (`doc` in the document):
/// dragging a handle, or drawing a new vector over the selection.
pub fn press(ws: &Workspace, map: &ScreenMap, origin: Pos2, doc: Point) -> Gesture {
    if ws.selection.is_empty() {
        return Gesture::Idle;
    }
    let point = handle_object(ws).and_then(|(o, g)| {
        handles(&o, &g)
            .into_iter()
            .find(|(_, p)| map.to_screen(*p).distance(origin) <= tokens::HANDLE_HIT_RADIUS)
            .map(|(which, _)| which)
    });
    Gesture::Gradient(GradientDrag {
        point,
        start: doc,
        originals: ws.selected_objects(),
        before: ws.snapshot(),
    })
}

/// `p` constrained to 45° steps around `pivot`.
fn constrain(pivot: Point, p: Point) -> Point {
    pivot + snap_direction(p - pivot)
}

/// Applies the drag to `pointer` (`shift`: 45° steps).
pub fn update(ws: &mut Workspace, drag: &GradientDrag, pointer: Point, shift: bool) {
    let target = ws.panels.color_target;
    let mut objects = drag.originals.clone();
    for o in &mut objects {
        o.for_each_shape(&mut |o: &mut Object| {
            if !o.visible || o.locked {
                return;
            }
            let Some(paint) = paint_of(o, target) else {
                return;
            };
            let frame = o.frame;
            let g = match drag.point {
                // A handle of the single object's existing gradient.
                Some(which) => {
                    let Some(mut g) = paint.gradient().copied() else {
                        return;
                    };
                    let (s, e, _) = g.document_points(&frame);
                    match which {
                        GradientPoint::Start => {
                            let p = if shift {
                                constrain(e, pointer)
                            } else {
                                pointer
                            };
                            g.set_points(&frame, p, e);
                        }
                        GradientPoint::End => {
                            let p = if shift {
                                constrain(s, pointer)
                            } else {
                                pointer
                            };
                            g.set_points(&frame, s, p);
                        }
                        GradientPoint::Minor => {
                            let main = (e - s).hypot();
                            if main > 1e-9 {
                                g.set_aspect(&frame, (pointer - s).hypot() / main);
                            }
                        }
                    }
                    g
                }
                // A new vector from the press point.
                None => {
                    // Solid paints first become the default linear gradient.
                    let mut g = match paint {
                        Paint::Gradient(g) => g,
                        Paint::Solid(c) => Gradient::from_color(GradientKind::Linear, c),
                    };
                    let end = if shift {
                        constrain(drag.start, pointer)
                    } else {
                        pointer
                    };
                    g.set_points(&frame, drag.start, end);
                    g
                }
            };
            set_paint(o, target, Paint::Gradient(g));
        });
    }
    ws.project.surface_mut().replace(&objects);
}

/// Undo label of a gradient drag.
pub fn label(target: ColorTarget) -> &'static str {
    match target {
        ColorTarget::Fill => "undo-change-fill-gradient",
        ColorTarget::Stroke => "undo-change-stroke-gradient",
    }
}

fn handle_dot(painter: &Painter, at: Pos2, square: bool) {
    let size = tokens::HANDLE_SIZE;
    if square {
        let r = egui::Rect::from_center_size(at, egui::Vec2::splat(size));
        painter.rect_filled(r.expand(1.0), 1, tokens::HALO);
        painter.rect(
            r,
            1,
            tokens::HANDLE_FILL,
            Stroke::new(tokens::LINE, tokens::SELECTION),
            egui::StrokeKind::Inside,
        );
    } else {
        painter.circle_filled(at, size / 2.0 + 1.0, tokens::HALO);
        painter.circle(
            at,
            size / 2.0,
            tokens::HANDLE_FILL,
            Stroke::new(tokens::LINE, tokens::SELECTION),
        );
    }
}

fn halo_line(painter: &Painter, points: Vec<Pos2>, closed: bool) {
    let halo = Stroke::new(tokens::LINE + 2.0, tokens::HALO);
    let line = Stroke::new(tokens::LINE, tokens::SELECTION);
    for stroke in [halo, line] {
        painter.add(if closed {
            Shape::closed_line(points.clone(), stroke)
        } else {
            Shape::line(points.clone(), stroke)
        });
    }
}

/// Draws the handles of the selected object's gradient: the vector, the
/// stop marks along it, and for radial gradients the outer ellipse.
pub fn draw(painter: &Painter, ws: &Workspace, map: &ScreenMap) {
    let Some((o, g)) = handle_object(ws) else {
        return;
    };
    let (s, e, m) = g.document_points(&o.frame);
    let (ss, se) = (map.to_screen(s), map.to_screen(e));
    if g.kind == GradientKind::Radial
        && let Some(to_doc) = g.to_document(&o.frame)
    {
        let ring: Vec<Pos2> = (0..64)
            .map(|k| {
                let a = k as f64 / 64.0 * std::f64::consts::TAU;
                map.to_screen(to_doc * Point::new(a.cos(), a.sin()))
            })
            .collect();
        halo_line(painter, ring, true);
        halo_line(painter, vec![ss, map.to_screen(m)], false);
    }
    halo_line(painter, vec![ss, se], false);
    for stop in g.stops() {
        let at = ss + (se - ss) * stop.offset;
        let c = stop.color;
        painter.circle_filled(at, 4.0, tokens::HALO);
        painter.circle(
            at,
            3.0,
            Color32::from_rgb(c.r, c.g, c.b),
            Stroke::new(1.0, tokens::SELECTION),
        );
    }
    for (which, p) in handles(&o, &g) {
        handle_dot(painter, map.to_screen(p), which == GradientPoint::Minor);
    }
}
