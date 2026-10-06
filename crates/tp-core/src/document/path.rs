//! Path geometry (subpaths of anchor points with Bézier handles), editing
//! operations, and regular polygon / star vertices.
//!
//! Path coordinates are in the owning object's local space (origin at the
//! frame center, unrotated); see `Object::refit`.

use super::object::ObjectId;
use kurbo::{
    Affine, BezPath, CubicBez, Line, ParamCurve, ParamCurveNearest, PathSeg, Point, Rect, Shape,
    Vec2,
};

/// An anchor point with optional handles (absolute positions).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub point: Point,
    pub handle_in: Option<Point>,
    pub handle_out: Option<Point>,
    /// Handles are kept aligned while editing.
    pub smooth: bool,
}

impl Node {
    pub fn corner(point: Point) -> Self {
        Self {
            point,
            handle_in: None,
            handle_out: None,
            smooth: false,
        }
    }

    /// A smooth point whose incoming handle mirrors `handle_out`.
    pub fn smooth(point: Point, handle_out: Point) -> Self {
        Self {
            point,
            handle_in: Some(point - (handle_out - point)),
            handle_out: Some(handle_out),
            smooth: true,
        }
    }

    fn transform(&mut self, a: Affine) {
        self.point = a * self.point;
        self.handle_in = self.handle_in.map(|h| a * h);
        self.handle_out = self.handle_out.map(|h| a * h);
    }

    fn translate(&mut self, d: Vec2) {
        self.transform(Affine::translate(d));
    }
}

/// An ordered list of anchor points, open or closed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Subpath {
    pub nodes: Vec<Node>,
    pub closed: bool,
}

impl Subpath {
    pub fn new(nodes: Vec<Node>, closed: bool) -> Self {
        Self { nodes, closed }
    }

    /// Number of segments (a closed subpath also joins last to first).
    pub fn segment_count(&self) -> usize {
        match self.nodes.len() {
            0 | 1 => 0,
            n if self.closed => n,
            n => n - 1,
        }
    }

    /// Segment `i`, from node `i` to the next one.
    pub fn segment(&self, i: usize) -> PathSeg {
        let a = &self.nodes[i];
        let b = &self.nodes[(i + 1) % self.nodes.len()];
        if a.handle_out.is_none() && b.handle_in.is_none() {
            PathSeg::Line(Line::new(a.point, b.point))
        } else {
            PathSeg::Cubic(CubicBez::new(
                a.point,
                a.handle_out.unwrap_or(a.point),
                b.handle_in.unwrap_or(b.point),
                b.point,
            ))
        }
    }

    fn append_to(&self, path: &mut BezPath) {
        let Some(first) = self.nodes.first() else {
            return;
        };
        path.move_to(first.point);
        let last = self.segment_count().saturating_sub(1);
        for i in 0..self.segment_count() {
            match self.segment(i) {
                // `close_path` draws a straight closing segment.
                PathSeg::Line(_) if self.closed && i == last => {}
                PathSeg::Line(l) => path.line_to(l.p1),
                PathSeg::Cubic(c) => path.curve_to(c.p1, c.p2, c.p3),
                PathSeg::Quad(q) => path.quad_to(q.p1, q.p2),
            }
        }
        if self.closed && self.nodes.len() > 1 {
            path.close_path();
        }
    }
}

/// Reference to one anchor point of a path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeRef {
    pub subpath: usize,
    pub node: usize,
}

impl NodeRef {
    pub fn new(subpath: usize, node: usize) -> Self {
        Self { subpath, node }
    }
}

/// A path point of a given object (point selection).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PointRef {
    pub object: ObjectId,
    pub node: NodeRef,
}

impl PointRef {
    pub fn new(object: ObjectId, node: NodeRef) -> Self {
        Self { object, node }
    }
}

/// Which handle of a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HandleSide {
    In,
    Out,
}

/// Where a point lies on a path's outline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentHit {
    pub subpath: usize,
    pub segment: usize,
    /// Curve parameter on the segment.
    pub t: f64,
    pub distance: f64,
}

/// Width of new lines (open subpaths), in texture pixels.
pub const DEFAULT_LINE_WIDTH: f64 = 8.0;

