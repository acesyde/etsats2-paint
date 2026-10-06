use std::sync::Arc;

use kurbo::{Point, Rect, Vec2};
use serde::{Deserialize, Serialize};

use crate::document::{Object, ObjectId, convex_polygons_overlap};

/// Name given to a project created without a name.
pub const DEFAULT_PROJECT_NAME: &str = "Untitled";
/// Name of the surface every new project starts with.
pub const MAIN_SURFACE_NAME: &str = "Main texture";

/// Square texture resolution a livery is authored for.
///
/// The document itself stays vector-based; the resolution is the nominal
/// texture size used for coordinates and default export.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureResolution {
    R2048,
    #[default]
    R4096,
    R8192,
}

impl TextureResolution {
    pub const ALL: [Self; 3] = [Self::R2048, Self::R4096, Self::R8192];

    /// Side length in pixels.
    pub fn side(self) -> u32 {
        match self {
            Self::R2048 => 2048,
            Self::R4096 => 4096,
            Self::R8192 => 8192,
        }
    }

    /// Human-readable label, e.g. `4096 × 4096`.
    pub fn label(self) -> String {
        let side = self.side();
        format!("{side} × {side}")
    }
}

/// One texture of the vehicle: a square artboard holding ordered objects
/// (later objects are drawn on top).
#[derive(Clone, Debug, PartialEq)]
pub struct Surface {
    pub name: String,
    /// Side length in texture pixels.
    pub size: f64,
    pub objects: Vec<Arc<Object>>,
}

impl Surface {
    pub fn new(name: impl Into<String>, size: f64) -> Self {
        Self {
            name: name.into(),
            size,
            objects: Vec::new(),
        }
    }

    /// The artboard rectangle in texture pixels.
    pub fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, self.size, self.size)
    }

    pub fn index_of(&self, id: ObjectId) -> Option<usize> {
        self.objects.iter().position(|o| o.id == id)
    }

    pub fn get(&self, id: ObjectId) -> Option<&Arc<Object>> {
        self.objects.iter().find(|o| o.id == id)
    }

    /// Topmost object containing `point`, with a tolerance in texture pixels.
    pub fn hit_test(&self, point: Point, tolerance: f64) -> Option<ObjectId> {
        self.objects
            .iter()
            .rev()
            .find(|o| o.contains(point, tolerance))
            .map(|o| o.id)
    }

    /// Objects touching `rect`, bottom to top.
    pub fn objects_in_rect(&self, rect: Rect) -> Vec<ObjectId> {
        let rect = rect.abs();
        let corners = [
            Point::new(rect.x0, rect.y0),
            Point::new(rect.x1, rect.y0),
            Point::new(rect.x1, rect.y1),
            Point::new(rect.x0, rect.y1),
        ];
        self.objects
            .iter()
            .filter(|o| {
                rect_touches(o.bounding_box(), rect)
                    && convex_polygons_overlap(&o.flattened(0.5), &corners)
            })
            .map(|o| o.id)
            .collect()
    }

    /// Removes the given objects; returns how many were removed.
    pub fn remove(&mut self, ids: &[ObjectId]) -> usize {
        let before = self.objects.len();
        self.objects.retain(|o| !ids.contains(&o.id));
        before - self.objects.len()
    }

    /// Replaces objects with the same ids, keeping their stacking position.
    pub fn replace(&mut self, objects: &[Object]) {
        for object in objects {
            if let Some(i) = self.index_of(object.id) {
                self.objects[i] = Arc::new(object.clone());
            }
        }
    }

    /// Moves each selected object one step up past the next unselected one.
    pub fn bring_forward(&mut self, ids: &[ObjectId]) {
        let n = self.objects.len();
        for i in (0..n.saturating_sub(1)).rev() {
            if ids.contains(&self.objects[i].id) && !ids.contains(&self.objects[i + 1].id) {
                self.objects.swap(i, i + 1);
            }
        }
    }

    /// Moves each selected object one step down past the previous unselected one.
    pub fn send_backward(&mut self, ids: &[ObjectId]) {
        for i in 1..self.objects.len() {
            if ids.contains(&self.objects[i].id) && !ids.contains(&self.objects[i - 1].id) {
                self.objects.swap(i, i - 1);
            }
        }
    }
}

