//! Pen paths being drawn, lines and polygon settings for new shapes, and
//! point editing with the Direct Selection tool.

use std::collections::BTreeSet;
use std::sync::Arc;

use tp_core::Snapshot;
use tp_core::document::{
    HandleSide, Node, NodeRef, Object, ObjectId, PathData, PointRef, ShapeKind, Subpath,
    snap_direction,
};
use tp_core::kurbo::{Affine, Point, Rect, Vec2};

use crate::workspace::Workspace;

/// Sides and star settings for new polygons (set from the Properties panel).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolygonStyle {
    pub sides: u8,
    pub star: bool,
    /// Inner radius of stars, 0.1..=0.9.
    pub inner: f64,
}

impl Default for PolygonStyle {
    fn default() -> Self {
        Self {
            sides: 6,
            star: false,
            inner: 0.5,
        }
    }
}

impl PolygonStyle {
    pub fn kind(&self) -> ShapeKind {
        ShapeKind::Polygon {
            sides: self.sides.clamp(3, 12),
            star: self.star.then_some(self.inner.clamp(0.1, 0.9)),
        }
    }
}

/// A path being drawn with the Pen tool (document coordinates). It is not
/// part of the document until finished.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PenSession {
    pub nodes: Vec<Node>,
}

/// Point `p` constrained to 45° steps from `from`.
pub fn constrain(from: Point, p: Point) -> Point {
    from + snap_direction(p - from)
}

impl Workspace {
    pub fn is_drawing_pen(&self) -> bool {
        self.pen.is_some()
    }

    /// A path object with the current style; its open subpaths use the
    /// line width last set.
    pub fn styled_path(&self, mut data: PathData, name: &str) -> Object {
        data.line_width = self.line_width;
        let mut object = Object::from_path(ObjectId(0), data);
        object.name = name.to_owned();
        object.fill = self.style.fill;
        object.stroke = self.style.stroke();
        object
    }

    /// Whether `doc` is over the first point of the pen path (to close it).
    pub fn pen_over_first(&self, doc: Point, radius: f64) -> bool {
        self.pen
            .as_ref()
            .is_some_and(|s| s.nodes.len() >= 3 && s.nodes[0].point.distance(doc) <= radius)
    }

    /// Adds a point at `doc` (Shift: 45° from the previous point). Starts a
    /// path when none is being drawn.
    pub fn pen_add(&mut self, doc: Point, shift: bool) {
        let session = self.pen.get_or_insert_with(PenSession::default);
        let at = match session.nodes.last() {
            Some(last) if shift => constrain(last.point, doc),
            _ => doc,
        };
        session.nodes.push(Node::corner(at));
    }

    /// Drags the outgoing handle of the last point (the incoming one
    /// mirrors it): makes it a smooth point.
    pub fn pen_drag_handle(&mut self, doc: Point, shift: bool) {
        let Some(node) = self.pen.as_mut().and_then(|s| s.nodes.last_mut()) else {
            return;
        };
        let handle = if shift {
            constrain(node.point, doc)
        } else {
            doc
        };
        if (handle - node.point).hypot() < 1e-9 {
            *node = Node::corner(node.point);
        } else {
            *node = Node::smooth(node.point, handle);
        }
    }

    /// Removes the last point; removing the only point discards the path.
    pub fn pen_pop(&mut self) {
        if let Some(session) = &mut self.pen {
            session.nodes.pop();
            if session.nodes.is_empty() {
                self.pen = None;
            }
        }
    }

    /// Finishes the pen path, open or `closed`. Fewer than 2 points: nothing
    /// is created.
    pub fn finish_pen(&mut self, closed: bool, now: f64) -> Option<ObjectId> {
        let session = self.pen.take()?;
        if session.nodes.len() < 2 {
            return None;
        }
        let closed = closed && session.nodes.len() >= 3;
        let data = PathData::new(vec![Subpath::new(session.nodes, closed)]);
        let object = self.styled_path(data, "Path");
        Some(self.create_object(object, now))
    }

    /// Creates a line from `a` to `b` with the current style.
    /// The line's rotation is its own angle, so its frame follows it.
    pub fn create_line(&mut self, a: Point, b: Point, now: f64) -> ObjectId {
        let data = PathData::new(vec![Subpath::new(
            vec![Node::corner(a), Node::corner(b)],
            false,
        )]);
        let mut object = self.styled_path(data, "Line");
        object.set_path_rotation(line_angle(a, b));
        self.create_object(object, now)
    }
}

