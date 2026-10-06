//! Vector objects, their frames and geometry.

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
    Rectangle { corner_radius: f64 },
    Ellipse,
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
        }
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
    pub frame: Frame,
    pub fill: Rgba,
    pub stroke: Option<StrokeStyle>,
    /// 0.0..=1.0
    pub opacity: f32,
}

impl Object {
    /// A shape with the default appearance.
    pub fn new(id: ObjectId, kind: ShapeKind, frame: Frame) -> Self {
        Self {
            id,
            kind,
            frame: frame.sanitized(),
            fill: DEFAULT_FILL,
            stroke: None,
            opacity: 1.0,
        }
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
            ShapeKind::Rectangle { .. } => rect.to_path(0.01),
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
        }
    }

    /// Whether `point` (document space) is inside the filled shape, accepting
    /// points up to `tolerance` texture pixels outside it.
    pub fn contains(&self, point: Point, tolerance: f64) -> bool {
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
