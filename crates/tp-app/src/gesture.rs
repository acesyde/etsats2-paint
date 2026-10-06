//! Canvas gestures (drags in progress) and press classification.

use std::collections::BTreeSet;

use egui::Pos2;
use tp_core::Snapshot;
use tp_core::document::{Frame, Handle, HandleSide, Object, ObjectId, PointRef, ShapeKind};
use tp_core::kurbo::Point;
use tp_ui::tokens::canvas;

use crate::path_edit::PointDrag;
use crate::viewport::ScreenMap;

/// A drag in progress on the canvas.
#[derive(Clone, Debug, Default)]
pub enum Gesture {
    #[default]
    Idle,
    Panning,
    Drawing {
        kind: ShapeKind,
        start: Point,
    },
    Marquee {
        start: Point,
        /// Selection kept under the marquee (non-empty with Shift).
        base: Vec<ObjectId>,
    },
    Moving(Transforming),
    Resizing {
        handle: Handle,
        bounds: Frame,
        base: Transforming,
    },
    Rotating {
        pivot: Point,
        start_angle: f64,
        base: Transforming,
    },
    ZoomRect {
        start: Point,
    },
    /// Selecting characters of the text being edited.
    TextSelect,
    /// Dragging the handle of the point just added with the Pen tool.
    PenHandle,
    /// Direct Selection: moving the selected path points.
    MovingPoints(PointDrag),
    /// Direct Selection: moving one handle.
    MovingHandle {
        drag: PointDrag,
        point: PointRef,
        side: HandleSide,
    },
    /// Direct Selection: selecting points inside a rectangle.
    PointMarquee {
        start: Point,
        base: BTreeSet<PointRef>,
        base_objects: Vec<ObjectId>,
    },
}

/// What every transform gesture remembers from its start.
#[derive(Clone, Debug)]
pub struct Transforming {
    pub start: Point,
    pub originals: Vec<Object>,
    pub before: Snapshot,
}

impl Gesture {
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Idle)
    }

    /// Whether the gesture edits the document (and so may be cancelled).
    pub fn edits_document(&self) -> bool {
        matches!(
            self,
            Self::Moving(_)
                | Self::Resizing { .. }
                | Self::Rotating { .. }
                | Self::MovingPoints(_)
                | Self::MovingHandle { .. }
        )
    }
}

/// What a press on the selection overlay targets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OverlayTarget {
    Handle(Handle),
    Rotate,
}

/// Selection corners and handle positions on screen.
pub fn screen_handles(bounds: &Frame, map: &ScreenMap) -> [(Handle, Pos2); 8] {
    Handle::ALL.map(|h| (h, map.to_screen(h.position(bounds))))
}

/// Classifies a press on the selection overlay: resize handles first, then
/// the rotation zone just outside the corners. `None` means the press falls
/// through to objects or empty canvas.
pub fn overlay_target(bounds: Option<&Frame>, map: &ScreenMap, pos: Pos2) -> Option<OverlayTarget> {
    let bounds = bounds?;
    let handles = screen_handles(bounds, map);
    let hit = handles
        .iter()
        .filter(|(_, p)| p.distance(pos) <= canvas::HANDLE_HIT_RADIUS)
        .min_by(|a, b| a.1.distance(pos).total_cmp(&b.1.distance(pos)));
    if let Some((handle, _)) = hit {
        return Some(OverlayTarget::Handle(*handle));
    }
    let inside = point_in_quad(&bounds.corners().map(|c| map.to_screen(c)), pos);
    let near_corner = handles
        .iter()
        .filter(|(h, _)| h.is_corner())
        .any(|(_, p)| p.distance(pos) <= canvas::ROTATE_ZONE);
    (near_corner && !inside).then_some(OverlayTarget::Rotate)
}

