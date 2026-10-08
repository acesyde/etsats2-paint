//! Move, resize and rotate for one or many objects.
//!
//! All functions start from the objects as they were when the gesture began,
//! so live updates never accumulate rounding errors.

use std::sync::Arc;

use kurbo::{Affine, Point, Rect, Vec2};

use super::object::{Frame, Object, ShapeKind, normalize_degrees};

/// Bounds a selection is transformed against: the object's own frame for a
/// single object, the axis-aligned union of bounds for several objects.
pub fn selection_frame(objects: &[Object]) -> Option<Frame> {
    match objects {
        [] => None,
        [single] => Some(single.frame),
        many => {
            let rect = many
                .iter()
                .map(Object::bounding_box)
                .reduce(|a, b| a.union(b))
                .unwrap_or(Rect::ZERO);
            Some(Frame::from_rect(rect))
        }
    }
}

/// A resize handle, as the sign of its position along each local axis:
/// `(-1, -1)` is the top-left corner, `(1, 0)` the right edge, etc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle {
    pub x: i8,
    pub y: i8,
}

impl Handle {
    pub const ALL: [Handle; 8] = [
        Handle { x: -1, y: -1 },
        Handle { x: 0, y: -1 },
        Handle { x: 1, y: -1 },
        Handle { x: 1, y: 0 },
        Handle { x: 1, y: 1 },
        Handle { x: 0, y: 1 },
        Handle { x: -1, y: 1 },
        Handle { x: -1, y: 0 },
    ];

    pub fn is_corner(self) -> bool {
        self.x != 0 && self.y != 0
    }

    /// Handle position in document space for `frame`.
    pub fn position(self, frame: &Frame) -> Point {
        let half = Vec2::new(frame.size.width / 2.0, frame.size.height / 2.0);
        frame.affine() * Point::new(f64::from(self.x) * half.x, f64::from(self.y) * half.y)
    }
}

/// Moves objects by `delta`. With `constrain`, the delta is snapped to the
/// nearest multiple of 45°.
pub fn translate(objects: &[Object], delta: Vec2, constrain: bool) -> Vec<Object> {
    let delta = if constrain {
        snap_direction(delta)
    } else {
        delta
    };
    objects
        .iter()
        .map(|o| {
            let mut o = o.clone();
            o.translate_deep(delta);
            o
        })
        .collect()
}

/// Projects `v` on the closest horizontal, vertical or diagonal direction.
pub fn snap_direction(v: Vec2) -> Vec2 {
    if v.hypot2() == 0.0 {
        return v;
    }
    let step = std::f64::consts::FRAC_PI_4;
    let angle = (v.atan2() / step).round() * step;
    let dir = Vec2::from_angle(angle);
    dir * v.dot(dir)
}

/// Options for [`resize`].
#[derive(Clone, Copy, Debug, Default)]
pub struct ResizeOptions {
    /// Keep proportions (Shift).
    pub proportional: bool,
    /// Scale around the center (Alt/Option).
    pub from_center: bool,
}

/// Resizes the selection by dragging `handle` of `bounds` to `pointer`.
pub fn resize(
    objects: &[Object],
    bounds: Frame,
    handle: Handle,
    pointer: Point,
    options: ResizeOptions,
) -> Vec<Object> {
    let to_local = bounds.affine().inverse();
    let local = to_local * pointer;
    let half = Vec2::new(bounds.size.width / 2.0, bounds.size.height / 2.0);
    let axis_scale = |sign: i8, p: f64, h: f64| -> f64 {
        if sign == 0 || h <= 0.0 {
            return 1.0;
        }
        let s = f64::from(sign);
        if options.from_center {
            p / (s * h)
        } else {
            (p + s * h) / (2.0 * s * h)
        }
    };
    let mut sx = axis_scale(handle.x, local.x, half.x);
    let mut sy = axis_scale(handle.y, local.y, half.y);
    if options.proportional {
        let m = if handle.is_corner() {
            sx.abs().max(sy.abs())
        } else if handle.x != 0 {
            sx.abs()
        } else {
            sy.abs()
        };
        sx = if sx < 0.0 { -m } else { m };
        sy = if sy < 0.0 { -m } else { m };
    }
    let anchor = if options.from_center {
        Point::ORIGIN
    } else {
        Point::new(-f64::from(handle.x) * half.x, -f64::from(handle.y) * half.y)
    };
    let local_transform = Affine::translate(anchor.to_vec2())
        * Affine::scale_non_uniform(sx, sy)
        * Affine::translate(-anchor.to_vec2());
    let transform = bounds.affine() * local_transform * to_local;

    objects
        .iter()
        .map(|o| resize_one(o, transform, sx, sy, bounds.rotation_deg))
        .collect()
}