fn rect_touches(a: Rect, b: Rect) -> bool {
    a.x0 <= b.x1 && b.x0 <= a.x1 && a.y0 <= b.y1 && b.y0 <= a.y1
}

/// A livery project: one or more surfaces of vector objects.
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub name: String,
    pub resolution: TextureResolution,
    pub surfaces: Vec<Surface>,
    pub active_surface: usize,
    next_id: u64,
}

impl Project {
    /// Creates a project with one empty surface, trimming the name and
    /// falling back to [`DEFAULT_PROJECT_NAME`] when it is empty.
    pub fn new(name: &str, resolution: TextureResolution) -> Self {
        let trimmed = name.trim();
        let name = if trimmed.is_empty() {
            DEFAULT_PROJECT_NAME.to_owned()
        } else {
            trimmed.to_owned()
        };
        Self {
            name,
            resolution,
            surfaces: vec![Surface::new(
                MAIN_SURFACE_NAME,
                f64::from(resolution.side()),
            )],
            active_surface: 0,
            next_id: 1,
        }
    }

    /// Texture size in pixels (width, height).
    pub fn texture_size(&self) -> (u32, u32) {
        let side = self.resolution.side();
        (side, side)
    }

    pub fn surface(&self) -> &Surface {
        &self.surfaces[self.active_surface]
    }

    pub fn surface_mut(&mut self) -> &mut Surface {
        &mut self.surfaces[self.active_surface]
    }

    /// A new, never-used object id.
    pub fn next_object_id(&mut self) -> ObjectId {
        let id = ObjectId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Adds an object on top of the active surface (its id is replaced by a
    /// fresh one) and returns the id.
    pub fn add(&mut self, mut object: Object) -> ObjectId {
        object.id = self.next_object_id();
        let id = object.id;
        self.surface_mut().objects.push(Arc::new(object));
        id
    }

    /// Adds copies of `objects` offset by `offset`, on top, keeping their
    /// relative order; returns the new ids.
    pub fn add_copies(&mut self, objects: &[Object], offset: Vec2) -> Vec<ObjectId> {
        objects
            .iter()
            .map(|o| {
                let mut copy = o.clone();
                copy.frame.center += offset;
                self.add(copy)
            })
            .collect()
    }

    /// Duplicates the given objects (bottom-to-top order preserved).
    pub fn duplicate(&mut self, ids: &[ObjectId], offset: Vec2) -> Vec<ObjectId> {
        let originals = self.selected_objects(ids);
        self.add_copies(&originals, offset)
    }

    /// The given objects in stacking order (bottom to top), cloned.
    pub fn selected_objects(&self, ids: &[ObjectId]) -> Vec<Object> {
        self.surface()
            .objects
            .iter()
            .filter(|o| ids.contains(&o.id))
            .map(|o| (**o).clone())
            .collect()
    }

    /// Captures the document state (cheap: objects are shared).
    pub fn snapshot(&self, selection: &[ObjectId]) -> Snapshot {
        Snapshot {
            surfaces: self.surfaces.iter().map(|s| s.objects.clone()).collect(),
            active_surface: self.active_surface,
            selection: selection.to_vec(),
        }
    }

    /// Restores a snapshot and returns its selection. Ids stay unique because
    /// the id counter is never rewound.
    pub fn restore(&mut self, snapshot: &Snapshot) -> Vec<ObjectId> {
        for (surface, objects) in self.surfaces.iter_mut().zip(&snapshot.surfaces) {
            surface.objects = objects.clone();
        }
        self.active_surface = snapshot.active_surface.min(self.surfaces.len() - 1);
        snapshot.selection.clone()
    }
}

/// Document state stored in the undo history.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    surfaces: Vec<Vec<Arc<Object>>>,
    active_surface: usize,
    selection: Vec<ObjectId>,
}

impl Snapshot {
    /// Whether both snapshots hold the same document, ignoring selection.
    /// Unchanged objects share their `Arc`, so this is mostly pointer checks.
    pub fn same_document(&self, other: &Snapshot) -> bool {
        self.active_surface == other.active_surface
            && self.surfaces.len() == other.surfaces.len()
            && self.surfaces.iter().zip(&other.surfaces).all(|(a, b)| {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| Arc::ptr_eq(x, y) || x == y)
            })
    }
}