/// The geometry of a path object.
#[derive(Clone, Debug, PartialEq)]
pub struct PathData {
    pub subpaths: Vec<Subpath>,
    /// Width of the lines open subpaths are drawn as (fill color), in
    /// texture pixels. Not scaled by resizes.
    pub line_width: f64,
}

impl Default for PathData {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl PathData {
    pub fn new(subpaths: Vec<Subpath>) -> Self {
        Self {
            subpaths,
            line_width: DEFAULT_LINE_WIDTH,
        }
    }

    /// Converts a kurbo path (absolute coordinates kept): quadratic curves
    /// become exact cubics, a closing point repeating the first node is
    /// merged into it, and nodes with aligned handles are marked smooth.
    pub fn from_bezpath(path: &BezPath) -> Self {
        let mut subpaths: Vec<Subpath> = Vec::new();
        let mut current: Option<Subpath> = None;
        let finish = |sub: Subpath, out: &mut Vec<Subpath>| {
            if !sub.nodes.is_empty() {
                out.push(sub);
            }
        };
        for el in path.elements() {
            match *el {
                kurbo::PathEl::MoveTo(p) => {
                    if let Some(sub) = current.take() {
                        finish(sub, &mut subpaths);
                    }
                    current = Some(Subpath::new(vec![Node::corner(p)], false));
                }
                kurbo::PathEl::LineTo(p) => {
                    current
                        .get_or_insert_with(Subpath::default)
                        .nodes
                        .push(Node::corner(p));
                }
                kurbo::PathEl::QuadTo(c, p) => {
                    let sub = current.get_or_insert_with(Subpath::default);
                    let p0 = sub.nodes.last().map_or(p, |n| n.point);
                    let c1 = p0 + (c - p0) * (2.0 / 3.0);
                    let c2 = p + (c - p) * (2.0 / 3.0);
                    if let Some(last) = sub.nodes.last_mut() {
                        last.handle_out = Some(c1);
                    }
                    sub.nodes.push(Node {
                        handle_in: Some(c2),
                        ..Node::corner(p)
                    });
                }
                kurbo::PathEl::CurveTo(c1, c2, p) => {
                    let sub = current.get_or_insert_with(Subpath::default);
                    if let Some(last) = sub.nodes.last_mut() {
                        last.handle_out = Some(c1);
                    }
                    sub.nodes.push(Node {
                        handle_in: Some(c2),
                        ..Node::corner(p)
                    });
                }
                kurbo::PathEl::ClosePath => {
                    if let Some(mut sub) = current.take() {
                        sub.closed = true;
                        // A final node repeating the first one: merge them.
                        if sub.nodes.len() > 1 {
                            let last = *sub.nodes.last().expect("non-empty");
                            if last.point.distance(sub.nodes[0].point) < 1e-9 {
                                sub.nodes.pop();
                                sub.nodes[0].handle_in = last.handle_in;
                            }
                        }
                        finish(sub, &mut subpaths);
                    }
                }
            }
        }
        if let Some(sub) = current.take() {
            finish(sub, &mut subpaths);
        }
        for sub in &mut subpaths {
            for n in &mut sub.nodes {
                if let (Some(a), Some(b)) = (n.handle_in, n.handle_out) {
                    let (u, v) = (a - n.point, b - n.point);
                    let len = u.hypot() * v.hypot();
                    n.smooth = len > 1e-12 && u.cross(v).abs() <= 1e-6 * len && u.dot(v) < 0.0;
                }
            }
        }
        Self::new(subpaths)
    }

    pub fn is_empty(&self) -> bool {
        self.subpaths.iter().all(|s| s.nodes.is_empty())
    }

    pub fn has_closed(&self) -> bool {
        self.subpaths.iter().any(|s| s.closed && s.nodes.len() > 2)
    }

    pub fn has_open(&self) -> bool {
        self.subpaths.iter().any(|s| !s.closed && s.nodes.len() > 1)
    }

    /// Outline of every subpath (what the stroke follows).
    pub fn outline(&self) -> BezPath {
        let mut path = BezPath::new();
        for s in &self.subpaths {
            s.append_to(&mut path);
        }
        path
    }

    /// Closed subpaths only (what is filled).
    pub fn fill_outline(&self) -> BezPath {
        let mut path = BezPath::new();
        for s in self.subpaths.iter().filter(|s| s.closed) {
            s.append_to(&mut path);
        }
        path
    }

