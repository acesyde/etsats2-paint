//! Cache of flattened object outlines and tessellated meshes, so panning and
//! zooming only re-map points instead of recomputing geometry.

use std::collections::HashMap;
use std::sync::Arc;

use tp_core::document::{Frame, Object, ObjectId, PathData, ShapeKind, flatten_subpaths};
use tp_core::kurbo::Point;
use tp_text::mesh::{self, Mesh};

/// Triangles of a shape that cannot be drawn as a convex polygon (paths,
/// polygons and stars), in document space.
pub struct ShapeMesh {
    /// Filled area (closed subpaths), if any.
    pub fill: Option<Mesh>,
    /// Stroke, when the object has one.
    pub stroke: Option<Mesh>,
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

/// Whether a kind is drawn from meshes rather than as a convex polygon.
pub fn uses_mesh(kind: ShapeKind) -> bool {
    matches!(kind, ShapeKind::Path | ShapeKind::Polygon { .. })
}

/// What the cached geometry was computed from.
#[derive(PartialEq)]
struct Key {
    frame: Frame,
    kind: ShapeKind,
    stroke_width: Option<f64>,
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
    let mesh = uses_mesh(object.kind).then(|| {
        let fill_path = object.fill_path();
        let fill = (!fill_path.elements().is_empty()).then(|| mesh::fill(&fill_path, tol));
        let stroke = object
            .stroke
            .filter(|s| s.width > 0.0)
            .map(|s| mesh::stroke_with_caps(&object.path(), s.width, tol, true));
        ShapeMesh { fill, stroke }
    });
    Geometry { lines, mesh }
}

impl GeometryCache {
    /// Geometry of `object` for the given zoom bucket.
    pub fn geometry(&mut self, object: &Arc<Object>, bucket: i32) -> Arc<Geometry> {
        let ptr = Arc::as_ptr(object) as usize;
        let key = Key {
            frame: object.frame,
            kind: object.kind,
            stroke_width: object.stroke.map(|s| s.width),
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
        assert!(mesh.fill.is_some() && mesh.stroke.is_none());
        // Same allocation, edited in place: the key still sees the change.
        Arc::make_mut(&mut o).edit_path(|p| p.subpaths[0].nodes[2].point.y = 120.0);
        cache.geometry(&o, 0);
        assert_eq!(cache.misses, 2);
    }

    #[test]
    fn finer_tolerance_at_higher_zoom() {
        assert!(tolerance(zoom_bucket(8.0)) < tolerance(zoom_bucket(0.1)));
    }
}