/// Applies a resize (global `transform` plus the scale factors in the bounds'
/// axes) to one object and, recursively, to its children.
fn resize_one(o: &Object, transform: Affine, sx: f64, sy: f64, bounds_rotation: f64) -> Object {
    let (from, fill, stroke) = (o.frame, o.fill, o.stroke.map(|s| s.paint));
    let mut o = o.clone();
    // Full document transform of the object's local space.
    let full = transform * o.frame.affine();
    // Object axis relative to the bounds axis.
    let rel = (o.frame.rotation_deg - bounds_rotation).to_radians();
    let (sin, cos) = rel.sin_cos();
    let w_factor = (sx * cos).hypot(sy * sin);
    let h_factor = (sx * sin).hypot(sy * cos);
    // New direction of the object's x axis; shapes are symmetric, so the angle
    // is folded into (-90°, 90°] to avoid 180° jumps on flips.
    let mut new_rel = (sy * sin).atan2(sx * cos).to_degrees();
    if new_rel > 90.0 {
        new_rel -= 180.0;
    } else if new_rel <= -90.0 {
        new_rel += 180.0;
    }
    if rel.to_degrees().abs() > 90.0 {
        // Preserve the original half-turn for objects that had one.
        new_rel += 180.0;
    }
    o.frame = Frame {
        center: transform * o.frame.center,
        size: kurbo::Size::new(
            o.frame.size.width * w_factor,
            o.frame.size.height * h_factor,
        ),
        rotation_deg: bounds_rotation + new_rel,
    }
    .sanitized();
    match o.kind {
        // Paths take the exact transform (mirror and skew included) in their
        // points; the frame only carries the rotation and is refitted.
        ShapeKind::Path => {
            let to_local = o.frame.affine().inverse() * full;
            o.edit_path(|p| p.transform(to_local));
        }
        // Polygons are symmetric about their vertical axis only: their
        // rotation follows where the local "up" direction goes, so a
        // vertical flip turns them upside down.
        ShapeKind::Polygon { .. } => {
            o.frame.rotation_deg = rotation_of_up(full, o.frame.rotation_deg)
        }
        // Texts and images are not symmetric: a mirroring transform toggles
        // their mirror (a reflection across their vertical axis, which keeps
        // "up"), and their rotation follows where "up" goes.
        ShapeKind::Text | ShapeKind::Image { .. } if transform.determinant() < 0.0 => {
            o.mirrored = !o.mirrored;
            o.frame.rotation_deg = rotation_of_up(full, o.frame.rotation_deg);
        }
        _ => {}
    }
    o.sync_text_scale();
    // Gradients take the exact transform (mirror and skew included), like
    // paths, whatever the frame became (path refits moved them meanwhile).
    o.fill = fill;
    if let (Some(s), Some(paint)) = (&mut o.stroke, stroke) {
        s.paint = paint;
    }
    o.remap_paints(&from, transform);
    o.place_by(transform);
    for child in &mut o.children {
        *child = Arc::new(resize_one(child, transform, sx, sy, bounds_rotation));
    }
    o.refresh_group_frame();
    o
}

/// Direction a flip reverses: Horizontal swaps left and right, Vertical
/// swaps top and bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlipAxis {
    Horizontal,
    Vertical,
}

impl FlipAxis {
    pub const ALL: [Self; 2] = [Self::Horizontal, Self::Vertical];
}

