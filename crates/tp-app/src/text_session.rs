//! Text objects in the workspace: creating texts, on-canvas editing
//! sessions and character style changes.

use tp_core::Snapshot;
use tp_core::document::{CharStyle, Object, ObjectId, ShapeKind, TextBlock, tree};
use tp_core::kurbo::Point;
use tp_text::{EditSession, Motion, TextLayout};

use crate::text_engine::layout_to_doc;
use crate::workspace::Workspace;

/// A text being edited on the canvas.
pub struct TextSession {
    pub id: ObjectId,
    pub edit: EditSession,
    /// Document before the session (one undo step on commit).
    pub before: Snapshot,
    /// The text was created by this session ("Create Text").
    pub created: bool,
    /// Time of the last caret change, so the caret is solid while typing.
    pub caret_since: f64,
}

impl Workspace {
    pub fn is_editing_text(&self) -> bool {
        self.text_session.is_some()
    }

    pub fn editing_text(&self) -> Option<ObjectId> {
        self.text_session.as_ref().map(|s| s.id)
    }

    /// Creates an empty text with its anchor at `anchor`, using the current
    /// character style and fill, and starts editing it.
    pub fn start_new_text(&mut self, anchor: Point, now: f64) -> ObjectId {
        self.end_text_session(now);
        self.commit_pending(now);
        let before = self.snapshot();
        let mut object = Object::text(
            ObjectId(0),
            TextBlock::new("", self.text_style.clone()),
            anchor,
        );
        object.fill = self.style.fill;
        object.stroke = self.style.stroke();
        self.text.place_at(&mut object, anchor);
        let layer = self.active_layer();
        if let Some(layer) = layer {
            self.panels.expanded.insert(layer);
        }
        let id = self.project.add_to(layer, object);
        self.selection = vec![id];
        self.text_session = Some(TextSession {
            id,
            edit: EditSession::new(""),
            before,
            created: true,
            caret_since: now,
        });
        id
    }

    /// Whether `id` is a text that can be edited (visible and unlocked,
    /// including its groups).
    pub fn can_edit_text(&self, id: ObjectId) -> bool {
        let objects = &self.project.surface().objects;
        tree::get(objects, id).is_some_and(|o| o.kind == ShapeKind::Text)
            && tree::effective_flags(objects, id) == Some((false, false))
    }

    /// Starts editing an existing text, with the caret at `at` (document
    /// space) or at the end.
    pub fn start_editing(&mut self, id: ObjectId, at: Option<Point>, now: f64) -> bool {
        if self.editing_text() == Some(id) {
            if let Some(at) = at {
                self.text_click(at, false, now);
            }
            return true;
        }
        self.end_text_session(now);
        if !self.can_edit_text(id) {
            return false;
        }
        self.commit_pending(now);
        let Some(object) = self.project.surface().get(id).cloned() else {
            return false;
        };
        let content = object.text.as_ref().map_or("", |t| t.content.as_str());
        let mut edit = EditSession::new(content);
        if let Some(at) = at
            && let Some(layout) = self.session_layout_of(&object)
        {
            let local = layout_to_doc(&object).inverse() * at;
            edit.click(&layout, local, false);
        }
        self.selection = vec![id];
        self.text_session = Some(TextSession {
            id,
            edit,
            before: self.snapshot(),
            created: false,
            caret_since: now,
        });
        true
    }

    fn session_layout_of(&mut self, object: &Object) -> Option<std::sync::Arc<TextLayout>> {
        let block = object.text.as_ref()?;
        Some(self.text.block_layout(block))
    }

    /// Layout of the text being edited.
    pub fn session_layout(&mut self) -> Option<(Object, std::sync::Arc<TextLayout>)> {
        let id = self.editing_text()?;
        let object = (**self.project.surface().get(id)?).clone();
        let layout = self.session_layout_of(&object)?;
        Some((object, layout))
    }

    /// Ends the session: an emptied text is removed; the session becomes one
    /// "Create Text" or "Edit Text" step.
    pub fn end_text_session(&mut self, now: f64) {
        let Some(session) = self.text_session.take() else {
            return;
        };
        let empty = session.edit.text().trim().is_empty();
        if empty {
            self.project.surface_mut().remove(&[session.id]);
            self.selection.retain(|id| *id != session.id);
        }
        let label = if session.created {
            "Create Text"
        } else {
            "Edit Text"
        };
        self.record(label, session.before, now, false);
    }

