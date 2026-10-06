//! Vector objects, their frames and geometry.

use std::sync::Arc;

use kurbo::{Affine, BezPath, Ellipse, PathEl, Point, Rect, RoundedRect, Shape, Size, Vec2};

use super::color::{DEFAULT_FILL, Rgba};
use super::path::{Node, PathData, Subpath, polygon_points};

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
    /// A regular polygon, or a star when `star` holds the inner radius
    /// ratio (0.1..=0.9); its bounds fill the frame.
    Polygon {
        sides: u8,
        star: Option<f64>,
    },
    /// Holds `Object::path` (frame-local coordinates, see `Object::refit`).
    Path,
    /// Holds `Object::children`; its frame is derived from them.
    Group,
    /// Holds `Object::text`; its frame is the laid-out size × scale.
    Text,
    /// Draws a project asset in its frame.
    Image {
        asset: AssetId,
    },
}

/// Identifier of a project asset (imported image or SVG).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetId(pub u64);

/// Horizontal text alignment around the text anchor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// One style for a whole text object.
#[derive(Clone, Debug, PartialEq)]
pub struct CharStyle {
    pub family: String,
    /// 100..=900
    pub weight: u16,
    pub italic: bool,
    /// Font size in texture pixels.
    pub size: f64,
    pub align: TextAlign,
    /// Tracking in thousandths of an em.
    pub letter_spacing: f64,
    /// Line height in percent of the size.
    pub line_height: f64,
}

impl Default for CharStyle {
    fn default() -> Self {
        Self {
            family: "Inter".to_owned(),
            weight: 700,
            italic: false,
            size: 200.0,
            align: TextAlign::Left,
            letter_spacing: 0.0,
            line_height: 120.0,
        }
    }
}

/// Content and style of a text object.
#[derive(Clone, Debug, PartialEq)]
pub struct TextBlock {
    pub content: String,
    pub style: CharStyle,
    /// Laid-out size at scale 1, written by the text engine.
    pub layout_size: Size,
    /// Scale applied by resizing with handles.
    pub scale: Vec2,
}

