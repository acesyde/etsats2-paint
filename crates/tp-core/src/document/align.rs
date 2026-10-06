//! Align and distribute objects by their visible axis-aligned bounds.

use kurbo::{Rect, Vec2};

use super::object::Object;
use super::transform::translate;

/// Which edge or center line to align.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Edge {
    Left,
    HCenter,
    Right,
    Top,
    VCenter,
    Bottom,
}

impl Edge {
    pub const ALL: [Self; 6] = [
        Self::Left,
        Self::HCenter,
        Self::Right,
        Self::Top,
        Self::VCenter,
        Self::Bottom,
    ];

    /// The coordinate of this edge on `r`.
    fn of(self, r: Rect) -> f64 {
        match self {
            Self::Left => r.x0,
            Self::HCenter => r.center().x,
            Self::Right => r.x1,
            Self::Top => r.y0,
            Self::VCenter => r.center().y,
            Self::Bottom => r.y1,
        }
    }

    fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::HCenter | Self::Right)
    }
}

/// Direction of a distribution: horizontal moves objects along x.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DistributeAxis {
    Horizontal,
    Vertical,
}

/// Evenly spaced centers, or equal gaps between neighbors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DistributeMode {
    Centers,
    Spacing,
}

/// Moves every object along one axis so that its `edge` is on `target`'s.
pub fn align(objects: &[Object], edge: Edge, target: Rect) -> Vec<Object> {
    let to = edge.of(target);
    objects
        .iter()
        .map(|o| {
            let d = to - edge.of(o.bounding_box());
            let delta = if edge.is_horizontal() {
                Vec2::new(d, 0.0)
            } else {
                Vec2::new(0.0, d)
            };
            translate(std::slice::from_ref(o), delta, false).remove(0)
        })
        .collect()
}