/// Point-in-convex-quad test (screen space, any winding).
pub fn point_in_quad(quad: &[Pos2; 4], p: Pos2) -> bool {
    let mut sign = 0.0f32;
    for i in 0..4 {
        let a = quad[i];
        let b = quad[(i + 1) % 4];
        let cross = (b - a).x * (p - a).y - (b - a).y * (p - a).x;
        if cross.abs() < f32::EPSILON {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    true
}

/// Frame of a shape being drawn from `start` to `current`.
/// `square` constrains to equal sides, `from_center` uses `start` as center.
pub fn drawing_frame(start: Point, current: Point, square: bool, from_center: bool) -> Frame {
    drawing_frame_aspect(start, current, square.then_some(1.0), from_center)
}

/// Like [`drawing_frame`], constraining width / height to `aspect` when set.
pub fn drawing_frame_aspect(
    start: Point,
    current: Point,
    aspect: Option<f64>,
    from_center: bool,
) -> Frame {
    let mut dx = current.x - start.x;
    let mut dy = current.y - start.y;
    if let Some(aspect) = aspect.filter(|a| *a > 0.0) {
        let h = dy.abs().max(dx.abs() / aspect);
        dx = (h * aspect).copysign(if dx == 0.0 { 1.0 } else { dx });
        dy = h.copysign(if dy == 0.0 { 1.0 } else { dy });
    }
    let (center, w, h) = if from_center {
        (start, 2.0 * dx.abs(), 2.0 * dy.abs())
    } else {
        (
            Point::new(start.x + dx / 2.0, start.y + dy / 2.0),
            dx.abs(),
            dy.abs(),
        )
    };
    Frame::new(center, tp_core::kurbo::Size::new(w, h), 0.0)
}

#[cfg(test)]
mod tests {
    use egui::{Vec2, pos2};
    use tp_core::kurbo::Size;

    use super::*;

    fn identity_map() -> ScreenMap {
        ScreenMap {
            scale: 1.0,
            offset: Vec2::ZERO,
        }
    }

    fn bounds() -> Frame {
        Frame::new(Point::new(200.0, 200.0), Size::new(200.0, 100.0), 0.0)
    }

    #[test]
    fn handle_has_priority() {
        let t = overlay_target(Some(&bounds()), &identity_map(), pos2(301.0, 251.0));
        assert_eq!(t, Some(OverlayTarget::Handle(Handle { x: 1, y: 1 })));
    }

    #[test]
    fn rotation_zone_is_outside_near_corners() {
        let t = overlay_target(Some(&bounds()), &identity_map(), pos2(315.0, 265.0));
        assert_eq!(t, Some(OverlayTarget::Rotate));
        // Inside the bounds near a corner: not rotation (falls through).
        let t = overlay_target(Some(&bounds()), &identity_map(), pos2(285.0, 238.0));
        assert_eq!(t, None);
        // Far away: nothing.
        assert_eq!(
            overlay_target(Some(&bounds()), &identity_map(), pos2(600.0, 600.0)),
            None
        );
    }

    #[test]
    fn no_selection_no_overlay() {
        assert_eq!(overlay_target(None, &identity_map(), pos2(0.0, 0.0)), None);
    }

    #[test]
    fn drawing_frames() {
        let f = drawing_frame(
            Point::new(100.0, 100.0),
            Point::new(500.0, 300.0),
            false,
            false,
        );
        assert_eq!(f.center, Point::new(300.0, 200.0));
        assert_eq!(f.size, Size::new(400.0, 200.0));
        let f = drawing_frame(
            Point::new(500.0, 500.0),
            Point::new(600.0, 550.0),
            false,
            true,
        );
        assert_eq!(f.center, Point::new(500.0, 500.0));
        assert_eq!(f.size, Size::new(200.0, 100.0));
        let f = drawing_frame(Point::new(0.0, 0.0), Point::new(-50.0, 20.0), true, false);
        assert_eq!(f.size, Size::new(50.0, 50.0));
        assert_eq!(f.center, Point::new(-25.0, 25.0));
    }
}
