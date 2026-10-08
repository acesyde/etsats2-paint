//! Symbols: drawings defined once and placed as instances on the textures.
//!
//! A symbol's content is a [`Surface`], so every tool edits it as it edits
//! a texture (see [`Project::editing_symbol`]). An instance is an object of
//! kind [`ShapeKind::Instance`]: its placement maps the symbol onto the
//! texture, and its children are the content expanded through it, so it
//! renders, exports and hit-tests like a group.

use std::collections::HashMap;
use std::sync::Arc;

use kurbo::{Affine, Point, Size, Vec2};

use crate::brand::{numbered_name, update_tree};
use crate::document::{Frame, Object, ObjectId, ShapeKind, SymbolId, apply_affine, tree};
use crate::project::{Project, Surface};

/// A named drawing, placed as instances.
#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    /// The content, on the symbol's own square artboard.
    pub surface: Surface,
}

impl Symbol {
    /// Whether both hold the same document (objects compared by pointer
    /// first).
    pub(crate) fn same_document(&self, other: &Symbol) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.surface.size == other.surface.size
            && self.surface.guides == other.surface.guides
            && self.surface.objects.len() == other.surface.objects.len()
            && self
                .surface
                .objects
                .iter()
                .zip(&other.surface.objects)
                .all(|(x, y)| Arc::ptr_eq(x, y) || x == y)
    }
}

/// Largest object id in `list` and its descendants.
fn max_id(list: &[Arc<Object>]) -> u64 {
    list.iter()
        .map(|o| o.id.0.max(max_id(&o.children)))
        .max()
        .unwrap_or(0)
}

/// Ids of `list` and its descendants, in paint order (parents first).
fn ids_in_order(list: &[Arc<Object>], out: &mut Vec<ObjectId>) {
    for o in list {
        out.push(o.id);
        ids_in_order(&o.children, out);
    }
}

/// Gives `o` and its descendants the next ids of `reuse`, then fresh ids
/// from `next`.
fn assign_ids(o: &mut Object, reuse: &mut impl Iterator<Item = ObjectId>, next: &mut u64) {
    o.id = reuse.next().unwrap_or_else(|| {
        *next += 1;
        ObjectId(*next - 1)
    });
    for child in &mut o.children {
        assign_ids(Arc::make_mut(child), reuse, next);
    }
}

/// Turns `o` and every instance inside it into plain groups.
fn detach_deep(o: &mut Object) {
    if o.is_instance() {
        o.kind = ShapeKind::Group;
    }
    for child in &mut o.children {
        detach_deep(Arc::make_mut(child));
    }
}

/// Re-expands one instance from its symbol's `content`, or turns it into a
/// group when its symbol is gone; returns whether it changed.
fn refresh_one(
    o: &mut Object,
    symbols: &HashMap<SymbolId, Vec<Arc<Object>>>,
    next: &mut u64,
) -> bool {
    let ShapeKind::Instance { symbol, placement } = o.kind else {
        return false;
    };
    let Some(content) = symbols.get(&symbol) else {
        o.kind = ShapeKind::Group;
        o.refresh_group_frame();
        return true;
    };
    let objects: Vec<Object> = content.iter().map(|c| (**c).clone()).collect();
    let mut expanded = apply_affine(&objects, placement);
    let mut previous = Vec::new();
    ids_in_order(&o.children, &mut previous);
    let mut reuse = previous.into_iter();
    for e in &mut expanded {
        assign_ids(e, &mut reuse, next);
    }
    let children: Vec<Arc<Object>> = expanded.into_iter().map(Arc::new).collect();
    if children.len() == o.children.len() && children.iter().zip(&o.children).all(|(a, b)| a == b) {
        return false;
    }
    o.children = children;
    o.refresh_group_frame();
    true
}

impl Project {
    /// Re-expands every instance of every texture from its symbol, so its
    /// content is the symbol's seen through its placement; an instance
    /// whose symbol is gone (pasted from another project) becomes a group.
    /// Unchanged instances keep their `Arc`.
    pub fn refresh_instances(&mut self) {
        let symbols: HashMap<SymbolId, Vec<Arc<Object>>> = self
            .symbols
            .iter()
            .map(|s| (s.id, s.surface.objects.clone()))
            .collect();
        let mut next = self.id_counter();
        for surface in &mut self.surfaces {
            update_tree(&mut surface.objects, &mut |o| {
                refresh_one(o, &symbols, &mut next)
            });
        }
        self.reserve_ids_up_to(next.saturating_sub(1));
    }

    /// An instance of symbol `id` showing it through `placement`, named
    /// after the symbol (its content is expanded by
    /// [`Project::refresh_instances`]).
    pub fn new_instance(&self, id: SymbolId, placement: Affine) -> Option<Object> {
        let symbol = self.symbol(id)?;
        let mut o = Object::new(
            ObjectId(0),
            ShapeKind::Instance {
                symbol: id,
                placement,
            },
            Frame::new(Point::ORIGIN, Size::ZERO, 0.0),
        );
        o.name = symbol.name.clone();
        Some(o)
    }

    /// The placement showing symbol `id` at 100% centered on `at`.
    pub fn placement_at(&self, id: SymbolId, at: Point) -> Option<Affine> {
        let half = self.symbol(id)?.surface.size / 2.0;
        Some(Affine::translate(at.to_vec2() - Vec2::new(half, half)))
    }