#[cfg(test)]
mod tests {
    use kurbo::Size;

    use super::*;
    use crate::document::{Frame, ShapeKind};

    fn rect_at(x: f64, y: f64) -> Object {
        Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(Point::new(x, y), Size::new(100.0, 100.0), 0.0),
        )
    }

    #[test]
    fn new_project_has_main_surface() {
        let p = Project::new("  ", TextureResolution::R4096);
        assert_eq!(p.name, DEFAULT_PROJECT_NAME);
        assert_eq!(p.surfaces.len(), 1);
        assert_eq!(p.surface().name, MAIN_SURFACE_NAME);
        assert_eq!(p.surface().bounds(), Rect::new(0.0, 0.0, 4096.0, 4096.0));
    }

    #[test]
    fn name_is_trimmed() {
        let p = Project::new("  ACE Logistics ", TextureResolution::R2048);
        assert_eq!(p.name, "ACE Logistics");
        assert_eq!(p.texture_size(), (2048, 2048));
    }

    #[test]
    fn resolution_labels() {
        assert_eq!(TextureResolution::R2048.label(), "2048 × 2048");
        assert_eq!(TextureResolution::default().side(), 4096);
    }

    #[test]
    fn ids_are_unique_and_stable() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let b = p.add(rect_at(150.0, 150.0));
        assert_ne!(a, b);
        p.surface_mut().bring_forward(&[a]);
        let mut moved = (**p.surface().get(a).unwrap()).clone();
        moved.frame.center = Point::new(500.0, 500.0);
        p.surface_mut().replace(&[moved]);
        assert!(p.surface().get(a).is_some());
        assert_eq!(p.surface().index_of(a), Some(1));
    }

    #[test]
    fn topmost_object_wins() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let _bottom = p.add(rect_at(100.0, 100.0));
        let top = p.add(rect_at(120.0, 120.0));
        assert_eq!(
            p.surface().hit_test(Point::new(110.0, 110.0), 0.0),
            Some(top)
        );
        assert_eq!(p.surface().hit_test(Point::new(1000.0, 1000.0), 0.0), None);
    }

    #[test]
    fn marquee_selects_touching_objects() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let b = p.add(rect_at(300.0, 100.0));
        let _c = p.add(rect_at(800.0, 800.0));
        let hits = p
            .surface()
            .objects_in_rect(Rect::new(140.0, 90.0, 260.0, 110.0));
        assert_eq!(hits, vec![a, b]);
    }

    #[test]
    fn marquee_misses_ellipse_corner() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let mut e = rect_at(100.0, 100.0);
        e.kind = ShapeKind::Ellipse;
        p.add(e);
        // Inside the bounding box corner, outside the ellipse.
        assert!(
            p.surface()
                .objects_in_rect(Rect::new(52.0, 52.0, 58.0, 58.0))
                .is_empty()
        );
    }

    #[test]
    fn duplicate_offsets_and_stacks_on_top() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let copies = p.duplicate(&[a], Vec2::new(20.0, 20.0));
        assert_eq!(copies.len(), 1);
        let copy = p.surface().get(copies[0]).unwrap();
        assert_eq!(copy.frame.center, Point::new(120.0, 120.0));
        assert_eq!(p.surface().index_of(copies[0]), Some(1));
    }

    #[test]
    fn reorder_steps() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(0.0, 0.0));
        let b = p.add(rect_at(0.0, 0.0));
        let c = p.add(rect_at(0.0, 0.0));
        p.surface_mut().bring_forward(&[a]);
        let order: Vec<_> = p.surface().objects.iter().map(|o| o.id).collect();
        assert_eq!(order, vec![b, a, c]);
        p.surface_mut().send_backward(&[c]);
        let order: Vec<_> = p.surface().objects.iter().map(|o| o.id).collect();
        assert_eq!(order, vec![b, c, a]);
    }

    #[test]
    fn remove_and_restore() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(0.0, 0.0));
        let snap = p.snapshot(&[a]);
        assert_eq!(p.surface_mut().remove(&[a]), 1);
        assert!(p.surface().objects.is_empty());
        assert_eq!(p.restore(&snap), vec![a]);
        assert_eq!(p.surface().objects.len(), 1);
        // The id counter is not rewound.
        assert_ne!(p.next_object_id(), a);
    }
}
