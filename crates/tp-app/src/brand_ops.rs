//! The brand kit in the workspace: applying swatches with their links,
//! editing swatches live, and the shared style actions of the Styles panel.

use tp_core::document::{Object, Paint, Rgba, StyleId, SwatchId};
use tp_i18n::tr;

use crate::workspace::{ColorTarget, Workspace};

/// State of the Edit Swatch popup: the swatch, its name being typed and
/// the picker's color.
#[derive(Clone, Debug)]
pub struct SwatchEdit {
    pub id: SwatchId,
    pub name: String,
    pub hsv: tp_ui::widgets::Hsv,
    pub hex: String,
}

/// The swatch `o`'s `target` color is linked to: the solid paint's, or
/// the stop `stop`'s of a gradient.
fn link_of(o: &Object, target: ColorTarget, stop: usize) -> Option<SwatchId> {
    let (paint, link) = match target {
        ColorTarget::Fill => (o.fill, o.fill_swatch),
        ColorTarget::Stroke => {
            let s = o.stroke?;
            (s.paint, s.swatch)
        }
    };
    match paint {
        Paint::Solid(_) => link,
        Paint::Gradient(g) => g.stops()[stop.min(g.stops().len() - 1)].swatch,
    }
}

impl Workspace {
    /// The swatch every selected shape's `target` color (or selected stop)
    /// is linked to, if they share one.
    pub fn linked_swatch(&self, target: ColorTarget) -> Option<SwatchId> {
        let stop = self.panels.gradient_stop;
        let shapes = self.selected_shapes();
        let mut links = shapes.iter().map(|o| link_of(o, target, stop));
        let first = links.next()??;
        links.all(|l| l == Some(first)).then_some(first)
    }

    /// Applies swatch `id` to `target` of the selection (live) and links
    /// it: the solid color, or the selected stop of a gradient. With
    /// nothing selected, the current style takes the color, unlinked.
    pub fn apply_swatch(&mut self, target: ColorTarget, id: SwatchId) {
        let Some(color) = self.project.swatch(id).map(|s| s.color) else {
            return;
        };
        self.apply_color(target, color);
        if self.selection.is_empty() {
            return;
        }
        let stop = self.panels.gradient_stop;
        let label = match target {
            ColorTarget::Fill => "undo-change-fill",
            ColorTarget::Stroke => "undo-change-stroke",
        };
        self.map_selected_shapes(label, |o| match target {
            ColorTarget::Fill => match &mut o.fill {
                Paint::Solid(_) => o.fill_swatch = Some(id),
                Paint::Gradient(g) => {
                    let i = stop.min(g.stops().len() - 1);
                    g.stops_mut()[i].swatch = Some(id);
                }
            },
            ColorTarget::Stroke => {
                if let Some(s) = &mut o.stroke {
                    match &mut s.paint {
                        Paint::Solid(_) => s.swatch = Some(id),
                        Paint::Gradient(g) => {
                            let i = stop.min(g.stops().len() - 1);
                            g.stops_mut()[i].swatch = Some(id);
                        }
                    }
                }
            }
        });
    }

    /// Add to palette: a swatch of `color` (the existing one when a swatch
    /// has it), linked to the current target. One undo step.
    pub fn add_to_palette(&mut self, target: ColorTarget, color: Rgba, now: f64) {
        let mut added = None;
        self.live_edit("colors-add-to-palette", |project, _| {
            added = Some(project.add_swatch(color, &tr("colors-swatch-prefix")).0);
        });
        if let Some(id) = added {
            self.apply_swatch(target, id);
        }
        self.commit_pending(now);
    }

    /// Opens the Edit Swatch popup on swatch `id`.
    pub fn start_swatch_edit(&mut self, id: SwatchId) {
        let Some(s) = self.project.swatch(id) else {
            return;
        };
        let h = tp_core::document::Hsva::from(s.color);
        self.panels.editing_swatch = Some(SwatchEdit {
            id,
            name: s.name.clone(),
            hsv: tp_ui::widgets::Hsv::new(h.h / 360.0, h.s, h.v, h.a),
            hex: s.color.to_hex(),
        });
    }

    /// Recolors the edited swatch and every color linked to it (live).
    pub fn preview_swatch_color(&mut self, color: Rgba) {
        let Some(id) = self.panels.editing_swatch.as_ref().map(|e| e.id) else {
            return;
        };
        self.live_edit("undo-edit-swatch", |project, _| {
            project.set_swatch_color(id, color);
        });
    }

    /// OK in Edit Swatch: renames the swatch (an empty name keeps the old
    /// one) and records the edit as one undo step.
    pub fn finish_swatch_edit(&mut self, now: f64) {
        let Some(edit) = self.panels.editing_swatch.take() else {
            return;
        };
        let renamed = self
            .project
            .swatch(edit.id)
            .is_some_and(|s| s.name != edit.name.trim() && !edit.name.trim().is_empty());
        if renamed {
            self.live_edit("undo-edit-swatch", |project, _| {
                project.rename_swatch(edit.id, &edit.name);
            });
        }
        self.commit_pending(now);
    }

