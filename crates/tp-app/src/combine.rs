//! Object › Combine: Unite, Minus Front, Intersect and Exclude.

use tp_core::document::{
    BooleanError, BooleanOp, Object, OperandProblem, combine, operand_problem, tree,
};
use tp_core::kurbo::Affine;
use tp_i18n::tr;

use crate::workspace::Workspace;

/// Undo label and menu label of a combine command.
pub fn combine_label(op: BooleanOp) -> &'static str {
    match op {
        BooleanOp::Unite => "op-unite",
        BooleanOp::MinusFront => "op-minus-front",
        BooleanOp::Intersect => "op-intersect",
        BooleanOp::Exclude => "op-exclude",
    }
}

impl Workspace {
    /// Selected objects from bottom to top (stacking order).
    fn selection_bottom_to_top(&self) -> Vec<Object> {
        let objects = &self.project.surface().objects;
        tree::in_paint_order(objects, &self.selection)
            .into_iter()
            .filter_map(|id| tree::get(objects, id).map(|o| (**o).clone()))
            .collect()
    }

    /// Why the selection cannot be combined, if it cannot.
    pub fn combine_block(&self) -> Option<&'static str> {
        let objects = self.selected_objects();
        if let Some(problem) = objects.iter().find_map(operand_problem) {
            return Some(match problem {
                OperandProblem::OpenLine => "reason-combine-open-line",
                OperandProblem::Text => "reason-combine-text",
                OperandProblem::Image => "reason-combine-image",
                OperandProblem::Instance => "reason-instance-look",
            });
        }
        (objects.len() < 2).then_some("reason-select-two-shapes")
    }

    /// Combines the selection into one path (one undo step), or shows why
    /// it could not and leaves the document unchanged.
    pub fn combine_selection(&mut self, op: BooleanOp, now: f64) {
        if self.combine_block().is_some() {
            return;
        }
        let objects = self.selection_bottom_to_top();
        let data = match combine(&objects, op) {
            Ok(data) => data,
            Err(err) => {
                let text = match (err, op) {
                    (BooleanError::Empty, BooleanOp::MinusFront) => "hint-nothing-remains",
                    (BooleanError::Empty, _) => "hint-no-overlap",
                    (BooleanError::Failed, _) => "hint-combine-failed",
                };
                self.show_hint(tr(text), now);
                return;
            }
        };
        let top = objects.last().expect("two objects").clone();
        let style = if op == BooleanOp::MinusFront {
            &objects[0]
        } else {
            &top
        };
        // The result takes the topmost object's place (and id).
        let mut result = Object::from_path(top.id, data);
        result.name.clone_from(&style.name);
        result.fill = style.fill;
        result.stroke = style.stroke;
        // Gradients stay where they were on the style's object.
        result.remap_paints(&style.frame, Affine::IDENTITY);
        result.opacity = style.opacity;
        result.visible = style.visible;
        result.locked = style.locked;
        let others: Vec<_> = objects[..objects.len() - 1].iter().map(|o| o.id).collect();
        let id = top.id;
        self.edit(combine_label(op), now, false, |project, selection| {
            let surface = project.surface_mut();
            surface.replace(&[result]);
            surface.remove(&others);
            *selection = vec![id];
        });
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::{Frame, ObjectId, Rgba, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    use tp_core::{Project, TextureResolution};

    use super::*;

    fn ws() -> Workspace {
        Workspace::new(Project::new("T", TextureResolution::R2048))
    }

    fn rect(
        ws: &mut Workspace,
        x0: f64,
        y0: f64,
        w: f64,
        h: f64,
        fill: Rgba,
        name: &str,
    ) -> ObjectId {
        let mut o = Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(Point::new(x0 + w / 2.0, y0 + h / 2.0), Size::new(w, h), 0.0),
        );
        o.fill = fill.into();
        o.name = name.into();
        ws.project.add(o)
    }

    #[test]
    fn minus_front_takes_the_bottom_style_at_the_top_place() {
        let mut ws = ws();
        let blue = Rgba::rgb(0, 0, 255);
        let stripe = rect(&mut ws, 0.0, 0.0, 1000.0, 100.0, blue, "Stripe");
        let cut = rect(
            &mut ws,
            900.0,
            0.0,
            50.0,
            100.0,
            Rgba::rgb(255, 0, 0),
            "Cut",
        );
        let above = rect(
            &mut ws,
            0.0,
            500.0,
            10.0,
            10.0,
            Rgba::rgb(0, 255, 0),
            "Above",
        );
        // Selection order does not matter: stacking order does.
        ws.selection = vec![cut, stripe];
        ws.combine_selection(BooleanOp::MinusFront, 1.0);
        let objects = &ws.project.surface().objects;
        assert_eq!(objects.len(), 2, "one result plus the unselected object");
        let result = &objects[0];
        assert_eq!(result.id, cut, "the topmost object's place");
        assert_eq!(result.kind, ShapeKind::Path);
        assert_eq!(
            (result.fill, result.name.as_str()),
            (tp_core::document::Paint::Solid(blue), "Stripe")
        );
        assert_eq!(objects[1].id, above);
        assert_eq!(ws.selection, vec![cut]);
        assert_eq!(ws.history.undo_label(), Some("op-minus-front"));
        ws.undo();
        let ids: Vec<ObjectId> = ws.project.surface().objects.iter().map(|o| o.id).collect();
        assert_eq!(
            ids,
            vec![stripe, cut, above],
            "originals back with their ids"
        );
    }

    #[test]
    fn result_stays_in_its_group() {
        let mut ws = ws();
        let a = rect(&mut ws, 0.0, 0.0, 100.0, 100.0, Rgba::rgb(1, 1, 1), "A");
        let b = rect(&mut ws, 50.0, 50.0, 100.0, 100.0, Rgba::rgb(2, 2, 2), "B");
        let c = rect(&mut ws, 900.0, 900.0, 10.0, 10.0, Rgba::rgb(3, 3, 3), "C");
        let g = ws.project.group(&[a, b, c]).unwrap();
        ws.selection = vec![a, b];
        ws.combine_selection(BooleanOp::Unite, 1.0);
        let group = ws.project.surface().get(g).unwrap();
        assert_eq!(group.children.len(), 2);
        assert_eq!(group.children[0].id, b);
        assert_eq!(group.children[0].name, "B", "the topmost style");
    }

    #[test]
    fn no_overlap_leaves_the_document_and_explains() {
        let mut ws = ws();
        let a = rect(&mut ws, 0.0, 0.0, 10.0, 10.0, Rgba::rgb(1, 1, 1), "A");
        let b = rect(&mut ws, 100.0, 0.0, 10.0, 10.0, Rgba::rgb(2, 2, 2), "B");
        ws.selection = vec![a, b];
        let history = ws.history.len();
        ws.combine_selection(BooleanOp::Intersect, 1.0);
        assert_eq!(ws.project.surface().objects.len(), 2);
        assert_eq!(ws.history.len(), history);
        assert_eq!(ws.hint.as_ref().unwrap().text, "The shapes don't overlap.");
    }

    #[test]
    fn blocks() {
        let mut ws = ws();
        let a = rect(&mut ws, 0.0, 0.0, 10.0, 10.0, Rgba::rgb(1, 1, 1), "A");
        ws.selection = vec![a];
        assert_eq!(ws.combine_block(), Some("reason-select-two-shapes"));
        let mut t = Object::new(
            ObjectId(0),
            ShapeKind::Text,
            Frame::new(Point::ZERO, Size::new(5.0, 5.0), 0.0),
        );
        t.text = Some(tp_core::document::TextBlock::new("A", Default::default()));
        let t = ws.project.add(t);
        ws.selection = vec![a, t];
        assert_eq!(ws.combine_block(), Some("reason-combine-text"));
    }

    #[test]
    fn unite_keeps_the_topmost_gradient_in_place() {
        use tp_core::document::{ColorStop, Gradient, GradientKind, Paint};
        let mut ws = ws();
        let a = rect(&mut ws, 0.0, 0.0, 400.0, 100.0, Rgba::rgb(1, 1, 1), "A");
        let b = rect(&mut ws, 300.0, 0.0, 400.0, 300.0, Rgba::rgb(1, 1, 1), "B");
        let g = Gradient::new(
            GradientKind::Linear,
            &[
                ColorStop::new(0.0, Rgba::rgb(255, 0, 0)),
                ColorStop::new(1.0, Rgba::rgb(0, 0, 255)),
            ],
        );
        ws.selection = vec![b];
        ws.apply_paint(
            crate::workspace::ColorTarget::Fill,
            Paint::Gradient(g),
            "undo-change-fill",
        );
        ws.commit_pending(1.0);
        let top = ws.project.surface().get(b).unwrap().clone();
        let before = top.fill.gradient().unwrap().document_points(&top.frame);
        ws.selection = vec![a, b];
        ws.combine_selection(BooleanOp::Unite, 2.0);
        let result = ws.project.surface().get(b).unwrap();
        assert_ne!(result.frame, top.frame, "the union is larger");
        let after = result
            .fill
            .gradient()
            .unwrap()
            .document_points(&result.frame);
        assert!((before.0 - after.0).hypot() < 1e-6 && (before.1 - after.1).hypot() < 1e-6);
    }
}
