//! The brand kit in the workspace: applying swatches with their links,
//! the usage of swatches, styles and symbols, the before/after editor of
//! swatches and graphic styles, and the shared style actions of the Styles
//! panel.

use std::sync::Arc;

use tp_core::document::{Object, Paint, Rgba, StyleId, SwatchId, SymbolId};
use tp_core::{Count, Look, Project, Usage};
use tp_i18n::tr;

use crate::layout::Space;
use crate::workspace::{ColorTarget, Workspace};

/// The usage of the project's swatches, styles and symbols, computed again
/// only when the project changed.
///
/// Every edit replaces at least one top-level `Arc` of the surface it
/// changes (see `surface_thumbnails`), so the top-level objects of every
/// texture and symbol, with the ids of the swatches, styles and symbols,
/// tell whether the usage may have changed. The key holds the `Arc`s, so a
/// pointer can't be reused by another object while it is compared.
#[derive(Default)]
pub struct UsageCache {
    key: Option<UsageKey>,
    value: Usage,
    /// Number of times the usage was computed, for tests.
    pub computations: u64,
}

#[derive(PartialEq)]
struct UsageKey {
    lists: Vec<ArcList>,
    swatches: Vec<SwatchId>,
    styles: Vec<StyleId>,
    symbols: Vec<SymbolId>,
}

/// Top-level objects, compared by pointer.
struct ArcList(Vec<Arc<Object>>);

impl PartialEq for ArcList {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len() && self.0.iter().zip(&other.0).all(|(a, b)| Arc::ptr_eq(a, b))
    }
}

impl UsageKey {
    fn of(p: &Project) -> Self {
        let symbols = p.symbols.iter().map(|s| &s.surface);
        Self {
            lists: p
                .surfaces
                .iter()
                .chain(symbols)
                .map(|s| ArcList(s.objects.clone()))
                .collect(),
            swatches: p.palette.iter().map(|s| s.id).collect(),
            styles: p
                .graphic_styles
                .iter()
                .map(|s| s.id)
                .chain(p.text_styles.iter().map(|s| s.id))
                .collect(),
            symbols: p.symbols.iter().map(|s| s.id).collect(),
        }
    }
}

impl UsageCache {
    /// The usage of `project`'s elements.
    pub fn get(&mut self, project: &Project) -> &Usage {
        let key = UsageKey::of(project);
        if self.key.as_ref() != Some(&key) {
            self.value = project.usage();
            self.key = Some(key);
            self.computations += 1;
        }
        &self.value
    }
}

/// The usage of a swatch or a style: "11 textures · 38 objects", or
/// "Unused".
pub fn usage_label(count: &Count) -> String {
    if count.is_unused() {
        return tr("brand-unused");
    }
    tr!(
        "brand-usage",
        textures = count.textures(),
        objects = count.objects
    )
}

/// The usage of a symbol: "14 instances · 9 textures", or "Unused".
pub fn symbol_usage_label(count: &Count) -> String {
    if count.is_unused() {
        return tr("brand-unused");
    }
    tr!(
        "brand-symbol-usage",
        instances = count.objects,
        textures = count.textures()
    )
}

/// The impact of an edit: "Impact: 11 textures, 38 objects", or a line
/// saying nothing uses the element yet.
pub fn impact_label(count: &Count) -> String {
    if count.is_unused() {
        return tr("brand-impact-none");
    }
    tr!(
        "brand-impact",
        textures = count.textures(),
        objects = count.objects
    )
}

/// What the before/after editor edits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrandEditTarget {
    Swatch(SwatchId),
    /// A graphic style (text styles have no editor).
    Style(StyleId),
}

/// The value being edited: a swatch's color or a graphic style's look.
// Only the open editor holds values (two at a time): their size doesn't
// matter, and copying them keeps the editor simple.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditValue {
    Color(Rgba),
    Look(Look),
}

