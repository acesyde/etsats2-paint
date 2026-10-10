//! Where the swatches, styles and symbols are used: how many objects each
//! changes and on which textures, so that an edit can announce its reach
//! before it is made.
//!
//! One walk over the textures counts everything. A swatch changes the
//! objects whose fill, stroke, shadow or gradient stops link to it, and each
//! instance of a symbol whose content has such an object (the instance
//! counts once). A style changes the objects that follow it, and the
//! instances whose symbol content follows it. Objects inside groups count
//! one by one; a group itself is not counted.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::document::{Object, Paint, ShapeKind, StyleId, SwatchId, SymbolId};
use crate::project::Project;

/// How far an element reaches: the objects it changes and the textures
/// holding them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Count {
    /// Objects changed (for a symbol: its instances).
    pub objects: usize,
    /// Indices of the textures holding them, in order.
    pub surfaces: Vec<usize>,
}

impl Count {
    /// Number of textures holding the objects.
    pub fn textures(&self) -> usize {
        self.surfaces.len()
    }

    pub fn is_unused(&self) -> bool {
        self.objects == 0
    }

    /// Counts one object on texture `surface` (textures are walked in
    /// order).
    fn add(&mut self, surface: usize) {
        self.objects += 1;
        if self.surfaces.last() != Some(&surface) {
            self.surfaces.push(surface);
        }
    }
}

static UNUSED: Count = Count {
    objects: 0,
    surfaces: Vec::new(),
};

/// The usage of every swatch, style (graphic and text) and symbol of a
/// project. An element nothing uses has no entry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Usage {
    swatches: HashMap<SwatchId, Count>,
    styles: HashMap<StyleId, Count>,
    symbols: HashMap<SymbolId, Count>,
}

impl Usage {
    pub fn swatch(&self, id: SwatchId) -> &Count {
        self.swatches.get(&id).unwrap_or(&UNUSED)
    }

    /// A graphic or a text style.
    pub fn style(&self, id: StyleId) -> &Count {
        self.styles.get(&id).unwrap_or(&UNUSED)
    }

    /// Instances of the symbol and the textures holding them.
    pub fn symbol(&self, id: SymbolId) -> &Count {
        self.symbols.get(&id).unwrap_or(&UNUSED)
    }
}

/// The swatches and styles one object links to, each once.
#[derive(Default)]
struct Links {
    swatches: HashSet<SwatchId>,
    styles: HashSet<StyleId>,
}

impl Links {
    fn paint(&mut self, paint: &Paint, link: Option<SwatchId>) {
        match paint {
            Paint::Solid(_) => self.swatches.extend(link),
            Paint::Gradient(g) => self
                .swatches
                .extend(g.stops().iter().filter_map(|s| s.swatch)),
        }
    }

    /// Adds what `o` itself links to (not its children).
    fn object(&mut self, o: &Object) {
        self.paint(&o.fill, o.fill_swatch);
        if let Some(s) = &o.stroke {
            self.paint(&s.paint, s.swatch);
        }
        self.swatches.extend(o.shadow.and_then(|s| s.swatch));
        self.styles.extend(o.style);
        self.styles.extend(o.text.as_ref().and_then(|t| t.style_id));
    }
}

/// Adds what the objects of `list` and their descendants link to, an
/// instance's expanded content left out.
fn content_links(list: &[Arc<Object>], out: &mut Links) {
    for o in list {
        match o.kind {
            ShapeKind::Instance { .. } => {}
            ShapeKind::Group => content_links(&o.children, out),
            _ => out.object(o),
        }
    }
}

impl Project {
    /// The usage of every swatch, style and symbol, over every texture.
    pub fn usage(&self) -> Usage {
        // What each symbol's content links to: its instances change with it.
        let symbols: HashMap<SymbolId, Links> = self
            .symbols
            .iter()
            .map(|s| {
                let mut links = Links::default();
                content_links(&s.surface.objects, &mut links);
                (s.id, links)
            })
            .collect();
        let mut usage = Usage::default();
        for (i, surface) in self.surfaces.iter().enumerate() {
            walk(&surface.objects, i, &symbols, &mut usage);
        }
        usage
    }
}

fn walk(
    list: &[Arc<Object>],
    surface: usize,
    symbols: &HashMap<SymbolId, Links>,
    usage: &mut Usage,
) {
    for o in list {
        let mut own = Links::default();
        let links = match o.kind {
            ShapeKind::Group => {
                walk(&o.children, surface, symbols, usage);
                continue;
            }
            ShapeKind::Instance { symbol, .. } => {
                usage.symbols.entry(symbol).or_default().add(surface);
                match symbols.get(&symbol) {
                    Some(links) => links,
                    None => continue,
                }
            }
            _ => {
                own.object(o);
                &own
            }
        };
        for id in &links.swatches {
            usage.swatches.entry(*id).or_default().add(surface);
        }
        for id in &links.styles {
            usage.styles.entry(*id).or_default().add(surface);
        }
    }
}

#[cfg(test)]
mod tests;