impl TextBlock {
    pub fn new(content: impl Into<String>, style: CharStyle) -> Self {
        Self {
            content: content.into(),
            style,
            layout_size: Size::new(MIN_SIZE, MIN_SIZE),
            scale: Vec2::new(1.0, 1.0),
        }
    }
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
            Self::Polygon { .. } => "Polygon",
            Self::Path => "Path",
            Self::Group => "Group",
            Self::Text => "Text",
            Self::Image { .. } => "Image",
        }
    }

    pub fn is_group(self) -> bool {
        matches!(self, Self::Group)
    }

    /// Shapes that Convert to Path turns into paths.
    pub fn is_convertible(self) -> bool {
        matches!(
            self,
            Self::Rectangle { .. } | Self::Ellipse | Self::Polygon { .. }
        )
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
    /// Content and style of a text object.
    pub text: Option<TextBlock>,
    /// Geometry of a path object, in frame-local coordinates.
    pub path: Option<Arc<PathData>>,
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
            text: None,
            path: None,
        }
    }

    /// A path object from geometry given in document coordinates; the
    /// frame is fitted to it (unrotated).
    pub fn from_path(id: ObjectId, data: PathData) -> Self {
        let mut object = Self::new(
            id,
            ShapeKind::Path,
            Frame::new(Point::ORIGIN, Size::ZERO, 0.0),
        );
        object.path = Some(Arc::new(data));
        object.refit();
        object
    }

    /// Path geometry (empty for other kinds).
    pub fn path_data(&self) -> Option<&PathData> {
        self.path.as_deref()
    }

    /// Edits the path geometry, then refits the frame.
    pub fn edit_path(&mut self, f: impl FnOnce(&mut PathData)) {
        if let Some(path) = &mut self.path {
            f(Arc::make_mut(path));
            self.refit();
        }
    }

    /// Turns a rectangle, ellipse or polygon into a path that looks the
    /// same, keeping identity, name, style and frame. Returns false (and
    /// changes nothing) for other kinds.
    pub fn convert_to_path(&mut self) -> bool {
        let r = self.frame.local_rect();
        let subpath = match self.kind {
            ShapeKind::Rectangle { corner_radius } => {
                Subpath::new(rect_nodes(r, corner_radius), true)
            }
            ShapeKind::Ellipse => Subpath::new(ellipse_nodes(r), true),
            ShapeKind::Polygon { sides, star } => Subpath::new(
                polygon_points(sides, star, r)
                    .into_iter()
                    .map(Node::corner)
                    .collect(),
                true,
            ),
            _ => return false,
        };
        self.kind = ShapeKind::Path;
        self.path = Some(Arc::new(PathData::new(vec![subpath])));
        self.refit();
        true
    }

    /// Recenters a path's geometry on its exact bounds and makes the frame
    /// those bounds (in the frame's own rotation). No-op for other kinds.
    pub fn refit(&mut self) {
        let Some(path) = &mut self.path else {
            return;
        };
        let Some(bounds) = path.bounds() else {
            return;
        };
        let offset = bounds.center().to_vec2();
        if offset.hypot2() > 0.0 {
            Arc::make_mut(path).transform(Affine::translate(-offset));
            self.frame.center = self.frame.affine() * offset.to_point();
        }
        self.frame.size = Size::new(bounds.width(), bounds.height());
        self.frame = self.frame.sanitized();
    }

    /// A text object (size from `block.layout_size` × `block.scale`).
    pub fn text(id: ObjectId, block: TextBlock, center: Point) -> Self {
        let mut object = Self::new(id, ShapeKind::Text, Frame::new(center, Size::ZERO, 0.0));
        object.frame.size = Size::new(
            block.layout_size.width * block.scale.x,
            block.layout_size.height * block.scale.y,
        );
        object.frame = object.frame.sanitized();
        object.text = Some(block);
        object
    }

    /// Updates a text's scale from its frame size (after a resize).
    pub fn sync_text_scale(&mut self) {
        if let Some(block) = &mut self.text {
            let w = block.layout_size.width.max(MIN_SIZE);
            let h = block.layout_size.height.max(MIN_SIZE);
            block.scale = Vec2::new(self.frame.size.width / w, self.frame.size.height / h);
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
            ShapeKind::Rectangle { .. }
            | ShapeKind::Text
            | ShapeKind::Image { .. }
            | ShapeKind::Path => {
                out.extend(self.frame.corners());
            }
            ShapeKind::Ellipse | ShapeKind::Polygon { .. } => out.extend(self.flattened(0.5)),
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
            ShapeKind::Rectangle { .. }
            | ShapeKind::Group
            | ShapeKind::Text
            | ShapeKind::Image { .. } => rect.to_path(0.01),
            ShapeKind::Ellipse => Ellipse::from_rect(rect).to_path(0.01),
            ShapeKind::Polygon { sides, star } => {
                let mut path = BezPath::new();
                for (i, p) in polygon_points(sides, star, rect).into_iter().enumerate() {
                    if i == 0 {
                        path.move_to(p);
                    } else {
                        path.line_to(p);
                    }
                }
                path.close_path();
                path
            }
            ShapeKind::Path => self
                .path
                .as_ref()
                .map_or_else(BezPath::new, |p| p.outline()),
        }
    }

    /// Outline in document space: every subpath of a path (what the stroke
    /// follows).
    pub fn path(&self) -> BezPath {
        self.frame.affine() * self.local_path(0.0)
    }

    /// Filled area in document space: for a path, its closed subpaths only.
    pub fn fill_path(&self) -> BezPath {
        match &self.path {
            Some(p) if self.kind == ShapeKind::Path => self.frame.affine() * p.fill_outline(),
            _ => self.path(),
        }
    }

    /// Whether the outline is drawn as an open line somewhere (never filled).
    pub fn has_open_path(&self) -> bool {
        self.path.as_ref().is_some_and(|p| p.has_open())
    }

    /// Outline flattened to a closed polygon in document space, accurate to
    /// `tolerance` texture pixels. The last point is not repeated.
    pub fn flattened(&self, tolerance: f64) -> Vec<Point> {
        flatten_closed(self.path(), tolerance)
    }

    /// Axis-aligned bounds of the shape in document space.
    pub fn bounding_box(&self) -> Rect {
        match self.kind {
            ShapeKind::Ellipse | ShapeKind::Polygon { .. } | ShapeKind::Path => {
                self.path().bounding_box()
            }
            ShapeKind::Rectangle { .. } | ShapeKind::Text | ShapeKind::Image { .. } => {
                self.frame.bounding_box()
            }
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
        let tolerance = tolerance.max(0.0);
        let half_stroke = self
            .stroke
            .filter(|s| s.width > 0.0)
            .map_or(0.0, |s| s.width / 2.0);
        match self.kind {
            ShapeKind::Polygon { .. } | ShapeKind::Path => {
                let fill = match &self.path {
                    Some(p) => p.fill_outline(),
                    None => self.local_path(0.0),
                };
                if fill.contains(local) {
                    return true;
                }
                local_outline_distance(&self.local_path(0.0), local) <= half_stroke + tolerance
            }
            _ => self.local_path(tolerance + half_stroke).contains(local),
        }
    }
}