/// What a press with the Direct Selection tool grabs, in priority order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointTarget {
    Handle(PointRef, HandleSide),
    Point(PointRef),
    /// A segment of a selected path: object, subpath, segment, parameter.
    Segment(ObjectId, usize, usize, f64),
}

/// Point and handle drags with the Direct Selection tool, computed from the
/// objects as they were when the drag started.
#[derive(Clone, Debug)]
pub struct PointDrag {
    pub start: Point,
    /// Document position of the point grabbed, which is the one snapped.
    pub grabbed: Option<Point>,
    pub originals: Vec<Arc<Object>>,
    pub before: Snapshot,
}

/// Paths a point may belong to: visible, unlocked, at any depth.
fn editable_paths(list: &[Arc<Object>], out: &mut Vec<Arc<Object>>) {
    for o in list.iter().filter(|o| o.visible && !o.locked) {
        if o.is_group() {
            editable_paths(&o.children, out);
        } else if o.kind == ShapeKind::Path {
            out.push(o.clone());
        }
    }
}

/// Document → local vector for an object (rotation only).
fn local_vector(o: &Object, v: Vec2) -> Vec2 {
    Affine::rotate(-o.frame.rotation_deg.to_radians()) * v.to_point() - Point::ORIGIN
}

impl Workspace {
    /// Selected, editable path objects.
    pub fn selected_paths(&self) -> Vec<Arc<Object>> {
        let mut all = Vec::new();
        editable_paths(&self.project.surface().objects, &mut all);
        all.retain(|o| self.selection.contains(&o.id));
        all
    }

    /// Handles shown for a path: those of selected points and of the points
    /// next to them.
    pub fn shown_handles(&self, o: &Object) -> Vec<(PointRef, HandleSide, Point)> {
        let Some(path) = o.path_data() else {
            return Vec::new();
        };
        let affine = o.frame.affine();
        let mut out = Vec::new();
        for (s, sub) in path.subpaths.iter().enumerate() {
            let n = sub.nodes.len();
            for (i, node) in sub.nodes.iter().enumerate() {
                let selected = |j: usize| {
                    self.points
                        .contains(&PointRef::new(o.id, NodeRef::new(s, j)))
                };
                let near_selected = selected(i)
                    || (i + 1 < n || sub.closed) && selected((i + 1) % n)
                    || (i > 0 || sub.closed) && selected((i + n - 1) % n);
                if !near_selected {
                    continue;
                }
                let r = PointRef::new(o.id, NodeRef::new(s, i));
                if let Some(h) = node.handle_in {
                    out.push((r, HandleSide::In, affine * h));
                }
                if let Some(h) = node.handle_out {
                    out.push((r, HandleSide::Out, affine * h));
                }
            }
        }
        out
    }

