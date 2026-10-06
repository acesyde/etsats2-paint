//! Vector objects, their frames and geometry.

use std::sync::Arc;

use kurbo::{Affine, BezPath, Ellipse, PathEl, Point, Rect, RoundedRect, Shape, Size, Vec2};

use super::color::{DEFAULT_FILL, Rgba};

/// Smallest width or height an object may have, in texture pixels.
pub const MIN_SIZE: f64 = 1.0;

/// Stable identifier of an object within a project.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(pub u64);

/// The geometric kind of an object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeKind {
    Rectangle {
        corner_radius: f64,
    },
    Ellipse,
    /// Holds `Object::children`; its frame is derived from them.
    Group,
}

impl ShapeKind {
    pub fn rectangle() -> Self {
        Self::Rectangle { corner_radius: 0.0 }
    }

    /// Short human name, used for undo labels.
    pub fn name(self) -> &'static str {
        match self {
            Self::Rectangle { .. } => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Group => "Group",
        }
    }

    pub fn is_group(self) -> bool {
        matches!(self, Self::Group)
    }
}

/// Placement of an object: center, size and clockwise rotation (degrees),
/// all in texture pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub center: Point,
    pub size: Size,
    pub rotation_deg: f64,
}

impl Frame {
    pub fn new(center: Point, size: Size, rotation_deg: f64) -> Self {
        Self {
            center,
            size,
            rotation_deg,
        }
    }

    /// Unrotated frame covering `rect`.
    pub fn from_rect(rect: Rect) -> Self {
        Self::new(rect.center(), rect.size(), 0.0)
    }

    /// Maps local coordinates (origin at the center, unrotated) to the
    /// document. With y pointing down, positive angles turn clockwise.
    pub fn affine(&self) -> Affine {
        Affine::translate(self.center.to_vec2()) * Affine::rotate(self.rotation_deg.to_radians())
    }

    /// The unrotated local rectangle, centered on the origin.
    pub fn local_rect(&self) -> Rect {
        Rect::from_center_size(Point::ORIGIN, self.size)
    }

    /// Corners in document space, clockwise from top-left (in local space).
    pub fn corners(&self) -> [Point; 4] {
        let r = self.local_rect();
        let a = self.affine();
        [
            a * Point::new(r.x0, r.y0),
            a * Point::new(r.x1, r.y0),
            a * Point::new(r.x1, r.y1),
            a * Point::new(r.x0, r.y1),
        ]
    }

    /// Axis-aligned bounding box of the rotated frame.
    pub fn bounding_box(&self) -> Rect {
        let c = self.corners();
        c[1..]
            .iter()
            .fold(Rect::from_points(c[0], c[0]), |r, p| r.union_pt(*p))
    }

    /// Clamps the size to the minimum and normalizes the rotation.
    pub fn sanitized(mut self) -> Self {
        self.size = Size::new(
            self.size.width.max(MIN_SIZE),
            self.size.height.max(MIN_SIZE),
        );
        self.rotation_deg = normalize_degrees(self.rotation_deg);
        self
    }
}

/// Normalizes an angle to `(-180, 180]`.
pub fn normalize_degrees(deg: f64) -> f64 {
    let mut d = deg % 360.0;
    if d <= -180.0 {
        d += 360.0;
    } else if d > 180.0 {
        d -= 360.0;
    }
    if d.abs() < 1e-9 { 0.0 } else { d }
}

/// Outline stroke.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokeStyle {
    pub color: Rgba,
    /// Width in texture pixels.
    pub width: f64,
}

/// A vector object on a surface.
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub id: ObjectId,
    pub kind: ShapeKind,
    pub name: String,
    pub frame: Frame,
    pub fill: Rgba,
    pub stroke: Option<StrokeStyle>,
    /// 0.0..=1.0; for groups, multiplied into the children.
    pub opacity: f32,
    pub visible: bool,
    pub locked: bool,
    /// Children of a group, bottom to top. Empty for shapes.
    pub children: Vec<Arc<Object>>,
}

