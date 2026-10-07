//! Boolean operations on closed shapes (Unite, Minus Front, Intersect,
//! Exclude), keeping curves. Built on `flo_curves`, whose types stay inside
//! this module.

use std::panic::{AssertUnwindSafe, catch_unwind};

use flo_curves::Coord2;
use flo_curves::bezier::path::{
    SimpleBezierPath, path_add, path_intersect, path_intersects_path, path_remove_interior_points,
    path_sub,
};
use kurbo::{BezPath, ParamCurve, PathEl, Point, Rect, Shape};

use super::object::{Object, ShapeKind};
use super::path::PathData;

/// Accuracy of the curve arithmetic, in texture pixels.
const ACCURACY: f64 = 0.01;

/// Which combination.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BooleanOp {
    Unite,
    MinusFront,
    Intersect,
    Exclude,
}

impl BooleanOp {
    pub const ALL: [Self; 4] = [
        Self::Unite,
        Self::MinusFront,
        Self::Intersect,
        Self::Exclude,
    ];
}

/// Why a combination produced nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BooleanError {
    /// The result has no area (no overlap, or everything removed).
    Empty,
    /// The geometry could not be combined reliably.
    Failed,
}

/// Why an object cannot be combined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperandProblem {
    /// Open lines have no area.
    OpenLine,
    /// Texts must be outlined first.
    Text,
    Image,
}

/// Why an object cannot take part in a combination, if it cannot.
pub fn operand_problem(o: &Object) -> Option<OperandProblem> {
    match o.kind {
        ShapeKind::Rectangle { .. } | ShapeKind::Ellipse | ShapeKind::Polygon { .. } => None,
        ShapeKind::Path if o.path_data().is_some_and(PathData::has_closed) => None,
        ShapeKind::Path => Some(OperandProblem::OpenLine),
        ShapeKind::Text => Some(OperandProblem::Text),
        ShapeKind::Image { .. } => Some(OperandProblem::Image),
        ShapeKind::Group => o
            .children
            .iter()
            .filter(|c| c.visible)
            .find_map(|c| operand_problem(c)),
    }
}

type Paths = Vec<SimpleBezierPath>;

fn c(p: Point) -> Coord2 {
    Coord2(p.x, p.y)
}

fn p(c: Coord2) -> Point {
    Point::new(c.0, c.1)
}

/// Closed subpaths of a kurbo path as `flo_curves` paths (lines and
/// quadratics become cubics).
fn to_flo(path: &BezPath) -> Paths {
    let mut out: Paths = Vec::new();
    let mut start = Point::ORIGIN;
    let mut last = Point::ORIGIN;
    let mut segments: Vec<(Coord2, Coord2, Coord2)> = Vec::new();
    let mut flush = |start: Point, last: Point, segments: &mut Vec<(Coord2, Coord2, Coord2)>| {
        if segments.is_empty() {
            return;
        }
        // Closed: end where it started.
        if last.distance(start) > 1e-9 {
            let (a, b) = (last, start);
            segments.push((c(a.lerp(b, 1.0 / 3.0)), c(a.lerp(b, 2.0 / 3.0)), c(b)));
        }
        out.push((c(start), std::mem::take(segments)));
    };
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(q) => {
                flush(start, last, &mut segments);
                start = q;
                last = q;
            }
            PathEl::LineTo(q) => {
                segments.push((c(last.lerp(q, 1.0 / 3.0)), c(last.lerp(q, 2.0 / 3.0)), c(q)));
                last = q;
            }
            PathEl::QuadTo(k, q) => {
                let c1 = last + (k - last) * (2.0 / 3.0);
                let c2 = q + (k - q) * (2.0 / 3.0);
                segments.push((c(c1), c(c2), c(q)));
                last = q;
            }
            PathEl::CurveTo(c1, c2, q) => {
                segments.push((c(c1), c(c2), c(q)));
                last = q;
            }
            PathEl::ClosePath => {
                flush(start, last, &mut segments);
                last = start;
            }
        }
    }
    flush(start, last, &mut segments);
    out
}