    /// What a press at `doc` grabs among the selected paths (`radius` in
    /// document units).
    pub fn point_target(&self, doc: Point, radius: f64) -> Option<PointTarget> {
        let paths = self.selected_paths();
        let nearest = |items: Vec<(f64, PointTarget)>| {
            items
                .into_iter()
                .filter(|(d, _)| *d <= radius)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, t)| t)
        };
        let points = paths.iter().flat_map(|o| {
            let a = o.frame.affine();
            o.path_data().map_or_else(Vec::new, |p| {
                p.node_refs()
                    .into_iter()
                    .map(|r| {
                        let at = a * p.node(r).expect("ref").point;
                        (at.distance(doc), PointTarget::Point(PointRef::new(o.id, r)))
                    })
                    .collect()
            })
        });
        if let Some(t) = nearest(points.collect()) {
            return Some(t);
        }
        let handles = paths
            .iter()
            .flat_map(|o| self.shown_handles(o))
            .map(|(r, side, at)| (at.distance(doc), PointTarget::Handle(r, side)));
        if let Some(t) = nearest(handles.collect()) {
            return Some(t);
        }
        let segments = paths.iter().filter_map(|o| {
            let local = o.frame.affine().inverse() * doc;
            let hit = o.path_data()?.nearest_segment(local)?;
            Some((
                hit.distance,
                PointTarget::Segment(o.id, hit.subpath, hit.segment, hit.t),
            ))
        });
        nearest(segments.collect())
    }

    /// Keeps only points of selected paths that still exist.
    pub fn prune_points(&mut self) {
        if self.points.is_empty() {
            return;
        }
        let surface = self.project.surface();
        let selection = &self.selection;
        self.points.retain(|r| {
            selection.contains(&r.object)
                && surface
                    .get(r.object)
                    .and_then(|o| o.path_data())
                    .is_some_and(|p| p.node(r.node).is_some())
        });
    }

    /// Selects one point alone (and its path), or toggles it with Shift.
    pub fn click_point(&mut self, r: PointRef, shift: bool) {
        if shift {
            if !self.points.remove(&r) {
                self.points.insert(r);
            }
        } else {
            self.points = BTreeSet::from([r]);
        }
        if !self.selection.contains(&r.object) {
            if !shift {
                self.selection.clear();
            }
            self.selection.push(r.object);
        }
    }

    /// Escape with Direct Selection: points first, then objects.
    pub fn clear_points_or_selection(&mut self) {
        if self.points.is_empty() {
            self.selection.clear();
        } else {
            self.points.clear();
        }
    }

    /// Starts a point or handle drag.
    pub fn point_drag(&self, start: Point) -> PointDrag {
        PointDrag {
            start,
            grabbed: None,
            originals: self.selected_paths(),
            before: self.snapshot(),
        }
    }

    /// Moves the selected points by the drag from `drag.start` to `doc`.
    pub fn drag_points(&mut self, drag: &PointDrag, doc: Point, shift: bool) {
        let delta = if shift {
            snap_direction(doc - drag.start)
        } else {
            doc - drag.start
        };
        let moved: Vec<Object> = drag
            .originals
            .iter()
            .filter_map(|o| {
                let refs: Vec<NodeRef> = self
                    .points
                    .iter()
                    .filter(|r| r.object == o.id)
                    .map(|r| r.node)
                    .collect();
                if refs.is_empty() {
                    return None;
                }
                let mut o = (**o).clone();
                let local = local_vector(&o, delta);
                o.edit_path(|p| p.move_nodes(&refs, local));
                Some(o)
            })
            .collect();
        self.project.surface_mut().replace(&moved);
    }

    /// Moves one handle to `doc`; `broken` (Alt) moves it alone and makes
    /// the point a corner.
    pub fn drag_handle(
        &mut self,
        drag: &PointDrag,
        r: PointRef,
        side: HandleSide,
        doc: Point,
        broken: bool,
    ) {
        let Some(original) = drag.originals.iter().find(|o| o.id == r.object) else {
            return;
        };
        let mut o = (**original).clone();
        let local = o.frame.affine().inverse() * doc;
        o.edit_path(|p| p.set_handle(r.node, side, local, broken));
        self.project.surface_mut().replace(&[o]);
    }

    /// Records a finished point or handle drag.
    pub fn finish_point_drag(&mut self, drag: PointDrag, now: f64) {
        self.record("Move Points", drag.before, now, false);
    }

    /// Selects the points of editable paths inside `rect` (added to `base`),
    /// and their paths.
    pub fn marquee_points(
        &mut self,
        rect: Rect,
        base: &BTreeSet<PointRef>,
        base_objects: &[ObjectId],
    ) {
        let rect = rect.abs();
        let mut paths = Vec::new();
        editable_paths(&self.project.surface().objects, &mut paths);
        let mut points = base.clone();
        let mut selection = base_objects.to_vec();
        for o in paths {
            let a = o.frame.affine();
            let Some(data) = o.path_data() else {
                continue;
            };
            for r in data.node_refs() {
                if rect.contains(a * data.node(r).expect("ref").point) {
                    points.insert(PointRef::new(o.id, r));
                    if !selection.contains(&o.id) {
                        selection.push(o.id);
                    }
                }
            }
        }
        self.points = points;
        self.selection = selection;
    }

    /// Applies `f` to the path of `id` as one undo step.
    fn edit_one_path(
        &mut self,
        label: &'static str,
        id: ObjectId,
        now: f64,
        f: impl FnOnce(&mut PathData),
    ) {
        let Some(mut o) = self.project.surface().get(id).map(|o| (**o).clone()) else {
            return;
        };
        let before = self.snapshot();
        o.edit_path(f);
        self.project.surface_mut().replace(&[o]);
        self.record(label, before, now, false);
    }

    /// Toggles a point between corner and smooth.
    pub fn toggle_point(&mut self, r: PointRef, now: f64) {
        self.edit_one_path("Change Point", r.object, now, |p| p.toggle_smooth(r.node));
    }

    /// Inserts a point on a segment and selects it.
    pub fn insert_point(&mut self, id: ObjectId, subpath: usize, segment: usize, t: f64, now: f64) {
        let mut inserted = None;
        let before = self.snapshot();
        if let Some(mut o) = self.project.surface().get(id).map(|o| (**o).clone()) {
            o.edit_path(|p| inserted = p.insert_at(subpath, segment, t));
            self.project.surface_mut().replace(&[o]);
        }
        if let Some(node) = inserted {
            self.points = BTreeSet::from([PointRef::new(id, node)]);
        }
        self.record("Add Point", before, now, false);
    }

    /// Deletes the selected points; paths left empty are deleted.
    pub fn delete_points(&mut self, now: f64) {
        let points = std::mem::take(&mut self.points);
        let before = self.snapshot().with_points(points.iter().copied());
        let mut emptied = Vec::new();
        let mut edited = Vec::new();
        for id in points.iter().map(|r| r.object).collect::<BTreeSet<_>>() {
            let Some(mut o) = self.project.surface().get(id).map(|o| (**o).clone()) else {
                continue;
            };
            let refs: Vec<NodeRef> = points
                .iter()
                .filter(|r| r.object == id)
                .map(|r| r.node)
                .collect();
            o.edit_path(|p| p.delete_nodes(&refs));
            if o.path_data().is_none_or(PathData::is_empty) {
                emptied.push(id);
            } else {
                edited.push(o);
            }
        }
        let surface = self.project.surface_mut();
        surface.replace(&edited);
        surface.remove(&emptied);
        self.selection.retain(|id| !emptied.contains(id));
        self.record("Delete Points", before, now, false);
    }

    /// Nudges the selected points (consecutive nudges are one step).
    pub fn nudge_points(&mut self, dx: f64, dy: f64, now: f64) {
        let drag = self.point_drag(Point::ORIGIN);
        let before = drag.before.clone();
        self.drag_points(&drag, Point::new(dx, dy), false);
        self.record("Nudge", before, now, true);
    }

    /// Converts the selected rectangles, ellipses and polygons (also inside
    /// selected groups) to paths.
    pub fn convert_selection_to_path(&mut self, now: f64) {
        let converted: Vec<Object> = self
            .selected_objects()
            .into_iter()
            .map(|mut o| {
                o.for_each_shape(&mut |shape| {
                    shape.convert_to_path();
                });
                o
            })
            .collect();
        self.edit("Convert to Path", now, false, |project, _| {
            project.surface_mut().replace(&converted);
        });
    }

    /// Whether the selection holds a shape Convert to Path can turn into a
    /// path.
    pub fn selection_has_convertible(&self) -> bool {
        self.selected_objects()
            .iter()
            .any(|o| o.shapes().iter().any(|s| s.kind.is_convertible()))
    }
}