    /// Open subpaths only (what is drawn as lines).
    pub fn open_outline(&self) -> BezPath {
        let mut path = BezPath::new();
        for s in self.subpaths.iter().filter(|s| !s.closed) {
            s.append_to(&mut path);
        }
        path
    }

    /// Exact bounds, curve extrema included; `None` without points.
    pub fn bounds(&self) -> Option<Rect> {
        let mut points = self.subpaths.iter().flat_map(|s| &s.nodes).map(|n| n.point);
        let first = points.next()?;
        let mut rect = Rect::from_points(first, first).union(self.outline().bounding_box());
        for p in points {
            rect = rect.union_pt(p);
        }
        Some(rect)
    }

    /// Applies `a` to every point and handle.
    pub fn transform(&mut self, a: Affine) {
        for s in &mut self.subpaths {
            for n in &mut s.nodes {
                n.transform(a);
            }
        }
    }

    pub fn node(&self, r: NodeRef) -> Option<&Node> {
        self.subpaths.get(r.subpath)?.nodes.get(r.node)
    }

    pub fn node_mut(&mut self, r: NodeRef) -> Option<&mut Node> {
        self.subpaths.get_mut(r.subpath)?.nodes.get_mut(r.node)
    }

    /// Every node reference, in order.
    pub fn node_refs(&self) -> Vec<NodeRef> {
        self.subpaths
            .iter()
            .enumerate()
            .flat_map(|(s, sub)| (0..sub.nodes.len()).map(move |n| NodeRef::new(s, n)))
            .collect()
    }

    /// Moves points (with their handles) by `delta`.
    pub fn move_nodes(&mut self, refs: &[NodeRef], delta: Vec2) {
        for r in refs {
            if let Some(n) = self.node_mut(*r) {
                n.translate(delta);
            }
        }
    }

    /// Moves one handle to `pos`. Unless `broken`, a smooth point keeps its
    /// opposite handle aligned (keeping that handle's length); `broken`
    /// makes the point a corner.
    pub fn set_handle(&mut self, r: NodeRef, side: HandleSide, pos: Point, broken: bool) {
        let Some(n) = self.node_mut(r) else {
            return;
        };
        let (this, other) = match side {
            HandleSide::In => (&mut n.handle_in, &mut n.handle_out),
            HandleSide::Out => (&mut n.handle_out, &mut n.handle_in),
        };
        *this = Some(pos);
        if broken {
            n.smooth = false;
            return;
        }
        if n.smooth
            && let Some(o) = other
        {
            let dir = pos - n.point;
            let len = (*o - n.point).hypot();
            if dir.hypot() > 1e-9 {
                *o = n.point - dir.normalize() * len;
            }
        }
    }

    /// Toggles a point between corner (no handles) and smooth (aligned
    /// handles along the direction of its neighbors, a third of each
    /// adjacent segment long).
    pub fn toggle_smooth(&mut self, r: NodeRef) {
        let Some(sub) = self.subpaths.get_mut(r.subpath) else {
            return;
        };
        let len = sub.nodes.len();
        if r.node >= len {
            return;
        }
        let n = sub.nodes[r.node];
        if n.smooth {
            let node = &mut sub.nodes[r.node];
            node.smooth = false;
            node.handle_in = None;
            node.handle_out = None;
            return;
        }
        let neighbor = |i: isize| -> Option<Point> {
            if len < 2 {
                return None;
            }
            let j = r.node as isize + i;
            if sub.closed {
                Some(sub.nodes[j.rem_euclid(len as isize) as usize].point)
            } else if (0..len as isize).contains(&j) {
                Some(sub.nodes[j as usize].point)
            } else {
                None
            }
        };
        let (prev, next) = (neighbor(-1), neighbor(1));
        let dir = match (prev, next) {
            (Some(a), Some(b)) => b - a,
            (None, Some(b)) => b - n.point,
            (Some(a), None) => n.point - a,
            (None, None) => Vec2::ZERO,
        };
        let node = &mut sub.nodes[r.node];
        node.smooth = true;
        if dir.hypot() < 1e-9 {
            return;
        }
        let dir = dir.normalize();
        node.handle_in = prev.map(|a| n.point - dir * (a - n.point).hypot() / 3.0);
        node.handle_out = next.map(|b| n.point + dir * (b - n.point).hypot() / 3.0);
    }