/// `flo_curves` paths back to a kurbo path; cubics whose control points lie
/// on their chord become lines.
fn from_flo(paths: &Paths) -> BezPath {
    let mut out = BezPath::new();
    for (start, segments) in paths {
        out.move_to(p(*start));
        let mut last = p(*start);
        for (c1, c2, end) in segments {
            let (a, b, e) = (p(*c1), p(*c2), p(*end));
            let chord = e - last;
            let len = chord.hypot();
            let off = |q: Point| {
                if len < 1e-12 {
                    (q - last).hypot()
                } else {
                    ((q - last).cross(chord) / len).abs()
                }
            };
            if off(a) <= 1e-6 * len.max(1.0) && off(b) <= 1e-6 * len.max(1.0) {
                out.line_to(e);
            } else {
                out.curve_to(a, b, e);
            }
            last = e;
        }
        out.close_path();
    }
    out
}

/// Whether two subpaths of an operand cross each other.
fn subpaths_cross(paths: &Paths) -> bool {
    for i in 0..paths.len() {
        for j in i + 1..paths.len() {
            if !path_intersects_path(&paths[i], &paths[j], ACCURACY).is_empty() {
                return true;
            }
        }
    }
    false
}

/// One shape's filled area as an even-odd set (what `flo_curves` expects).
fn operand(o: &Object) -> Paths {
    if o.is_group() {
        let mut acc: Paths = Vec::new();
        for child in o.children.iter().filter(|c| c.visible) {
            let part = operand(child);
            acc = if acc.is_empty() {
                part
            } else {
                path_add(&acc, &part, ACCURACY)
            };
        }
        return acc;
    }
    // Shapes as Convert to Path builds them (an ellipse is 4 points), so
    // results keep as few points as possible.
    let mut shape = o.clone();
    shape.convert_to_path();
    let paths = to_flo(&shape.fill_path());
    // Non-crossing subpaths (holes nested in outer contours): even-odd
    // and non-zero agree. Crossing subpaths: keep everything they enclose.
    if subpaths_cross(&paths) {
        path_remove_interior_points(&paths, ACCURACY)
    } else {
        paths
    }
}

fn xor(a: &Paths, b: &Paths) -> Paths {
    let left: Paths = path_sub(a, b, ACCURACY);
    let right: Paths = path_sub(b, a, ACCURACY);
    if left.is_empty() {
        right
    } else if right.is_empty() {
        left
    } else {
        path_add(&left, &right, ACCURACY)
    }
}

fn compute(objects: &[Object], op: BooleanOp) -> Paths {
    let operands: Vec<Paths> = objects.iter().map(operand).collect();
    match op {
        BooleanOp::Unite => operands
            .into_iter()
            .reduce(|a, b| path_add(&a, &b, ACCURACY))
            .unwrap_or_default(),
        BooleanOp::MinusFront => {
            let mut it = operands.into_iter();
            let bottom = it.next().unwrap_or_default();
            match it.reduce(|a, b| path_add(&a, &b, ACCURACY)) {
                Some(front) => path_sub(&bottom, &front, ACCURACY),
                None => bottom,
            }
        }
        BooleanOp::Intersect => operands
            .into_iter()
            .reduce(|a, b| path_intersect(&a, &b, ACCURACY))
            .unwrap_or_default(),
        BooleanOp::Exclude => operands
            .into_iter()
            .reduce(|a, b| xor(&a, &b))
            .unwrap_or_default(),
    }
}

/// Signed area of a closed kurbo subpath (flattened).
fn signed_area(path: &BezPath) -> f64 {
    let mut pts = Vec::new();
    kurbo::flatten(path.iter(), 0.05, |el| match el {
        PathEl::MoveTo(q) | PathEl::LineTo(q) => pts.push(q),
        _ => {}
    });
    let n = pts.len();
    (0..n)
        .map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum::<f64>()
        / 2.0
}