/// Mirrors the selection across the vertical (Horizontal) or horizontal
/// (Vertical) line through the center of its bounds, along the texture's
/// axes whatever its rotation. The bounds stay where they were.
pub fn flip(objects: &[Object], axis: FlipAxis) -> Vec<Object> {
    let Some(bounds) = selection_frame(objects) else {
        return Vec::new();
    };
    let (sx, sy) = match axis {
        FlipAxis::Horizontal => (-1.0, 1.0),
        FlipAxis::Vertical => (1.0, -1.0),
    };
    let c = bounds.center.to_vec2();
    let transform =
        Affine::translate(c) * Affine::scale_non_uniform(sx, sy) * Affine::translate(-c);
    objects
        .iter()
        .map(|o| resize_one(o, transform, sx, sy, 0.0))
        .collect()
}

/// Rotation (degrees) that points a frame's local "up" where `full` maps
/// it; `fallback` when `full` collapses it.
fn rotation_of_up(full: Affine, fallback: f64) -> f64 {
    let up = full * Point::new(0.0, -1.0) - full * Point::ORIGIN;
    if up.hypot2() > 1e-18 {
        normalize_degrees(up.x.atan2(-up.y).to_degrees())
    } else {
        fallback
    }
}

/// Rotates the selection by `angle_deg` around `pivot`. With `snap`, a single
/// object's resulting rotation (or the delta, for several objects) is rounded
/// to a multiple of 15°.
pub fn rotate(objects: &[Object], pivot: Point, angle_deg: f64, snap: bool) -> Vec<Object> {
    let angle = match (snap, objects) {
        (true, [single]) => {
            let target = (single.frame.rotation_deg + angle_deg) / 15.0;
            target.round() * 15.0 - single.frame.rotation_deg
        }
        (true, _) => (angle_deg / 15.0).round() * 15.0,
        (false, _) => angle_deg,
    };
    let around = Affine::translate(pivot.to_vec2())
        * Affine::rotate(angle.to_radians())
        * Affine::translate(-pivot.to_vec2());
    objects
        .iter()
        .map(|o| rotate_one(o, around, angle))
        .collect()
}

fn rotate_one(o: &Object, around: Affine, angle: f64) -> Object {
    let mut o = o.clone();
    o.frame.center = around * o.frame.center;
    o.frame.rotation_deg = normalize_degrees(o.frame.rotation_deg + angle);
    o.place_by(around);
    for child in &mut o.children {
        *child = Arc::new(rotate_one(child, around, angle));
    }
    o.refresh_group_frame();
    o
}

/// Maps objects by any affine transform `a`. Paths and gradients take it
/// exactly; frames take its rotation and per-axis scales, a skew being
/// approximated as when a group is resized.
pub fn apply_affine(objects: &[Object], a: Affine) -> Vec<Object> {
    let [m0, m1, m2, m3, e, f] = a.as_coeffs();
    let sx = m0.hypot(m1);
    if sx < 1e-12 || (m0 * m3 - m1 * m2).abs() < 1e-12 {
        return objects.to_vec();
    }
    // a = translate(e, f) · rotate(θ) · m, with m upper triangular: the
    // per-axis scales (and a shear) in unrotated axes.
    let theta = m1.atan2(m0);
    let sy = (m0 * m3 - m1 * m2) / sx;
    let m = Affine::rotate(-theta) * Affine::new([m0, m1, m2, m3, 0.0, 0.0]);
    let around = Affine::translate((e, f)) * Affine::rotate(theta);
    objects
        .iter()
        .map(|o| {
            let scaled = resize_one(o, m, sx, sy, 0.0);
            rotate_one(&scaled, around, theta.to_degrees())
        })
        .collect()
}

/// Angle in degrees of `point` around `pivot` (clockwise on screen).
pub fn angle_around(pivot: Point, point: Point) -> f64 {
    (point - pivot).atan2().to_degrees()
}

#[cfg(test)]
mod tests {
    use kurbo::Size;

    use super::*;
    use crate::document::{ObjectId, ShapeKind};

