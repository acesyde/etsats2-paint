//! The brand kit of a project: palette swatches and shared styles, and the
//! links that make colors and objects follow them.
//!
//! Objects keep their resolved colors and character settings; a link only
//! says where a value comes from. A link holds while the value equals its
//! source: [`Project::relink`] drops the others after every edit, so tools
//! that change a look detach it without knowing about links.

use std::collections::HashMap;
use std::sync::Arc;

use crate::document::{
    CharStyle, Object, ObjectId, Paint, Rgba, ShapeKind, StrokeStyle, StyleId, SwatchId, tree,
};
use crate::import::LibraryKey;
use crate::project::Project;

/// A named palette color.
#[derive(Clone, Debug, PartialEq)]
pub struct Swatch {
    pub id: SwatchId,
    pub name: String,
    pub color: Rgba,
    /// The library entry it came from or was added to (not part of the
    /// undo history).
    pub origin: Option<LibraryKey>,
}

/// A fill, a stroke and an opacity: what a style gives an object besides
/// character settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub fill: Paint,
    pub fill_swatch: Option<SwatchId>,
    pub stroke: Option<StrokeStyle>,
    pub opacity: f32,
}

/// A named look for shapes and texts.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphicStyle {
    pub id: StyleId,
    pub name: String,
    pub look: Look,
    /// The library entry it came from or was added to.
    pub origin: Option<LibraryKey>,
}

/// A named lettering: character settings and look.
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub id: StyleId,
    pub name: String,
    pub style: CharStyle,
    pub look: Look,
    /// The library entry it came from or was added to.
    pub origin: Option<LibraryKey>,
}

/// The palette and the shared styles, as stored.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BrandKit {
    pub palette: Vec<Swatch>,
    pub graphic_styles: Vec<GraphicStyle>,
    pub text_styles: Vec<TextStyle>,
}

impl BrandKit {
    /// Whether both hold the same swatches and styles, library origins
    /// aside (they are not part of the undo history).
    pub(crate) fn same_content(&self, other: &BrandKit) -> bool {
        fn same<T>(a: &[T], b: &[T], eq: impl Fn(&T, &T) -> bool) -> bool {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| eq(x, y))
        }
        same(&self.palette, &other.palette, |a, b| {
            a.id == b.id && a.name == b.name && a.color == b.color
        }) && same(&self.graphic_styles, &other.graphic_styles, |a, b| {
            a.id == b.id && a.name == b.name && a.look == b.look
        }) && same(&self.text_styles, &other.text_styles, |a, b| {
            a.id == b.id && a.name == b.name && a.style == b.style && a.look == b.look
        })
    }

    /// The library origins of the swatches and styles, in order.
    pub(crate) fn origins(&self) -> impl Iterator<Item = Option<&LibraryKey>> {
        let palette = self.palette.iter().map(|s| s.origin.as_ref());
        let graphic = self.graphic_styles.iter().map(|s| s.origin.as_ref());
        let text = self.text_styles.iter().map(|s| s.origin.as_ref());
        palette.chain(graphic).chain(text)
    }
}

/// Whether two paints are the same, a gradient's position aside: a style
/// gives a gradient's kind and stops, its position stays the object's.
fn same_paint(a: &Paint, b: &Paint) -> bool {
    match (a, b) {
        (Paint::Solid(x), Paint::Solid(y)) => x == y,
        (Paint::Gradient(g), Paint::Gradient(h)) => g.kind == h.kind && g.stops() == h.stops(),
        _ => false,
    }
}

/// `style`'s paint, keeping `own`'s gradient position when both are
/// gradients of the same kind.
fn take_paint(own: &Paint, style: &Paint) -> Paint {
    match (own, style) {
        (Paint::Gradient(o), Paint::Gradient(s)) if o.kind == s.kind => {
            let mut g = *s;
            g.start = o.start;
            g.end = o.end;
            g.minor = o.minor;
            Paint::Gradient(g)
        }
        _ => *style,
    }
}