fn reversed(path: &BezPath) -> BezPath {
    // Rebuild the segments in the opposite order.
    let segs: Vec<kurbo::PathSeg> = path.segments().collect();
    let mut out = BezPath::new();
    let Some(first) = segs.last() else {
        return path.clone();
    };
    out.move_to(first.end());
    for seg in segs.iter().rev() {
        match seg.reverse() {
            kurbo::PathSeg::Line(l) => out.line_to(l.p1),
            kurbo::PathSeg::Quad(q) => out.quad_to(q.p1, q.p2),
            kurbo::PathSeg::Cubic(c) => out.curve_to(c.p1, c.p2, c.p3),
        }
    }
    out.close_path();
    out
}

/// Orients even-odd contours for the non-zero rule: contours nested an
/// even number of times wind one way, the others the opposite way.
fn orient_for_nonzero(paths: &Paths) -> BezPath {
    let subs: Vec<BezPath> = paths.iter().map(|q| from_flo(&vec![q.clone()])).collect();
    let mut out = BezPath::new();
    for (i, sub) in subs.iter().enumerate() {
        let probe = sub.segments().next().map_or(Point::ORIGIN, |s| s.eval(0.5));
        let depth = subs
            .iter()
            .enumerate()
            .filter(|(j, other)| *j != i && other.winding(probe) != 0)
            .count();
        let area = signed_area(sub);
        let want_positive = depth % 2 == 0;
        let fixed = if (area > 0.0) == want_positive {
            sub.clone()
        } else {
            reversed(sub)
        };
        out.extend(fixed.elements().iter().copied());
    }
    out
}

/// Fraction of a sampling grid over `bounds` where `expected` and `got`
/// disagree.
fn disagreement(bounds: Rect, expected: impl Fn(Point) -> bool, got: &BezPath) -> f64 {
    const N: usize = 96;
    let mut wrong = 0usize;
    for iy in 0..N {
        for ix in 0..N {
            let q = Point::new(
                bounds.x0 + (ix as f64 + 0.5) / N as f64 * bounds.width(),
                bounds.y0 + (iy as f64 + 0.5) / N as f64 * bounds.height(),
            );
            if expected(q) != (got.winding(q) != 0) {
                wrong += 1;
            }
        }
    }
    wrong as f64 / (N * N) as f64
}

/// Whether `q` is in the filled area of `o` (non-zero, groups: any child).
fn inside(o: &Object, q: Point) -> bool {
    if o.is_group() {
        return o.children.iter().any(|c| c.visible && inside(c, q));
    }
    o.fill_path().winding(q) != 0
}

/// Combines `objects` (bottom to top) into one path's geometry, in
/// document coordinates.
pub fn combine(objects: &[Object], op: BooleanOp) -> Result<PathData, BooleanError> {
    if objects.len() < 2 || objects.iter().any(|o| operand_problem(o).is_some()) {
        return Err(BooleanError::Failed);
    }
    let result = catch_unwind(AssertUnwindSafe(|| compute(objects, op)))
        .map_err(|_| BooleanError::Failed)?;
    let finite = result.iter().all(|(s, segs)| {
        s.0.is_finite()
            && s.1.is_finite()
            && segs
                .iter()
                .all(|(a, b, e)| [a.0, a.1, b.0, b.1, e.0, e.1].iter().all(|v| v.is_finite()))
    });
    if !finite {
        return Err(BooleanError::Failed);
    }
    let path = orient_for_nonzero(&result);
    let expected = |q: Point| {
        let ins: Vec<bool> = objects.iter().map(|o| inside(o, q)).collect();
        match op {
            BooleanOp::Unite => ins.iter().any(|b| *b),
            BooleanOp::MinusFront => ins[0] && !ins[1..].iter().any(|b| *b),
            BooleanOp::Intersect => ins.iter().all(|b| *b),
            BooleanOp::Exclude => ins.iter().filter(|b| **b).count() % 2 == 1,
        }
    };
    let bounds = objects
        .iter()
        .map(Object::bounding_box)
        .reduce(|a, b| a.union(b))
        .unwrap_or(Rect::ZERO);
    let has_area =
        signed_area(&path).abs() > 1e-6 || path.elements().len() > 2 && path.area().abs() > 1e-6;
    if path.elements().is_empty() || !has_area {
        // An empty result is only trusted when nothing is expected.
        return if disagreement(bounds, expected, &BezPath::new()) <= 0.002 {
            Err(BooleanError::Empty)
        } else {
            Err(BooleanError::Failed)
        };
    }
    if disagreement(bounds, expected, &path) > 0.01 {
        return Err(BooleanError::Failed);
    }
    Ok(PathData::from_bezpath(&path))
}