impl Object {
    /// A shape with the default appearance and the default name of its kind.
    pub fn new(id: ObjectId, kind: ShapeKind, frame: Frame) -> Self {
        Self {
            id,
            kind,
            name: kind.name().to_owned(),
            frame: frame.sanitized(),
            fill: DEFAULT_FILL,
            stroke: None,
            opacity: 1.0,
            visible: true,
            locked: false,
            children: Vec::new(),
        }
    }

    /// A group of `children` (bottom to top), unrotated, frame derived.
    pub fn group(id: ObjectId, children: Vec<Arc<Object>>) -> Self {
        let mut group = Self::new(
            id,
            ShapeKind::Group,
            Frame::new(Point::ORIGIN, Size::ZERO, 0.0),
        );
        group.children = children;
        group.refresh_group_frame();
        group
    }

    pub fn is_group(&self) -> bool {
        self.kind.is_group()
    }

    /// Applies `f` to this object if it is a shape, or to every shape in
    /// its subtree if it is a group (then refreshes group frames).
    pub fn for_each_shape(&mut self, f: &mut impl FnMut(&mut Object)) {
        if self.is_group() {
            for child in &mut self.children {
                Arc::make_mut(child).for_each_shape(f);
            }
            self.refresh_group_frame();
        } else {
            f(self);
        }
    }

    /// Shapes of this object's subtree (itself if it is a shape).
    pub fn shapes(&self) -> Vec<&Object> {
        if self.is_group() {
            self.children.iter().flat_map(|c| c.shapes()).collect()
        } else {
            vec![self]
        }
    }

    /// Moves the object and all its descendants by `delta`.
    pub fn translate_deep(&mut self, delta: Vec2) {
        self.frame.center += delta;
        for child in &mut self.children {
            Arc::make_mut(child).translate_deep(delta);
        }
    }

    /// Points describing the visible extent of the object (document space).
    fn extent_points(&self, out: &mut Vec<Point>) {
        if !self.visible {
            return;
        }
        match self.kind {
            ShapeKind::Group => {
                for child in &self.children {
                    child.extent_points(out);
                }
            }
            ShapeKind::Rectangle { .. } => out.extend(self.frame.corners()),
            ShapeKind::Ellipse => out.extend(self.flattened(0.5)),
        }
    }

    /// Recomputes a group's frame as the bounds of its visible children,
    /// measured in the group's own rotation. No-op for shapes.
    pub fn refresh_group_frame(&mut self) {
        if !self.is_group() {
            return;
        }
        let mut points = Vec::new();
        for child in &self.children {
            child.extent_points(&mut points);
        }
        let Some(first) = points.first().copied() else {
            self.frame.size = Size::new(MIN_SIZE, MIN_SIZE);
            return;
        };
        let rotation = Affine::rotate(self.frame.rotation_deg.to_radians());
        let unrotate = rotation.inverse();
        let local = points.iter().map(|p| unrotate * *p).fold(
            Rect::from_points(unrotate * first, unrotate * first),
            |r, p| r.union_pt(p),
        );
        self.frame.center = rotation * local.center();
        self.frame.size = Size::new(local.width().max(MIN_SIZE), local.height().max(MIN_SIZE));
    }

    /// Outline in local space (centered, unrotated), grown by `grow` on each
    /// side (used for hit-test tolerance).
    fn local_path(&self, grow: f64) -> BezPath {
        let rect = self.frame.local_rect().inflate(grow, grow);
        match self.kind {
            ShapeKind::Rectangle { corner_radius } if corner_radius > 0.0 => {
                let max = rect.width().min(rect.height()) / 2.0;
                RoundedRect::from_rect(rect, (corner_radius + grow).min(max)).to_path(0.01)
            }
            ShapeKind::Rectangle { .. } | ShapeKind::Group => rect.to_path(0.01),
            ShapeKind::Ellipse => Ellipse::from_rect(rect).to_path(0.01),
        }
    }