fn same_stroke(a: Option<&StrokeStyle>, b: Option<&StrokeStyle>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            same_paint(&a.paint, &b.paint)
                && a.width == b.width
                && a.align == b.align
                && a.line == b.line
                && a.swatch == b.swatch
        }
        _ => false,
    }
}

/// Whether an object can follow a graphic style (shapes and texts).
fn takes_graphic_style(o: &Object) -> bool {
    !matches!(
        o.kind,
        ShapeKind::Group | ShapeKind::Image { .. } | ShapeKind::Instance { .. }
    )
}

impl Look {
    /// `o`'s look.
    pub fn of(o: &Object) -> Self {
        Self {
            fill: o.fill,
            fill_swatch: o.fill_swatch,
            stroke: o.stroke,
            opacity: o.opacity,
        }
    }

    /// Whether `o`'s look is this one (gradient positions aside).
    pub fn matches(&self, o: &Object) -> bool {
        same_paint(&self.fill, &o.fill)
            && self.fill_swatch == o.fill_swatch
            && same_stroke(self.stroke.as_ref(), o.stroke.as_ref())
            && self.opacity == o.opacity
    }

    /// Gives `o` this look, keeping its own gradient positions.
    fn apply_to(&self, o: &mut Object) {
        o.fill = take_paint(&o.fill, &self.fill);
        o.fill_swatch = self.fill_swatch;
        o.stroke = self.stroke.map(|s| match &o.stroke {
            Some(own) => StrokeStyle {
                paint: take_paint(&own.paint, &s.paint),
                ..s
            },
            None => s,
        });
        o.opacity = self.opacity;
    }

    /// Recolors the parts linked to swatch `id`.
    pub(crate) fn recolor(&mut self, id: SwatchId, color: Rgba) {
        recolor(&mut self.fill, self.fill_swatch, id, color);
        if let Some(s) = &mut self.stroke {
            let link = s.swatch;
            recolor(&mut s.paint, link, id, color);
        }
    }

    /// Drops the swatch links that no longer match.
    fn unlink(&mut self, swatches: &HashMap<SwatchId, Rgba>) {
        unlink_paint(&mut self.fill, &mut self.fill_swatch, swatches);
        if let Some(s) = &mut self.stroke {
            let mut link = s.swatch;
            unlink_paint(&mut s.paint, &mut link, swatches);
            s.swatch = link;
        }
    }
}

impl GraphicStyle {
    /// Whether `o`'s look is this style's (gradient positions aside).
    pub fn matches(&self, o: &Object) -> bool {
        self.look.matches(o)
    }

    /// Gives `o` this style's look and makes it follow the style (a text
    /// then follows no text style).
    fn apply_to(&self, o: &mut Object) {
        self.look.apply_to(o);
        o.style = Some(self.id);
        if let Some(t) = &mut o.text {
            t.style_id = None;
        }
    }

    pub fn from_object(id: StyleId, name: String, o: &Object) -> Self {
        Self {
            id,
            name,
            look: Look::of(o),
            origin: None,
        }
    }
}

impl TextStyle {
    /// Whether text `o` has this style's character settings and look.
    pub fn matches(&self, o: &Object) -> bool {
        o.text.as_ref().is_some_and(|t| t.style == self.style) && self.look.matches(o)
    }

    /// Gives text `o` this style's lettering and makes it follow the style
    /// (it then follows no graphic style). Returns false for a non-text.
    fn apply_to(&self, o: &mut Object) -> bool {
        let Some(t) = &mut o.text else {
            return false;
        };
        t.style = self.style.clone();
        t.style_id = Some(self.id);
        self.look.apply_to(o);
        o.style = None;
        true
    }

    /// The lettering of text `o`.
    pub fn from_object(id: StyleId, name: String, o: &Object) -> Option<Self> {
        Some(Self {
            id,
            name,
            style: o.text.as_ref()?.style.clone(),
            look: Look::of(o),
            origin: None,
        })
    }
}