    /// Cancel in Edit Swatch: the swatch and linked colors are restored.
    pub fn cancel_swatch_edit(&mut self) {
        if self.panels.editing_swatch.take().is_some() {
            self.cancel_pending();
        }
    }

    /// Deletes swatch `id`; colors keep their values. One undo step.
    pub fn delete_swatch(&mut self, id: SwatchId, now: f64) {
        self.edit("colors-delete-swatch", now, false, |project, _| {
            project.delete_swatch(id);
        });
    }

    /// Ends a text being typed on the canvas, so that it is its own undo
    /// step before a style change (its step is recorded from a snapshot
    /// taken before the text existed).
    fn end_typing(&mut self, now: f64) {
        self.end_text_session(now);
    }

    /// The single selected object, if exactly one is selected.
    fn single_selected(&self) -> Option<tp_core::document::ObjectId> {
        match self.selection.as_slice() {
            [id] => Some(*id),
            _ => None,
        }
    }

    /// New Style from Selection (graphic): from the single selected shape
    /// or text.
    pub fn new_graphic_style(&mut self, now: f64) -> Option<StyleId> {
        self.end_typing(now);
        let from = self.single_selected()?;
        let mut made = None;
        self.edit("styles-new", now, false, |project, _| {
            made = project.new_graphic_style(from, &tr("styles-graphic-prefix"));
        });
        made
    }

    /// New Style from Selection (text): from the single selected text.
    pub fn new_text_style(&mut self, now: f64) -> Option<StyleId> {
        self.end_typing(now);
        let from = self.single_selected()?;
        let mut made = None;
        self.text_style_edit("styles-new", now, |project, _| {
            made = project.new_text_style(from, &tr("styles-text-prefix"));
        });
        made
    }

    /// Applies a style (graphic or text) to the selection.
    pub fn apply_style(&mut self, id: StyleId, now: f64) {
        self.end_typing(now);
        let ids = self.selection.clone();
        if self.project.graphic_style(id).is_some() {
            self.edit("styles-apply", now, false, |project, _| {
                project.apply_graphic_style(id, &ids);
            });
        } else {
            self.text_style_edit("styles-apply", now, |project, _| {
                project.apply_text_style(id, &ids);
            });
        }
    }

    /// Redefine from Selection: the style takes the single selected
    /// object's look, and every object following it changes.
    pub fn redefine_style(&mut self, id: StyleId, now: f64) {
        self.end_typing(now);
        let Some(from) = self.single_selected() else {
            return;
        };
        if self.project.graphic_style(id).is_some() {
            self.edit("undo-redefine-style", now, false, |project, _| {
                project.redefine_graphic_style(id, from);
            });
        } else {
            self.text_style_edit("undo-redefine-style", now, |project, _| {
                project.redefine_text_style(id, from);
            });
        }
    }

    /// Renames a style; returns false (nothing recorded) when the name is
    /// empty or taken.
    pub fn rename_style(&mut self, id: StyleId, name: &str, now: f64) -> bool {
        self.end_typing(now);
        let mut done = false;
        self.edit("undo-rename-style", now, false, |project, _| {
            done = project.rename_style(id, name);
        });
        done
    }

    /// Deletes a style; objects keep their look.
    pub fn delete_style(&mut self, id: StyleId, now: f64) {
        self.end_typing(now);
        self.edit("styles-delete", now, false, |project, _| {
            project.delete_style(id);
        });
    }

    /// Selects the objects of the active texture following style `id`.
    pub fn select_style_users(&mut self, id: StyleId) {
        self.selection = self.project.style_users(id, self.project.active_surface);
        self.normalize_selection();
    }
}

#[cfg(test)]
mod tests {
    use tp_core::document::{CharStyle, Frame, ObjectId, ShapeKind, TextBlock};
    use tp_core::kurbo::{Point, Size};
    use tp_core::{Project, TextureResolution};

    use super::*;

    const RED: Rgba = Rgba::rgb(200, 0, 0);
    const BLUE: Rgba = Rgba::rgb(0, 0, 255);

    fn ws() -> Workspace {
        Workspace::new(Project::new("t", TextureResolution::R2048))
    }

    fn rect(ws: &mut Workspace) -> ObjectId {
        ws.create_shape(
            ShapeKind::rectangle(),
            Frame::new(Point::new(100.0, 100.0), Size::new(100.0, 50.0), 0.0),
            0.0,
        )
    }

    fn object(ws: &Workspace, id: ObjectId) -> Object {
        (**ws.project.surface().get(id).unwrap()).clone()
    }