    /// Ends the session when its text is no longer the only selected object
    /// or no longer exists.
    pub fn check_text_session(&mut self, now: f64) {
        let Some(id) = self.editing_text() else {
            return;
        };
        if self.selection != [id] || self.project.surface().get(id).is_none() {
            self.end_text_session(now);
        }
    }

    /// Applies `f` to the editing state, then updates the text object.
    pub fn with_session(&mut self, now: f64, f: impl FnOnce(&mut EditSession, &TextLayout)) {
        let Some((_, layout)) = self.session_layout() else {
            return;
        };
        let Some(session) = self.text_session.as_mut() else {
            return;
        };
        f(&mut session.edit, &layout);
        session.caret_since = now;
        self.sync_session_text();
    }

    /// Writes the session's content into its object, keeping the anchor.
    fn sync_session_text(&mut self) {
        let Some(session) = &self.text_session else {
            return;
        };
        let content = session.edit.text().to_owned();
        let Some(object) = self.project.surface().get(session.id) else {
            return;
        };
        let mut object = (**object).clone();
        let Some(previous) = object.text.clone() else {
            return;
        };
        if previous.content == content {
            return;
        }
        if let Some(text) = &mut object.text {
            text.content = content;
        }
        self.text.relayout(&mut object, &previous);
        self.project.surface_mut().replace(&[object]);
    }

    pub fn text_insert(&mut self, text: &str, now: f64) {
        self.with_session(now, |edit, _| edit.insert(text));
    }

    pub fn text_move(&mut self, motion: Motion, extend: bool, now: f64) {
        self.with_session(now, |edit, layout| edit.move_cursor(motion, extend, layout));
    }

    /// Places the caret at a document point (drag extends the selection).
    pub fn text_click(&mut self, at: Point, extend: bool, now: f64) {
        let Some((object, _)) = self.session_layout() else {
            return;
        };
        let local = layout_to_doc(&object).inverse() * at;
        self.with_session(now, |edit, layout| edit.click(layout, local, extend));
    }

    /// Selects the word under a document point.
    pub fn text_select_word(&mut self, at: Point, now: f64) {
        let Some((object, _)) = self.session_layout() else {
            return;
        };
        let local = layout_to_doc(&object).inverse() * at;
        self.with_session(now, |edit, layout| {
            let offset = layout.hit(local);
            edit.select_word(offset);
        });
    }

    /// Undoes within the session; returns false when nothing is left.
    pub fn text_undo(&mut self, now: f64) -> bool {
        let mut done = false;
        self.with_session(now, |edit, _| done = edit.undo());
        done
    }

    pub fn text_redo(&mut self, now: f64) -> bool {
        let mut done = false;
        self.with_session(now, |edit, _| done = edit.redo());
        done
    }

    /// Selected texts, descending into groups.
    pub fn selected_texts(&self) -> Vec<Object> {
        self.selected_shapes()
            .into_iter()
            .filter(|o| o.kind == ShapeKind::Text)
            .collect()
    }

    /// Changes the character style of the selected texts (live edit), or the
    /// style for new texts when nothing is selected.
    pub fn set_char_style(&mut self, label: &'static str, f: impl Fn(&mut CharStyle)) {
        if self.selection.is_empty() {
            f(&mut self.text_style);
            return;
        }
        let mut objects = self.selected_objects();
        let engine = &mut self.text;
        for object in &mut objects {
            object.for_each_shape(&mut |shape| {
                let Some(previous) = shape.text.clone() else {
                    return;
                };
                if let Some(text) = &mut shape.text {
                    f(&mut text.style);
                }
                engine.relayout(shape, &previous);
            });
        }
        if self.is_editing_text() {
            // Part of the editing session's undo step.
            self.project.surface_mut().replace(&objects);
        } else {
            self.live_edit(label, |project, _| project.surface_mut().replace(&objects));
        }
    }

    /// Lays out every text again (fonts changed).
    pub fn relayout_all_texts(&mut self) {
        let mut changed = Vec::new();
        let engine = &mut self.text;
        for object in &self.project.surface().objects {
            let mut copy = (**object).clone();
            let mut any = false;
            copy.for_each_shape(&mut |shape| {
                if let Some(block) = shape.text.clone() {
                    engine.relayout(shape, &block);
                    any |= shape.text.as_ref().map(|t| t.layout_size) != Some(block.layout_size);
                }
            });
            if any {
                changed.push(copy);
            }
        }
        if !changed.is_empty() {
            self.project.surface_mut().replace(&changed);
        }
    }
}