/// Recolors the paint parts linked to `id`; returns whether anything
/// changed.
fn recolor(paint: &mut Paint, link: Option<SwatchId>, id: SwatchId, color: Rgba) -> bool {
    let mut changed = false;
    match paint {
        Paint::Solid(c) => {
            if link == Some(id) && *c != color {
                *c = color;
                changed = true;
            }
        }
        Paint::Gradient(g) => {
            for stop in g.stops_mut() {
                if stop.swatch == Some(id) && stop.color != color {
                    stop.color = color;
                    changed = true;
                }
            }
        }
    }
    changed
}

/// Drops the swatch links of a paint that no longer match; returns
/// whether anything changed.
fn unlink_paint(
    paint: &mut Paint,
    link: &mut Option<SwatchId>,
    swatches: &HashMap<SwatchId, Rgba>,
) -> bool {
    let mut changed = false;
    if let Some(id) = *link {
        let keep = matches!(paint, Paint::Solid(c) if swatches.get(&id) == Some(c));
        if !keep {
            *link = None;
            changed = true;
        }
    }
    if let Paint::Gradient(g) = paint {
        for stop in g.stops_mut() {
            if let Some(id) = stop.swatch
                && swatches.get(&id) != Some(&stop.color)
            {
                stop.swatch = None;
                changed = true;
            }
        }
    }
    changed
}

/// Runs `f` on every object of `list` and their descendants (children
/// first). An object is replaced only when `f` or a descendant changed it,
/// so unchanged objects keep their `Arc`. An instance's children, its
/// expanded content, are left out: they follow its symbol.
pub(crate) fn update_tree(
    list: &mut [Arc<Object>],
    f: &mut dyn FnMut(&mut Object) -> bool,
) -> bool {
    let mut any = false;
    for arc in list.iter_mut() {
        let mut o = (**arc).clone();
        let mut changed = !o.is_instance() && update_tree(&mut o.children, f);
        changed |= f(&mut o);
        if changed {
            *arc = Arc::new(o);
            any = true;
        }
    }
    any
}

/// Runs `f` on every object of the textures and of the symbols' content.
fn update_surfaces(project: &mut Project, f: &mut dyn FnMut(&mut Object) -> bool) {
    let symbols = project.symbols.iter_mut().map(|s| &mut s.surface);
    for s in project.surfaces.iter_mut().chain(symbols) {
        update_tree(&mut s.objects, f);
    }
}

/// The first "`prefix` N" not in `names`.
pub(crate) fn numbered_name<'a>(
    prefix: &str,
    names: impl Iterator<Item = &'a str> + Clone,
) -> String {
    (1..)
        .map(|n| format!("{prefix} {n}"))
        .find(|candidate| !names.clone().any(|n| n == candidate))
        .expect("an unused number")
}

/// The ids of `ids` and of their descendants, as objects of `list`.
fn with_descendants(list: &[Arc<Object>], ids: &[ObjectId]) -> Vec<ObjectId> {
    fn walk(o: &Object, out: &mut Vec<ObjectId>) {
        out.push(o.id);
        if o.is_instance() {
            return;
        }
        for c in &o.children {
            walk(c, out);
        }
    }
    let mut out = Vec::new();
    for id in ids {
        if let Some(o) = tree::get(list, *id) {
            walk(o, &mut out);
        }
    }
    out
}

impl Project {
    /// The palette and the shared styles.
    pub fn brand_kit(&self) -> BrandKit {
        BrandKit {
            palette: self.palette.clone(),
            graphic_styles: self.graphic_styles.clone(),
            text_styles: self.text_styles.clone(),
        }
    }

    pub fn swatch(&self, id: SwatchId) -> Option<&Swatch> {
        self.palette.iter().find(|s| s.id == id)
    }

    /// Adds a swatch of `color` named "`prefix` N"; when a swatch already
    /// has that color, returns it instead. Returns the swatch and whether
    /// it was added.
    pub fn add_swatch(&mut self, color: Rgba, prefix: &str) -> (SwatchId, bool) {
        if let Some(s) = self.palette.iter().find(|s| s.color == color) {
            return (s.id, false);
        }
        let id = SwatchId(self.fresh_id());
        let name = numbered_name(prefix, self.palette.iter().map(|s| s.name.as_str()));
        self.palette.push(Swatch {
            id,
            name,
            color,
            origin: None,
        });
        (id, true)
    }