/// Handle length of a quarter circle approximated by one cubic.
const KAPPA: f64 = 0.552_284_749_830_793_4;

/// Nodes of a (rounded) rectangle, clockwise from the top edge.
fn rect_nodes(r: Rect, radius: f64) -> Vec<Node> {
    let radius = radius.clamp(0.0, r.width().min(r.height()) / 2.0);
    if radius <= 0.0 {
        return [(r.x0, r.y0), (r.x1, r.y0), (r.x1, r.y1), (r.x0, r.y1)]
            .map(|p| Node::corner(p.into()))
            .to_vec();
    }
    let k = radius * (1.0 - KAPPA);
    let node = |p: (f64, f64), hin: Option<(f64, f64)>, hout: Option<(f64, f64)>| Node {
        point: p.into(),
        handle_in: hin.map(Into::into),
        handle_out: hout.map(Into::into),
        smooth: false,
    };
    let (x0, y0, x1, y1) = (r.x0, r.y0, r.x1, r.y1);
    vec![
        node((x0 + radius, y0), Some((x0 + k, y0)), None),
        node((x1 - radius, y0), None, Some((x1 - k, y0))),
        node((x1, y0 + radius), Some((x1, y0 + k)), None),
        node((x1, y1 - radius), None, Some((x1, y1 - k))),
        node((x1 - radius, y1), Some((x1 - k, y1)), None),
        node((x0 + radius, y1), None, Some((x0 + k, y1))),
        node((x0, y1 - radius), Some((x0, y1 - k)), None),
        node((x0, y0 + radius), None, Some((x0, y0 + k))),
    ]
}

/// Four smooth nodes of the ellipse inscribed in `r`, clockwise from the top.
fn ellipse_nodes(r: Rect) -> Vec<Node> {
    let c = r.center();
    let (rx, ry) = (r.width() / 2.0, r.height() / 2.0);
    let (kx, ky) = (rx * KAPPA, ry * KAPPA);
    [
        ((0.0, -ry), (kx, 0.0)),
        ((rx, 0.0), (0.0, ky)),
        ((0.0, ry), (-kx, 0.0)),
        ((-rx, 0.0), (0.0, -ky)),
    ]
    .map(|((x, y), (hx, hy))| {
        let p = Point::new(c.x + x, c.y + y);
        Node::smooth(p, Point::new(p.x + hx, p.y + hy))
    })
    .to_vec()
}

/// Distance from `p` to the nearest point of `path`'s outline.
fn local_outline_distance(path: &BezPath, p: Point) -> f64 {
    path.segments()
        .map(|s| kurbo::ParamCurveNearest::nearest(&s, p, 1e-6).distance_sq)
        .fold(f64::INFINITY, f64::min)
        .sqrt()
}

