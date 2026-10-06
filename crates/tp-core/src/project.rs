use std::sync::Arc;

use kurbo::{Point, Rect, Vec2};
use serde::{Deserialize, Serialize};

use crate::document::tree::{self, Hit, Placement};
use crate::document::{Object, ObjectId, Rgba};

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

    /// The object with `id`, anywhere in the tree.
    pub fn get(&self, id: ObjectId) -> Option<&Arc<Object>> {
        tree::get(&self.objects, id)
    }

    /// Topmost visible, unlocked object under `point` (tolerance in texture
    /// pixels): the top-level object a click selects and the innermost one.
    pub fn hit_test(&self, point: Point, tolerance: f64) -> Option<Hit> {
        tree::hit_test(&self.objects, point, tolerance)
    }

    /// Visible, unlocked top-level objects touching `rect`, bottom to top.
    pub fn objects_in_rect(&self, rect: Rect) -> Vec<ObjectId> {
        tree::top_level_in_rect(&self.objects, rect)
    }

    /// Removes the given objects (with their subtrees); returns how many.
    pub fn remove(&mut self, ids: &[ObjectId]) -> usize {
        tree::remove(&mut self.objects, ids).len()
    }

    /// Replaces objects with the same ids, anywhere in the tree.
    pub fn replace(&mut self, objects: &[Object]) {
        tree::replace(&mut self.objects, objects);
    }

    /// Moves each selected object one step up within its parent.
    pub fn bring_forward(&mut self, ids: &[ObjectId]) {
        tree::bring_forward(&mut self.objects, ids);
    }

    /// Moves each selected object one step down within its parent.
    pub fn send_backward(&mut self, ids: &[ObjectId]) {
        tree::send_backward(&mut self.objects, ids);
    }
}

/// A livery project: one or more surfaces of vector objects.
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub name: String,
    pub resolution: TextureResolution,
    pub surfaces: Vec<Surface>,
    pub active_surface: usize,
    /// Saved colors, without duplicates.
    pub palette: Vec<Rgba>,
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
            palette: Vec::new(),
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

    /// Gives `object` and all its descendants fresh ids.
    fn assign_fresh_ids(&mut self, object: &mut Object) {
        object.id = self.next_object_id();
        for child in &mut object.children {
            let mut c = (**child).clone();
            self.assign_fresh_ids(&mut c);
            *child = Arc::new(c);
        }
    }

    /// Adds an object (fresh ids) on top of the active surface's top level.
    pub fn add(&mut self, object: Object) -> ObjectId {
        self.add_to(None, object)
    }

    /// Adds an object (fresh ids) at the top of `parent` (a group) or of the
    /// top level. Falls back to the top level if `parent` is not a group.
    pub fn add_to(&mut self, parent: Option<ObjectId>, mut object: Object) -> ObjectId {
        self.assign_fresh_ids(&mut object);
        object.refresh_group_frame();
        let id = object.id;
        let objects = &mut self.surface_mut().objects;
        let parent = parent.filter(|p| tree::get(objects, *p).is_some_and(|g| g.is_group()));
        let len = match parent {
            Some(p) => tree::get(objects, p).map_or(0, |g| g.children.len()),
            None => objects.len(),
        };
        tree::insert(objects, parent, len, vec![Arc::new(object)]);
        id
    }

    /// Adds copies of `objects` offset by `offset` at the top of `parent`,
    /// keeping their relative order; returns the new ids.
    pub fn add_copies(
        &mut self,
        objects: &[Object],
        offset: Vec2,
        parent: Option<ObjectId>,
    ) -> Vec<ObjectId> {
        objects
            .iter()
            .map(|o| {
                let mut copy = o.clone();
                copy.translate_deep(offset);
                self.add_to(parent, copy)
            })
            .collect()
    }

    /// Duplicates the given objects, each copy directly above its original.
    pub fn duplicate(&mut self, ids: &[ObjectId], offset: Vec2) -> Vec<ObjectId> {
        let originals = self.selected_objects(ids);
        let mut copies = Vec::new();
        for original in originals {
            let mut copy = original.clone();
            copy.translate_deep(offset);
            self.assign_fresh_ids(&mut copy);
            let id = copy.id;
            let objects = &mut self.surface_mut().objects;
            let parent = tree::parent_of(objects, original.id).flatten();
            let index = tree::find_path(objects, original.id)
                .and_then(|p| p.last().copied())
                .map_or(0, |i| i + 1);
            tree::insert(objects, parent, index, vec![Arc::new(copy)]);
            copies.push(id);
        }
        copies
    }

    /// The given objects (normalized: no descendant of another), in paint
    /// order, cloned.
    pub fn selected_objects(&self, ids: &[ObjectId]) -> Vec<Object> {
        let objects = &self.surface().objects;
        let ids = tree::normalize_selection(objects, ids);
        tree::in_paint_order(objects, &ids)
            .into_iter()
            .filter_map(|id| tree::get(objects, id).map(|o| (**o).clone()))
            .collect()
    }

    /// Wraps the given objects in a new group; returns its id.
    pub fn group(&mut self, ids: &[ObjectId]) -> Option<ObjectId> {
        let id = self.next_object_id();
        tree::group(&mut self.surface_mut().objects, ids, id)
    }

    /// Releases the given groups' children; returns their ids.
    pub fn ungroup(&mut self, ids: &[ObjectId]) -> Vec<ObjectId> {
        tree::ungroup(&mut self.surface_mut().objects, ids)
    }

    /// Moves objects in the active surface's tree.
    pub fn move_objects(&mut self, ids: &[ObjectId], placement: Placement) -> bool {
        tree::move_to(&mut self.surface_mut().objects, ids, placement)
    }

    /// Adds a color to the palette unless already present.
    pub fn add_to_palette(&mut self, color: Rgba) -> bool {
        if self.palette.contains(&color) {
            return false;
        }
        self.palette.push(color);
        true
    }

    /// Captures the document state (cheap: objects are shared).
    pub fn snapshot(&self, selection: &[ObjectId]) -> Snapshot {
        Snapshot {
            surfaces: self.surfaces.iter().map(|s| s.objects.clone()).collect(),
            active_surface: self.active_surface,
            palette: self.palette.clone(),
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
        self.palette = snapshot.palette.clone();
        snapshot.selection.clone()
    }
}

