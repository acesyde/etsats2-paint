//! An open project and its editor state: selection, view, history, gesture.

use std::collections::HashSet;

use egui::Rect;
use tp_core::document::tree::{self, Placement};
use tp_core::document::{
    CharStyle, DEFAULT_FILL, Frame, History, Object, ObjectId, Rgba, ShapeKind, StrokeStyle,
    translate,
};
use tp_core::kurbo::Vec2;
use tp_core::{Project, Snapshot};

use crate::geometry_cache::GeometryCache;
use crate::gesture::Gesture;
use crate::image_cache::ImageCache;
use crate::text_engine::TextEngine;
use crate::text_session::TextSession;
use crate::tool::Tool;
use crate::viewport::Viewport;

/// Offset applied by Duplicate and repeated Paste, in texture pixels.
pub const COPY_OFFSET: f64 = 20.0;

/// Document save state shown in the status bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveState {
    Saved,
    Unsaved,
    /// A save to the project file is running.
    Saving,
}

/// A save to the project file in progress.
#[derive(Clone, Debug)]
pub struct PendingSave {
    pub id: u64,
    /// The document being written: it becomes the saved state on success.
    pub snapshot: Snapshot,
    pub path: std::path::PathBuf,
}

/// A short message shown over the canvas until `until` (seconds).
#[derive(Clone, Debug)]
pub struct Hint {
    pub text: String,
    pub until: f64,
}

/// Default width of a newly added stroke, in texture pixels.
pub const DEFAULT_STROKE_WIDTH: f64 = 4.0;

/// Which color the Colors panel edits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorTarget {
    #[default]
    Fill,
    Stroke,
}

/// Color model shown by the Colors panel sliders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorModel {
    #[default]
    Rgb,
    Hsv,
    Hsl,
}

/// Fill and stroke used for new shapes (not part of the document history).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub fill: Rgba,
    pub stroke: StrokeStyle,
    pub stroke_enabled: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: DEFAULT_FILL,
            stroke: StrokeStyle {
                color: Rgba::rgb(0, 0, 0),
                width: DEFAULT_STROKE_WIDTH,
            },
            stroke_enabled: false,
        }
    }
}

impl Style {
    pub fn stroke(&self) -> Option<StrokeStyle> {
        self.stroke_enabled.then_some(self.stroke)
    }
}

/// UI state of the editing panels (not part of the document history).
#[derive(Debug, Default)]
pub struct PanelState {
    pub color_target: ColorTarget,
    pub color_model: ColorModel,
    /// Last color shown in the picker with its HSV, so hue survives at zero
    /// saturation or value.
    pub picker: Option<(Rgba, tp_ui::widgets::Hsv)>,
    pub lock_proportions: bool,
    /// Groups expanded in the Layers panel.
    pub expanded: HashSet<ObjectId>,
    /// Anchor row for Shift+click range selection in the Layers panel.
    pub layers_anchor: Option<ObjectId>,
    /// Row being renamed, with its edit buffer.
    pub renaming: Option<(ObjectId, String)>,
    /// Rows being dragged in the Layers panel.
    pub layers_drag: Option<Vec<ObjectId>>,
    /// Open font picker.
    pub font_picker: Option<FontPicker>,
    /// Asset being renamed in the Assets panel, with its edit buffer.
    pub renaming_asset: Option<(tp_core::document::AssetId, String)>,
}

/// State of the font family picker popup.
#[derive(Debug, Default)]
pub struct FontPicker {
    pub query: String,
    /// Highlighted row among the filtered families.
    pub highlighted: usize,
    pub focus_requested: bool,
}