    /// Inserts a point on segment `segment` at parameter `t` without
    /// changing the shape. Returns the new node's reference.
    pub fn insert_at(&mut self, subpath: usize, segment: usize, t: f64) -> Option<NodeRef> {
        let sub = self.subpaths.get_mut(subpath)?;
        if segment >= sub.segment_count() {
            return None;
        }
        let t = t.clamp(0.0, 1.0);
        let next = (segment + 1) % sub.nodes.len();
        let new = match sub.segment(segment) {
            PathSeg::Cubic(c) => {
                let left = c.subsegment(0.0..t);
                let right = c.subsegment(t..1.0);
                sub.nodes[segment].handle_out = Some(left.p1);
                sub.nodes[next].handle_in = Some(right.p2);
                Node {
                    point: left.p3,
                    handle_in: Some(left.p2),
                    handle_out: Some(right.p1),
                    smooth: true,
                }
            }
            seg => Node::corner(seg.eval(t)),
        };
        let index = segment + 1;
        sub.nodes.insert(index, new);
        Some(NodeRef::new(subpath, index))
    }

    /// Removes points; segments reconnect across them. An open subpath left
    /// with fewer than 2 points, or a closed one with fewer than 3, is
    /// removed.
    pub fn delete_nodes(&mut self, refs: &[NodeRef]) {
        for (s, sub) in self.subpaths.iter_mut().enumerate() {
            let mut i = 0;
            sub.nodes.retain(|_| {
                let keep = !refs.contains(&NodeRef::new(s, i));
                i += 1;
                keep
            });
        }
        self.subpaths
            .retain(|s| s.nodes.len() >= if s.closed { 3 } else { 2 });
    }

