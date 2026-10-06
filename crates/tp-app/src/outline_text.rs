//! Object › Create Outlines: texts become groups of letter paths.

use std::sync::Arc;

use tp_core::document::{Object, ObjectId, PathData, ShapeKind};

use crate::workspace::Workspace;

/// Longest group name taken from the text, in characters.
const NAME_MAX: usize = 32;

/// Group name for a text: its first line, shortened.
pub fn group_name(content: &str) -> String {
    let line = content.lines().next().unwrap_or("").trim();
    let mut name: String = line.chars().take(NAME_MAX).collect();
    if line.chars().count() > NAME_MAX {
        name.push('…');
    }
    if name.is_empty() {
        "Text".to_owned()
    } else {
        name
    }
}

impl Workspace {
    /// The letter paths of a text, in reading order (empty without visible
    /// letters). Each letter keeps the text's rotation, fill and stroke.
    pub fn text_letters(&mut self, text: &Object) -> Vec<Object> {
        let Some(block) = &text.text else {
            return Vec::new();
        };
        let content = block.content.clone();
        let rotation = text.frame.rotation_deg;
        self.text
            .glyph_outlines(text)
            .into_iter()
            .map(|g| {
                let mut letter = Object::from_path(ObjectId(0), PathData::from_bezpath(&g.path));
                // Same geometry, in a frame rotated like the text.
                letter.set_path_rotation(rotation);
                letter.name = content.get(g.range).unwrap_or("?").to_owned();
                letter.fill = text.fill;
                letter.stroke = text.stroke;
                letter
            })
            .collect()
    }

    /// Texts in the selection, directly or inside selected groups.
    pub fn selected_text_ids(&self) -> Vec<ObjectId> {
        self.selected_objects()
            .iter()
            .flat_map(|o| {
                o.shapes()
                    .into_iter()
                    .filter(|s| s.kind == ShapeKind::Text)
                    .map(|s| s.id)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn selection_has_text(&self) -> bool {
        !self.selected_text_ids().is_empty()
    }

    /// Replaces each selected text by a group of its letters (one undo
    /// step). Texts without visible letters are left unchanged.
    pub fn create_outlines(&mut self, now: f64) {
        let mut groups = Vec::new();
        for id in self.selected_text_ids() {
            let Some(text) = self.project.surface().get(id).map(|o| (**o).clone()) else {
                continue;
            };
            let letters = self.text_letters(&text);
            if letters.is_empty() {
                continue;
            }
            let content = text.text.as_ref().map_or("", |b| b.content.as_str());
            let mut group = Object::group(ObjectId(0), letters.into_iter().map(Arc::new).collect());
            group.name = group_name(content);
            group.opacity = text.opacity;
            group.visible = text.visible;
            group.locked = text.locked;
            groups.push((id, group));
        }
        if groups.is_empty() {
            return;
        }
        self.edit("Create Outlines", now, false, |project, _| {
            for (id, group) in groups {
                project.replace_with_group(id, group);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::{CharStyle, TextBlock};
    use tp_core::kurbo::{Point, Shape};
    use tp_core::{Project, TextureResolution};

    use super::*;

    fn ws_with_text(content: &str) -> (Workspace, ObjectId) {
        let mut ws = Workspace::new(Project::new("T", TextureResolution::R4096));
        let anchor = Point::new(1000.0, 1000.0);
        let mut text = Object::text(
            ObjectId(0),
            TextBlock::new(content, CharStyle::default()),
            anchor,
        );
        ws.text.place_at(&mut text, anchor);
        let id = ws.project.add(text);
        ws.selection = vec![id];
        (ws, id)
    }

    #[test]
    fn outline_a_word() {
        let (mut ws, id) = ws_with_text("ACE");
        ws.create_outlines(1.0);
        let group = ws.project.surface().get(id).unwrap().clone();
        assert!(group.is_group());
        assert_eq!(group.name, "ACE");
        let names: Vec<&str> = group.children.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["A", "C", "E"]);
        assert!(group.children.iter().all(|c| c.kind == ShapeKind::Path));
        assert_eq!(ws.selection, vec![id], "the group (same id) stays selected");
        assert_eq!(ws.history.undo_label(), Some("Create Outlines"));
        ws.undo();
        assert_eq!(ws.project.surface().get(id).unwrap().kind, ShapeKind::Text);
    }

    #[test]
    fn only_spaces_are_left_unchanged() {
        let (mut ws, id) = ws_with_text("   ");
        let history = ws.history.len();
        ws.create_outlines(1.0);
        assert_eq!(ws.project.surface().get(id).unwrap().kind, ShapeKind::Text);
        assert_eq!(ws.history.len(), history);
    }

    #[test]
    fn letters_keep_rotation_and_stretch() {
        let (mut ws, id) = ws_with_text("H");
        let mut text = (**ws.project.surface().get(id).unwrap()).clone();
        text.frame.rotation_deg = 30.0;
        text.frame.size.width *= 1.5;
        text.sync_text_scale();
        ws.project.surface_mut().replace(&[text.clone()]);
        let before = ws.text.outline(&text);
        let letters = ws.text_letters(&text);
        assert_eq!(letters.len(), 1);
        assert!((letters[0].frame.rotation_deg - 30.0).abs() < 1e-9);
        // Same geometry in the document, point for point along the curves.
        let after = letters[0].path();
        let flat = |p: &tp_core::kurbo::BezPath| {
            let mut out = Vec::new();
            tp_core::kurbo::flatten(p.iter(), 0.01, |el| match el {
                tp_core::kurbo::PathEl::MoveTo(q) | tp_core::kurbo::PathEl::LineTo(q) => {
                    out.push(q)
                }
                _ => {}
            });
            out
        };
        let dist = |points: &[Point], path: &tp_core::kurbo::BezPath| {
            points
                .iter()
                .map(|q| {
                    path.segments()
                        .map(|s| {
                            tp_core::kurbo::ParamCurveNearest::nearest(&s, *q, 1e-9)
                                .distance_sq
                                .sqrt()
                        })
                        .fold(f64::INFINITY, f64::min)
                })
                .fold(0.0, f64::max)
        };
        assert!(dist(&flat(&before), &after) < 1e-3);
        assert!(dist(&flat(&after), &before) < 1e-3);
        let a = before.bounding_box();
        let b = after.bounding_box();
        assert!(
            (a.x0 - b.x0).abs() < 1e-6 && (a.y1 - b.y1).abs() < 1e-6,
            "{a:?} {b:?}"
        );
    }

    #[test]
    fn group_names() {
        assert_eq!(group_name("ACE\nLOGISTICS"), "ACE");
        assert_eq!(group_name("  "), "Text");
        assert_eq!(group_name(&"x".repeat(40)).chars().count(), 33);
    }
}