/// State of the before/after editor of a swatch or a graphic style. The
/// document is never touched before Apply to Fleet: the new value is shown
/// on a copy of the project.
#[derive(Clone, Debug)]
pub struct BrandEdit {
    pub target: BrandEditTarget,
    /// The name being typed.
    pub name: String,
    /// The current value, and the new one.
    pub before: EditValue,
    pub after: EditValue,
    /// Picker state: the color of the swatch, or of the look's fill or
    /// stroke (`look_target`), as hex and HSV.
    pub hex: String,
    pub hsv: tp_ui::widgets::Hsv,
    /// Which color of a look the picker edits.
    pub look_target: ColorTarget,
    /// The project with `after` applied, once a value was set.
    pub preview: Option<Project>,
    /// The textures holding the objects the edit changes (from the usage
    /// when the editor opened).
    pub affected: Vec<usize>,
    /// The impact: the element's usage when the editor opened.
    pub impact: Count,
    /// The space the editor was opened in: showing another one cancels it.
    pub space: Space,
}

/// The picker's HSV for `color`.
fn hsv_of(color: Rgba) -> tp_ui::widgets::Hsv {
    let h = tp_core::document::Hsva::from(color);
    tp_ui::widgets::Hsv::new(h.h / 360.0, h.s, h.v, h.a)
}

/// The solid color of `look`'s fill or stroke, if it has one.
pub fn look_color(look: &Look, target: ColorTarget) -> Option<Rgba> {
    let paint = match target {
        ColorTarget::Fill => look.fill,
        ColorTarget::Stroke => look.stroke?.paint,
    };
    match paint {
        Paint::Solid(c) => Some(c),
        Paint::Gradient(_) => None,
    }
}

impl BrandEdit {
    fn new(ws: &mut Workspace, target: BrandEditTarget, name: String, value: EditValue) -> Self {
        let usage = ws.usage();
        let impact = match target {
            BrandEditTarget::Swatch(id) => usage.swatch(id),
            BrandEditTarget::Style(id) => usage.style(id),
        }
        .clone();
        let color = match &value {
            EditValue::Color(c) => *c,
            EditValue::Look(look) => look_color(look, ColorTarget::Fill)
                .or_else(|| look_color(look, ColorTarget::Stroke))
                .unwrap_or(Rgba::rgb(0, 0, 0)),
        };
        Self {
            target,
            name,
            before: value,
            after: value,
            hex: color.to_hex(),
            hsv: hsv_of(color),
            look_target: ColorTarget::Fill,
            preview: None,
            affected: impact.surfaces.clone(),
            impact,
            space: ws.space,
        }
    }

    /// Whether Apply to Fleet would change anything.
    pub fn changed(&self, project: &Project) -> bool {
        let name = self.name.trim();
        let current = match self.target {
            BrandEditTarget::Swatch(id) => project.swatch(id).map(|s| s.name.as_str()),
            BrandEditTarget::Style(id) => project.graphic_style(id).map(|s| s.name.as_str()),
        };
        self.after != self.before || current != Some(name)
    }

    /// Why Apply to Fleet is refused, as the key of the reason shown: an
    /// empty name, or a style name another graphic style already has.
    pub fn problem(&self, project: &Project) -> Option<&'static str> {
        let name = self.name.trim();
        match self.target {
            BrandEditTarget::Swatch(_) if name.is_empty() => Some("colors-swatch-name-empty"),
            BrandEditTarget::Style(_) if name.is_empty() => Some("styles-name-empty"),
            BrandEditTarget::Style(id)
                if project
                    .graphic_styles
                    .iter()
                    .any(|s| s.id != id && s.name == name) =>
            {
                Some("styles-name-taken")
            }
            _ => None,
        }
    }
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

/// Empty thumbnails for the editor's previews.
fn preview_thumbs() -> crate::surface_thumbnails::SurfaceThumbnails {
    crate::surface_thumbnails::SurfaceThumbnails::named("preview_thumbnail")
}

/// Applies `value` to the swatch or style of `target` in `project`.
fn apply_value(project: &mut Project, target: BrandEditTarget, value: EditValue) {
    match (target, value) {
        (BrandEditTarget::Swatch(id), EditValue::Color(c)) => project.set_swatch_color(id, c),
        (BrandEditTarget::Style(id), EditValue::Look(look)) => {
            project.set_graphic_style_look(id, look);
        }
        _ => {}
    }
}

