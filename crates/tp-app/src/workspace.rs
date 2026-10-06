//! An open project and its editor state: selection, view, history, gesture.

use egui::Rect;
use tp_core::document::{Frame, History, Object, ObjectId, ShapeKind, translate};
use tp_core::kurbo::Vec2;
use tp_core::{Project, Snapshot};

use crate::geometry_cache::GeometryCache;
use crate::gesture::Gesture;
use crate::tool::Tool;
use crate::viewport::Viewport;

/// Offset applied by Duplicate and repeated Paste, in texture pixels.
pub const COPY_OFFSET: f64 = 20.0;

/// Document save state shown in the status bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveState {
    Saved,
    Unsaved,
}

/// A short message shown over the canvas until `until` (seconds).
#[derive(Clone, Debug)]
pub struct Hint {
    pub text: String,
    pub until: f64,
}

pub struct Workspace {
    pub project: Project,
    pub tool: Tool,
    /// Tool to restore when the temporary Hand tool (Space) is released.
    pub tool_before_space: Option<Tool>,
    pub save_state: SaveState,
    /// Selected objects of the active surface, in selection order.
    pub selection: Vec<ObjectId>,
    /// `None` until the canvas size is known (first frame).
    pub viewport: Option<Viewport>,
    pub history: History<Snapshot>,
    pub gesture: Gesture,
    /// Canvas area on screen during the last frame.
    pub canvas_rect: Option<Rect>,
    pub hint: Option<Hint>,
    pub geometry: GeometryCache,
}

impl Workspace {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            tool: Tool::default(),
            tool_before_space: None,
            save_state: SaveState::Unsaved,
            selection: Vec::new(),
            viewport: None,
            history: History::default(),
            gesture: Gesture::Idle,
            canvas_rect: None,
            hint: None,
            geometry: GeometryCache::default(),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.project.snapshot(&self.selection)
    }

    /// Records the change from `before` to the current state, unless nothing
    /// changed in the document.
    pub fn record(&mut self, label: &'static str, before: Snapshot, now: f64, coalesce: bool) {
        let after = self.snapshot();
        if before.same_document(&after) {
            return;
        }
        self.history.record(label, before, after, now, coalesce);
        self.save_state = SaveState::Unsaved;
    }

    /// Runs `edit` on the project and selection as one undoable step.
    pub fn edit(
        &mut self,
        label: &'static str,
        now: f64,
        coalesce: bool,
        edit: impl FnOnce(&mut Project, &mut Vec<ObjectId>),
    ) {
        let before = self.snapshot();
        edit(&mut self.project, &mut self.selection);
        self.record(label, before, now, coalesce);
    }

    pub fn undo(&mut self) {
        if let Some(state) = self.history.undo() {
            self.selection = self.project.restore(&state);
            self.gesture = Gesture::Idle;
            self.save_state = SaveState::Unsaved;
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.history.redo() {
            self.selection = self.project.restore(&state);
            self.gesture = Gesture::Idle;
            self.save_state = SaveState::Unsaved;
        }
    }

    /// Selected objects in stacking order (bottom to top).
    pub fn selected_objects(&self) -> Vec<Object> {
        self.project.selected_objects(&self.selection)
    }

    /// Keeps only selected ids that still exist on the active surface.
    pub fn prune_selection(&mut self) {
        let surface = self.project.surface();
        self.selection.retain(|id| surface.get(*id).is_some());
    }

    pub fn select_all(&mut self) {
        self.selection = self
            .project
            .surface()
            .objects
            .iter()
            .map(|o| o.id)
            .collect();
    }

    pub fn deselect(&mut self) {
        self.selection.clear();
    }

    /// Adds a shape with the default appearance, selects it, records it.
    pub fn create_shape(&mut self, kind: ShapeKind, frame: Frame, now: f64) -> ObjectId {
        let label = match kind {
            ShapeKind::Rectangle { .. } => "Create Rectangle",
            ShapeKind::Ellipse => "Create Ellipse",
        };
        let mut id = None;
        self.edit(label, now, false, |project, selection| {
            let new = project.add(Object::new(ObjectId(0), kind, frame));
            *selection = vec![new];
            id = Some(new);
        });
        id.expect("shape created")
    }

    pub fn delete_selection(&mut self, now: f64) {
        self.edit("Delete", now, false, |project, selection| {
            project.surface_mut().remove(selection);
            selection.clear();
        });
    }

    pub fn duplicate_selection(&mut self, now: f64) {
        self.edit("Duplicate", now, false, |project, selection| {
            *selection = project.duplicate(selection, Vec2::new(COPY_OFFSET, COPY_OFFSET));
        });
    }

    /// Inserts copies of `objects` with `offset`, selecting them.
    pub fn paste(&mut self, objects: &[Object], offset: f64, now: f64) {
        self.edit("Paste", now, false, |project, selection| {
            *selection = project.add_copies(objects, Vec2::new(offset, offset));
        });
    }

    pub fn bring_forward(&mut self, now: f64) {
        self.edit("Bring Forward", now, false, |project, selection| {
            project.surface_mut().bring_forward(selection);
        });
    }

    pub fn send_backward(&mut self, now: f64) {
        self.edit("Send Backward", now, false, |project, selection| {
            project.surface_mut().send_backward(selection);
        });
    }

    /// Moves the selection by `(dx, dy)` texture pixels; consecutive nudges
    /// within a second are one undo step.
    pub fn nudge(&mut self, dx: f64, dy: f64, now: f64) {
        let moved = translate(&self.selected_objects(), Vec2::new(dx, dy), false);
        self.edit("Nudge", now, true, |project, _| {
            project.surface_mut().replace(&moved);
        });
    }

    /// Document-to-screen map of the last frame, if the canvas was shown.
    pub fn screen_map(&self, pixels_per_point: f32) -> Option<crate::viewport::ScreenMap> {
        Some(self.viewport?.map(self.canvas_rect?, pixels_per_point))
    }

    /// The artboard on screen during the last frame.
    pub fn artboard_rect(&self, pixels_per_point: f32) -> Option<Rect> {
        let map = self.screen_map(pixels_per_point)?;
        let size = self.project.surface().size;
        Some(Rect::from_two_pos(
            map.to_screen(tp_core::kurbo::Point::ORIGIN),
            map.to_screen(tp_core::kurbo::Point::new(size, size)),
        ))
    }

    /// Shows a transient message over the canvas.
    pub fn show_hint(&mut self, text: impl Into<String>, now: f64) {
        self.hint = Some(Hint {
            text: text.into(),
            until: now + 2.5,
        });
    }
}

