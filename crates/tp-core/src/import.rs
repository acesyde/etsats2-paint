//! Bringing elements from one project into another: the personal library,
//! Add to Library and paste across projects all copy symbols, swatches,
//! styles and objects with everything they use.
//!
//! The library is a [`Project`] without vehicles, so one function serves
//! every direction. Each element is reused when the target already has it
//! (the same library entry, or a swatch or style with the same name and
//! value), else added under a new id and a free name; links are remapped to
//! the target's elements.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::brand::{GraphicStyle, Look, Swatch, TextStyle};
use crate::document::{
    AssetId, Object, Paint, ShapeKind, StrokeStyle, StyleId, SwatchId, SymbolId,
};
use crate::project::{Project, Surface};
use crate::symbols::Symbol;

/// Identifies a library entry for good: 32 hex characters.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LibraryKey(pub String);

/// What to bring from a project.
#[derive(Clone, Debug, Default)]
pub struct Picks {
    pub symbols: Vec<SymbolId>,
    pub swatches: Vec<SwatchId>,
    pub graphic_styles: Vec<StyleId>,
    pub text_styles: Vec<StyleId>,
    /// Objects being pasted, as copied from the project.
    pub objects: Vec<Object>,
}

impl Picks {
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
            && self.swatches.is_empty()
            && self.graphic_styles.is_empty()
            && self.text_styles.is_empty()
            && self.objects.is_empty()
    }
}

/// The elements picks use, themselves included, in their project's order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Closure {
    pub symbols: Vec<SymbolId>,
    pub swatches: Vec<SwatchId>,
    pub graphic_styles: Vec<StyleId>,
    pub text_styles: Vec<StyleId>,
    pub assets: Vec<AssetId>,
}

/// Ids referenced, before they are sorted out.
#[derive(Default)]
struct Found {
    symbols: HashSet<SymbolId>,
    swatches: HashSet<SwatchId>,
    styles: HashSet<StyleId>,
    assets: HashSet<AssetId>,
}

impl Found {
    fn paint(&mut self, paint: &Paint, link: Option<SwatchId>) {
        self.swatches.extend(link);
        if let Paint::Gradient(g) = paint {
            self.swatches
                .extend(g.stops().iter().filter_map(|s| s.swatch));
        }
    }

    fn stroke(&mut self, stroke: Option<&StrokeStyle>) {
        if let Some(s) = stroke {
            self.paint(&s.paint, s.swatch);
        }
    }

    fn look(&mut self, look: &Look) {
        self.paint(&look.fill, look.fill_swatch);
        self.stroke(look.stroke.as_ref());
    }

    fn object(&mut self, o: &Object) {
        self.paint(&o.fill, o.fill_swatch);
        self.stroke(o.stroke.as_ref());
        self.styles.extend(o.style);
        self.styles.extend(o.text.as_ref().and_then(|t| t.style_id));
        match o.kind {
            ShapeKind::Image { asset } => {
                self.assets.insert(asset);
            }
            // An instance's content is its symbol's.
            ShapeKind::Instance { symbol, .. } => {
                self.symbols.insert(symbol);
                return;
            }
            _ => {}
        }
        for c in &o.children {
            self.object(c);
        }
    }
}

/// The symbols, swatches, styles and assets `picks` use in `from`, picks
/// included: through objects, symbol content and style looks.
pub fn closure(from: &Project, picks: &Picks) -> Closure {
    let mut found = Found::default();
    found.symbols.extend(&picks.symbols);
    found.swatches.extend(&picks.swatches);
    found.styles.extend(&picks.graphic_styles);
    found.styles.extend(&picks.text_styles);
    for o in &picks.objects {
        found.object(o);
    }
    // Symbols don't nest today; walking until nothing new shows up keeps
    // this right if they ever do.
    let mut walked = HashSet::new();
    loop {
        let todo: Vec<SymbolId> = found.symbols.difference(&walked).copied().collect();
        if todo.is_empty() {
            break;
        }
        for id in todo {
            walked.insert(id);
            if let Some(s) = from.symbol(id) {
                for o in &s.surface.objects {
                    found.object(o);
                }
            }
        }
    }
    for s in &from.graphic_styles {
        if found.styles.contains(&s.id) {
            found.look(&s.look);
        }
    }
    for s in &from.text_styles {
        if found.styles.contains(&s.id) {
            found.look(&s.look);
        }
    }
    Closure {
        symbols: from
            .symbols
            .iter()
            .map(|s| s.id)
            .filter(|id| found.symbols.contains(id))
            .collect(),
        swatches: from
            .palette
            .iter()
            .map(|s| s.id)
            .filter(|id| found.swatches.contains(id))
            .collect(),
        graphic_styles: from
            .graphic_styles
            .iter()
            .map(|s| s.id)
            .filter(|id| found.styles.contains(id))
            .collect(),
        text_styles: from
            .text_styles
            .iter()
            .map(|s| s.id)
            .filter(|id| found.styles.contains(id))
            .collect(),
        assets: from
            .assets
            .keys()
            .filter(|id| found.assets.contains(id))
            .copied()
            .collect(),
    }
}