/// Flattens every subpath of `path` into a polyline, with whether it is
/// closed. Closed polylines do not repeat their first point.
pub fn flatten_subpaths(
    path: impl IntoIterator<Item = PathEl>,
    tolerance: f64,
) -> Vec<(Vec<Point>, bool)> {
    let mut out: Vec<(Vec<Point>, bool)> = Vec::new();
    kurbo::flatten(path, tolerance.max(1e-4), |el| match el {
        PathEl::MoveTo(p) => out.push((vec![p], false)),
        PathEl::LineTo(p) => {
            if let Some((points, _)) = out.last_mut() {
                points.push(p);
            }
        }
        PathEl::ClosePath => {
            if let Some((points, closed)) = out.last_mut() {
                *closed = true;
                if points.len() > 1 && points[0].distance(*points.last().unwrap()) < 1e-9 {
                    points.pop();
                }
            }
        }
        _ => {}
    });
    out
}

/// Whether segment `a`–`b` crosses segment `c`–`d`.
pub fn segments_intersect(a: Point, b: Point, c: Point, d: Point) -> bool {
    let cross = |o: Point, p: Point, q: Point| (p - o).cross(q - o);
    let (d1, d2) = (cross(c, d, a), cross(c, d, b));
    let (d3, d4) = (cross(a, b, c), cross(a, b, d));
    (d1 * d2 <= 0.0) && (d3 * d4 <= 0.0) && !(d1 == 0.0 && d2 == 0.0 && d3 == 0.0 && d4 == 0.0)
}