#[cfg(test)]
mod tests {
    use kurbo::Size;

    use super::*;
    use crate::document::path::{Node, Subpath};
    use crate::document::{Frame, ObjectId};

    fn rect(x0: f64, y0: f64, w: f64, h: f64) -> Object {
        Object::new(
            ObjectId(1),
            ShapeKind::rectangle(),
            Frame::new(Point::new(x0 + w / 2.0, y0 + h / 2.0), Size::new(w, h), 0.0),
        )
    }

    fn circle(cx: f64, cy: f64, r: f64) -> Object {
        Object::new(
            ObjectId(2),
            ShapeKind::Ellipse,
            Frame::new(Point::new(cx, cy), Size::new(2.0 * r, 2.0 * r), 0.0),
        )
    }

    fn area(data: &PathData) -> f64 {
        // Holes wind the other way: the signed sum is the filled area.
        data.fill_outline().area().abs()
    }

    fn points(data: &PathData) -> usize {
        data.subpaths.iter().map(|s| s.nodes.len()).sum()
    }

    fn filled(data: &PathData, q: Point) -> bool {
        data.fill_outline().winding(q) != 0
    }

    #[test]
    fn unite_two_squares() {
        let r = combine(
            &[rect(0.0, 0.0, 2.0, 2.0), rect(1.0, 1.0, 2.0, 2.0)],
            BooleanOp::Unite,
        )
        .unwrap();
        assert_eq!(r.subpaths.len(), 1);
        assert!((area(&r) - 7.0).abs() < 1e-6, "{}", area(&r));
        assert_eq!(points(&r), 8, "an L-like outline with straight edges");
        assert!(
            r.subpaths[0]
                .nodes
                .iter()
                .all(|n| n.handle_in.is_none() && n.handle_out.is_none())
        );
    }

    #[test]
    fn minus_front_cuts_a_notch() {
        let stripe = rect(0.0, 0.0, 1000.0, 100.0);
        let disk = circle(1000.0, 50.0, 40.0);
        let r = combine(&[stripe, disk], BooleanOp::MinusFront).unwrap();
        assert!(filled(&r, Point::new(500.0, 50.0)));
        assert!(!filled(&r, Point::new(990.0, 50.0)), "notch");
        assert!(filled(&r, Point::new(990.0, 2.0)), "corner kept");
    }

    #[test]
    fn exclude_makes_a_hole() {
        let r = combine(
            &[rect(0.0, 0.0, 100.0, 100.0), rect(25.0, 25.0, 50.0, 50.0)],
            BooleanOp::Exclude,
        )
        .unwrap();
        assert!(filled(&r, Point::new(10.0, 10.0)));
        assert!(!filled(&r, Point::new(50.0, 50.0)), "hole");
        assert_eq!(r.subpaths.len(), 2);
    }

    #[test]
    fn intersect_of_separate_circles_is_empty() {
        let e = combine(
            &[circle(0.0, 0.0, 10.0), circle(100.0, 0.0, 10.0)],
            BooleanOp::Intersect,
        );
        assert_eq!(e, Err(BooleanError::Empty));
        let all = combine(
            &[rect(10.0, 10.0, 10.0, 10.0), rect(0.0, 0.0, 100.0, 100.0)],
            BooleanOp::MinusFront,
        );
        assert_eq!(all, Err(BooleanError::Empty), "nothing would remain");
    }

    #[test]
    fn united_circles_keep_few_curved_points() {
        let (a, b) = (circle(0.0, 0.0, 100.0), circle(120.0, 0.0, 100.0));
        let r = combine(&[a, b], BooleanOp::Unite).unwrap();
        assert!(points(&r) <= 8, "{} points", points(&r));
        // Within 0.05 px of the circles' outer arcs.
        let outline = r.outline();
        let mut pts = Vec::new();
        kurbo::flatten(outline.iter(), 0.01, |el| match el {
            PathEl::MoveTo(q) | PathEl::LineTo(q) => pts.push(q),
            _ => {}
        });
        for q in pts {
            let d0 = (q.distance(Point::ZERO) - 100.0).abs();
            let d1 = (q.distance(Point::new(120.0, 0.0)) - 100.0).abs();
            assert!(d0.min(d1) < 0.05, "{q:?}");
        }
    }