    /// Outline in document space.
    pub fn path(&self) -> BezPath {
        self.frame.affine() * self.local_path(0.0)
    }

    /// Outline flattened to a closed polygon in document space, accurate to
    /// `tolerance` texture pixels. The last point is not repeated.
    pub fn flattened(&self, tolerance: f64) -> Vec<Point> {
        flatten_closed(self.path(), tolerance)
    }

    /// Axis-aligned bounds of the shape in document space.
    pub fn bounding_box(&self) -> Rect {
        match self.kind {
            ShapeKind::Ellipse => self.path().bounding_box(),
            ShapeKind::Rectangle { .. } => self.frame.bounding_box(),
            ShapeKind::Group => {
                let mut points = Vec::new();
                self.extent_points(&mut points);
                match points.first() {
                    Some(first) => points
                        .iter()
                        .fold(Rect::from_points(*first, *first), |r, p| r.union_pt(*p)),
                    None => self.frame.bounding_box(),
                }
            }
        }
    }

    /// Whether `point` (document space) is inside the filled shape, accepting
    /// points up to `tolerance` texture pixels outside it. For a group, whether
    /// any visible child contains it.
    pub fn contains(&self, point: Point, tolerance: f64) -> bool {
        if self.is_group() {
            return self
                .children
                .iter()
                .any(|c| c.visible && c.contains(point, tolerance));
        }
        let local = self.frame.affine().inverse() * point;
        self.local_path(tolerance.max(0.0)).contains(local)
    }
}

/// Flattens a path into the points of its first closed subpath.
pub fn flatten_closed(path: impl IntoIterator<Item = PathEl>, tolerance: f64) -> Vec<Point> {
    let mut points = Vec::new();
    kurbo::flatten(path, tolerance.max(1e-4), |el| match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => points.push(p),
        _ => {}
    });
    if points.len() > 1
        && points
            .first()
            .map(|p| p.distance(*points.last().unwrap()) < 1e-9)
            == Some(true)
    {
        points.pop();
    }
    points
}

