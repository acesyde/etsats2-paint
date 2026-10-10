//! Drop shadows of the selection (the inspector's Shadow row): adding,
//! editing and removing them. Only the selected shapes, paths, texts and
//! images themselves take a shadow: a selected group's or instance's
//! content keeps its own.

use tp_core::document::{Object, Rgba, Shadow, SwatchId};

use crate::workspace::Workspace;

impl Workspace {
    /// The shadow of each selected object that can have one, in stacking
    /// order; empty when none can (only groups and instances).
    pub fn selected_shadows(&self) -> Vec<Option<Shadow>> {
        self.selected_objects()
            .iter()
            .filter(|o| o.takes_shadow())
            .map(|o| o.shadow)
            .collect()
    }

    /// Applies `f` to every selected object that can have a shadow, as a
    /// live edit labelled `label`.
    fn map_shadowed(&mut self, label: &'static str, mut f: impl FnMut(&mut Object)) {
        let mut objects: Vec<Object> = self
            .selected_objects()
            .into_iter()
            .filter(Object::takes_shadow)
            .collect();
        if objects.is_empty() {
            return;
        }
        objects.iter_mut().for_each(&mut f);
        self.live_edit(label, |project, _| project.surface_mut().replace(&objects));
    }

    /// Gives the default shadow to every selected object that can have one
    /// and has none (+ Add a shadow). One undo step.
    pub fn add_shadow(&mut self, now: f64) {
        self.map_shadowed("undo-add-shadow", |o| {
            o.shadow.get_or_insert(Shadow::DEFAULT);
        });
        self.commit_pending(now);
    }

    /// Removes the selected objects' shadows. One undo step.
    pub fn remove_shadow(&mut self, now: f64) {
        self.map_shadowed("undo-remove-shadow", |o| o.shadow = None);
        self.commit_pending(now);
    }

    /// Changes every selected shadow with `f` (live; values are kept in
    /// their ranges). The interaction becomes one undo step on
    /// [`Workspace::commit_pending`].
    pub fn set_shadow(&mut self, f: impl Fn(&mut Shadow)) {
        self.map_shadowed("undo-change-shadow", |o| {
            if let Some(shadow) = &mut o.shadow {
                f(shadow);
                *shadow = shadow.clamped();
            }
        });
    }

    /// Gives the selected shadows `color` (live). Like a fill, a shadow
    /// linked to a swatch stays linked while it keeps the swatch's color.
    pub fn set_shadow_color(&mut self, color: Rgba) {
        self.recent_candidate = Some(color);
        self.set_shadow(|s| s.color = color);
    }

    /// Gives the selected shadows the color of swatch `id` and links them
    /// to it (live).
    pub fn link_shadow(&mut self, id: SwatchId) {
        let Some(color) = self.project.swatch(id).map(|s| s.color) else {
            return;
        };
        self.set_shadow(|s| {
            s.color = color;
            s.swatch = Some(id);
        });
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::{Frame, ObjectId, ShapeKind};
    use tp_core::kurbo::{Point, Size};

    use super::*;

    fn workspace() -> (Workspace, ObjectId, ObjectId) {
        let mut ws = Workspace::new(tp_core::Project::new(
            "Test",
            tp_core::TextureResolution::default(),
        ));
        let frame = Frame::new(Point::new(100.0, 100.0), Size::new(50.0, 50.0), 0.0);
        let a = ws
            .project
            .add(Object::new(ObjectId(0), ShapeKind::rectangle(), frame));
        let b = ws
            .project
            .add(Object::new(ObjectId(0), ShapeKind::Ellipse, frame));
        (ws, a, b)
    }

    fn shadow(ws: &Workspace, id: ObjectId) -> Option<Shadow> {
        ws.project.surface().get(id).unwrap().shadow
    }

    #[test]
    fn add_edit_and_remove_are_one_step_each() {
        let (mut ws, a, b) = workspace();
        ws.selection = vec![a, b];
        ws.add_shadow(1.0);
        assert_eq!(shadow(&ws, a), Some(Shadow::DEFAULT));
        assert_eq!(shadow(&ws, b), Some(Shadow::DEFAULT));
        // A drag: many live changes, one step; values stay in range.
        ws.set_shadow(|s| s.blur = 12.0);
        ws.set_shadow(|s| s.blur = 500.0);
        ws.commit_pending(2.0);
        assert_eq!(shadow(&ws, a).unwrap().blur, 200.0);
        ws.undo();
        assert_eq!(shadow(&ws, b).unwrap().blur, 8.0);
        ws.remove_shadow(3.0);
        assert_eq!(shadow(&ws, a), None);
        ws.undo();
        ws.undo();
        assert_eq!(shadow(&ws, a), None);
    }

    #[test]
    fn a_group_takes_no_shadow_and_its_content_keeps_its_own() {
        let (mut ws, a, b) = workspace();
        ws.selection = vec![a];
        ws.add_shadow(1.0);
        ws.selection = vec![a, b];
        ws.group_selection(2.0);
        let group = ws.selection[0];
        assert!(ws.selected_shadows().is_empty());
        ws.add_shadow(3.0);
        ws.remove_shadow(4.0);
        assert_eq!(shadow(&ws, group), None);
        assert_eq!(shadow(&ws, a), Some(Shadow::DEFAULT));
        assert_eq!(shadow(&ws, b), None);
    }
}