    #[test]
    fn coincident_edges_and_tangent_circles() {
        // Two squares sharing a side.
        let r = combine(
            &[rect(0.0, 0.0, 10.0, 10.0), rect(10.0, 0.0, 10.0, 10.0)],
            BooleanOp::Unite,
        );
        let d = r.expect("squares sharing a side unite");
        assert!((area(&d) - 200.0).abs() < 1e-3, "{}", area(&d));
        // Tangent circles: either a correct union or a safe refusal.
        let r = combine(
            &[circle(0.0, 0.0, 10.0), circle(20.0, 0.0, 10.0)],
            BooleanOp::Unite,
        );
        if let Ok(d) = r {
            assert!(filled(&d, Point::new(0.0, 0.0)) && filled(&d, Point::new(20.0, 0.0)));
            assert!(!filled(&d, Point::new(10.0, 8.0)));
        }
    }

    #[test]
    fn glyph_like_hole_is_kept_as_operand() {
        // A square with a counter (opposite winding), minus a bar across it.
        let outer = Subpath::new(
            [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]
                .map(|q| Node::corner(q.into()))
                .to_vec(),
            true,
        );
        let hole = Subpath::new(
            [(30.0, 30.0), (30.0, 70.0), (70.0, 70.0), (70.0, 30.0)]
                .map(|q| Node::corner(q.into()))
                .to_vec(),
            true,
        );
        let o = Object::from_path(ObjectId(3), PathData::new(vec![outer, hole]));
        let r = combine(&[o, rect(-10.0, 45.0, 120.0, 10.0)], BooleanOp::MinusFront).unwrap();
        assert!(
            !filled(&r, Point::new(50.0, 40.0)),
            "the counter stays empty"
        );
        assert!(!filled(&r, Point::new(10.0, 50.0)), "the bar is cut");
        assert!(filled(&r, Point::new(10.0, 10.0)));
    }

    #[test]
    fn rounded_rectangle_round_trip_is_exact() {
        let mut r = rect(0.0, 0.0, 400.0, 200.0);
        r.kind = ShapeKind::Rectangle {
            corner_radius: 40.0,
        };
        r.convert_to_path();
        let src = r.fill_path();
        let back = from_flo(&to_flo(&src));
        let flat = |path: &BezPath| {
            let mut out = Vec::new();
            kurbo::flatten(path.iter(), 0.01, |el| match el {
                PathEl::MoveTo(q) | PathEl::LineTo(q) => out.push(q),
                _ => {}
            });
            out
        };
        let near = |q: Point, path: &BezPath| {
            path.segments()
                .map(|s| {
                    kurbo::ParamCurveNearest::nearest(&s, q, 1e-9)
                        .distance_sq
                        .sqrt()
                })
                .fold(f64::INFINITY, f64::min)
        };
        assert!(flat(&src).iter().all(|q| near(*q, &back) < 0.01));
        assert!(flat(&back).iter().all(|q| near(*q, &src) < 0.01));
        // Straight sides come back as straight segments.
        assert_eq!(
            back.elements()
                .iter()
                .filter(|e| matches!(e, PathEl::LineTo(_)))
                .count(),
            4
        );
    }

    #[test]
    fn operand_problems() {
        let mut t = rect(0.0, 0.0, 1.0, 1.0);
        t.kind = ShapeKind::Text;
        assert_eq!(operand_problem(&t), Some(OperandProblem::Text));
        let line = Object::from_path(
            ObjectId(4),
            PathData::new(vec![Subpath::new(
                vec![
                    Node::corner((0.0, 0.0).into()),
                    Node::corner((10.0, 0.0).into()),
                ],
                false,
            )]),
        );
        assert!(operand_problem(&line).is_some());
        assert!(operand_problem(&circle(0.0, 0.0, 1.0)).is_none());
    }
}
