//! Align, distribute and flip the selection (Object menu, inspector › Layout).

use tp_core::document::{
    DistributeAxis, DistributeMode, Edge, FlipAxis, ObjectId, align, distribute, flip,
};
use tp_core::kurbo::Rect;

use crate::workspace::Workspace;

/// What the align commands align to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AlignTo {
    /// The selection's bounds; the artboard for a single object.
    #[default]
    Selection,
    Artboard,
    /// The most recently selected object, which does not move.
    KeyObject,
}

impl AlignTo {
    pub const ALL: [Self; 3] = [Self::Selection, Self::Artboard, Self::KeyObject];

    pub fn label(self) -> &'static str {
        match self {
            Self::Selection => "align-to-selection",
            Self::Artboard => "align-to-artboard",
            Self::KeyObject => "align-to-key-object",
        }
    }
}

/// Undo label of an align command.
pub fn align_label(edge: Edge) -> &'static str {
    match edge {
        Edge::Left => "op-align-left",
        Edge::HCenter => "op-align-horizontal-centers",
        Edge::Right => "op-align-right",
        Edge::Top => "op-align-top",
        Edge::VCenter => "op-align-vertical-centers",
        Edge::Bottom => "op-align-bottom",
    }
}

/// Label (and undo label) of a flip command.
pub fn flip_label(axis: FlipAxis) -> &'static str {
    match axis {
        FlipAxis::Horizontal => "op-flip-horizontal",
        FlipAxis::Vertical => "op-flip-vertical",
    }
}

/// Undo label of a distribute command.
pub fn distribute_label(axis: DistributeAxis, mode: DistributeMode) -> &'static str {
    match (axis, mode) {
        (DistributeAxis::Horizontal, DistributeMode::Centers) => "op-distribute-horizontal-centers",
        (DistributeAxis::Vertical, DistributeMode::Centers) => "op-distribute-vertical-centers",
        (DistributeAxis::Horizontal, DistributeMode::Spacing) => "op-distribute-horizontal-spacing",
        (DistributeAxis::Vertical, DistributeMode::Spacing) => "op-distribute-vertical-spacing",
    }
}

impl Workspace {
    /// The key object (Align to: Key object, two or more selected).
    pub fn key_object(&self) -> Option<ObjectId> {
        (self.panels.align_to == AlignTo::KeyObject && self.selection.len() >= 2)
            .then(|| self.selection.last().copied())
            .flatten()
    }

    /// What the align commands align to, and the object that stays put.
    pub fn align_target(&self) -> Option<(Rect, Option<ObjectId>)> {
        let objects = self.selected_objects();
        let artboard = self.project.surface().bounds();
        match self.panels.align_to {
            AlignTo::Artboard if !objects.is_empty() => Some((artboard, None)),
            AlignTo::Selection => match objects.as_slice() {
                [] => None,
                [_] => Some((artboard, None)),
                many => many
                    .iter()
                    .map(|o| o.bounding_box())
                    .reduce(|a, b| a.union(b))
                    .map(|r| (r, None)),
            },
            AlignTo::KeyObject => {
                let key = self.key_object()?;
                let rect = objects.iter().find(|o| o.id == key)?.bounding_box();
                Some((rect, Some(key)))
            }
            AlignTo::Artboard => None,
        }
    }

    /// Aligns the selection (one undo step).
    pub fn align_selection(&mut self, edge: Edge, now: f64) {
        let Some((target, key)) = self.align_target() else {
            return;
        };
        let moving: Vec<_> = self
            .selected_objects()
            .into_iter()
            .filter(|o| Some(o.id) != key)
            .collect();
        let moved = align(&moving, edge, target);
        self.edit(align_label(edge), now, false, |project, _| {
            project.surface_mut().replace(&moved);
        });
    }

    /// Mirrors the selection across the center of its bounds (one undo
    /// step).
    pub fn flip_selection(&mut self, axis: FlipAxis, now: f64) {
        let flipped = flip(&self.selected_objects(), axis);
        if flipped.is_empty() {
            return;
        }
        self.edit(flip_label(axis), now, false, |project, _| {
            project.surface_mut().replace(&flipped);
        });
    }