    /// The closest point of the outline to `p`.
    pub fn nearest_segment(&self, p: Point) -> Option<SegmentHit> {
        let mut best: Option<SegmentHit> = None;
        for (s, sub) in self.subpaths.iter().enumerate() {
            for i in 0..sub.segment_count() {
                let n = sub.segment(i).nearest(p, 1e-6);
                let distance = n.distance_sq.sqrt();
                if best.is_none_or(|b| distance < b.distance) {
                    best = Some(SegmentHit {
                        subpath: s,
                        segment: i,
                        t: n.t,
                        distance,
                    });
                }
            }
        }
        best
    }
}

/// Unit vertices of a regular polygon or star, first vertex pointing up.
fn unit_polygon(sides: u8, star: Option<f64>) -> Vec<Point> {
    let sides = usize::from(sides.clamp(3, 12));
    let count = if star.is_some() { sides * 2 } else { sides };
    let inner = star.unwrap_or(1.0).clamp(0.1, 0.9);
    (0..count)
        .map(|i| {
            let angle =
                -std::f64::consts::FRAC_PI_2 + i as f64 * std::f64::consts::TAU / count as f64;
            let r = if star.is_some() && i % 2 == 1 {
                inner
            } else {
                1.0
            };
            Point::new(r * angle.cos(), r * angle.sin())
        })
        .collect()
}

fn bounds_of(points: &[Point]) -> Rect {
    points
        .iter()
        .fold(Rect::from_points(points[0], points[0]), |r, p| {
            r.union_pt(*p)
        })
}

/// Vertices of a polygon (or star) whose bounds fill `rect`.
pub fn polygon_points(sides: u8, star: Option<f64>, rect: Rect) -> Vec<Point> {
    let unit = unit_polygon(sides, star);
    let b = bounds_of(&unit);
    unit.iter()
        .map(|p| {
            Point::new(
                rect.x0 + (p.x - b.x0) / b.width() * rect.width(),
                rect.y0 + (p.y - b.y0) / b.height() * rect.height(),
            )
        })
        .collect()
}

/// Width / height of a regular polygon (or star) with these settings.
pub fn regular_aspect(sides: u8, star: Option<f64>) -> f64 {
    let b = bounds_of(&unit_polygon(sides, star));
    b.width() / b.height()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(closed: bool) -> PathData {
        PathData::new(vec![Subpath::new(
            [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]
                .map(|p| Node::corner(p.into()))
                .to_vec(),
            closed,
        )])
    }

    #[test]
    fn outlines_and_bounds() {
        let mut p = square(true);
        p.subpaths.push(Subpath::new(
            vec![
                Node::corner((0.0, 200.0).into()),
                Node::corner((50.0, 250.0).into()),
            ],
            false,
        ));
        assert_eq!(p.fill_outline().elements().len(), 5);
        assert!(p.outline().elements().len() > 5);
        assert_eq!(p.bounds(), Some(Rect::new(0.0, 0.0, 100.0, 250.0)));
        assert!(p.has_closed() && p.has_open());
    }

    #[test]
    fn curve_bounds_include_extrema() {
        let p = PathData::new(vec![Subpath::new(
            vec![
                Node {
                    handle_out: Some((0.0, -100.0).into()),
                    ..Node::corner((0.0, 0.0).into())
                },
                Node {
                    handle_in: Some((100.0, -100.0).into()),
                    ..Node::corner((100.0, 0.0).into())
                },
            ],
            false,
        )]);
        let b = p.bounds().unwrap();
        assert!((b.y0 + 75.0).abs() < 1e-6, "{b:?}");
    }

    #[test]
    fn delete_corner_of_square_gives_triangle() {
        let mut p = square(true);
        p.delete_nodes(&[NodeRef::new(0, 2)]);
        assert_eq!(p.subpaths[0].nodes.len(), 3);
        assert!(p.subpaths[0].closed);
        p.delete_nodes(&[NodeRef::new(0, 0)]);
        assert!(p.subpaths.is_empty(), "a closed subpath needs 3 points");
        let mut open = square(false);
        open.delete_nodes(&[NodeRef::new(0, 0), NodeRef::new(0, 1), NodeRef::new(0, 2)]);
        assert!(open.is_empty());
    }

    #[test]
    fn insert_keeps_the_shape() {
        let mut p = PathData::new(vec![Subpath::new(
            vec![
                Node::smooth((0.0, 0.0).into(), (50.0, -80.0).into()),
                Node::smooth((200.0, 0.0).into(), (260.0, 90.0).into()),
                Node::corner((300.0, 200.0).into()),
            ],
            true,
        )]);
        let before: Vec<Point> = flat(&p.outline());
        let r = p.insert_at(0, 0, 0.3).unwrap();
        assert_eq!(r, NodeRef::new(0, 1));
        let r2 = p.insert_at(0, 3, 0.5).unwrap();
        assert_eq!(r2, NodeRef::new(0, 4));
        assert_eq!(p.subpaths[0].nodes.len(), 5);
        let path = p.outline();
        for q in before {
            let d = path
                .segments()
                .map(|s| s.nearest(q, 1e-9).distance_sq.sqrt())
                .fold(f64::INFINITY, f64::min);
            assert!(d < 0.01, "{d}");
        }
    }

    fn flat(path: &BezPath) -> Vec<Point> {
        let mut out = Vec::new();
        kurbo::flatten(path, 0.1, |el| match el {
            kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => out.push(p),
            _ => {}
        });
        out
    }

    #[test]
    fn aligned_handles_keep_their_length() {
        let mut p = PathData::new(vec![Subpath::new(
            vec![Node::smooth((500.0, 500.0).into(), (600.0, 500.0).into())],
            false,
        )]);
        p.set_handle(
            NodeRef::new(0, 0),
            HandleSide::Out,
            (500.0, 400.0).into(),
            false,
        );
        let n = p.subpaths[0].nodes[0];
        assert!(n.smooth);
        let hin = n.handle_in.unwrap();
        assert!((hin.x - 500.0).abs() < 1e-9 && (hin.y - 600.0).abs() < 1e-9);
        p.set_handle(
            NodeRef::new(0, 0),
            HandleSide::In,
            (450.0, 550.0).into(),
            true,
        );
        let n = p.subpaths[0].nodes[0];
        assert!(!n.smooth);
        assert_eq!(n.handle_out, Some((500.0, 400.0).into()));
    }

    #[test]
    fn toggle_smooth() {
        let mut p = square(true);
        let r = NodeRef::new(0, 1);
        p.toggle_smooth(r);
        let n = *p.node(r).unwrap();
        assert!(n.smooth);
        // Neighbors (0,0) and (100,100): direction (1,1).
        let out = n.handle_out.unwrap() - n.point;
        let inn = n.handle_in.unwrap() - n.point;
        assert!((out.x - out.y).abs() < 1e-9 && out.x > 0.0);
        assert!((out + inn).hypot() < 1e-9);
        p.toggle_smooth(r);
        assert_eq!(*p.node(r).unwrap(), Node::corner((100.0, 0.0).into()));
    }

    #[test]
    fn nearest_segment() {
        let p = square(true);
        let hit = p.nearest_segment((50.0, 103.0).into()).unwrap();
        assert_eq!((hit.subpath, hit.segment), (0, 2));
        assert!((hit.distance - 3.0).abs() < 1e-6);
    }

    fn flat_all(path: &BezPath) -> Vec<Point> {
        let mut out = Vec::new();
        kurbo::flatten(path, 0.01, |el| match el {
            kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => out.push(p),
            _ => {}
        });
        out
    }

    fn max_distance(points: &[Point], path: &BezPath) -> f64 {
        points
            .iter()
            .map(|p| {
                path.segments()
                    .map(|s| s.nearest(*p, 1e-9).distance_sq.sqrt())
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max)
    }

    #[test]
    fn from_bezpath_quads_are_exact() {
        // A TrueType-like contour: quadratic curves, closed back on its start.
        let mut src = BezPath::new();
        src.move_to((0.0, 0.0));
        src.quad_to((50.0, -40.0), (100.0, 0.0));
        src.quad_to((140.0, 50.0), (100.0, 100.0));
        src.line_to((0.0, 100.0));
        src.line_to((0.0, 0.0));
        src.close_path();
        let p = PathData::from_bezpath(&src);
        assert_eq!(p.subpaths.len(), 1);
        assert!(p.subpaths[0].closed);
        assert_eq!(p.subpaths[0].nodes.len(), 4, "closing point merged");
        let out = p.outline();
        assert!(max_distance(&flat_all(&src), &out) < 1e-6);
        assert!(max_distance(&flat_all(&out), &src) < 1e-6);
        // An open path stays open.
        let open = {
            let mut b = BezPath::new();
            b.move_to((0.0, 0.0));
            b.line_to((10.0, 0.0));
            b
        };
        let o = PathData::from_bezpath(&open);
        assert!(!o.subpaths[0].closed && o.subpaths[0].nodes.len() == 2);
    }

    #[test]
    fn from_bezpath_square_and_smooth_join() {
        let square = Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.1);
        let p = PathData::from_bezpath(&square);
        assert_eq!(p.subpaths[0].nodes.len(), 4);
        assert!(p.subpaths[0].nodes.iter().all(|n| !n.smooth));
        let mut s = BezPath::new();
        s.move_to((0.0, 0.0));
        s.curve_to((10.0, 0.0), (20.0, 10.0), (30.0, 10.0));
        s.curve_to((40.0, 10.0), (50.0, 0.0), (60.0, 0.0));
        let p = PathData::from_bezpath(&s);
        assert!(p.subpaths[0].nodes[1].smooth, "aligned handles");
        assert!(!p.subpaths[0].nodes[0].smooth);
    }

    #[test]
    fn polygons() {
        let tri = polygon_points(3, None, Rect::new(0.0, 0.0, 300.0, 200.0));
        assert!(tri[0].distance((150.0, 0.0).into()) < 1e-9);
        assert!(tri[1].distance((300.0, 200.0).into()) < 1e-9);
        assert!(tri[2].distance((0.0, 200.0).into()) < 1e-9);
        let hex = polygon_points(6, None, Rect::new(100.0, 100.0, 500.0, 500.0));
        assert_eq!(hex.len(), 6);
        assert!(hex[0].distance((300.0, 100.0).into()) < 1e-9);
        assert_eq!(
            polygon_points(5, Some(0.5), Rect::new(0.0, 0.0, 1.0, 1.0)).len(),
            10
        );
        // Regular: drawn at its natural aspect, every side is equal.
        let aspect = regular_aspect(5, None);
        let pent = polygon_points(5, None, Rect::new(0.0, 0.0, 100.0 * aspect, 100.0));
        let sides: Vec<f64> = (0..5)
            .map(|i| pent[i].distance(pent[(i + 1) % 5]))
            .collect();
        for s in &sides {
            assert!((s - sides[0]).abs() < 1e-9);
        }
    }
}