impl Object {
    /// Whether the shape (filled area, stroke or open line) touches `rect`
    /// (document space). Works for concave shapes and open paths.
    pub fn touches_rect(&self, rect: Rect) -> bool {
        let rect = rect.abs();
        let lines = flatten_subpaths(self.path(), 0.5);
        if lines.iter().flat_map(|(p, _)| p).any(|p| rect.contains(*p)) {
            return true;
        }
        let corners = [
            Point::new(rect.x0, rect.y0),
            Point::new(rect.x1, rect.y0),
            Point::new(rect.x1, rect.y1),
            Point::new(rect.x0, rect.y1),
        ];
        if corners.iter().any(|c| self.contains(*c, 0.0)) {
            return true;
        }
        lines.iter().any(|(points, closed)| {
            let n = points.len();
            let edges = if *closed { n } else { n.saturating_sub(1) };
            (0..edges).any(|i| {
                let (a, b) = (points[i], points[(i + 1) % n]);
                (0..4).any(|k| segments_intersect(a, b, corners[k], corners[(k + 1) % 4]))
            })
        })
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
    fn text_frame_is_layout_times_scale() {
        let mut block = TextBlock::new("ACE", CharStyle::default());
        block.layout_size = Size::new(400.0, 240.0);
        block.scale = Vec2::new(2.0, 1.0);
        let mut t = Object::text(ObjectId(5), block, Point::new(10.0, 20.0));
        assert_eq!(t.name, "Text");
        assert_eq!(t.frame.size, Size::new(800.0, 240.0));
        // Clicking between letters hits the laid-out box.
        assert!(t.contains(Point::new(10.0, 20.0), 0.0));
        t.frame.size = Size::new(400.0, 480.0);
        t.sync_text_scale();
        assert_eq!(t.text.as_ref().unwrap().scale, Vec2::new(1.0, 2.0));
    }

    #[test]
    fn image_hits_its_frame() {
        let i = object(
            ShapeKind::Image { asset: AssetId(1) },
            (0.0, 0.0),
            (100.0, 50.0),
            0.0,
        );
        assert_eq!(i.name, "Image");
        assert!(i.contains(Point::new(49.0, 24.0), 0.0));
        assert!(!i.contains(Point::new(49.0, 26.0), 0.0));
    }

    fn polyline(points: &[(f64, f64)], closed: bool) -> Subpath {
        Subpath::new(
            points.iter().map(|p| Node::corner((*p).into())).collect(),
            closed,
        )
    }

    fn path_object(subpaths: Vec<Subpath>) -> Object {
        Object::from_path(ObjectId(7), PathData::new(subpaths))
    }

    #[test]
    fn refit_centers_geometry() {
        let o = path_object(vec![polyline(
            &[(100.0, 100.0), (500.0, 100.0), (500.0, 400.0)],
            false,
        )]);
        assert_eq!(o.name, "Path");
        assert_eq!(o.frame.center, Point::new(300.0, 250.0));
        assert_eq!(o.frame.size, Size::new(400.0, 300.0));
        let local = o.path_data().unwrap().bounds().unwrap();
        assert_eq!(local.center(), Point::ORIGIN);
        // Document geometry is unchanged.
        let doc = o.path().bounding_box();
        assert_eq!(doc, Rect::new(100.0, 100.0, 500.0, 400.0));
    }

    #[test]
    fn refit_keeps_rotation_and_fixed_points() {
        let mut o = path_object(vec![polyline(
            &[(0.0, 0.0), (100.0, 0.0), (100.0, 50.0)],
            true,
        )]);
        o.frame.rotation_deg = 30.0;
        let doc = |o: &Object, i: usize| {
            o.frame.affine() * o.path_data().unwrap().subpaths[0].nodes[i].point
        };
        let first = doc(&o, 0);
        o.edit_path(|p| p.subpaths[0].nodes[2].point.y += 50.0);
        assert_eq!(o.frame.rotation_deg, 30.0);
        assert_eq!(o.frame.size, Size::new(100.0, 100.0));
        assert!(doc(&o, 0).distance(first) < 1e-9);
        assert_eq!(
            o.path_data().unwrap().bounds().unwrap().center(),
            Point::ORIGIN
        );
    }

    #[test]
    fn curve_extrema_and_minimum_size() {
        let mut curve = Subpath::new(
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
        );
        let o = path_object(vec![curve.clone()]);
        assert!((o.frame.size.height - 75.0).abs() < 1e-6);
        curve.nodes[0].handle_out = None;
        curve.nodes[1].handle_in = None;
        let line = path_object(vec![curve]);
        assert_eq!(line.frame.size, Size::new(100.0, MIN_SIZE));
        assert_eq!(line.frame.center, Point::new(50.0, 0.0));
    }

    fn star() -> Object {
        object(
            ShapeKind::Polygon {
                sides: 5,
                star: Some(0.4),
            },
            (0.0, 0.0),
            (200.0, 200.0),
            0.0,
        )
    }

    #[test]
    fn star_notch_is_not_hit() {
        let s = star();
        assert_eq!(s.name, "Polygon");
        assert!(s.contains(Point::new(0.0, 0.0), 0.0));
        // Between the two upper points, inside the bounds.
        let top = s.bounding_box();
        assert!(!s.contains(Point::new(top.x0 + 30.0, top.y0 + 30.0), 0.0));
        assert_eq!(s.flattened(0.5).len(), 10);
    }

    #[test]
    fn hole_is_not_hit() {
        let o = path_object(vec![
            polyline(
                &[(0.0, 0.0), (300.0, 0.0), (300.0, 300.0), (0.0, 300.0)],
                true,
            ),
            polyline(
                &[
                    (100.0, 100.0),
                    (100.0, 200.0),
                    (200.0, 200.0),
                    (200.0, 100.0),
                ],
                true,
            ),
        ]);
        assert!(o.contains(Point::new(50.0, 50.0), 0.0));
        assert!(!o.contains(Point::new(150.0, 150.0), 0.0));
    }

    #[test]
    fn open_path_hits_its_stroke_only() {
        let mut v = path_object(vec![polyline(
            &[(0.0, 0.0), (100.0, 200.0), (200.0, 0.0)],
            false,
        )]);
        v.stroke = Some(StrokeStyle {
            color: Rgba::rgb(0, 0, 0),
            width: 20.0,
        });
        assert!(v.contains(Point::new(100.0, 195.0), 0.0));
        assert!(v.contains(Point::new(52.0, 100.0), 0.0));
        assert!(
            !v.contains(Point::new(100.0, 100.0), 0.0),
            "between the arms"
        );
        assert!(v.has_open_path());
        assert!(v.fill_path().elements().is_empty());
    }

    #[test]
    fn thin_line_hit_within_tolerance() {
        let mut line = path_object(vec![polyline(&[(0.0, 0.0), (100.0, 0.0)], false)]);
        line.stroke = Some(StrokeStyle {
            color: Rgba::rgb(0, 0, 0),
            width: 1.0,
        });
        assert!(!line.contains(Point::new(50.0, 2.0), 0.0));
        assert!(line.contains(Point::new(50.0, 2.0), 2.0));
    }

    #[test]
    fn marquee_touches_concave_shapes() {
        let s = star();
        let b = s.bounding_box();
        // A small rect in the notch between two points does not touch.
        assert!(!s.touches_rect(Rect::new(
            b.x0 + 20.0,
            b.y0 + 20.0,
            b.x0 + 35.0,
            b.y0 + 35.0
        )));
        // Crossing an arm touches.
        assert!(s.touches_rect(Rect::new(-5.0, b.y0 - 10.0, 5.0, b.y0 + 10.0)));
        // Fully inside touches.
        assert!(s.touches_rect(Rect::new(-1.0, -1.0, 1.0, 1.0)));
        // Enclosing touches.
        assert!(s.touches_rect(b.inflate(10.0, 10.0)));
        let v = path_object(vec![polyline(
            &[(0.0, 0.0), (100.0, 200.0), (200.0, 0.0)],
            false,
        )]);
        assert!(!v.touches_rect(Rect::new(90.0, 90.0, 110.0, 110.0)));
        assert!(v.touches_rect(Rect::new(40.0, 90.0, 60.0, 110.0)));
    }

    fn assert_same_outline(converted: &Object, original: &Object) {
        let size = original.frame.size.width.max(original.frame.size.height);
        let reference = original.path();
        for p in flatten_closed(converted.path(), 0.05) {
            let d = local_outline_distance(&reference, p);
            assert!(d <= 0.0005 * size, "{d} > {}", 0.0005 * size);
        }
        assert_eq!(converted.frame.center, original.frame.center);
        assert_eq!(converted.frame.rotation_deg, original.frame.rotation_deg);
        assert!((converted.frame.size.width - original.frame.size.width).abs() < 1e-6);
    }

    #[test]
    fn convert_to_path_keeps_the_outline() {
        let mut shapes = vec![
            object(ShapeKind::rectangle(), (100.0, 100.0), (400.0, 200.0), 20.0),
            object(
                ShapeKind::Rectangle {
                    corner_radius: 40.0,
                },
                (0.0, 0.0),
                (400.0, 200.0),
                0.0,
            ),
            object(ShapeKind::Ellipse, (50.0, 50.0), (600.0, 300.0), -45.0),
            star(),
        ];
        shapes[1].name = "Badge".into();
        shapes[1].fill = Rgba::rgb(1, 2, 3);
        for original in shapes {
            let mut converted = original.clone();
            assert!(converted.convert_to_path());
            assert_eq!(converted.kind, ShapeKind::Path);
            assert_eq!(converted.id, original.id);
            assert_eq!(converted.name, original.name);
            assert_eq!(converted.fill, original.fill);
            assert!(!converted.has_open_path());
            assert_same_outline(&converted, &original);
        }
        let mut t = Object::new(
            ObjectId(1),
            ShapeKind::Text,
            Frame::from_rect(Rect::new(0.0, 0.0, 9.0, 9.0)),
        );
        assert!(!t.convert_to_path());
        assert_eq!(t.kind, ShapeKind::Text);
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