/// Keys while a pen path is drawn: Enter or Escape finishes it (open),
/// Backspace or Delete removes the last point. Consumed so they do not
/// trigger commands.
pub fn handle_pen_keys(ctx: &egui::Context, ws: &mut Workspace, now: f64) {
    if ws.pen.is_none() || ws.gesture.is_active() {
        return;
    }
    let pressed = |i: &mut egui::InputState, key| i.consume_key(egui::Modifiers::NONE, key);
    let (finish, pop) = ctx.input_mut(|i| {
        let finish = pressed(i, egui::Key::Enter) | pressed(i, egui::Key::Escape);
        let pop = pressed(i, egui::Key::Backspace) | pressed(i, egui::Key::Delete);
        (finish, pop)
    });
    if pop {
        ws.pen_pop();
    }
    if finish {
        ws.finish_pen(false, now);
    }
}

/// Angle of the line from `a` to `b` in degrees, clockwise, folded into
/// (−90°, 90°] so that a line drawn right to left is not upside down.
pub fn line_angle(a: Point, b: Point) -> f64 {
    let mut angle = (b - a).atan2().to_degrees();
    if angle > 90.0 {
        angle -= 180.0;
    } else if angle <= -90.0 {
        angle += 180.0;
    }
    if angle.abs() < 1e-9 { 0.0 } else { angle }
}

