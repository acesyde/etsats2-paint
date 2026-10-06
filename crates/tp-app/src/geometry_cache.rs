//! Cache of flattened object outlines, so panning and zooming only re-map
//! points instead of recomputing geometry.

use std::collections::HashMap;
use std::sync::Arc;

use tp_core::document::{Frame, Object, ObjectId, ShapeKind};
use tp_core::kurbo::Point;

struct Entry {
    /// Address of the `Arc<Object>` the points were computed from.
    ptr: usize,
    frame: Frame,
    kind: ShapeKind,
    bucket: i32,
    points: Arc<Vec<Point>>,
    used: bool,
}

/// Flattened outlines in document space, keyed by object.
#[derive(Default)]
pub struct GeometryCache {
    entries: HashMap<ObjectId, Entry>,
    /// Number of outlines computed (cache misses), for tests and diagnostics.
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

impl GeometryCache {
    /// Outline of `object` for the given zoom bucket.
    pub fn outline(&mut self, object: &Arc<Object>, bucket: i32) -> Arc<Vec<Point>> {
        let ptr = Arc::as_ptr(object) as usize;
        if let Some(e) = self.entries.get_mut(&object.id)
            && e.ptr == ptr
            && e.bucket == bucket
            && e.frame == object.frame
            && e.kind == object.kind
        {
            e.used = true;
            return e.points.clone();
        }
        self.misses += 1;
        let points = Arc::new(object.flattened(tolerance(bucket)));
        self.entries.insert(
            object.id,
            Entry {
                ptr,
                frame: object.frame,
                kind: object.kind,
                bucket,
                points: points.clone(),
                used: true,
            },
        );
        points
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
        cache.outline(&o, 0);
        cache.outline(&o, 0);
        assert_eq!(cache.misses, 1);
        cache.outline(&o, 3);
        assert_eq!(cache.misses, 2);
    }

    #[test]
    fn changed_object_misses_and_prune_drops_unused() {
        let mut cache = GeometryCache::default();
        let o = ellipse();
        cache.outline(&o, 0);
        let mut moved = (*o).clone();
        moved.frame.center.x += 10.0;
        cache.outline(&Arc::new(moved), 0);
        assert_eq!(cache.misses, 2);
        cache.prune();
        cache.prune();
        assert!(cache.is_empty());
    }

    #[test]
    fn finer_tolerance_at_higher_zoom() {
        assert!(tolerance(zoom_bucket(8.0)) < tolerance(zoom_bucket(0.1)));
    }
}
