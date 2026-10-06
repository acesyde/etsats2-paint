//! Move, resize and rotate for one or many objects.
//!
//! All functions start from the objects as they were when the gesture began,
//! so live updates never accumulate rounding errors.

use std::sync::Arc;

use kurbo::{Affine, Point, Rect, Vec2};

use super::object::{Frame, Object, normalize_degrees};

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
    let mut o = o.clone();
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
    for child in &mut o.children {
        *child = Arc::new(resize_one(child, transform, sx, sy, bounds_rotation));
    }
    o.refresh_group_frame();
    o
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
    for child in &mut o.children {
        *child = Arc::new(rotate_one(child, around, angle));
    }
    o.refresh_group_frame();
    o
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
}
