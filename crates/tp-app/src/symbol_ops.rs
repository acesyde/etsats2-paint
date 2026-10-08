//! Symbols in the workspace: converting the selection, placing, detaching,
//! duplicating, deleting and renaming symbols, and editing a symbol in its
//! own view.

use tp_core::document::{ObjectId, SymbolId};
use tp_core::kurbo::Point;
use tp_i18n::tr;

use crate::workspace::Workspace;

impl Workspace {
    /// Whether a symbol is being edited (instead of a texture).
    pub fn is_editing_symbol(&self) -> bool {
        self.project.editing_symbol.is_some()
    }

    /// Whether the selection holds an instance (maybe in a group).
    pub fn selection_has_instance(&self) -> bool {
        fn any(o: &tp_core::document::Object) -> bool {
            o.is_instance() || (o.is_group() && o.children.iter().any(|c| any(c)))
        }
        self.selected_objects().iter().any(any)
    }

    /// The symbol of the single selected instance.
    pub fn selected_instance_symbol(&self) -> Option<SymbolId> {
        match self.selected_objects().as_slice() {
            [o] => match o.kind {
                tp_core::document::ShapeKind::Instance { symbol, .. } => Some(symbol),
                _ => None,
            },
            _ => None,
        }
    }

    /// Convert to Symbol: the selection becomes a new symbol, replaced by
    /// its first instance, selected. One undo step.
    pub fn convert_to_symbol(&mut self, now: f64) -> Option<SymbolId> {
        self.end_text_session(now);
        if self.selection.is_empty() || self.is_editing_symbol() {
            return None;
        }
        let ids = self.selection.clone();
        let mut made = None;
        self.edit("cmd-convert-to-symbol", now, false, |project, selection| {
            if let Some((symbol, instance)) = project.convert_to_symbol(&ids, &tr("symbols-prefix"))
            {
                *selection = vec![instance];
                made = Some(symbol);
            }
        });
        made
    }

    /// Places an instance of `id` at 100%, centered on `at` (the middle of
    /// the view by default), in the active layer, selected.
    pub fn place_symbol(&mut self, id: SymbolId, at: Option<Point>, now: f64) {
        self.end_text_session(now);
        if self.is_editing_symbol() {
            return;
        }
        let at = at.unwrap_or_else(|| self.view_center());
        let Some(object) = self
            .project
            .placement_at(id, at)
            .and_then(|p| self.project.new_instance(id, p))
        else {
            return;
        };
        let layer = self.active_layer();
        self.edit("undo-place-symbol", now, false, |project, selection| {
            *selection = vec![project.add_to(layer, object)];
        });
    }

    /// Detach Instance: each selected instance becomes a group of plain
    /// objects that look the same.
    pub fn detach_selected_instances(&mut self, now: f64) {
        let ids = self.selection.clone();
        self.edit("cmd-detach-instance", now, false, |project, _| {
            project.detach_instances(&ids);
        });
    }

    pub fn duplicate_symbol(&mut self, id: SymbolId, now: f64) {
        self.edit("undo-duplicate-symbol", now, false, |project, _| {
            project.duplicate_symbol(id, &tr("symbols-copy-suffix"));
        });
    }

    /// Deletes a symbol; its instances become groups.
    pub fn delete_symbol(&mut self, id: SymbolId, now: f64) {
        if self.project.editing_symbol == Some(id) {
            self.finish_symbol_edit(now);
        }
        self.edit("undo-delete-symbol", now, false, |project, _| {
            project.delete_symbol(id);
        });
    }

    /// Renames a symbol; returns false (nothing recorded) for an empty or
    /// taken name.
    pub fn rename_symbol(&mut self, id: SymbolId, name: &str, now: f64) -> bool {
        let mut done = false;
        self.edit("undo-rename-symbol", now, false, |project, _| {
            done = project.rename_symbol(id, name);
        });
        done
    }

    /// Shows symbol `id` alone in its own view, to edit it.
    pub fn edit_symbol(&mut self, id: SymbolId, now: f64) {
        if self.project.symbol(id).is_none() || self.project.editing_symbol == Some(id) {
            return;
        }
        self.end_text_session(now);
        self.commit_pending(now);
        self.selection.clear();
        self.points.clear();
        let active = self.project.active_surface;
        self.show_view(active, Some(id));
    }

    /// Done: back to the texture the symbol was opened from.
    pub fn finish_symbol_edit(&mut self, now: f64) {
        if !self.is_editing_symbol() {
            return;
        }
        self.end_text_session(now);
        self.commit_pending(now);
        self.selection.clear();
        self.points.clear();
        let active = self.project.active_surface;
        self.show_view(active, None);
    }

    /// The instances of the active texture (top level or in groups).
    pub fn instances_on_texture(&self) -> Vec<ObjectId> {
        fn walk(list: &[std::sync::Arc<tp_core::document::Object>], out: &mut Vec<ObjectId>) {
            for o in list {
                if o.is_instance() {
                    out.push(o.id);
                } else {
                    walk(&o.children, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(
            &self.project.surfaces[self.project.active_surface].objects,
            &mut out,
        );
        out
    }
}

#[cfg(test)]
mod tests;