/// `look` with its fill and stroke links dropped when their color no
/// longer is their swatch's ("Linked, not copied").
fn linked_look(project: &Project, mut look: Look) -> Look {
    let holds = |paint: &Paint, link: Option<SwatchId>| match (paint, link) {
        (Paint::Solid(c), Some(id)) => project.swatch(id).is_some_and(|s| s.color == *c),
        _ => true,
    };
    if !holds(&look.fill, look.fill_swatch) {
        look.fill_swatch = None;
    }
    if let Some(s) = &mut look.stroke
        && !holds(&s.paint, s.swatch)
    {
        s.swatch = None;
    }
    look
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

    /// The usage of the project's swatches, styles and symbols (computed
    /// again only when the project changed).
    pub fn usage(&mut self) -> &Usage {
        self.usage_cache.get(&self.project)
    }

    /// Opens the before/after editor on swatch `id`.
    pub fn start_swatch_edit(&mut self, id: SwatchId) {
        let Some(s) = self.project.swatch(id) else {
            return;
        };
        let (name, color) = (s.name.clone(), s.color);
        let edit = BrandEdit::new(
            self,
            BrandEditTarget::Swatch(id),
            name,
            EditValue::Color(color),
        );
        self.open_brand_edit(edit);
    }

    /// Opens the before/after editor on graphic style `id` (a text style
    /// has none).
    pub fn start_style_edit(&mut self, id: StyleId) {
        let Some(s) = self.project.graphic_style(id) else {
            return;
        };
        let (name, look) = (s.name.clone(), s.look);
        let edit = BrandEdit::new(
            self,
            BrandEditTarget::Style(id),
            name,
            EditValue::Look(look),
        );
        self.open_brand_edit(edit);
    }

    fn open_brand_edit(&mut self, edit: BrandEdit) {
        self.panels.brand_edit = Some(edit);
        self.preview_thumbs = preview_thumbs();
    }

    /// Sets the editor's new value and shows it on a copy of the project
    /// (rebuilt only when the value changes); the document is untouched.
    pub fn set_edit_value(&mut self, value: EditValue) {
        let Some(edit) = &self.panels.brand_edit else {
            return;
        };
        let value = match value {
            EditValue::Look(look) => EditValue::Look(linked_look(&self.project, look)),
            color => color,
        };
        if edit.preview.is_some() && edit.after == value {
            return;
        }
        let target = edit.target;
        let mut preview = self.project.clone();
        apply_value(&mut preview, target, value);
        if let Some(edit) = &mut self.panels.brand_edit {
            edit.after = value;
            edit.preview = Some(preview);
        }
    }

    /// Apply to Fleet: records the new value and name as one undo step and
    /// closes the editor; with nothing changed, just closes it. Refuses an
    /// empty name, or a style name taken by another style (the editor stays
    /// open). Returns whether it closed.
    pub fn apply_brand_edit(&mut self, now: f64) -> bool {
        let Some(edit) = &self.panels.brand_edit else {
            return false;
        };
        if edit.problem(&self.project).is_some() {
            return false;
        }
        let edit = self.panels.brand_edit.take().expect("an open editor");
        self.preview_thumbs = preview_thumbs();
        if !edit.changed(&self.project) {
            return true;
        }
        self.end_typing(now);
        let label = match edit.target {
            BrandEditTarget::Swatch(_) => "undo-edit-swatch",
            BrandEditTarget::Style(_) => "undo-edit-style",
        };
        let value = edit.after;
        self.edit(label, now, false, |project, _| {
            if value != edit.before {
                apply_value(project, edit.target, value);
            }
            match edit.target {
                BrandEditTarget::Swatch(id) => project.rename_swatch(id, &edit.name),
                BrandEditTarget::Style(id) => project.rename_style(id, &edit.name),
            };
        });
        true
    }

    /// Cancel or Escape in the editor: closes it; the project was never
    /// touched.
    pub fn cancel_brand_edit(&mut self) {
        if self.panels.brand_edit.take().is_some() {
            self.preview_thumbs = preview_thumbs();
        }
    }

    /// Shows `space`; an open before/after editor is cancelled when the
    /// space changes.
    pub fn show_space(&mut self, space: Space) {
        if space != self.space {
            self.cancel_brand_edit();
        }
        self.space = space;
    }

    /// Once per frame: cancels the editor when another space is shown
    /// (whatever showed it), and renders the thumbnails of the affected
    /// textures with the new value.
    pub fn update_brand_edit(&mut self, ctx: &egui::Context) {
        let Some(edit) = &self.panels.brand_edit else {
            return;
        };
        if edit.space != self.space {
            self.cancel_brand_edit();
            return;
        }
        if let Some(preview) = &edit.preview {
            self.preview_thumbs
                .update_only(ctx, preview, &self.text.fonts, &edit.affected);
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
    const DARK_RED: Rgba = Rgba::rgb(0x8B, 0, 0);
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

    /// A rectangle on surface `surface` filled with `color`, linked to
    /// swatch `id`.
    fn linked_on(ws: &mut Workspace, surface: usize, color: Rgba, id: SwatchId) -> ObjectId {
        let mut o = Object::new(
            ObjectId(0),
            ShapeKind::rectangle(),
            Frame::new(Point::new(1024.0, 1024.0), Size::new(2048.0, 2048.0), 0.0),
        );
        o.fill = Paint::Solid(color);
        o.fill_swatch = Some(id);
        let active = ws.project.active_surface;
        ws.project.active_surface = surface;
        let id = ws.project.add(o);
        ws.project.active_surface = active;
        id
    }

    fn on(ws: &Workspace, surface: usize, id: ObjectId) -> Object {
        (**ws.project.surfaces[surface].get(id).unwrap()).clone()
    }

    /// Three textures; "Company red" fills a rectangle on the first and
    /// the third.
    fn fleet() -> (Workspace, SwatchId, ObjectId, ObjectId) {
        let mut ws = ws();
        for name in ["Chassis", "Trailer"] {
            ws.project
                .surfaces
                .push(tp_core::Surface::new(name, 2048.0));
        }
        let (red, _) = ws.project.add_swatch(RED, "Color");
        ws.project.rename_swatch(red, "Company red");
        let a = linked_on(&mut ws, 0, RED, red);
        let b = linked_on(&mut ws, 2, RED, red);
        (ws, red, a, b)
    }

    fn rename_edit(ws: &mut Workspace, name: &str) {
        ws.panels.brand_edit.as_mut().unwrap().name = name.into();
    }

    // ── Usage ───────────────────────────────────────────────────────────

    #[test]
    fn usage_follows_undo_and_redo() {
        let (mut ws, red, ..) = fleet();
        assert_eq!(ws.usage().swatch(red).objects, 2);
        ws.edit("undo-change-fill", 1.0, false, |p, _| {
            p.active_surface = 1;
            let mut o = (*p.surfaces[0].objects[0]).clone();
            o.id = ObjectId(0);
            p.add(o);
            p.active_surface = 0;
        });
        assert_eq!(ws.usage().swatch(red).objects, 3);
        assert_eq!(ws.usage().swatch(red).surfaces, [0, 1, 2]);
        ws.undo();
        assert_eq!(ws.usage().swatch(red).objects, 2);
        ws.redo();
        assert_eq!(ws.usage().swatch(red).objects, 3);
    }

    #[test]
    fn usage_is_computed_again_only_on_change() {
        let (mut ws, red, ..) = fleet();
        ws.usage();
        let computed = ws.usage_cache.computations;
        for _ in 0..3 {
            ws.usage();
        }
        assert_eq!(ws.usage_cache.computations, computed, "nothing changed");
        ws.delete_swatch(red, 1.0);
        assert!(ws.usage().swatch(red).is_unused());
        assert_eq!(ws.usage_cache.computations, computed + 1);
        ws.undo();
        assert_eq!(ws.usage().swatch(red).objects, 2, "undo restores the count");
    }

    #[test]
    fn usage_labels() {
        let count = |objects, surfaces: &[usize]| Count {
            objects,
            surfaces: surfaces.to_vec(),
        };
        assert_eq!(
            usage_label(&count(38, &[0; 11])),
            "11 textures · 38 objects"
        );
        assert_eq!(usage_label(&count(1, &[3])), "1 texture · 1 object");
        assert_eq!(usage_label(&Count::default()), "Unused");
        assert_eq!(
            symbol_usage_label(&count(14, &[0; 9])),
            "14 instances · 9 textures"
        );
        assert_eq!(
            symbol_usage_label(&count(1, &[0])),
            "1 instance · 1 texture"
        );
        assert_eq!(
            impact_label(&count(38, &[0; 11])),
            "Impact: 11 textures, 38 objects"
        );
        assert_eq!(impact_label(&Count::default()), "Not used yet");
    }

    // ── Before/after editor ─────────────────────────────────────────────

    #[test]
    fn nothing_changes_before_apply() {
        let (mut ws, red, a, b) = fleet();
        let steps = ws.history.len();
        let before = ws.project.clone();
        ws.start_swatch_edit(red);
        let edit = ws.panels.brand_edit.as_ref().unwrap();
        assert_eq!(edit.affected, [0, 2]);
        assert_eq!(edit.impact.objects, 2);
        ws.set_edit_value(EditValue::Color(DARK_RED));
        assert_eq!(ws.project, before, "the document is untouched");
        assert_eq!(ws.history.len(), steps);
        let preview = ws
            .panels
            .brand_edit
            .as_ref()
            .unwrap()
            .preview
            .as_ref()
            .unwrap();
        let shown = |surface: usize, id| preview.surfaces[surface].get(id).unwrap().fill;
        assert_eq!(shown(0, a), Paint::Solid(DARK_RED));
        assert_eq!(shown(2, b), Paint::Solid(DARK_RED));
        // Apply to Fleet: one step, the links hold.
        assert!(ws.apply_brand_edit(2.0));
        assert!(ws.panels.brand_edit.is_none());
        assert_eq!(ws.history.len(), steps + 1);
        for (surface, id) in [(0, a), (2, b)] {
            let o = on(&ws, surface, id);
            assert_eq!(o.fill, Paint::Solid(DARK_RED));
            assert_eq!(o.fill_swatch, Some(red));
        }
        ws.undo();
        assert_eq!(on(&ws, 2, b).fill, Paint::Solid(RED));
        assert_eq!(ws.project.swatch(red).unwrap().color, RED);
    }

    #[test]
    fn cancel_records_nothing() {
        let (mut ws, red, a, _) = fleet();
        let steps = ws.history.len();
        ws.start_swatch_edit(red);
        ws.set_edit_value(EditValue::Color(BLUE));
        rename_edit(&mut ws, "Company blue");
        ws.cancel_brand_edit();
        assert!(ws.panels.brand_edit.is_none());
        assert_eq!(on(&ws, 0, a).fill, Paint::Solid(RED));
        assert_eq!(ws.project.swatch(red).unwrap().name, "Company red");
        assert_eq!(ws.history.len(), steps);
    }

    #[test]
    fn rename_only_and_unchanged_apply() {
        let (mut ws, red, ..) = fleet();
        let steps = ws.history.len();
        // Unchanged: closes, records nothing.
        ws.start_swatch_edit(red);
        assert!(ws.apply_brand_edit(1.0));
        assert_eq!(ws.history.len(), steps);
        // A value set back to the current one is no change either.
        ws.start_swatch_edit(red);
        ws.set_edit_value(EditValue::Color(BLUE));
        ws.set_edit_value(EditValue::Color(RED));
        assert!(ws.apply_brand_edit(2.0));
        assert_eq!(ws.history.len(), steps);
        // Empty name: refused, the editor stays open.
        ws.start_swatch_edit(red);
        rename_edit(&mut ws, "  ");
        assert!(!ws.apply_brand_edit(3.0));
        assert!(ws.panels.brand_edit.is_some());
        // Rename only.
        rename_edit(&mut ws, "Ardent red");
        assert!(ws.apply_brand_edit(4.0));
        assert_eq!(ws.history.len(), steps + 1);
        let s = ws.project.swatch(red).unwrap();
        assert_eq!((s.name.as_str(), s.color), ("Ardent red", RED));
        ws.undo();
        assert_eq!(ws.project.swatch(red).unwrap().name, "Company red");
    }

    #[test]
    fn a_style_name_taken_by_another_style_is_refused() {
        let (mut ws, _, a, b) = fleet();
        let (mut stripe, mut band) = (None, None);
        ws.edit("styles-new", 1.0, false, |p, _| {
            stripe = p.new_graphic_style(a, "Stripe");
            p.active_surface = 2;
            band = p.new_graphic_style(b, "Band");
            p.active_surface = 0;
        });
        let (stripe, band) = (stripe.unwrap(), band.unwrap());
        let band_name = ws.project.graphic_style(band).unwrap().name.clone();
        let steps = ws.history.len();
        ws.start_style_edit(stripe);
        rename_edit(&mut ws, &band_name);
        let edit = ws.panels.brand_edit.as_ref().unwrap();
        assert_eq!(edit.problem(&ws.project), Some("styles-name-taken"));
        assert!(!ws.apply_brand_edit(2.0));
        assert!(ws.panels.brand_edit.is_some(), "the editor stays open");
        assert_eq!(ws.history.len(), steps);
        rename_edit(&mut ws, "");
        let edit = ws.panels.brand_edit.as_ref().unwrap();
        assert_eq!(edit.problem(&ws.project), Some("styles-name-empty"));
    }

    #[test]
    fn a_style_edit_changes_followers_on_two_textures() {
        let (mut ws, red, a, b) = fleet();
        let mut style = None;
        ws.edit("styles-new", 1.0, false, |p, _| {
            style = p.new_graphic_style(a, "Stripe");
        });
        let style = style.unwrap();
        ws.project.active_surface = 2;
        ws.selection = vec![b];
        ws.apply_style(style, 2.0);
        ws.project.active_surface = 0;
        let steps = ws.history.len();
        ws.start_style_edit(style);
        assert_eq!(ws.panels.brand_edit.as_ref().unwrap().affected, [0, 2]);
        let mut look = ws.project.graphic_style(style).unwrap().look;
        look.stroke = Some(tp_core::document::StrokeStyle {
            width: 8.0,
            ..Default::default()
        });
        // A fill color that is not the swatch's drops the link.
        look.fill = Paint::Solid(BLUE);
        ws.set_edit_value(EditValue::Look(look));
        let EditValue::Look(after) = ws.panels.brand_edit.as_ref().unwrap().after else {
            panic!("a look");
        };
        assert_eq!(after.fill_swatch, None);
        assert_eq!(on(&ws, 0, a).fill_swatch, Some(red), "untouched");
        assert!(ws.apply_brand_edit(3.0));
        assert_eq!(ws.history.len(), steps + 1);
        for (surface, id) in [(0, a), (2, b)] {
            let o = on(&ws, surface, id);
            assert_eq!(o.stroke.unwrap().width, 8.0);
            assert_eq!(o.fill, Paint::Solid(BLUE));
            assert_eq!(o.style, Some(style), "still follows");
        }
        ws.undo();
        assert_eq!(on(&ws, 2, b).stroke, None);
        assert_eq!(on(&ws, 2, b).fill_swatch, Some(red));
        // Text styles have no editor.
        assert!(ws.project.text_style(style).is_none());
    }

    #[test]
    fn showing_another_space_cancels_the_editor() {
        let (mut ws, red, a, _) = fleet();
        ws.space = Space::Brand;
        let steps = ws.history.len();
        ws.start_swatch_edit(red);
        ws.set_edit_value(EditValue::Color(DARK_RED));
        ws.show_space(Space::Brand);
        assert!(ws.panels.brand_edit.is_some(), "same space");
        ws.show_space(Space::Workshop);
        assert!(ws.panels.brand_edit.is_none());
        assert_eq!(on(&ws, 0, a).fill, Paint::Solid(RED));
        assert_eq!(ws.history.len(), steps);
        // Whatever shows another space, the next frame cancels it.
        ws.start_swatch_edit(red);
        ws.space = Space::Brand;
        ws.update_brand_edit(&egui::Context::default());
        assert!(ws.panels.brand_edit.is_none());
    }

    // ── Preview thumbnails ──────────────────────────────────────────────

    /// Runs frames until the preview thumbnails are rendered.
    fn settle(ws: &mut Workspace, ctx: &egui::Context) {
        for _ in 0..500 {
            ws.update_brand_edit(ctx);
            if !ws.preview_thumbs.is_rendering() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("the preview thumbnails never finished");
    }

    fn settle_own(ws: &mut Workspace, ctx: &egui::Context) {
        for _ in 0..500 {
            ws.thumbnails.update(ctx, &ws.project, &ws.text.fonts);
            if !ws.thumbnails.is_rendering() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("the thumbnails never finished");
    }

    #[test]
    fn previews_render_the_affected_textures_only() {
        let ctx = egui::Context::default();
        let (mut ws, red, ..) = fleet();
        settle_own(&mut ws, &ctx);
        let own = ws.thumbnails.renders;
        ws.start_swatch_edit(red);
        settle(&mut ws, &ctx);
        assert_eq!(ws.preview_thumbs.renders, 0, "nothing before a value");
        ws.set_edit_value(EditValue::Color(DARK_RED));
        settle(&mut ws, &ctx);
        assert_eq!(ws.preview_thumbs.renders, 2, "the two affected textures");
        assert!(ws.preview_thumbs.texture(1).is_none(), "never rendered");
        assert_eq!(ws.preview_thumbs.color_at(2, [0.5, 0.5]), Some(DARK_RED));
        // The same value: nothing rendered again.
        ws.set_edit_value(EditValue::Color(DARK_RED));
        settle(&mut ws, &ctx);
        assert_eq!(ws.preview_thumbs.renders, 2);
        ws.set_edit_value(EditValue::Color(BLUE));
        settle(&mut ws, &ctx);
        assert_eq!(ws.preview_thumbs.renders, 4);
        assert_eq!(ws.preview_thumbs.color_at(0, [0.5, 0.5]), Some(BLUE));
        // The workspace's own thumbnails are not rendered again.
        settle_own(&mut ws, &ctx);
        assert_eq!(ws.thumbnails.renders, own);
        assert_eq!(ws.thumbnails.color_at(0, [0.5, 0.5]), Some(RED));
        // A new edit starts with no preview.
        ws.cancel_brand_edit();
        ws.start_swatch_edit(red);
        assert_eq!(ws.preview_thumbs.renders, 0);
        assert!(ws.preview_thumbs.texture(0).is_none());
    }

    /// `set_edit_value` on a 40-texture fleet (the sample truck's textures
    /// duplicated), one swatch linked on all: well under a frame.
    #[test]
    #[cfg(not(debug_assertions))]
    fn editing_a_value_on_a_large_fleet_is_fast() {
        let package = tp_vehicles::Package::read(crate::vehicles::SAMPLES[0].bytes).unwrap();
        let textures = crate::vehicle_project::default_textures(&package.manifest);
        let mut p = crate::vehicle_project::fleet_project("Fleet", &package, &textures).unwrap();
        let base = p.surfaces.clone();
        while p.surfaces.len() < 40 {
            p.surfaces.push(base[p.surfaces.len() % base.len()].clone());
        }
        let mut ws = Workspace::new(p);
        let (red, _) = ws.project.add_swatch(RED, "Color");
        for surface in 0..40 {
            for i in 0..30 {
                let id = linked_on(&mut ws, surface, RED, red);
                if i % 3 != 0 {
                    let mut o = on(&ws, surface, id);
                    o.fill_swatch = None;
                    ws.project.active_surface = surface;
                    ws.project.surface_mut().replace(&[o]);
                    ws.project.active_surface = 0;
                }
            }
        }
        ws.start_swatch_edit(red);
        assert_eq!(ws.panels.brand_edit.as_ref().unwrap().affected.len(), 40);
        let calls = 20;
        let start = std::time::Instant::now();
        for k in 0..calls {
            ws.set_edit_value(EditValue::Color(Rgba::rgb(k as u8 * 10, 0, 0)));
        }
        let each = start.elapsed() / calls;
        eprintln!("set_edit_value on 40 textures: {each:?} per call");
        assert!(each < std::time::Duration::from_millis(5), "{each:?}");
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
