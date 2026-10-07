//! Cache of flattened object outlines and tessellated meshes, so panning and
//! zooming only re-map points instead of recomputing geometry.

use std::collections::HashMap;
use std::sync::Arc;

use tp_core::document::{
    Frame, LineStyle, Object, ObjectId, PathData, ShapeKind, StrokeAlign, StrokeStyle,
    flatten_subpaths, stroke_region,
};
use tp_core::kurbo::{BezPath, Point};
use tp_text::mesh::{self, Mesh};

/// Triangles of a shape that cannot be drawn as a convex polygon (paths,
/// polygons and stars), in document space.
pub struct ShapeMesh {
    /// Filled area (closed subpaths), if any.
    pub fill: Option<Mesh>,
    /// Outline of the lines of open subpaths (stroke color), when stroked.
    pub casing: Option<Mesh>,
    /// Lines of open subpaths (fill color), narrowed by the stroke width.
    pub line: Option<Mesh>,
    /// Stroke of the closed outline, when the object has one.
    pub stroke: Option<Mesh>,
}

/// A line `width` wide along `path`: round caps for the default style,
/// the filled expanded outline otherwise.
fn line_mesh(path: &BezPath, width: f64, line: &LineStyle, tolerance: f64) -> Mesh {
    if line.is_default() {
        mesh::stroke_with_caps(path, width, tolerance, true)
    } else {
        mesh::fill(
            &stroke_region::expand(path, width, line, tolerance),
            tolerance,
        )
    }
}

/// The stroke of the outline `stroke_path` around the filled area
/// `fill_path`: a centered stroke for the default style, the stroke region
/// otherwise (falling back to the centered stroke when degenerate).
pub fn stroke_mesh(
    stroke_path: &BezPath,
    fill_path: &BezPath,
    style: &StrokeStyle,
    tolerance: f64,
) -> Mesh {
    match stroke_region(
        stroke_path,
        Some(fill_path),
        style.width,
        style.align,
        &style.line,
        tolerance,
    ) {
        Some(region) if !region.elements().is_empty() => mesh::fill(&region, tolerance),
        _ => mesh::stroke(stroke_path, style.width, tolerance),
    }
}

/// Meshes of a path or polygon at `tolerance`, drawn in field order: fill,
/// casing, line, stroke (the stroke ends up where the export draws it).
pub fn shape_mesh(object: &Object, tolerance: f64) -> ShapeMesh {
    let non_empty = |p: &BezPath| !p.elements().is_empty();
    let fill_path = object.fill_path();
    let stroke = object.stroke.filter(|s| s.width > 0.0);
    let (casing, line) = match object.line_path() {
        Some((lines, width)) => {
            let style = object
                .path
                .as_ref()
                .map(|p| p.line_style)
                .unwrap_or_default();
            let (casing, body) = match stroke {
                Some(s) => stroke_region::line_widths(width, s.width, s.align),
                None => (0.0, width),
            };
            let casing = (stroke.is_some() && casing > 0.0)
                .then(|| line_mesh(&lines, casing, &style, tolerance));
            let line = (body > 0.0).then(|| line_mesh(&lines, body, &style, tolerance));
            (casing, line)
        }
        None => (None, None),
    };
    let stroke_path = object.stroke_path();
    ShapeMesh {
        fill: non_empty(&fill_path).then(|| mesh::fill(&fill_path, tolerance)),
        casing,
        line,
        stroke: stroke
            .filter(|_| non_empty(&stroke_path))
            .map(|s| stroke_mesh(&stroke_path, &fill_path, &s, tolerance)),
    }
}

/// Geometry of one object at one zoom bucket.
pub struct Geometry {
    /// Every subpath flattened, with whether it is closed.
    pub lines: Vec<(Vec<Point>, bool)>,
    /// Set for kinds drawn from meshes.
    pub mesh: Option<ShapeMesh>,
}

impl Geometry {
    /// Points of the first subpath (the whole outline of simple shapes).
    pub fn outline(&self) -> &[Point] {
        self.lines.first().map_or(&[], |(p, _)| p.as_slice())
    }
}