#[cfg(test)]
mod tests {
    use tp_core::{Project, TextureResolution};

    use super::*;

    fn ws() -> Workspace {
        Workspace::new(Project::new("t", TextureResolution::R2048))
    }

    fn type_text(ws: &mut Workspace, text: &str, now: f64) {
        for c in text.chars() {
            ws.text_insert(&c.to_string(), now);
        }
    }

    fn content(ws: &Workspace, id: ObjectId) -> String {
        ws.project
            .surface()
            .get(id)
            .unwrap()
            .text
            .as_ref()
            .unwrap()
            .content
            .clone()
    }

    #[test]
    fn create_type_and_commit_is_one_step() {
        let mut ws = ws();
        let id = ws.start_new_text(Point::new(300.0, 400.0), 0.0);
        type_text(&mut ws, "ACE Logistics", 0.1);
        let anchor = ws
            .text
            .anchor(&ws.project.surface().get(id).unwrap().clone())
            .unwrap();
        assert!((anchor - Point::new(300.0, 400.0)).hypot() < 1e-6);
        ws.end_text_session(1.0);
        assert_eq!(content(&ws, id), "ACE Logistics");
        assert_eq!(ws.history.len(), 1);
        assert_eq!(ws.history.undo_label(), Some("Create Text"));
        assert_eq!(ws.selection, vec![id]);
    }

    #[test]
    fn empty_text_is_removed_without_a_step() {
        let mut ws = ws();
        ws.start_new_text(Point::new(300.0, 400.0), 0.0);
        ws.end_text_session(1.0);
        assert!(ws.project.surface().objects.is_empty());
        assert_eq!(ws.history.len(), 0);
    }

    #[test]
    fn editing_session_undo() {
        let mut ws = ws();
        let id = ws.start_new_text(Point::new(300.0, 400.0), 0.0);
        type_text(&mut ws, "ACE", 0.1);
        ws.end_text_session(1.0);
        assert!(ws.start_editing(id, None, 2.0));
        type_text(&mut ws, " Logistics", 2.1);
        assert!(ws.text_undo(2.2));
        assert_eq!(content(&ws, id), "ACE");
        type_text(&mut ws, " Logistics", 2.3);
        ws.end_text_session(3.0);
        assert_eq!(ws.history.undo_label(), Some("Edit Text"));
        ws.undo();
        assert_eq!(content(&ws, id), "ACE");
    }

    #[test]
    fn char_style_applies_to_selected_texts_or_new_text_style() {
        let mut ws = ws();
        let a = ws.start_new_text(Point::new(100.0, 400.0), 0.0);
        type_text(&mut ws, "A", 0.0);
        ws.end_text_session(0.5);
        let b = ws.start_new_text(Point::new(100.0, 900.0), 1.0);
        type_text(&mut ws, "B", 1.0);
        ws.end_text_session(1.5);
        ws.selection = vec![a, b];
        let width = ws.project.surface().get(a).unwrap().frame.size.width;
        ws.set_char_style("Change Text Size", |s| s.size = 300.0);
        ws.commit_pending(2.0);
        assert_eq!(ws.history.undo_label(), Some("Change Text Size"));
        for id in [a, b] {
            let o = ws.project.surface().get(id).unwrap();
            assert_eq!(o.text.as_ref().unwrap().style.size, 300.0);
        }
        assert!(ws.project.surface().get(a).unwrap().frame.size.width > width);
        ws.undo();
        assert_eq!(
            ws.project
                .surface()
                .get(b)
                .unwrap()
                .text
                .as_ref()
                .unwrap()
                .style
                .size,
            200.0
        );
        ws.selection.clear();
        ws.set_char_style("Change Font", |s| s.family = "Oswald".into());
        assert_eq!(ws.text_style.family, "Oswald");
    }

    #[test]
    fn locked_text_cannot_be_edited() {
        let mut ws = ws();
        let id = ws.start_new_text(Point::new(100.0, 400.0), 0.0);
        type_text(&mut ws, "A", 0.0);
        ws.end_text_session(0.5);
        ws.set_locked(id, true, 1.0);
        assert!(!ws.start_editing(id, None, 2.0));
    }
}