/// End points of a line dragged from `start` to `current`. Shift snaps the
/// angle to 45°; Alt uses `start` as the middle.
pub fn line_points(start: Point, current: Point, shift: bool, alt: bool) -> (Point, Point) {
    let d: Vec2 = if shift {
        snap_direction(current - start)
    } else {
        current - start
    };
    if alt {
        (start - d, start + d)
    } else {
        (start, start + d)
    }
}

#[cfg(test)]
mod tests {
    use tp_core::{Project, TextureResolution};

    use super::*;

    fn ws() -> Workspace {
        Workspace::new(Project::new("T", TextureResolution::R2048))
    }

    #[test]
    fn pen_points_and_finish() {
        let mut ws = ws();
        ws.pen_add(Point::new(100.0, 100.0), false);
        ws.pen_add(Point::new(400.0, 130.0), true);
        assert_eq!(
            ws.pen.as_ref().unwrap().nodes[1].point,
            Point::new(400.0, 100.0)
        );
        ws.pen_add(Point::new(500.0, 100.0), false);
        ws.pen_drag_handle(Point::new(600.0, 100.0), false);
        let n = ws.pen.as_ref().unwrap().nodes[2];
        assert!(n.smooth && n.handle_in == Some(Point::new(400.0, 100.0)));
        let id = ws.finish_pen(false, 1.0).unwrap();
        let o = ws.project.surface().get(id).unwrap();
        assert_eq!(o.name, "Path");
        assert!(o.has_open_path());
        assert_eq!(ws.selection, vec![id]);
        assert_eq!(ws.history.undo_label(), Some("Create Path"));
    }

    #[test]
    fn single_point_is_discarded_and_pop_empties() {
        let mut ws = ws();
        ws.pen_add(Point::new(1.0, 1.0), false);
        assert_eq!(ws.finish_pen(false, 1.0), None);
        assert!(ws.pen.is_none() && ws.project.surface().objects.is_empty());
        ws.pen_add(Point::new(1.0, 1.0), false);
        ws.pen_pop();
        assert!(ws.pen.is_none());
    }

    #[test]
    fn new_lines_use_the_current_style_and_line_width() {
        let mut ws = ws();
        ws.style.fill = tp_core::document::Rgba::rgb(255, 0, 0);
        let id = ws.create_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), 1.0);
        let line = ws.project.surface().get(id).unwrap();
        assert_eq!(line.fill, ws.style.fill);
        assert!(line.stroke.is_none(), "stroke off by default");
        assert_eq!(line.path_data().unwrap().line_width, 8.0);
        ws.line_width = 40.0;
        let id = ws.create_line(Point::new(0.0, 0.0), Point::new(10.0, 0.0), 2.0);
        let line = ws.project.surface().get(id).unwrap();
        assert_eq!(line.path_data().unwrap().line_width, 40.0);
    }

    #[test]
    fn line_rotation_is_its_angle() {
        let mut ws = ws();
        let (a, b) = (Point::new(500.0, 1000.0), Point::new(3500.0, 1020.0));
        let id = ws.create_line(a, b, 1.0);
        let line = ws.project.surface().get(id).unwrap();
        let expected = (20.0f64 / 3000.0).atan().to_degrees();
        assert!((line.frame.rotation_deg - expected).abs() < 1e-9);
        assert!((line.frame.size.width - a.distance(b)).abs() < 1e-6);
        assert_eq!(line.frame.size.height, tp_core::document::MIN_SIZE);
        // Same end points in the document.
        let doc: Vec<Point> = line.path_data().unwrap().subpaths[0]
            .nodes
            .iter()
            .map(|n| line.frame.affine() * n.point)
            .collect();
        assert!(doc[0].distance(a) < 1e-9 && doc[1].distance(b) < 1e-9);
        // Drawn right to left: same angle, not upside down.
        assert!((line_angle(b, a) - expected).abs() < 1e-9);
        assert_eq!(line_angle(Point::ORIGIN, Point::new(0.0, 10.0)), 90.0);
    }

    #[test]
    fn line_constraints() {
        let (a, b) = line_points(
            Point::new(100.0, 100.0),
            Point::new(500.0, 120.0),
            true,
            false,
        );
        assert_eq!((a, b), (Point::new(100.0, 100.0), Point::new(500.0, 100.0)));
        let (a, b) = line_points(
            Point::new(500.0, 500.0),
            Point::new(600.0, 500.0),
            false,
            true,
        );
        assert_eq!((a, b), (Point::new(400.0, 500.0), Point::new(600.0, 500.0)));
    }
}