    /// Distributes the selection (three objects or more, one undo step).
    pub fn distribute_selection(&mut self, axis: DistributeAxis, mode: DistributeMode, now: f64) {
        let objects = self.selected_objects();
        if objects.len() < 3 {
            return;
        }
        let moved = distribute(&objects, axis, mode);
        self.edit(distribute_label(axis, mode), now, false, |project, _| {
            project.surface_mut().replace(&moved);
        });
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::{Frame, Object, ShapeKind};
    use tp_core::kurbo::{Point, Size};
    use tp_core::{Project, TextureResolution};

    use super::*;

    fn ws_with(rects: &[(f64, f64)]) -> (Workspace, Vec<ObjectId>) {
        let mut ws = Workspace::new(Project::new("T", TextureResolution::R4096));
        let ids = rects
            .iter()
            .map(|(x, y)| {
                ws.project.add(Object::new(
                    ObjectId(0),
                    ShapeKind::rectangle(),
                    Frame::new(Point::new(*x, *y), Size::new(100.0, 100.0), 0.0),
                ))
            })
            .collect();
        (ws, ids)
    }

    #[test]
    fn targets_for_every_mode() {
        let (mut ws, ids) = ws_with(&[(100.0, 100.0), (500.0, 300.0)]);
        assert_eq!(ws.align_target(), None, "nothing selected");
        ws.selection = vec![ids[0]];
        let board = Rect::new(0.0, 0.0, 4096.0, 4096.0);
        assert_eq!(ws.align_target(), Some((board, None)), "single → artboard");
        ws.selection = vec![ids[0], ids[1]];
        assert_eq!(
            ws.align_target(),
            Some((Rect::new(50.0, 50.0, 550.0, 350.0), None))
        );
        ws.panels.align_to = AlignTo::Artboard;
        assert_eq!(ws.align_target(), Some((board, None)));
        ws.panels.align_to = AlignTo::KeyObject;
        assert_eq!(
            ws.align_target(),
            Some((Rect::new(450.0, 250.0, 550.0, 350.0), Some(ids[1])))
        );
        ws.selection = vec![ids[1]];
        assert_eq!(ws.align_target(), None, "a key object needs two objects");
    }

    #[test]
    fn key_object_does_not_move_and_one_undo_step() {
        let (mut ws, ids) = ws_with(&[(100.0, 100.0), (500.0, 300.0)]);
        ws.panels.align_to = AlignTo::KeyObject;
        ws.selection = vec![ids[0], ids[1]];
        ws.align_selection(Edge::Top, 1.0);
        let top = |ws: &Workspace, id| ws.project.surface().get(id).unwrap().bounding_box().y0;
        assert_eq!(top(&ws, ids[1]), 250.0);
        assert_eq!(top(&ws, ids[0]), 250.0);
        assert_eq!(ws.history.undo_label(), Some("op-align-top"));
        ws.undo();
        assert_eq!(top(&ws, ids[0]), 50.0);
    }

    #[test]
    fn flipping_swaps_the_selection_in_one_undo_step() {
        let (mut ws, ids) = ws_with(&[(100.0, 100.0), (500.0, 300.0)]);
        let history = ws.history.len();
        ws.flip_selection(FlipAxis::Horizontal, 1.0);
        assert_eq!(ws.history.len(), history, "nothing selected, nothing done");
        ws.selection = ids.clone();
        ws.flip_selection(FlipAxis::Horizontal, 1.0);
        let x = |ws: &Workspace, id| ws.project.surface().get(id).unwrap().frame.center.x;
        assert_eq!((x(&ws, ids[0]), x(&ws, ids[1])), (500.0, 100.0));
        assert_eq!(ws.history.undo_label(), Some("op-flip-horizontal"));
        ws.undo();
        assert_eq!((x(&ws, ids[0]), x(&ws, ids[1])), (100.0, 500.0));
    }
}