    fn rect(center: (f64, f64), size: (f64, f64), rot: f64) -> Object {
        Object::new(
            ObjectId(1),
            ShapeKind::rectangle(),
            Frame::new(center.into(), size.into(), rot),
        )
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn assert_same(a: &Object, b: &Object) {
        assert!(
            a.frame.center.distance(b.frame.center) < 1e-6,
            "{:?} vs {:?}",
            a.frame,
            b.frame
        );
        assert!(
            close(a.frame.size.width, b.frame.size.width),
            "{:?} vs {:?}",
            a.frame,
            b.frame
        );
        assert!(close(a.frame.size.height, b.frame.size.height));
        assert!(close(
            normalize_degrees(a.frame.rotation_deg - b.frame.rotation_deg),
            0.0
        ));
    }

    #[test]
    fn apply_affine_matches_the_other_transforms() {
        let o = rect((100.0, 50.0), (200.0, 100.0), 30.0);
        // Identity and translation.
        assert_same(
            &apply_affine(std::slice::from_ref(&o), Affine::IDENTITY)[0],
            &o,
        );
        assert_same(
            &apply_affine(std::slice::from_ref(&o), Affine::translate((10.0, -5.0)))[0],
            &translate(std::slice::from_ref(&o), Vec2::new(10.0, -5.0), false)[0],
        );
        // A 2× scale about the origin.
        let doubled = &apply_affine(std::slice::from_ref(&o), Affine::scale(2.0))[0];
        assert_eq!(doubled.frame.center, Point::new(200.0, 100.0));
        assert!(close(doubled.frame.size.width, 400.0));
        assert!(close(doubled.frame.rotation_deg, 30.0));
        // A 90° rotation about the origin.
        assert_same(
            &apply_affine(std::slice::from_ref(&o), Affine::rotate(90f64.to_radians()))[0],
            &rotate(std::slice::from_ref(&o), Point::ORIGIN, 90.0, false)[0],
        );
        // A non-uniform scale on a rotated object: as resize does.
        let bounds = Frame::new(Point::new(0.0, 0.0), kurbo::Size::new(2.0, 2.0), 0.0);
        let resized = resize(
            std::slice::from_ref(&o),
            bounds,
            Handle { x: 1, y: 1 },
            Point::new(2.0, 0.5),
            ResizeOptions {
                proportional: false,
                from_center: true,
            },
        );
        assert_same(
            &apply_affine(
                std::slice::from_ref(&o),
                Affine::scale_non_uniform(2.0, 0.5),
            )[0],
            &resized[0],
        );
    }

    #[test]
    fn apply_affine_mirrors_paths_exactly() {
        use crate::document::{Node, PathData, Subpath};
        let path = Object::from_path(
            ObjectId(1),
            PathData::new(vec![Subpath::new(
                vec![
                    Node::corner(Point::new(10.0, 0.0)),
                    Node::corner(Point::new(30.0, 0.0)),
                    Node::corner(Point::new(30.0, 10.0)),
                ],
                false,
            )]),
        );
        let mirror = Affine::new([-1.0, 0.0, 0.0, 1.0, 100.0, 0.0]);
        let out = &apply_affine(std::slice::from_ref(&path), mirror)[0];
        let expected = mirror * path.path();
        let got = out.path();
        let pts = |p: &kurbo::BezPath| -> Vec<Point> {
            p.elements().iter().filter_map(|e| e.end_point()).collect()
        };
        for (a, b) in pts(&got).iter().zip(pts(&expected).iter()) {
            assert!(a.distance(*b) < 1e-6, "{a:?} vs {b:?}");
        }
    }

    #[test]
    fn proportional_corner_resize() {
        // 200×100 rectangle, drag the bottom-right handle until width is 400.
        let o = rect((100.0, 50.0), (200.0, 100.0), 0.0);
        let bounds = o.frame;
        let out = resize(
            &[o],
            bounds,
            Handle { x: 1, y: 1 },
            Point::new(400.0, 60.0),
            ResizeOptions {
                proportional: true,
                from_center: false,
            },
        );
        assert!(close(out[0].frame.size.width, 400.0));
        assert!(close(out[0].frame.size.height, 200.0));
        // Anchored at the top-left corner.
        assert!(close(out[0].frame.center.x, 200.0) && close(out[0].frame.center.y, 100.0));
    }

    #[test]
    fn resize_from_center_keeps_center() {
        let o = rect((500.0, 500.0), (100.0, 100.0), 0.0);
        let out = resize(
            std::slice::from_ref(&o),
            o.frame,
            Handle { x: 1, y: 1 },
            Point::new(600.0, 650.0),
            ResizeOptions {
                proportional: false,
                from_center: true,
            },
        );
        assert_eq!(out[0].frame.center, Point::new(500.0, 500.0));
        assert!(close(out[0].frame.size.width, 200.0) && close(out[0].frame.size.height, 300.0));
    }

    #[test]
    fn dragging_past_anchor_flips_without_negative_size() {
        let o = rect((50.0, 50.0), (100.0, 100.0), 0.0);
        let out = resize(
            std::slice::from_ref(&o),
            o.frame,
            Handle { x: 1, y: 0 },
            Point::new(-50.0, 50.0),
            ResizeOptions::default(),
        );
        assert!(close(out[0].frame.size.width, 50.0));
        assert!(close(out[0].frame.center.x, -25.0));
        assert!(close(out[0].frame.rotation_deg, 0.0));
    }

    #[test]
    fn rotated_single_resize_keeps_rotation() {
        let o = rect((0.0, 0.0), (100.0, 50.0), 30.0);
        let target = Handle { x: 1, y: 0 }.position(&Frame {
            size: Size::new(200.0, 50.0),
            center: o.frame.affine() * Point::new(50.0, 0.0),
            ..o.frame
        });
        let out = resize(
            std::slice::from_ref(&o),
            o.frame,
            Handle { x: 1, y: 0 },
            target,
            ResizeOptions::default(),
        );
        assert!(close(out[0].frame.size.width, 200.0));
        assert!(close(out[0].frame.rotation_deg, 30.0));
    }

    #[test]
    fn multi_resize_scales_positions() {
        let a = rect((50.0, 50.0), (100.0, 100.0), 0.0);
        let b = rect((250.0, 50.0), (100.0, 100.0), 0.0);
        let objects = [a, b];
        let bounds = selection_frame(&objects).unwrap();
        assert_eq!(bounds.size, Size::new(300.0, 100.0));
        let out = resize(
            &objects,
            bounds,
            Handle { x: 1, y: 0 },
            Point::new(600.0, 50.0),
            ResizeOptions::default(),
        );
        assert!(close(out[0].frame.center.x, 100.0) && close(out[1].frame.center.x, 500.0));
        assert!(close(out[1].frame.size.width, 200.0));
    }

    #[test]
    fn snapped_single_rotation_is_multiple_of_15() {
        let o = rect((0.0, 0.0), (10.0, 10.0), 7.0);
        let out = rotate(&[o], Point::ORIGIN, 31.0, true);
        assert!(close(out[0].frame.rotation_deg % 15.0, 0.0));
        assert!(close(out[0].frame.rotation_deg, 45.0));
    }

    #[test]
    fn multi_rotation_moves_centers_around_pivot() {
        let a = rect((100.0, 0.0), (10.0, 10.0), 0.0);
        let b = rect((-100.0, 0.0), (10.0, 10.0), 0.0);
        let out = rotate(&[a, b], Point::ORIGIN, 90.0, false);
        assert!(close(out[0].frame.center.x, 0.0) && close(out[0].frame.center.y, 100.0));
        assert!(close(out[1].frame.rotation_deg, 90.0));
    }

    #[test]
    fn constrained_move_snaps_to_axes() {
        let out = translate(
            &[rect((0.0, 0.0), (1.0, 1.0), 0.0)],
            Vec2::new(100.0, 8.0),
            true,
        );
        assert!(close(out[0].frame.center.y, 0.0));
        assert!(out[0].frame.center.x > 99.0);
    }

    fn group_of(a: Object, b: Object) -> Object {
        Object::group(ObjectId(9), vec![Arc::new(a), Arc::new(b)])
    }

    #[test]
    fn moving_a_group_moves_children() {
        let g = group_of(
            rect((0.0, 0.0), (10.0, 10.0), 0.0),
            rect((100.0, 0.0), (10.0, 10.0), 0.0),
        );
        let out = translate(&[g], Vec2::new(5.0, 7.0), false);
        assert_eq!(out[0].children[1].frame.center, Point::new(105.0, 7.0));
        assert_eq!(out[0].frame.center, Point::new(55.0, 7.0));
    }

    #[test]
    fn rotating_a_group_rotates_children_and_keeps_a_rotated_frame() {
        let g = group_of(
            rect((-100.0, 0.0), (20.0, 20.0), 0.0),
            rect((100.0, 0.0), (20.0, 20.0), 0.0),
        );
        let out = rotate(&[g], Point::ORIGIN, 90.0, false);
        let child = &out[0].children[1];
        assert!(close(child.frame.center.x, 0.0) && close(child.frame.center.y, 100.0));
        assert!(close(child.frame.rotation_deg, 90.0));
        assert!(close(out[0].frame.rotation_deg, 90.0));
        assert!(close(out[0].frame.size.width, 220.0) && close(out[0].frame.size.height, 20.0));
    }

    #[test]
    fn resizing_a_group_scales_children() {
        let g = group_of(
            rect((50.0, 50.0), (100.0, 100.0), 0.0),
            rect((250.0, 50.0), (100.0, 100.0), 0.0),
        );
        let bounds = g.frame;
        let out = resize(
            &[g],
            bounds,
            Handle { x: 1, y: 0 },
            Point::new(600.0, 50.0),
            ResizeOptions::default(),
        );
        assert!(close(out[0].children[1].frame.center.x, 500.0));
        assert!(close(out[0].children[1].frame.size.width, 200.0));
        assert!(close(out[0].frame.size.width, 600.0));
    }

    #[test]
    fn handle_positions() {
        let f = Frame::new(Point::new(10.0, 10.0), Size::new(20.0, 10.0), 0.0);
        assert_eq!(Handle { x: -1, y: -1 }.position(&f), Point::new(0.0, 5.0));
        assert_eq!(Handle { x: 1, y: 0 }.position(&f), Point::new(20.0, 10.0));
    }

    use crate::document::path::{Node, PathData, Subpath};

    /// An arrow pointing right: tip at (200, 50).
    fn arrow() -> Object {
        let pts = [
            (0.0, 40.0),
            (150.0, 40.0),
            (150.0, 0.0),
            (200.0, 50.0),
            (150.0, 100.0),
            (150.0, 60.0),
            (0.0, 60.0),
        ];
        Object::from_path(
            ObjectId(9),
            PathData::new(vec![Subpath::new(
                pts.iter().map(|p| Node::corner((*p).into())).collect(),
                true,
            )]),
        )
    }

    fn doc_points(o: &Object) -> Vec<Point> {
        let a = o.frame.affine();
        o.path_data().unwrap().subpaths[0]
            .nodes
            .iter()
            .map(|n| a * n.point)
            .collect()
    }

    fn linear_red_blue() -> crate::document::Paint {
        use crate::document::{ColorStop, Gradient, GradientKind, Paint, Rgba};
        Paint::Gradient(Gradient::new(
            GradientKind::Linear,
            &[
                ColorStop::new(0.0, Rgba::rgb(255, 0, 0)),
                ColorStop::new(1.0, Rgba::rgb(0, 0, 255)),
            ],
        ))
    }

    #[test]
    fn gradients_stay_in_place_when_path_points_move() {
        let mut a = arrow();
        a.fill = linear_red_blue();
        let before = a.fill.gradient().unwrap().document_points(&a.frame);
        // Move the tip far right: the bounds (and frame) grow.
        a.edit_path(|p| p.subpaths[0].nodes[3].point.x += 300.0);
        let after = a.fill.gradient().unwrap().document_points(&a.frame);
        assert!(close(before.0.x, after.0.x) && close(before.0.y, after.0.y));
        assert!(close(before.1.x, after.1.x) && close(before.1.y, after.1.y));
        // Changing the path's rotation keeps them in place too.
        a.set_path_rotation(30.0);
        let turned = a.fill.gradient().unwrap().document_points(&a.frame);
        assert!(close(before.1.x, turned.1.x) && close(before.1.y, turned.1.y));
    }

    #[test]
    fn flipping_mirrors_gradients() {
        for mut a in [
            arrow(),
            Object::new(ObjectId(2), ShapeKind::rectangle(), arrow().frame),
        ] {
            a.fill = linear_red_blue();
            let bounds = a.frame;
            let (s, e, _) = a.fill.gradient().unwrap().document_points(&a.frame);
            let out = resize(
                &[a],
                bounds,
                Handle { x: 1, y: 0 },
                Point::new(-200.0, 50.0),
                ResizeOptions::default(),
            );
            let o = &out[0];
            let (s2, e2, _) = o.fill.gradient().unwrap().document_points(&o.frame);
            // Mirrored about x = 0: red now on the right, blue on the left.
            assert!(close(s2.x, -s.x) && close(e2.x, -e.x), "{s2:?} {e2:?}");
            assert!(close(s2.y, s.y) && close(e2.y, e.y));
        }
    }

    #[test]
    fn flipping_a_path_mirrors_it() {
        let a = arrow();
        let bounds = a.frame;
        // Drag the right handle past the left side, to x = -200.
        let out = resize(
            &[a],
            bounds,
            Handle { x: 1, y: 0 },
            Point::new(-200.0, 50.0),
            ResizeOptions::default(),
        );
        let o = &out[0];
        let tip = doc_points(o)[3];
        let tail = doc_points(o)[0];
        assert!(
            tip.x < tail.x,
            "the arrow points left: tip {tip:?}, tail {tail:?}"
        );
        assert!(
            close(o.frame.rotation_deg, 0.0),
            "rotation {}",
            o.frame.rotation_deg
        );
        assert!(close(tip.x, -200.0) && close(tip.y, 50.0));
    }

    #[test]
    fn flipping_a_triangle_turns_it_upside_down() {
        let t = Object::new(
            ObjectId(4),
            ShapeKind::Polygon {
                sides: 3,
                star: None,
            },
            Frame::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0), 0.0),
        );
        let top = |o: &Object| o.bounding_box();
        let first_vertex = |o: &Object| o.flattened(0.1)[0];
        assert!(close(first_vertex(&t).y, top(&t).y0));
        let out = resize(
            std::slice::from_ref(&t),
            t.frame,
            Handle { x: 0, y: 1 },
            Point::new(0.0, -150.0),
            ResizeOptions::default(),
        );
        let f = &out[0];
        assert!(close(f.frame.rotation_deg.abs(), 180.0));
        assert!(close(first_vertex(f).y, top(f).y1), "points down");
        // A horizontal flip changes nothing visible.
        let h = resize(
            std::slice::from_ref(&t),
            t.frame,
            Handle { x: 1, y: 0 },
            Point::new(-150.0, 0.0),
            ResizeOptions::default(),
        );
        assert!(close(h[0].frame.rotation_deg, 0.0));
    }

    fn image(center: (f64, f64), size: (f64, f64), rot: f64) -> Object {
        Object::new(
            ObjectId(5),
            ShapeKind::Image {
                asset: crate::document::AssetId(1),
            },
            Frame::new(center.into(), size.into(), rot),
        )
    }

    #[test]
    fn dragging_past_the_opposite_side_mirrors_an_image() {
        let i = image((50.0, 50.0), (100.0, 100.0), 0.0);
        let out = resize(
            std::slice::from_ref(&i),
            i.frame,
            Handle { x: 1, y: 0 },
            Point::new(-50.0, 50.0),
            ResizeOptions::default(),
        );
        assert!(out[0].mirrored);
        assert!(close(out[0].frame.rotation_deg, 0.0));
        assert!(close(out[0].frame.size.width, 50.0));
        // Past the top: mirrored and upside down.
        let v = resize(
            std::slice::from_ref(&i),
            i.frame,
            Handle { x: 0, y: 1 },
            Point::new(50.0, -50.0),
            ResizeOptions::default(),
        );
        assert!(v[0].mirrored);
        assert!(close(v[0].frame.rotation_deg, 180.0));
        // A plain resize keeps the mirror as it was.
        let grown = resize(
            &out,
            out[0].frame,
            Handle { x: 1, y: 1 },
            Point::new(200.0, 200.0),
            ResizeOptions::default(),
        );
        assert!(grown[0].mirrored);
    }

    #[test]
    fn flipping_a_rotated_rectangle_reverses_its_rotation() {
        let r = rect((400.0, 400.0), (200.0, 100.0), 30.0);
        for axis in FlipAxis::ALL {
            let out = &flip(std::slice::from_ref(&r), axis)[0];
            assert!(out.frame.center.distance(r.frame.center) < 1e-9);
            assert!(close(out.frame.size.width, 200.0) && close(out.frame.size.height, 100.0));
            assert!(
                close(out.frame.rotation_deg, -30.0),
                "{axis:?}: {:?}",
                out.frame
            );
            assert!(!out.mirrored, "shapes carry no mirror");
        }
    }

    #[test]
    fn flipping_several_objects_swaps_their_ends() {
        let a = rect((100.0, 100.0), (100.0, 100.0), 0.0);
        let b = rect((500.0, 300.0), (50.0, 50.0), 0.0);
        let objects = [a, b];
        let before = selection_frame(&objects).unwrap();
        let out = flip(&objects, FlipAxis::Horizontal);
        let after = selection_frame(&out).unwrap();
        assert!(after.center.distance(before.center) < 1e-9);
        assert_eq!(after.size, before.size);
        assert!(close(out[0].frame.center.x, 475.0) && close(out[0].frame.center.y, 100.0));
        assert!(close(out[1].frame.center.x, 75.0) && close(out[1].frame.center.y, 300.0));
        let v = flip(&objects, FlipAxis::Vertical);
        assert!(close(v[0].frame.center.y, 275.0) && close(v[1].frame.center.y, 75.0));
    }

    #[test]
    fn flipping_twice_restores_a_path() {
        let a = arrow();
        let back = flip(
            &flip(std::slice::from_ref(&a), FlipAxis::Horizontal),
            FlipAxis::Horizontal,
        );
        assert!(back[0].frame.center.distance(a.frame.center) < 1e-9);
        for (p, q) in doc_points(&back[0]).iter().zip(doc_points(&a)) {
            assert!(p.distance(q) < 1e-9, "{p:?} vs {q:?}");
        }
        let once = &flip(std::slice::from_ref(&a), FlipAxis::Horizontal)[0];
        assert!(doc_points(once)[3].x < doc_points(once)[0].x, "points left");
    }

    #[test]
    fn flipping_an_image_both_ways_turns_it_upside_down() {
        let i = image((300.0, 200.0), (200.0, 100.0), 0.0);
        let h = flip(std::slice::from_ref(&i), FlipAxis::Horizontal);
        assert!(h[0].mirrored && close(h[0].frame.rotation_deg, 0.0));
        assert!(h[0].frame.center.distance(i.frame.center) < 1e-9);
        let hv = &flip(&h, FlipAxis::Vertical)[0];
        assert!(!hv.mirrored);
        assert!(close(hv.frame.rotation_deg, 180.0));
        assert!(close(hv.frame.size.width, 200.0) && close(hv.frame.size.height, 100.0));
        // A rotated image: −θ horizontally, 180° − θ vertically.
        let r = image((0.0, 0.0), (200.0, 100.0), 30.0);
        assert!(close(
            flip(std::slice::from_ref(&r), FlipAxis::Horizontal)[0]
                .frame
                .rotation_deg,
            -30.0
        ));
        assert!(close(
            flip(std::slice::from_ref(&r), FlipAxis::Vertical)[0]
                .frame
                .rotation_deg,
            150.0
        ));
    }

    #[test]
    fn multi_resize_of_rotated_path_is_exact() {
        let mut a = arrow();
        a.frame.rotation_deg = 30.0;
        let other = rect((1000.0, 1000.0), (100.0, 100.0), 0.0);
        let objects = [a.clone(), other];
        let bounds = selection_frame(&objects).unwrap();
        let handle = Handle { x: 1, y: 1 };
        let target = Point::new(
            bounds.center.x + bounds.size.width,
            bounds.center.y + bounds.size.height * 0.25,
        );
        let out = resize(&objects, bounds, handle, target, ResizeOptions::default());
        // Expected: the bounds transform applied to every original point.
        let half = Vec2::new(bounds.size.width / 2.0, bounds.size.height / 2.0);
        let to_local = bounds.affine().inverse();
        let local = to_local * target;
        let sx = (local.x + half.x) / (2.0 * half.x);
        let sy = (local.y + half.y) / (2.0 * half.y);
        let anchor = Vec2::new(-half.x, -half.y);
        let t = bounds.affine()
            * Affine::translate(anchor)
            * Affine::scale_non_uniform(sx, sy)
            * Affine::translate(-anchor)
            * to_local;
        for (got, orig) in doc_points(&out[0]).iter().zip(doc_points(&a)) {
            assert!(got.distance(t * orig) < 1e-6);
        }
    }
}