/// Spreads objects along `axis`, keeping the first and last (by position)
/// in place. Needs at least three objects; fewer are returned unchanged.
pub fn distribute(objects: &[Object], axis: DistributeAxis, mode: DistributeMode) -> Vec<Object> {
    if objects.len() < 3 {
        return objects.to_vec();
    }
    let span = |r: Rect| match axis {
        DistributeAxis::Horizontal => (r.x0, r.x1),
        DistributeAxis::Vertical => (r.y0, r.y1),
    };
    let boxes: Vec<(f64, f64)> = objects.iter().map(|o| span(o.bounding_box())).collect();
    let mut order: Vec<usize> = (0..objects.len()).collect();
    // Stable sort: ties keep the selection order.
    order.sort_by(|a, b| {
        let ca = (boxes[*a].0 + boxes[*a].1) / 2.0;
        let cb = (boxes[*b].0 + boxes[*b].1) / 2.0;
        ca.total_cmp(&cb)
    });
    let n = order.len();
    let first = boxes[order[0]];
    let last = boxes[order[n - 1]];
    let mut starts = vec![0.0; objects.len()];
    match mode {
        DistributeMode::Centers => {
            let c0 = (first.0 + first.1) / 2.0;
            let c1 = (last.0 + last.1) / 2.0;
            for (k, i) in order.iter().enumerate() {
                let center = c0 + (c1 - c0) * k as f64 / (n - 1) as f64;
                let half = (boxes[*i].1 - boxes[*i].0) / 2.0;
                starts[*i] = center - half;
            }
        }
        DistributeMode::Spacing => {
            let total: f64 = boxes.iter().map(|(a, b)| b - a).sum();
            let gap = ((last.1 - first.0) - total) / (n - 1) as f64;
            let mut at = first.0;
            for i in &order {
                starts[*i] = at;
                at += boxes[*i].1 - boxes[*i].0 + gap;
            }
        }
    }
    objects
        .iter()
        .zip(boxes.iter().zip(&starts))
        .map(|(o, (b, start))| {
            let d = start - b.0;
            let delta = match axis {
                DistributeAxis::Horizontal => Vec2::new(d, 0.0),
                DistributeAxis::Vertical => Vec2::new(0.0, d),
            };
            translate(std::slice::from_ref(o), delta, false).remove(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kurbo::{Point, Size};

    use super::*;
    use crate::document::path::{Node, PathData, Subpath};
    use crate::document::{Frame, ObjectId, ShapeKind};

    fn rect(x0: f64, y0: f64, w: f64, h: f64) -> Object {
        Object::new(
            ObjectId(1),
            ShapeKind::rectangle(),
            Frame::new(Point::new(x0 + w / 2.0, y0 + h / 2.0), Size::new(w, h), 0.0),
        )
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn align_left_edges_on_one_axis() {
        let objs = [
            rect(100.0, 0.0, 50.0, 50.0),
            rect(300.0, 70.0, 80.0, 20.0),
            rect(500.0, 200.0, 10.0, 10.0),
        ];
        let target = Rect::new(100.0, 0.0, 510.0, 210.0);
        let out = align(&objs, Edge::Left, target);
        for (o, before) in out.iter().zip(&objs) {
            assert!(close(o.bounding_box().x0, 100.0));
            assert!(close(o.bounding_box().y0, before.bounding_box().y0));
            assert_eq!(o.frame.size, before.frame.size);
        }
        let out = align(&objs, Edge::Bottom, target);
        assert!(out.iter().all(|o| close(o.bounding_box().y1, 210.0)));
        let out = align(&objs, Edge::HCenter, Rect::new(0.0, 0.0, 4096.0, 4096.0));
        assert!(
            out.iter()
                .all(|o| close(o.bounding_box().center().x, 2048.0))
        );
    }

    #[test]
    fn rotated_objects_align_by_visible_bounds() {
        let mut diamond = rect(0.0, 100.0, 100.0, 100.0);
        diamond.frame.rotation_deg = 45.0;
        let square = rect(300.0, 0.0, 100.0, 100.0);
        let out = align(&[diamond, square], Edge::Top, Rect::new(0.0, 0.0, 1.0, 1.0));
        assert!(
            close(out[0].bounding_box().y0, 0.0),
            "the top corner is at 0"
        );
        // The rotation is unchanged: only moved.
        assert_eq!(out[0].frame.rotation_deg, 45.0);
        assert!(close(out[1].bounding_box().y0, 0.0));
    }

    #[test]
    fn groups_move_as_a_whole() {
        let a = rect(0.0, 0.0, 10.0, 10.0);
        let b = rect(50.0, 30.0, 10.0, 10.0);
        let g = Object::group(ObjectId(9), vec![Arc::new(a), Arc::new(b)]);
        let out = align(&[g], Edge::Left, Rect::new(500.0, 0.0, 600.0, 1.0));
        let children: Vec<_> = out[0]
            .children
            .iter()
            .map(|c| c.bounding_box().x0)
            .collect();
        assert!(close(children[0], 500.0) && close(children[1], 550.0));
    }

    #[test]
    fn equal_gaps_keep_the_outermost_objects() {
        let objs = [
            rect(0.0, 0.0, 100.0, 10.0),
            rect(150.0, 0.0, 200.0, 10.0),
            rect(900.0, 0.0, 100.0, 10.0),
        ];
        let out = distribute(&objs, DistributeAxis::Horizontal, DistributeMode::Spacing);
        let b: Vec<Rect> = out.iter().map(Object::bounding_box).collect();
        assert!(close(b[0].x0, 0.0) && close(b[2].x0, 900.0));
        assert!(close(b[1].x0, 400.0));
        assert!(close(b[1].x0 - b[0].x1, 300.0) && close(b[2].x0 - b[1].x1, 300.0));
    }

    #[test]
    fn centers_are_evenly_spaced_in_any_selection_order() {
        let objs = [
            rect(0.0, 900.0, 10.0, 100.0),
            rect(0.0, 0.0, 10.0, 100.0),
            rect(0.0, 100.0, 10.0, 40.0),
        ];
        let out = distribute(&objs, DistributeAxis::Vertical, DistributeMode::Centers);
        let c: Vec<f64> = out.iter().map(|o| o.bounding_box().center().y).collect();
        // Ordered by position: 50 (second), 120 → 500, 950 (first).
        assert!(close(c[1], 50.0) && close(c[0], 950.0));
        assert!(close(c[2], 500.0));
    }

    #[test]
    fn overlapping_objects_get_a_negative_gap() {
        let objs = [
            rect(0.0, 0.0, 100.0, 10.0),
            rect(20.0, 0.0, 100.0, 10.0),
            rect(60.0, 0.0, 100.0, 10.0),
        ];
        let out = distribute(&objs, DistributeAxis::Horizontal, DistributeMode::Spacing);
        let b: Vec<Rect> = out.iter().map(Object::bounding_box).collect();
        assert!(close(b[0].x0, 0.0) && close(b[2].x1, 160.0));
        assert!(close(b[1].x0 - b[0].x1, -70.0) && close(b[2].x0 - b[1].x1, -70.0));
        // Two objects: unchanged.
        let two = distribute(
            &objs[..2],
            DistributeAxis::Horizontal,
            DistributeMode::Spacing,
        );
        assert_eq!(two, objs[..2].to_vec());
    }

    #[test]
    fn paths_move_exactly() {
        let path = Object::from_path(
            ObjectId(3),
            PathData::new(vec![Subpath::new(
                vec![
                    Node::smooth(Point::new(100.0, 100.0), Point::new(200.0, 0.0)),
                    Node::corner(Point::new(300.0, 300.0)),
                ],
                false,
            )]),
        );
        let before = path.path_data().cloned();
        let out = align(
            std::slice::from_ref(&path),
            Edge::Left,
            Rect::new(1000.0, 0.0, 2000.0, 1.0),
        );
        assert!(close(out[0].bounding_box().x0, 1000.0));
        // Only the frame moved: the local geometry is identical.
        assert_eq!(out[0].path_data().cloned(), before);
        assert_eq!(out[0].frame.rotation_deg, path.frame.rotation_deg);
    }
}