/// How [`import`] treats an element the target already has.
pub enum ImportMode<'a> {
    /// Leave it unchanged (Import from Library, Paste).
    Import,
    /// Add to Library: an element reused by its library entry gets the
    /// source's content and name; every element brought ends up with a
    /// library key, new ones from `new_key`.
    Publish(&'a mut dyn FnMut() -> LibraryKey),
}

/// What [`import`] did.
#[derive(Clone, Debug, Default)]
pub struct Imported {
    /// Source id → target id, for every element brought.
    pub swatches: HashMap<SwatchId, SwatchId>,
    pub styles: HashMap<StyleId, StyleId>,
    pub symbols: HashMap<SymbolId, SymbolId>,
    pub assets: HashMap<AssetId, AssetId>,
    /// The picked objects, linked to the target's elements (their ids are
    /// the source's: adding them gives fresh ones).
    pub objects: Vec<Object>,
    /// Publish: the library key of each source element brought, by id, to
    /// record in the source.
    pub origins: Vec<(u64, LibraryKey)>,
    /// Swatches, styles and symbols added to the target.
    pub added: usize,
}

/// `name`, or the first free "`name` N" (from 2) when `taken`.
fn unique_name(name: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(name) {
        return name.to_owned();
    }
    (2..)
        .map(|n| format!("{name} {n}"))
        .find(|n| !taken(n))
        .expect("a free name")
}

/// Points a paint's swatch links to the target's swatches (links to
/// swatches not brought are dropped).
fn remap_paint(paint: &mut Paint, link: &mut Option<SwatchId>, map: &HashMap<SwatchId, SwatchId>) {
    *link = link.and_then(|id| map.get(&id).copied());
    if let Paint::Gradient(g) = paint {
        for stop in g.stops_mut() {
            stop.swatch = stop.swatch.and_then(|id| map.get(&id).copied());
        }
    }
}

/// `look` linked to the target's swatches, with their colors.
fn remap_look(look: &Look, map: &HashMap<SwatchId, SwatchId>, into: &Project) -> Look {
    let mut look = *look;
    remap_paint(&mut look.fill, &mut look.fill_swatch, map);
    if let Some(s) = &mut look.stroke {
        remap_paint(&mut s.paint, &mut s.swatch, map);
    }
    for swatch in &into.palette {
        look.recolor(swatch.id, swatch.color);
    }
    look
}

/// Points `o`'s links (images, swatches, styles, symbols) to the target's
/// elements. An instance of a symbol not brought becomes a group.
fn remap_object(o: &mut Object, done: &Imported) {
    remap_paint(&mut o.fill, &mut o.fill_swatch, &done.swatches);
    if let Some(s) = &mut o.stroke {
        remap_paint(&mut s.paint, &mut s.swatch, &done.swatches);
    }
    o.style = o.style.and_then(|id| done.styles.get(&id).copied());
    if let Some(t) = &mut o.text {
        t.style_id = t.style_id.and_then(|id| done.styles.get(&id).copied());
    }
    match o.kind {
        ShapeKind::Image { asset } => {
            if let Some(&asset) = done.assets.get(&asset) {
                o.kind = ShapeKind::Image { asset };
            }
        }
        ShapeKind::Instance { symbol, placement } => {
            o.kind = match done.symbols.get(&symbol) {
                Some(&symbol) => ShapeKind::Instance { symbol, placement },
                None => ShapeKind::Group,
            };
        }
        _ => {}
    }
    for c in &mut o.children {
        remap_object(Arc::make_mut(c), done);
    }
}