    /// Renames a swatch; refuses an empty name.
    pub fn rename_swatch(&mut self, id: SwatchId, name: &str) -> bool {
        let name = name.trim();
        match self.palette.iter_mut().find(|s| s.id == id) {
            Some(s) if !name.is_empty() => {
                s.name = name.to_owned();
                true
            }
            _ => false,
        }
    }

    /// Deletes a swatch: every color keeps its value and stops being
    /// linked to it.
    pub fn delete_swatch(&mut self, id: SwatchId) {
        self.palette.retain(|s| s.id != id);
        self.relink();
    }

    /// Changes a swatch's color and every color linked to it: on every
    /// surface and in the graphic styles.
    pub fn set_swatch_color(&mut self, id: SwatchId, color: Rgba) {
        let Some(swatch) = self.palette.iter_mut().find(|s| s.id == id) else {
            return;
        };
        swatch.color = color;
        for style in &mut self.graphic_styles {
            style.look.recolor(id, color);
        }
        for style in &mut self.text_styles {
            style.look.recolor(id, color);
        }
        update_surfaces(self, &mut |o| {
            let mut changed = recolor(&mut o.fill, o.fill_swatch, id, color);
            if let Some(s) = &mut o.stroke {
                let link = s.swatch;
                changed |= recolor(&mut s.paint, link, id, color);
            }
            changed
        });
        self.refresh_instances();
    }

    /// Drops every link whose value no longer matches its source, or whose
    /// source no longer exists: swatch links of colors, and graphic and
    /// text style links of objects.
    pub fn relink(&mut self) {
        let swatches: HashMap<SwatchId, Rgba> =
            self.palette.iter().map(|s| (s.id, s.color)).collect();
        for style in &mut self.graphic_styles {
            style.look.unlink(&swatches);
        }
        for style in &mut self.text_styles {
            style.look.unlink(&swatches);
        }
        let graphic: HashMap<StyleId, GraphicStyle> = self
            .graphic_styles
            .iter()
            .map(|s| (s.id, s.clone()))
            .collect();
        let text: HashMap<StyleId, TextStyle> =
            self.text_styles.iter().map(|s| (s.id, s.clone())).collect();
        update_surfaces(self, &mut |o| {
            let mut changed = unlink_paint(&mut o.fill, &mut o.fill_swatch, &swatches);
            if let Some(s) = &mut o.stroke {
                let mut link = s.swatch;
                changed |= unlink_paint(&mut s.paint, &mut link, &swatches);
                if link != s.swatch {
                    s.swatch = link;
                    changed = true;
                }
            }
            if let Some(id) = o.text.as_ref().and_then(|t| t.style_id)
                && !text.get(&id).is_some_and(|s| s.matches(o))
            {
                if let Some(t) = &mut o.text {
                    t.style_id = None;
                }
                changed = true;
            }
            // A text follows one style: its text style wins.
            let follows_text = o.text.as_ref().is_some_and(|t| t.style_id.is_some());
            if let Some(id) = o.style {
                let keep = !follows_text
                    && takes_graphic_style(o)
                    && graphic.get(&id).is_some_and(|s| s.matches(o));
                if !keep {
                    o.style = None;
                    changed = true;
                }
            }
            changed
        });
        // Symbol content may have changed links: instances follow.
        self.refresh_instances();
    }

    pub fn graphic_style(&self, id: StyleId) -> Option<&GraphicStyle> {
        self.graphic_styles.iter().find(|s| s.id == id)
    }

    pub fn text_style(&self, id: StyleId) -> Option<&TextStyle> {
        self.text_styles.iter().find(|s| s.id == id)
    }