/// Whether an object is drawn from meshes rather than as a convex polygon:
/// paths, polygons, shapes whose stroke is not a plain centered line, and
/// shapes painted with a gradient.
pub fn uses_mesh(object: &Object) -> bool {
    matches!(object.kind, ShapeKind::Path | ShapeKind::Polygon { .. })
        || object.fill.gradient().is_some()
        || object.stroke.is_some_and(|s| {
            s.align != StrokeAlign::Center || !s.line.is_default() || s.paint.gradient().is_some()
        })
}

/// What the cached geometry was computed from.
#[derive(PartialEq)]
struct Key {
    frame: Frame,
    kind: ShapeKind,
    /// Stroke width, alignment and line style (not the color).
    stroke: Option<(f64, StrokeAlign, LineStyle)>,
    path: Option<Arc<PathData>>,
    bucket: i32,
}

struct Entry {
    /// Address of the `Arc<Object>` the geometry was computed from.
    ptr: usize,
    key: Key,
    geometry: Arc<Geometry>,
    used: bool,
}

/// Flattened outlines and meshes in document space, keyed by object.
#[derive(Default)]
pub struct GeometryCache {
    entries: HashMap<ObjectId, Entry>,
    /// Number of geometries computed (cache misses), for tests and
    /// diagnostics.
    pub misses: u64,
}

/// Zoom bucket: one per power of two, so tolerance adapts to the zoom level
/// without recomputing on every zoom step.
pub fn zoom_bucket(screen_points_per_texture_px: f64) -> i32 {
    screen_points_per_texture_px.max(1e-6).log2().floor() as i32
}

/// Flattening tolerance (texture pixels) for a bucket: about a quarter of a
/// screen point at the bucket's highest zoom.
pub fn tolerance(bucket: i32) -> f64 {
    (0.25 / 2f64.powi(bucket + 1)).max(1e-3)
}

fn compute(object: &Object, bucket: i32) -> Geometry {
    let tol = tolerance(bucket);
    let lines = flatten_subpaths(object.path(), tol);
    let mesh = uses_mesh(object).then(|| shape_mesh(object, tol));
    Geometry { lines, mesh }
}

impl GeometryCache {
    /// Geometry of `object` for the given zoom bucket.
    pub fn geometry(&mut self, object: &Arc<Object>, bucket: i32) -> Arc<Geometry> {
        let ptr = Arc::as_ptr(object) as usize;
        let key = Key {
            frame: object.frame,
            kind: object.kind,
            stroke: object.stroke.map(|s| (s.width, s.align, s.line)),
            path: object.path.clone(),
            bucket,
        };
        if let Some(e) = self.entries.get_mut(&object.id)
            && e.ptr == ptr
            && e.key == key
        {
            e.used = true;
            return e.geometry.clone();
        }
        self.misses += 1;
        let geometry = Arc::new(compute(object, bucket));
        self.entries.insert(
            object.id,
            Entry {
                ptr,
                key,
                geometry: geometry.clone(),
                used: true,
            },
        );
        geometry
    }