pub struct Workspace {
    pub project: Project,
    pub tool: Tool,
    /// Tool to restore when the temporary Hand tool (Space) is released.
    pub tool_before_space: Option<Tool>,
    /// The project's file, once saved or opened.
    pub path: Option<std::path::PathBuf>,
    /// The document as last saved or opened (`None`: never saved).
    pub saved: Option<Snapshot>,
    /// Save in progress, and a path to save again once it finishes.
    pub saving: Option<PendingSave>,
    pub save_queued: Option<std::path::PathBuf>,
    /// Last recovery copy: when, and of which document.
    pub recovery_at: f64,
    pub recovery_written: Option<Snapshot>,
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
    /// Fill and stroke for new shapes.
    pub style: Style,
    pub panels: PanelState,
    /// Live panel edit not yet recorded: label and document before it.
    pending: Option<(&'static str, Snapshot)>,
    /// Last color applied, waiting to be added to recent colors once the
    /// interaction ends.
    pub recent_candidate: Option<Rgba>,
    /// Fonts, text layouts and glyph meshes.
    pub text: TextEngine,
    /// Character style for new texts.
    pub text_style: CharStyle,
    /// Text being edited on the canvas.
    pub text_session: Option<TextSession>,
    /// Display textures of image assets.
    pub images: ImageCache,
    /// The file dialog should open to place images, at a point or the view
    /// center (Place command, Image tool).
    pub place_request: Option<Option<tp_core::kurbo::Point>>,
    /// Path being drawn with the Pen tool.
    pub pen: Option<crate::path_edit::PenSession>,
    /// Sides and star settings for new polygons.
    pub polygon_style: crate::path_edit::PolygonStyle,
    /// Selected path points (Direct Selection tool).
    pub points: std::collections::BTreeSet<tp_core::document::PointRef>,
}

impl Workspace {
    pub fn new(project: Project) -> Self {
        Self::with_text_engine(project, TextEngine::default())
    }