    /// Creates a graphic style named "`prefix` N" from the look of object
    /// `from` of the active surface, which then follows it.
    pub fn new_graphic_style(&mut self, from: ObjectId, prefix: &str) -> Option<StyleId> {
        let o = (**tree::get(&self.surface().objects, from)?).clone();
        if !takes_graphic_style(&o) {
            return None;
        }
        let id = StyleId(self.fresh_id());
        let name = numbered_name(prefix, self.graphic_styles.iter().map(|s| s.name.as_str()));
        let style = GraphicStyle::from_object(id, name, &o);
        self.graphic_styles.push(style);
        self.apply_graphic_style(id, &[from]);
        Some(id)
    }

    /// Creates a text style named "`prefix` N" from the lettering
    /// (character settings and look) of text `from` of the active surface,
    /// which then follows it.
    pub fn new_text_style(&mut self, from: ObjectId, prefix: &str) -> Option<StyleId> {
        let o = (**tree::get(&self.surface().objects, from)?).clone();
        o.text.as_ref()?;
        let id = StyleId(self.fresh_id());
        let name = numbered_name(prefix, self.text_styles.iter().map(|s| s.name.as_str()));
        let style = TextStyle::from_object(id, name, &o)?;
        self.text_styles.push(style);
        self.apply_text_style(id, &[from]);
        Some(id)
    }

    /// Gives the shapes and texts among `ids` of the active surface, and
    /// inside those that are groups, graphic style `id`'s look; they
    /// follow it. Images and groups themselves are left out.
    pub fn apply_graphic_style(&mut self, id: StyleId, ids: &[ObjectId]) {
        let Some(style) = self.graphic_style(id).cloned() else {
            return;
        };
        let targets = with_descendants(&self.surface().objects, ids);
        update_tree(&mut self.surface_mut().objects, &mut |o| {
            if !targets.contains(&o.id) || !takes_graphic_style(o) {
                return false;
            }
            let before = o.clone();
            style.apply_to(o);
            *o != before
        });
    }

    /// Gives the texts among `ids` of the active surface, and inside those
    /// that are groups, text style `id`'s lettering; they follow it.
    pub fn apply_text_style(&mut self, id: StyleId, ids: &[ObjectId]) {
        let Some(style) = self.text_style(id).cloned() else {
            return;
        };
        let targets = with_descendants(&self.surface().objects, ids);
        update_tree(&mut self.surface_mut().objects, &mut |o| {
            if !targets.contains(&o.id) {
                return false;
            }
            let before = o.clone();
            style.apply_to(o) && *o != before
        });
    }

    /// Gives graphic style `id` the look of object `from` of the active
    /// surface; every object following it, on every surface, changes too.
    pub fn redefine_graphic_style(&mut self, id: StyleId, from: ObjectId) -> bool {
        let Some(o) = tree::get(&self.surface().objects, from).map(|o| (**o).clone()) else {
            return false;
        };
        if !takes_graphic_style(&o) {
            return false;
        }
        let Some(style) = self.graphic_styles.iter_mut().find(|s| s.id == id) else {
            return false;
        };
        style.look = Look::of(&o);
        let style = style.clone();
        update_surfaces(self, &mut |o| {
            if o.style != Some(id) && o.id != from {
                return false;
            }
            let before = o.clone();
            style.apply_to(o);
            *o != before
        });
        self.refresh_instances();
        true
    }

    /// Gives text style `id` the lettering of text `from` of the active
    /// surface; every text following it, on every surface, changes too.
    pub fn redefine_text_style(&mut self, id: StyleId, from: ObjectId) -> bool {
        let Some(o) = tree::get(&self.surface().objects, from).map(|o| (**o).clone()) else {
            return false;
        };
        let Some(style) = self.text_styles.iter_mut().find(|s| s.id == id) else {
            return false;
        };
        let Some(mut new) = TextStyle::from_object(id, style.name.clone(), &o) else {
            return false;
        };
        new.origin = style.origin.take();
        *style = new.clone();
        update_surfaces(self, &mut |o| {
            let follows = o.id == from || o.text.as_ref().is_some_and(|t| t.style_id == Some(id));
            if !follows {
                return false;
            }
            let before = o.clone();
            new.apply_to(o) && *o != before
        });
        self.refresh_instances();
        true
    }