    #[test]
    fn swatch_links_and_unlinks() {
        let mut ws = ws();
        let r = rect(&mut ws);
        ws.apply_color(ColorTarget::Fill, RED);
        ws.commit_pending(1.0);
        ws.add_to_palette(ColorTarget::Fill, RED, 2.0);
        let swatch = ws.project.palette[0].id;
        assert_eq!(ws.linked_swatch(ColorTarget::Fill), Some(swatch));
        // Picking another color unlinks when the edit is recorded.
        ws.apply_color(ColorTarget::Fill, BLUE);
        ws.commit_pending(3.0);
        assert_eq!(object(&ws, r).fill_swatch, None);
        assert_eq!(ws.linked_swatch(ColorTarget::Fill), None);
        // Clicking the swatch links again.
        ws.apply_swatch(ColorTarget::Fill, swatch);
        ws.commit_pending(4.0);
        assert_eq!(object(&ws, r).fill, Paint::Solid(RED));
        assert_eq!(object(&ws, r).fill_swatch, Some(swatch));
    }

    #[test]
    fn cancelled_swatch_edit_restores_and_records_nothing() {
        let mut ws = ws();
        let r = rect(&mut ws);
        ws.add_to_palette(ColorTarget::Fill, RED, 1.0);
        let swatch = ws.project.palette[0].id;
        let steps = ws.history.len();
        ws.start_swatch_edit(swatch);
        ws.preview_swatch_color(BLUE);
        assert_eq!(object(&ws, r).fill, Paint::Solid(BLUE), "live preview");
        ws.cancel_swatch_edit();
        assert_eq!(object(&ws, r).fill, Paint::Solid(RED));
        assert_eq!(ws.project.swatch(swatch).unwrap().color, RED);
        assert_eq!(ws.history.len(), steps);
        // OK records one step, and the link holds.
        ws.start_swatch_edit(swatch);
        ws.preview_swatch_color(BLUE);
        if let Some(e) = &mut ws.panels.editing_swatch {
            e.name = "Company blue".into();
        }
        ws.finish_swatch_edit(5.0);
        assert_eq!(ws.history.len(), steps + 1);
        assert_eq!(ws.project.swatch(swatch).unwrap().name, "Company blue");
        assert_eq!(object(&ws, r).fill_swatch, Some(swatch));
        ws.undo();
        assert_eq!(object(&ws, r).fill, Paint::Solid(RED));
    }

    #[test]
    fn redefining_a_text_style_lays_texts_out_again() {
        let mut ws = ws();
        let mut t = Object::new(
            ObjectId(0),
            ShapeKind::Text,
            Frame::new(Point::new(300.0, 300.0), Size::new(10.0, 10.0), 0.0),
        );
        t.text = Some(TextBlock::new("ACE", CharStyle::default()));
        let mut t2 = t.clone();
        let t = ws.project.add(t);
        t2.frame.center.y = 800.0;
        let t2 = ws.project.add(t2);
        ws.selection = vec![t];
        let style = ws.new_text_style(1.0).unwrap();
        ws.selection = vec![t2];
        ws.apply_style(style, 2.0);
        let before = object(&ws, t2).text.unwrap().layout_size;
        // Change t's size, then redefine from it: t2 follows and is laid out.
        let mut bigger = object(&ws, t);
        bigger.text.as_mut().unwrap().style.size *= 2.0;
        ws.project.surface_mut().replace(&[bigger]);
        ws.selection = vec![t];
        ws.redefine_style(style, 3.0);
        let after = object(&ws, t2).text.unwrap();
        assert_eq!(after.style_id, Some(style));
        assert!(
            after.layout_size.height > before.height,
            "{before:?} → {:?}",
            after.layout_size
        );
    }
}

#[cfg(test)]
mod typing_tests {
    use tp_core::kurbo::Point;
    use tp_core::{Project, TextureResolution};

    use crate::workspace::Workspace;

    #[test]
    fn applying_a_style_while_typing() {
        let mut ws = Workspace::new(Project::new("t", TextureResolution::R2048));
        let source = ws.start_new_text(Point::new(300.0, 300.0), 0.0);
        ws.text_insert("Nice", 0.1);
        ws.end_text_session(0.2);
        ws.selection = vec![source];
        ws.set_char_style("undo-change-font", |s| s.size = 400.0);
        ws.commit_pending(1.0);
        let style = ws.new_text_style(2.0).unwrap();
        // A new text, still being typed, gets the style.
        let left = ws.start_new_text(Point::new(300.0, 900.0), 3.0);
        ws.text_insert("Left", 3.1);
        ws.apply_style(style, 4.0);
        let text = |ws: &Workspace| ws.project.surface().get(left).and_then(|o| o.text.clone());
        assert_eq!(text(&ws).unwrap().style_id, Some(style));
        // The user then leaves the text (Escape, or a click elsewhere).
        ws.end_text_session(5.0);
        assert_eq!(text(&ws).unwrap().style_id, Some(style));
        ws.undo();
        let t = text(&ws).expect("one Undo keeps the text");
        assert_eq!(t.content, "Left");
        assert_eq!(t.style_id, None);
        assert_ne!(t.style.size, 400.0);
    }
}