    pub fn with_text_engine(project: Project, text: TextEngine) -> Self {
        Self {
            project,
            tool: Tool::default(),
            tool_before_space: None,
            path: None,
            saved: None,
            saving: None,
            save_queued: None,
            recovery_at: 0.0,
            recovery_written: None,
            selection: Vec::new(),
            viewport: None,
            history: History::default(),
            gesture: Gesture::Idle,
            canvas_rect: None,
            hint: None,
            geometry: GeometryCache::default(),
            style: Style::default(),
            panels: PanelState::default(),
            pending: None,
            recent_candidate: None,
            text,
            text_style: CharStyle::default(),
            text_session: None,
            images: ImageCache::default(),
            place_request: None,
            pen: None,
            polygon_style: Default::default(),
            points: Default::default(),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.project
            .snapshot(&self.selection)
            .with_points(self.points.iter().copied())
    }

    /// Restores a snapshot with its object and point selection.
    pub fn restore(&mut self, state: &Snapshot) {
        self.selection = self.project.restore(state);
        self.points = state.points().iter().copied().collect();
    }

    /// Records the change from `before` to the current state, unless nothing
    /// changed in the document.
    pub fn record(&mut self, label: &'static str, before: Snapshot, now: f64, coalesce: bool) {
        let after = self.snapshot();
        if before.same_document(&after) {
            return;
        }
        self.history.record(label, before, after, now, coalesce);
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

    /// Applies a live change (a field being typed, a slider being dragged…).
    /// The first call of an interaction remembers the document; the whole
    /// interaction becomes one undo step on [`Self::commit_pending`].
    pub fn live_edit(
        &mut self,
        label: &'static str,
        edit: impl FnOnce(&mut Project, &mut Vec<ObjectId>),
    ) {
        if self.pending.is_none() {
            self.pending = Some((label, self.snapshot()));
        }
        edit(&mut self.project, &mut self.selection);
    }

    /// Records the pending live edit, if any, as one undo step.
    pub fn commit_pending(&mut self, now: f64) {
        if let Some((label, before)) = self.pending.take() {
            self.record(label, before, now, false);
        }
    }

    /// Reverts the pending live edit, if any.
    pub fn cancel_pending(&mut self) {
        if let Some((_, before)) = self.pending.take() {
            self.selection = self.project.restore(&before);
        }
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub fn undo(&mut self) {
        if let Some(state) = self.history.undo() {
            self.restore(&state);
            self.gesture = Gesture::Idle;
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.history.redo() {
            self.restore(&state);
            self.gesture = Gesture::Idle;
        }
    }

    /// Whether the document differs from its saved state (always true for a
    /// project never saved).
    pub fn has_unsaved_changes(&self) -> bool {
        self.saved
            .as_ref()
            .is_none_or(|saved| !self.snapshot().same_document(saved))
    }

    /// Save state shown in the status bar.
    pub fn save_state(&self) -> SaveState {
        if self.saving.is_some() {
            SaveState::Saving
        } else if self.has_unsaved_changes() {
            SaveState::Unsaved
        } else {
            SaveState::Saved
        }
    }

    /// Marks the current document as saved to `path`.
    pub fn mark_saved(&mut self, path: std::path::PathBuf) {
        self.saved = Some(self.snapshot());
        self.path = Some(path);
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

    /// Removes ids that are gone or have a selected ancestor.
    pub fn normalize_selection(&mut self) {
        self.selection =
            tree::normalize_selection(&self.project.surface().objects, &self.selection);
    }

    /// Selects every visible, unlocked top-level object.
    pub fn select_all(&mut self) {
        self.selection = tree::selectable_top_level(&self.project.surface().objects);
    }

    /// Where new objects go: the selected group, or the common parent of the
    /// selection, or the top level.
    pub fn active_layer(&self) -> Option<ObjectId> {
        let objects = &self.project.surface().objects;
        if let [single] = self.selection.as_slice()
            && tree::get(objects, *single).is_some_and(|o| o.is_group())
        {
            return Some(*single);
        }
        let mut parents = self
            .selection
            .iter()
            .map(|id| tree::parent_of(objects, *id).flatten());
        let first = parents.next()??;
        parents.all(|p| p == Some(first)).then_some(first)
    }

    /// Escape: clears the selected path points if any, else the selection.
    pub fn deselect(&mut self) {
        self.clear_points_or_selection();
    }

    /// A shape with the current style.
    pub fn styled_shape(&self, kind: ShapeKind, frame: Frame) -> Object {
        let mut object = Object::new(ObjectId(0), kind, frame);
        object.fill = self.style.fill;
        object.stroke = self.style.stroke();
        object
    }

    /// Adds a shape with the current style to the active layer, selects it,
    /// records it.
    pub fn create_shape(&mut self, kind: ShapeKind, frame: Frame, now: f64) -> ObjectId {
        let object = self.styled_shape(kind, frame);
        self.create_object(object, now)
    }

    /// Adds `object` to the active layer, selects it, records it.
    pub fn create_object(&mut self, object: Object, now: f64) -> ObjectId {
        let label = match object.kind {
            ShapeKind::Rectangle { .. } => "Create Rectangle",
            ShapeKind::Ellipse => "Create Ellipse",
            ShapeKind::Polygon { .. } => "Create Polygon",
            ShapeKind::Path if object.name == "Line" => "Create Line",
            ShapeKind::Path => "Create Path",
            ShapeKind::Group => "Create Group",
            ShapeKind::Text => "Create Text",
            ShapeKind::Image { .. } => "Place",
        };
        let layer = self.active_layer();
        if let Some(layer) = layer {
            self.panels.expanded.insert(layer);
        }
        let mut id = None;
        self.edit(label, now, false, |project, selection| {
            let new = project.add_to(layer, object);
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
        let layer = self.active_layer();
        self.edit("Paste", now, false, |project, selection| {
            *selection = project.add_copies(objects, Vec2::new(offset, offset), layer);
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

    /// Groups the selection (Cmd/Ctrl+G).
    pub fn group_selection(&mut self, now: f64) {
        self.edit("Group", now, false, |project, selection| {
            if let Some(group) = project.group(selection) {
                *selection = vec![group];
            }
        });
    }

    /// Ungroups the selected groups (Cmd/Ctrl+Shift+G).
    pub fn ungroup_selection(&mut self, now: f64) {
        self.edit("Ungroup", now, false, |project, selection| {
            let released = project.ungroup(selection);
            if !released.is_empty() {
                *selection = released;
            }
        });
    }

    /// Adds an empty top-level group "Layer N" at the top and selects it.
    pub fn new_layer(&mut self, now: f64) {
        let next = self
            .project
            .surface()
            .objects
            .iter()
            .filter_map(|o| o.name.strip_prefix("Layer ")?.parse::<u32>().ok())
            .max()
            .unwrap_or(0)
            + 1;
        self.edit("New Layer", now, false, |project, selection| {
            let mut layer = Object::group(ObjectId(0), Vec::new());
            layer.name = format!("Layer {next}");
            *selection = vec![project.add(layer)];
        });
    }

    /// Moves objects in the tree (Layers drag and drop).
    pub fn move_objects(&mut self, ids: &[ObjectId], placement: Placement, now: f64) {
        let label = if matches!(placement, Placement::IntoTop(_)) {
            "Move to Group"
        } else {
            "Reorder"
        };
        let ids = ids.to_vec();
        self.edit(label, now, false, |project, _| {
            project.move_objects(&ids, placement);
        });
    }

    /// Edits one object (rename, visibility, lock…) as one undo step.
    pub fn edit_object(
        &mut self,
        id: ObjectId,
        label: &'static str,
        now: f64,
        f: impl FnOnce(&mut Object),
    ) {
        let Some(mut object) = self.project.surface().get(id).map(|o| (**o).clone()) else {
            return;
        };
        f(&mut object);
        self.edit(label, now, false, |project, _| {
            project.surface_mut().replace(&[object])
        });
    }

    pub fn rename(&mut self, id: ObjectId, name: &str, now: f64) {
        let trimmed = name.trim().to_owned();
        self.edit_object(id, "Rename", now, |o| {
            o.name = if trimmed.is_empty() {
                o.kind.name().to_owned()
            } else {
                trimmed
            };
        });
    }

    /// Shows or hides an object; hidden objects leave the selection.
    pub fn set_visible(&mut self, id: ObjectId, visible: bool, now: f64) {
        let label = if visible { "Show" } else { "Hide" };
        self.edit_object(id, label, now, |o| o.visible = visible);
        if !visible {
            self.drop_from_selection(id);
        }
    }

    /// Locks or unlocks an object; locked objects leave the selection.
    pub fn set_locked(&mut self, id: ObjectId, locked: bool, now: f64) {
        let label = if locked { "Lock" } else { "Unlock" };
        self.edit_object(id, label, now, |o| o.locked = locked);
        if locked {
            self.drop_from_selection(id);
        }
    }

    /// Removes `id` and its descendants from the selection.
    fn drop_from_selection(&mut self, id: ObjectId) {
        let objects = &self.project.surface().objects;
        self.selection
            .retain(|s| *s != id && !tree::is_ancestor(objects, id, *s));
    }

    /// Applies a color to the selection's fills or strokes (live), or to the
    /// current style when nothing is selected.
    pub fn apply_color(&mut self, target: ColorTarget, color: Rgba) {
        self.recent_candidate = Some(color);
        if self.selection.is_empty() {
            match target {
                ColorTarget::Fill => self.style.fill = color,
                ColorTarget::Stroke => {
                    self.style.stroke.color = color;
                    self.style.stroke_enabled = true;
                }
            }
            return;
        }
        let label = match target {
            ColorTarget::Fill => "Change Fill",
            ColorTarget::Stroke => "Change Stroke",
        };
        let width = self.style.stroke.width;
        self.map_selected_shapes(label, |o| match target {
            ColorTarget::Fill => o.fill = color,
            ColorTarget::Stroke => {
                let w = o.stroke.map_or(width, |s| s.width);
                o.stroke = Some(StrokeStyle { color, width: w });
            }
        });
    }

    /// Swaps fill and stroke colors (Shift+X). Shapes without a stroke get
    /// one in their fill color.
    pub fn swap_fill_stroke(&mut self, now: f64) {
        if self.selection.is_empty() {
            let style = &mut self.style;
            std::mem::swap(&mut style.fill, &mut style.stroke.color);
            return;
        }
        let width = self.style.stroke.width;
        self.map_selected_shapes("Swap Fill and Stroke", |o| match o.stroke {
            Some(mut stroke) => {
                std::mem::swap(&mut o.fill, &mut stroke.color);
                o.stroke = Some(stroke);
            }
            None => {
                o.stroke = Some(StrokeStyle {
                    color: o.fill,
                    width,
                })
            }
        });
        self.commit_pending(now);
    }

    /// Default fill and no stroke (D), for the selection or the current style.
    pub fn default_colors(&mut self, now: f64) {
        if self.selection.is_empty() {
            self.style = Style::default();
            return;
        }
        self.map_selected_shapes("Default Colors", |o| {
            o.fill = DEFAULT_FILL;
            o.stroke = None;
        });
        self.commit_pending(now);
    }

    /// Applies `f` to every selected shape (descending into groups) as a live
    /// edit labelled `label`.
    pub fn map_selected_shapes(&mut self, label: &'static str, mut f: impl FnMut(&mut Object)) {
        let mut objects = self.selected_objects();
        for o in &mut objects {
            o.for_each_shape(&mut f);
        }
        self.live_edit(label, |project, _| project.surface_mut().replace(&objects));
    }

    /// Selected shapes (descending into groups).
    pub fn selected_shapes(&self) -> Vec<Object> {
        self.selected_objects()
            .iter()
            .flat_map(|o| o.shapes().into_iter().cloned().collect::<Vec<_>>())
            .collect()
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
    fn live_edit_is_one_step_and_cancel_restores() {
        let mut ws = ws();
        ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        for value in [10, 50, 200] {
            ws.apply_color(ColorTarget::Fill, Rgba::rgb(value, 0, 0));
        }
        ws.commit_pending(1.0);
        assert_eq!(ws.history.len(), 2);
        assert_eq!(ws.history.undo_label(), Some("Change Fill"));
        assert_eq!(ws.selected_objects()[0].fill, Rgba::rgb(200, 0, 0));

        ws.apply_color(ColorTarget::Fill, Rgba::rgb(0, 255, 0));
        ws.cancel_pending();
        assert_eq!(ws.selected_objects()[0].fill, Rgba::rgb(200, 0, 0));
        assert_eq!(ws.history.len(), 2);
    }

    #[test]
    fn color_with_empty_selection_sets_style_for_new_shapes() {
        let mut ws = ws();
        ws.apply_color(ColorTarget::Fill, Rgba::rgb(255, 255, 255));
        ws.apply_color(ColorTarget::Stroke, Rgba::rgb(0, 0, 0));
        ws.style.stroke.width = 8.0;
        assert!(!ws.has_pending());
        ws.create_shape(ShapeKind::Ellipse, frame(100.0), 0.0);
        let o = &ws.selected_objects()[0];
        assert_eq!(o.fill, Rgba::rgb(255, 255, 255));
        assert_eq!(o.stroke.unwrap().width, 8.0);
    }

    #[test]
    fn new_shapes_go_to_the_active_layer() {
        let mut ws = ws();
        ws.new_layer(0.0);
        let layer = ws.selection[0];
        assert_eq!(ws.project.surface().get(layer).unwrap().name, "Layer 1");
        let id = ws.create_shape(ShapeKind::Ellipse, frame(100.0), 1.0);
        let group = ws.project.surface().get(layer).unwrap();
        assert_eq!(group.children.last().unwrap().id, id);
        ws.new_layer(2.0);
        assert_eq!(
            ws.project.surface().get(ws.selection[0]).unwrap().name,
            "Layer 2"
        );
    }

    #[test]
    fn hiding_removes_from_selection() {
        let mut ws = ws();
        let id = ws.create_shape(ShapeKind::rectangle(), frame(100.0), 0.0);
        ws.set_visible(id, false, 1.0);
        assert!(ws.selection.is_empty());
        assert_eq!(ws.history.undo_label(), Some("Hide"));
        ws.rename(id, "  ", 2.0);
        assert_eq!(ws.project.surface().get(id).unwrap().name, "Rectangle");
    }

    #[test]
    fn save_state_follows_the_saved_document() {
        let mut ws = ws();
        assert_eq!(ws.save_state(), SaveState::Unsaved, "never saved");
        ws.mark_saved("/tmp/a.truckpaint".into());
        assert_eq!(ws.save_state(), SaveState::Saved);
        ws.create_shape(ShapeKind::Ellipse, frame(100.0), 0.0);
        assert_eq!(ws.save_state(), SaveState::Unsaved);
        ws.undo();
        assert_eq!(ws.save_state(), SaveState::Saved);
        ws.redo();
        assert_eq!(ws.save_state(), SaveState::Unsaved);
        // Selection changes are not document changes.
        ws.undo();
        ws.select_all();
        assert_eq!(ws.save_state(), SaveState::Saved);
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