/// Document state stored in the undo history.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    surfaces: Vec<Vec<Arc<Object>>>,
    active_surface: usize,
    palette: Vec<Rgba>,
    selection: Vec<ObjectId>,
}

impl Snapshot {
    /// Whether both snapshots hold the same document, ignoring selection.
    /// Unchanged objects share their `Arc`, so this is mostly pointer checks.
    pub fn same_document(&self, other: &Snapshot) -> bool {
        self.active_surface == other.active_surface
            && self.palette == other.palette
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
            Some(Hit { top, inner: top })
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
    fn palette_has_no_duplicates_and_is_snapshotted() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let red = Rgba::rgb(255, 0, 0);
        assert!(p.add_to_palette(red));
        assert!(!p.add_to_palette(red));
        assert_eq!(p.palette, vec![red]);
        let snap = p.snapshot(&[]);
        p.palette.clear();
        assert!(!snap.same_document(&p.snapshot(&[])));
        p.restore(&snap);
        assert_eq!(p.palette, vec![red]);
    }

    #[test]
    fn duplicate_inside_group_stays_in_group() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let g = p.group(&[a]).unwrap();
        let copies = p.duplicate(&[a], Vec2::new(20.0, 20.0));
        let group = p.surface().get(g).unwrap();
        assert_eq!(group.children.len(), 2);
        assert_eq!(group.children[1].id, copies[0]);
        assert_eq!(group.children[1].frame.center, Point::new(120.0, 120.0));
    }

    #[test]
    fn copies_of_groups_get_fresh_ids() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let b = p.add(rect_at(300.0, 100.0));
        let g = p.group(&[a, b]).unwrap();
        let copy = p.duplicate(&[g], Vec2::new(0.0, 50.0))[0];
        let copied = p.surface().get(copy).unwrap();
        assert!(copied.children.iter().all(|c| c.id != a && c.id != b));
        assert_eq!(copied.children[0].frame.center, Point::new(100.0, 150.0));
    }

    #[test]
    fn add_to_group() {
        let mut p = Project::new("p", TextureResolution::R2048);
        let a = p.add(rect_at(100.0, 100.0));
        let g = p.group(&[a]).unwrap();
        let b = p.add_to(Some(g), rect_at(500.0, 100.0));
        let group = p.surface().get(g).unwrap();
        assert_eq!(group.children.last().unwrap().id, b);
        assert_eq!(group.frame.size.width, 500.0);
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