    /// Turns the objects `ids` of the active surface into a new symbol named
    /// "`prefix` N"; they are replaced by its first instance, at the topmost
    /// one's stacking position and parent, showing them where they were.
    /// Instances among them (or inside them) are detached first. Returns the
    /// symbol and the instance.
    pub fn convert_to_symbol(
        &mut self,
        ids: &[ObjectId],
        prefix: &str,
    ) -> Option<(SymbolId, ObjectId)> {
        if ids.is_empty() || self.editing_symbol.is_some() {
            return None;
        }
        let group = self.group(ids)?;
        let mut content: Vec<Object> = tree::get(&self.surface().objects, group)?
            .children
            .iter()
            .map(|c| (**c).clone())
            .collect();
        for o in &mut content {
            detach_deep(o);
        }
        let bounds = content
            .iter()
            .map(Object::bounding_box)
            .reduce(|a, b| a.union(b))?;
        let side = bounds.width().max(bounds.height()).ceil().max(1.0);
        // The content centered on the symbol's artboard.
        let offset = Vec2::new(side / 2.0, side / 2.0) - bounds.center().to_vec2();
        let mut next = self.id_counter();
        let content: Vec<Arc<Object>> = content
            .into_iter()
            .map(|mut o| {
                o.translate_deep(offset);
                assign_ids(&mut o, &mut std::iter::empty(), &mut next);
                Arc::new(o)
            })
            .collect();
        self.reserve_ids_up_to(next.saturating_sub(1));
        let id = SymbolId(self.fresh_id());
        let name = numbered_name(prefix, self.symbols.iter().map(|s| s.name.as_str()));
        let mut surface = Surface::new(name.clone(), side);
        surface.objects = content;
        self.symbols.push(Symbol { id, name, surface });
        let mut instance = self.new_instance(id, Affine::translate(-offset))?;
        instance.id = group;
        self.surface_mut().replace(&[instance]);
        self.refresh_instances();
        Some((id, group))
    }

    /// Replaces each instance among `ids` (active surface) by a group of
    /// its current content, keeping its name, opacity, visibility and lock.
    pub fn detach_instances(&mut self, ids: &[ObjectId]) -> Vec<ObjectId> {
        let objects = &self.surface().objects;
        let detached: Vec<Object> = ids
            .iter()
            .filter_map(|id| tree::get(objects, *id))
            .filter(|o| o.is_instance())
            .map(|o| {
                let mut o = (**o).clone();
                o.kind = ShapeKind::Group;
                o
            })
            .collect();
        let out = detached.iter().map(|o| o.id).collect();
        self.surface_mut().replace(&detached);
        out
    }

    /// A copy of symbol `id` named "<name> `suffix`" (made unique), with
    /// the same content and no instance.
    pub fn duplicate_symbol(&mut self, id: SymbolId, suffix: &str) -> Option<SymbolId> {
        let source = self.symbol(id)?.clone();
        let mut next = self.id_counter();
        let objects = source
            .surface
            .objects
            .iter()
            .map(|o| {
                let mut o = (**o).clone();
                assign_ids(&mut o, &mut std::iter::empty(), &mut next);
                Arc::new(o)
            })
            .collect();
        self.reserve_ids_up_to(next.saturating_sub(1));
        let base = format!("{} {suffix}", source.name);
        let taken = |n: &str| self.symbols.iter().any(|s| s.name == n);
        let name = if taken(&base) {
            (2..)
                .map(|n| format!("{base} {n}"))
                .find(|n| !taken(n))
                .expect("a free name")
        } else {
            base
        };
        let new = SymbolId(self.fresh_id());
        let mut surface = source.surface;
        surface.objects = objects;
        surface.name.clone_from(&name);
        self.symbols.push(Symbol {
            id: new,
            name,
            surface,
        });
        Some(new)
    }

    /// Deletes symbol `id`: its instances, on every texture, become groups
    /// that look the same.
    pub fn delete_symbol(&mut self, id: SymbolId) {
        for surface in &mut self.surfaces {
            update_tree(&mut surface.objects, &mut |o| match o.kind {
                ShapeKind::Instance { symbol, .. } if symbol == id => {
                    o.kind = ShapeKind::Group;
                    true
                }
                _ => false,
            });
        }
        self.symbols.retain(|s| s.id != id);
        if self.editing_symbol == Some(id) {
            self.editing_symbol = None;
        }
    }

    /// Renames a symbol; refuses an empty name or one another symbol has.
    pub fn rename_symbol(&mut self, id: SymbolId, name: &str) -> bool {
        let name = name.trim();
        if name.is_empty() || self.symbols.iter().any(|s| s.id != id && s.name == name) {
            return false;
        }
        match self.symbols.iter_mut().find(|s| s.id == id) {
            Some(s) => {
                s.name = name.to_owned();
                s.surface.name = name.to_owned();
                true
            }
            None => false,
        }
    }

    /// Number of instances of symbol `id` on every texture.
    pub fn instance_count(&self, id: SymbolId) -> usize {
        fn count(list: &[Arc<Object>], id: SymbolId) -> usize {
            list.iter()
                .map(|o| match o.kind {
                    ShapeKind::Instance { symbol, .. } if symbol == id => 1,
                    ShapeKind::Instance { .. } => 0,
                    _ => count(&o.children, id),
                })
                .sum()
        }
        self.surfaces.iter().map(|s| count(&s.objects, id)).sum()
    }

    pub fn symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.id == id)
    }

    /// Sets the symbols read from a file; new ids continue after theirs.
    pub fn set_symbols(&mut self, symbols: Vec<Symbol>) {
        let largest = symbols
            .iter()
            .map(|s| s.id.0.max(max_id(&s.surface.objects)))
            .max()
            .unwrap_or(0);
        self.reserve_ids_up_to(largest);
        self.symbols = symbols;
    }
}

#[cfg(test)]
mod tests;