    /// Drops entries not used since the previous call.
    pub fn prune(&mut self) {
        self.entries.retain(|_, e| std::mem::take(&mut e.used));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::{Node, Subpath};
    use tp_core::kurbo::Size;

    use super::*;

    fn ellipse() -> Arc<Object> {
        Arc::new(Object::new(
            ObjectId(1),
            ShapeKind::Ellipse,
            Frame::new(Point::new(100.0, 100.0), Size::new(200.0, 100.0), 0.0),
        ))
    }

    #[test]
    fn same_object_and_bucket_hits() {
        let mut cache = GeometryCache::default();
        let o = ellipse();
        cache.geometry(&o, 0);
        cache.geometry(&o, 0);
        assert_eq!(cache.misses, 1);
        cache.geometry(&o, 3);
        assert_eq!(cache.misses, 2);
        assert!(cache.geometry(&o, 3).mesh.is_none());
    }

    #[test]
    fn changed_object_misses_and_prune_drops_unused() {
        let mut cache = GeometryCache::default();
        let o = ellipse();
        cache.geometry(&o, 0);
        let mut moved = (*o).clone();
        moved.frame.center.x += 10.0;
        cache.geometry(&Arc::new(moved), 0);
        assert_eq!(cache.misses, 2);
        cache.prune();
        cache.prune();
        assert!(cache.is_empty());
    }

    #[test]
    fn path_edit_rebuilds_the_mesh() {
        let mut cache = GeometryCache::default();
        let corner = |x: f64, y: f64| Node::corner(Point::new(x, y));
        let mut o = Arc::new(Object::from_path(
            ObjectId(2),
            PathData::new(vec![
                Subpath::new(
                    vec![corner(0.0, 0.0), corner(100.0, 0.0), corner(50.0, 80.0)],
                    true,
                ),
                Subpath::new(vec![corner(200.0, 0.0), corner(300.0, 0.0)], false),
            ]),
        ));
        let g = cache.geometry(&o, 0);
        assert_eq!(g.lines.len(), 2);
        assert!(!g.lines[1].1, "the second subpath is open");
        let mesh = g.mesh.as_ref().unwrap();
        assert!(mesh.fill.is_some() && mesh.line.is_some());
        assert!(
            mesh.stroke.is_none() && mesh.casing.is_none(),
            "no stroke set"
        );
        // Same allocation, edited in place: the key still sees the change.
        Arc::make_mut(&mut o).edit_path(|p| p.subpaths[0].nodes[2].point.y = 120.0);
        cache.geometry(&o, 0);
        assert_eq!(cache.misses, 2);
    }

    #[test]
    fn stroke_alignment_rebuilds_the_mesh() {
        let mut cache = GeometryCache::default();
        let mut o = ellipse();
        Arc::make_mut(&mut o).stroke = Some(StrokeStyle::default());
        assert!(cache.geometry(&o, 0).mesh.is_none(), "plain outline");
        // Same allocation, edited in place: a color change keeps the
        // geometry, an alignment change rebuilds it.
        Arc::make_mut(&mut o).stroke.as_mut().unwrap().paint =
            tp_core::document::Rgba::rgb(255, 0, 0).into();
        cache.geometry(&o, 0);
        assert_eq!(cache.misses, 1);
        let s = Arc::make_mut(&mut o).stroke.as_mut().unwrap();
        s.align = StrokeAlign::Outside;
        s.width = 10.0;
        let outside = cache.geometry(&o, 0);
        assert_eq!(cache.misses, 2);
        let stroke = outside.mesh.as_ref().unwrap().stroke.as_ref().unwrap();
        // Every stroke vertex lies outside the ellipse (up to tolerance).
        let inside = stroke.vertices.iter().filter(|v| {
            let (x, y) = (
                (f64::from(v[0]) - 100.0) / 100.0,
                (f64::from(v[1]) - 100.0) / 50.0,
            );
            x * x + y * y < 0.99
        });
        assert_eq!(inside.count(), 0);
    }

    #[test]
    fn finer_tolerance_at_higher_zoom() {
        assert!(tolerance(zoom_bucket(8.0)) < tolerance(zoom_bucket(0.1)));
    }

    #[test]
    fn gradients_are_drawn_from_meshes() {
        use tp_core::document::{Gradient, GradientKind, Paint, Rgba};
        let mut o = (*ellipse()).clone();
        assert!(!uses_mesh(&o), "solid ellipse: a convex polygon");
        let g = Paint::Gradient(Gradient::from_color(
            GradientKind::Linear,
            Rgba::rgb(1, 2, 3),
        ));
        o.fill = g;
        assert!(uses_mesh(&o));
        o.fill = Rgba::rgb(1, 2, 3).into();
        o.stroke = Some(tp_core::document::StrokeStyle {
            paint: g,
            ..Default::default()
        });
        assert!(uses_mesh(&o));
        let mesh = compute(&Arc::new(o), 0).mesh.expect("meshes");
        assert!(mesh.fill.is_some() && mesh.stroke.is_some());
    }
}