/// Separating-axis overlap test between two convex polygons.
pub fn convex_polygons_overlap(a: &[Point], b: &[Point]) -> bool {
    fn separated(a: &[Point], b: &[Point]) -> bool {
        for i in 0..a.len() {
            let p = a[i];
            let q = a[(i + 1) % a.len()];
            let edge = q - p;
            let axis = Vec2::new(-edge.y, edge.x);
            if axis.hypot2() < 1e-18 {
                continue;
            }
            let project = |pts: &[Point]| {
                pts.iter()
                    .map(|pt| axis.dot(pt.to_vec2()))
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                        (lo.min(v), hi.max(v))
                    })
            };
            let (a_lo, a_hi) = project(a);
            let (b_lo, b_hi) = project(b);
            if a_hi < b_lo || b_hi < a_lo {
                return true;
            }
        }
        false
    }
    if a.is_empty() || b.is_empty() {
        return false;
    }
    !separated(a, b) && !separated(b, a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(kind: ShapeKind, center: (f64, f64), size: (f64, f64), rot: f64) -> Object {
        Object::new(
            ObjectId(1),
            kind,
            Frame::new(center.into(), size.into(), rot),
        )
    }

    #[test]
    fn rotated_bounds() {
        let o = object(ShapeKind::rectangle(), (0.0, 0.0), (100.0, 100.0), 45.0);
        let b = o.bounding_box();
        let half_diag = 50.0 * 2f64.sqrt();
        assert!((b.x1 - half_diag).abs() < 1e-6 && (b.y0 + half_diag).abs() < 1e-6);
    }

    #[test]
    fn rotation_is_clockwise() {
        let f = Frame::new(Point::ORIGIN, Size::new(10.0, 10.0), 90.0);
        // Local +x maps to document +y (down): clockwise on screen.
        let p = f.affine() * Point::new(1.0, 0.0);
        assert!(p.x.abs() < 1e-9 && (p.y - 1.0).abs() < 1e-9);
    }

    #[test]
    fn flattened_points_are_within_tolerance() {
        let o = object(ShapeKind::Ellipse, (0.0, 0.0), (1000.0, 1000.0), 0.0);
        let coarse = o.flattened(5.0);
        let fine = o.flattened(0.1);
        assert!(coarse.len() >= 8 && fine.len() > coarse.len());
        for p in &fine {
            assert!((p.distance(Point::ORIGIN) - 500.0).abs() < 0.2);
        }
    }

    #[test]
    fn ellipse_corner_is_not_hit() {
        let o = object(ShapeKind::Ellipse, (0.0, 0.0), (100.0, 100.0), 0.0);
        assert!(o.contains(Point::new(0.0, 0.0), 0.0));
        assert!(!o.contains(Point::new(45.0, 45.0), 0.0));
    }

    #[test]
    fn rounded_corner_is_not_hit() {
        let o = object(
            ShapeKind::Rectangle {
                corner_radius: 30.0,
            },
            (0.0, 0.0),
            (100.0, 100.0),
            0.0,
        );
        assert!(!o.contains(Point::new(49.0, 49.0), 0.0));
        assert!(o.contains(Point::new(49.0, 0.0), 0.0));
    }

    #[test]
    fn rotation_is_respected_by_hit_test() {
        let o = object(ShapeKind::rectangle(), (0.0, 0.0), (200.0, 20.0), 90.0);
        assert!(o.contains(Point::new(0.0, 90.0), 0.0));
        assert!(!o.contains(Point::new(90.0, 0.0), 0.0));
    }

    #[test]
    fn tolerance_grows_the_hit_area() {
        let o = object(ShapeKind::rectangle(), (0.0, 0.0), (2.0, 2.0), 0.0);
        assert!(!o.contains(Point::new(3.0, 0.0), 0.0));
        assert!(o.contains(Point::new(3.0, 0.0), 3.0));
    }

    #[test]
    fn minimum_size_is_enforced() {
        let o = object(ShapeKind::rectangle(), (0.0, 0.0), (0.0, -5.0), 0.0);
        assert_eq!(o.frame.size, Size::new(MIN_SIZE, MIN_SIZE));
    }

    #[test]
    fn default_names_and_flags() {
        let o = object(ShapeKind::Ellipse, (0.0, 0.0), (10.0, 10.0), 0.0);
        assert_eq!(o.name, "Ellipse");
        assert!(o.visible && !o.locked && o.children.is_empty());
        let g = Object::group(ObjectId(2), vec![Arc::new(o)]);
        assert_eq!(g.name, "Group");
    }

    #[test]
    fn group_frame_ignores_hidden_children_and_follows_rotation() {
        let a = object(ShapeKind::rectangle(), (0.0, 0.0), (100.0, 100.0), 0.0);
        let mut b = object(ShapeKind::rectangle(), (1000.0, 0.0), (100.0, 100.0), 0.0);
        b.visible = false;
        let mut g = Object::group(ObjectId(3), vec![Arc::new(a), Arc::new(b)]);
        assert_eq!(g.frame.size, Size::new(100.0, 100.0));
        g.frame.rotation_deg = 45.0;
        g.refresh_group_frame();
        let diag = 100.0 * 2f64.sqrt();
        assert!((g.frame.size.width - diag).abs() < 1e-6);
    }

    #[test]
    fn normalizes_angles() {
        assert_eq!(normalize_degrees(370.0), 10.0);
        assert_eq!(normalize_degrees(-190.0), 170.0);
        assert_eq!(normalize_degrees(180.0), 180.0);
    }

    #[test]
    fn sat_overlap() {
        let square = |x: f64| {
            vec![
                Point::new(x, 0.0),
                Point::new(x + 10.0, 0.0),
                Point::new(x + 10.0, 10.0),
                Point::new(x, 10.0),
            ]
        };
        assert!(convex_polygons_overlap(&square(0.0), &square(5.0)));
        assert!(!convex_polygons_overlap(&square(0.0), &square(20.0)));
    }
}