    /// Renames a style; refuses an empty name, or one another style of the
    /// same kind has.
    pub fn rename_style(&mut self, id: StyleId, name: &str) -> bool {
        let name = name.trim();
        if name.is_empty() {
            return false;
        }
        if self.graphic_styles.iter().any(|s| s.id == id) {
            if self
                .graphic_styles
                .iter()
                .any(|s| s.id != id && s.name == name)
            {
                return false;
            }
            if let Some(s) = self.graphic_styles.iter_mut().find(|s| s.id == id) {
                s.name = name.to_owned();
            }
            return true;
        }
        if self
            .text_styles
            .iter()
            .any(|s| s.id != id && s.name == name)
        {
            return false;
        }
        match self.text_styles.iter_mut().find(|s| s.id == id) {
            Some(s) => {
                s.name = name.to_owned();
                true
            }
            None => false,
        }
    }

    /// Gives `o` and its descendants the current values of the swatches
    /// and styles they link to, so the links hold.
    pub(crate) fn follow_links(&self, o: &mut Object) {
        if let Some(id) = o.text.as_ref().and_then(|t| t.style_id)
            && let Some(style) = self.text_style(id)
        {
            style.apply_to(o);
        } else if let Some(id) = o.style
            && let Some(style) = self.graphic_style(id)
            && takes_graphic_style(o)
        {
            style.apply_to(o);
        }
        for swatch in &self.palette {
            recolor(&mut o.fill, o.fill_swatch, swatch.id, swatch.color);
            if let Some(s) = &mut o.stroke {
                let link = s.swatch;
                recolor(&mut s.paint, link, swatch.id, swatch.color);
            }
        }
        // An instance's content follows its symbol.
        if !o.is_instance() {
            for child in &mut o.children {
                self.follow_links(Arc::make_mut(child));
            }
        }
    }

    /// Gives every object following style `id` (graphic or text), on every
    /// surface and in the symbols, the style's current look.
    pub(crate) fn reapply_style(&mut self, id: StyleId) {
        let graphic = self.graphic_style(id).cloned();
        let text = self.text_style(id).cloned();
        update_surfaces(self, &mut |o| {
            let before = o.clone();
            if let Some(style) = &text
                && o.text.as_ref().is_some_and(|t| t.style_id == Some(id))
            {
                style.apply_to(o);
            } else if let Some(style) = &graphic
                && o.style == Some(id)
            {
                style.apply_to(o);
            }
            *o != before
        });
        self.refresh_instances();
    }

    /// Deletes a style: objects keep their look and stop following it.
    pub fn delete_style(&mut self, id: StyleId) {
        self.graphic_styles.retain(|s| s.id != id);
        self.text_styles.retain(|s| s.id != id);
        self.relink();
    }

    /// Objects of surface `index` that follow style `id` (graphic or text).
    pub fn style_users(&self, id: StyleId, index: usize) -> Vec<ObjectId> {
        fn walk(list: &[Arc<Object>], id: StyleId, out: &mut Vec<ObjectId>) {
            for o in list {
                if o.style == Some(id) || o.text.as_ref().is_some_and(|t| t.style_id == Some(id)) {
                    out.push(o.id);
                }
                if !o.is_instance() {
                    walk(&o.children, id, out);
                }
            }
        }
        let mut out = Vec::new();
        if let Some(s) = self.surfaces.get(index) {
            walk(&s.objects, id, &mut out);
        }
        out
    }

    /// The graphic and text styles followed by `ids` (and the objects
    /// inside them) on the active surface.
    pub fn styles_of(&self, ids: &[ObjectId]) -> Vec<StyleId> {
        let list = &self.surface().objects;
        let mut out = Vec::new();
        for id in with_descendants(list, ids) {
            if let Some(o) = tree::get(list, id) {
                for s in [o.style, o.text.as_ref().and_then(|t| t.style_id)]
                    .into_iter()
                    .flatten()
                {
                    if !out.contains(&s) {
                        out.push(s);
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests;