#[cfg(test)]
mod tests {
    use tp_core::TextureResolution;
    use tp_core::kurbo::{Point, Size};

    use super::*;

    fn ws() -> Workspace {
        Workspace::new(Project::new("t", TextureResolution::R2048))
    }

    fn frame(x: f64) -> Frame {
        Frame::new(Point::new(x, 100.0), Size::new(100.0, 100.0), 0.0)
    }

    #[test]
    fn undo_creation_and_redo_restores_selection() {
        let mut ws = ws();
        let id = ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        assert_eq!(ws.history.undo_label(), Some("Create Rectangle"));
        ws.undo();
        assert!(ws.project.surface().objects.is_empty());
        assert!(ws.selection.is_empty());
        ws.redo();
        assert_eq!(ws.selection, vec![id]);
    }

    #[test]
    fn duplicate_selects_copies_with_offset() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        ws.duplicate_selection(1.0);
        let copy = &ws.selected_objects()[0];
        assert_eq!(copy.frame.center, Point::new(120.0, 120.0));
        assert_eq!(ws.project.surface().objects.len(), 2);
    }

    #[test]
    fn nudges_coalesce() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        ws.nudge(10.0, 0.0, 5.0);
        ws.nudge(10.0, 0.0, 5.5);
        assert_eq!(ws.history.len(), 2);
        ws.undo();
        assert_eq!(ws.selected_objects()[0].frame.center.x, 100.0);
    }

    #[test]
    fn selection_only_changes_are_not_recorded() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        let before = ws.snapshot();
        ws.deselect();
        ws.record("Select", before, 1.0, false);
        assert_eq!(ws.history.len(), 1);
    }

    #[test]
    fn delete_and_undo() {
        let mut ws = ws();
        let id = ws.create_shape(ShapeKind::Ellipse, frame(100.0), 0.0);
        ws.delete_selection(1.0);
        assert!(ws.project.surface().objects.is_empty());
        ws.undo();
        assert_eq!(ws.selection, vec![id]);
    }
}