impl ImportMode<'_> {
    fn publishing(&self) -> bool {
        matches!(self, Self::Publish(_))
    }

    /// The library entry of a new element copied from one linked to
    /// `origin`.
    fn origin_of_new(&mut self, origin: Option<&LibraryKey>) -> Option<LibraryKey> {
        match self {
            Self::Import => origin.cloned(),
            Self::Publish(new_key) => Some(origin.cloned().unwrap_or_else(new_key)),
        }
    }

    /// Publish: makes sure the target element has a library key and
    /// records it for source element `source`.
    fn record(&mut self, source: u64, target: &mut Option<LibraryKey>, done: &mut Imported) {
        if let Self::Publish(new_key) = self {
            let key = target.get_or_insert_with(new_key).clone();
            done.origins.push((source, key));
        }
    }
}

/// Copies `picks` from `from` into `into` with everything they use, in
/// dependency order (assets, swatches, styles, symbols, objects). See the
/// module documentation for the reuse rules.
pub fn import(from: &Project, picks: &Picks, into: &mut Project, mut mode: ImportMode) -> Imported {
    let closure = closure(from, picks);
    let mut done = Imported::default();

    for id in &closure.assets {
        let a = &from.assets[id];
        let (target, _) = into.add_asset(&a.name, a.kind, a.bytes.clone(), a.size);
        done.assets.insert(*id, target);
    }

    for id in &closure.swatches {
        let Some(s) = from.swatch(*id) else { continue };
        let by_origin = s.origin.as_ref().and_then(|k| {
            into.palette
                .iter()
                .position(|t| t.origin.as_ref() == Some(k))
        });
        let index = if let Some(i) = by_origin {
            if mode.publishing() {
                let target = into.palette[i].id;
                into.palette[i].name = unique_name(&s.name, |n| {
                    into.palette.iter().any(|t| t.id != target && t.name == n)
                });
                into.set_swatch_color(target, s.color);
            }
            i
        } else if let Some(i) = into
            .palette
            .iter()
            .position(|t| t.name == s.name && t.color == s.color)
        {
            i
        } else {
            let swatch = Swatch {
                id: SwatchId(into.fresh_id()),
                name: unique_name(&s.name, |n| into.palette.iter().any(|t| t.name == n)),
                color: s.color,
                origin: mode.origin_of_new(s.origin.as_ref()),
            };
            into.palette.push(swatch);
            done.added += 1;
            into.palette.len() - 1
        };
        let target = &mut into.palette[index];
        done.swatches.insert(*id, target.id);
        mode.record(id.0, &mut target.origin, &mut done);
    }

    for id in &closure.graphic_styles {
        let Some(s) = from.graphic_style(*id) else {
            continue;
        };
        let look = remap_look(&s.look, &done.swatches, into);
        let by_origin = s.origin.as_ref().and_then(|k| {
            into.graphic_styles
                .iter()
                .position(|t| t.origin.as_ref() == Some(k))
        });
        let index = if let Some(i) = by_origin {
            if mode.publishing() {
                let target = into.graphic_styles[i].id;
                into.graphic_styles[i].name = unique_name(&s.name, |n| {
                    into.graphic_styles
                        .iter()
                        .any(|t| t.id != target && t.name == n)
                });
                into.graphic_styles[i].look = look;
                into.reapply_style(target);
            }
            i
        } else if let Some(i) = into
            .graphic_styles
            .iter()
            .position(|t| t.name == s.name && t.look == look)
        {
            i
        } else {
            let style = GraphicStyle {
                id: StyleId(into.fresh_id()),
                name: unique_name(&s.name, |n| into.graphic_styles.iter().any(|t| t.name == n)),
                look,
                origin: mode.origin_of_new(s.origin.as_ref()),
            };
            into.graphic_styles.push(style);
            done.added += 1;
            into.graphic_styles.len() - 1
        };
        let target = &mut into.graphic_styles[index];
        done.styles.insert(*id, target.id);
        mode.record(id.0, &mut target.origin, &mut done);
    }

    for id in &closure.text_styles {
        let Some(s) = from.text_style(*id) else {
            continue;
        };
        let look = remap_look(&s.look, &done.swatches, into);
        let by_origin = s.origin.as_ref().and_then(|k| {
            into.text_styles
                .iter()
                .position(|t| t.origin.as_ref() == Some(k))
        });
        let index = if let Some(i) = by_origin {
            if mode.publishing() {
                let target = into.text_styles[i].id;
                into.text_styles[i].name = unique_name(&s.name, |n| {
                    into.text_styles
                        .iter()
                        .any(|t| t.id != target && t.name == n)
                });
                into.text_styles[i].style = s.style.clone();
                into.text_styles[i].look = look;
                into.reapply_style(target);
            }
            i
        } else if let Some(i) = into
            .text_styles
            .iter()
            .position(|t| t.name == s.name && t.style == s.style && t.look == look)
        {
            i
        } else {
            let style = TextStyle {
                id: StyleId(into.fresh_id()),
                name: unique_name(&s.name, |n| into.text_styles.iter().any(|t| t.name == n)),
                style: s.style.clone(),
                look,
                origin: mode.origin_of_new(s.origin.as_ref()),
            };
            into.text_styles.push(style);
            done.added += 1;
            into.text_styles.len() - 1
        };
        let target = &mut into.text_styles[index];
        done.styles.insert(*id, target.id);
        mode.record(id.0, &mut target.origin, &mut done);
    }

    // Symbols are reused by library entry only: there is no cheap test of
    // two drawings being the same. Their ids are known before any content
    // is copied, so instances inside a symbol can be remapped.
    let mut copy = Vec::new();
    for id in &closure.symbols {
        let Some(s) = from.symbol(*id) else { continue };
        let by_origin = s.origin.as_ref().and_then(|k| {
            into.symbols
                .iter()
                .find(|t| t.origin.as_ref() == Some(k))
                .map(|t| t.id)
        });
        let target = match by_origin {
            Some(t) => {
                if mode.publishing() {
                    copy.push((s, t));
                }
                t
            }
            None => {
                let t = SymbolId(into.fresh_id());
                into.symbols.push(Symbol {
                    id: t,
                    name: String::new(),
                    surface: Surface::new("", s.surface.size),
                    origin: mode.origin_of_new(s.origin.as_ref()),
                });
                done.added += 1;
                copy.push((s, t));
                t
            }
        };
        done.symbols.insert(*id, target);
    }
    for (s, target) in copy {
        let mut objects = Vec::with_capacity(s.surface.objects.len());
        for o in &s.surface.objects {
            let mut o = (**o).clone();
            remap_object(&mut o, &done);
            into.follow_links(&mut o);
            into.assign_fresh_ids(&mut o);
            objects.push(Arc::new(o));
        }
        let name = unique_name(&s.name, |n| {
            into.symbols.iter().any(|t| t.id != target && t.name == n)
        });
        let t = into
            .symbols
            .iter_mut()
            .find(|t| t.id == target)
            .expect("symbol just added or found");
        t.name.clone_from(&name);
        t.surface.name = name;
        t.surface.size = s.surface.size;
        t.surface.guides.clone_from(&s.surface.guides);
        t.surface.objects = objects;
    }
    for id in &closure.symbols {
        let Some(&target) = done.symbols.get(id) else {
            continue;
        };
        if let Some(t) = into.symbols.iter_mut().find(|s| s.id == target) {
            mode.record(id.0, &mut t.origin, &mut done);
        }
    }

    done.objects = picks
        .objects
        .iter()
        .map(|o| {
            let mut o = o.clone();
            remap_object(&mut o, &done);
            into.follow_links(&mut o);
            o
        })
        .collect();

    // Symbol content may now follow other values: instances follow.
    into.relink();
    done
}

impl Project {
    /// Whether a swatch, style or symbol of the project is linked to
    /// library entry `key`.
    pub fn has_origin(&self, key: &LibraryKey) -> bool {
        let is = |o: &Option<LibraryKey>| o.as_ref() == Some(key);
        self.palette.iter().any(|s| is(&s.origin))
            || self.graphic_styles.iter().any(|s| is(&s.origin))
            || self.text_styles.iter().any(|s| is(&s.origin))
            || self.symbols.iter().any(|s| is(&s.origin))
    }

    /// Links the swatches, styles and symbols with the given ids to their
    /// library entries (see [`Imported::origins`]).
    pub fn record_origins(&mut self, origins: &[(u64, LibraryKey)]) {
        let map: HashMap<u64, Option<LibraryKey>> = origins
            .iter()
            .map(|(id, key)| (*id, Some(key.clone())))
            .collect();
        self.set_origins(&map);
    }
}

#[cfg(test)]
mod tests;
