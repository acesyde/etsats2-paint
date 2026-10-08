## Context

See proposal.md (Why) and the specs for the behavior. Today:
- **Groups:** a group (`ShapeKind::Group`) holds `children: Vec<Arc<Object>>` in document coordinates.
  - Its frame is derived from them (`refresh_group_frame`).
  - The transform functions (`translate`, `resize_one`, `rotate_one`) update an object's frame and recurse into its children, whatever its kind.
  - Rendering (`tree::draw_list`), hit testing and bounds walk children.
- **Active surface:** `Project.surfaces` are the textures. `Project::surface()` and `surface_mut()` return `surfaces[active_surface]`, and every tool edits through them.
  - About 30 places index `surfaces[...]` directly, mostly vehicle, template, export and sidebar code.
  - `Workspace` keeps one viewport per texture index (`swap_view`).
- **Undo:** the history stores `Snapshot`s: surface metadata, objects, guides, vehicles, the brand kit and assets. `Workspace::record` is the single place an edit becomes an undo step, and since `brand-kit` it calls `Project::relink()` first.
- **Brand kit:** `update_tree` and `update_surfaces` walk every object recursively for swatch propagation and relinking.
- **Assets panel:** asset usage counts image objects in the textures.
- **File format:** `tp-file` v1 uses serde with `#[serde(default)]` for optional fields. Unknown enum variants fail to parse.

## Goals / Non-Goals

**Goals:**
- **Rendering unchanged:** instances render, export, hit-test and snap with the existing code, because they carry their expanded content as children.
- **Same tools in a symbol:** editing a symbol uses the same tools, panels and undo as a texture.
- **No drift:** an instance's content always equals its symbol's content seen through its placement, whatever transforms were applied.

**Non-Goals:**
- Overrides of an instance's look, and nested symbols (decided with the user).
- Symbol thumbnails in the Symbols panel. They are a later improvement; the panel shows names and counts.
- Sharing symbols across projects (roadmap open question).
- Editing a symbol in place on a texture (decided in `brand-kit`: own view).

## Decisions

### 1. A symbol's content is a `Surface`
`Symbol { id: SymbolId, name: String, surface: Surface }`, stored in `Project.symbols: Vec<Symbol>`.
- `surface.size` is the square artboard side.
- `surface.objects` is the content, and `surface.guides` the symbol's guides. `template` is always `None`.

Storing the content as a `Surface` lets every tool, panel and the canvas work on a symbol unchanged.

**Alternative:** a dedicated content type. Rejected: every tool would need a second target type.

### 2. Editing a symbol: `Project.editing_symbol`
- `Project.editing_symbol: Option<SymbolId>`. When it is set, `surface()` and `surface_mut()` return the symbol's surface. `active_surface` stays the texture the symbol was opened from, so Done returns there.
- Code that reads `surfaces[active_surface]` for vehicle data (templates, Copy From Cabin, Next and Previous Texture, export, the sidebar highlight) checks `editing_symbol`. Its commands are disabled with a `reason-editing-symbol` tooltip.
- `editing_symbol` and the symbols are part of `Snapshot`. Undoing a change made in the symbol view goes back to that view: `Workspace::restore` applies `editing_symbol` and swaps the view.
- `Workspace` keeps one viewport per symbol (`HashMap<SymbolId, Viewport>`) next to the texture viewports. A symbol first opened is fitted to the window.

**Alternative:** an `Active { Texture(usize) | Symbol(SymbolId) }` enum replacing `active_surface`. Rejected for now: about 60 call sites would change for the same behavior.

### 3. An instance is `ShapeKind::Instance { symbol, placement }` with expanded children
- **Placement:** `placement` is an `Affine` mapping the symbol's coordinates to the texture. A negative determinant means the instance is mirrored, so no separate flip flag is needed.
- **Expanded children:** `children` holds the symbol's objects mapped by the placement. The instance's frame is derived from them, exactly as a group's (`refresh_group_frame`). The selection box therefore fits the content, and rendering, export, hit testing, snapping and bounds use the existing group walk.
- **Transforms:** `translate`, `rotate_one`, `resize_one` and `translate_deep` already move children and frames. For an instance, they also compose their transform into the placement (`placement = transform * placement`).
- **Mapping:** a new `transform::apply_affine(objects, a)` maps objects by any affine:
  - it decomposes `a` into rotation, signed scales and translation;
  - it calls `resize_one`, so paths and gradients take the exact transform, and frames are approximated as when a group is resized.

  Expansion and Copy From Cabin's scaling (`scale_objects`) both use it.
- **Group-like checks:** a helper `Object::has_content()` (Group or Instance) replaces `is_group()` where children are walked: drawing, bounds, hit testing, sampling and frame refresh. `is_group()` keeps meaning a real group where structure is edited: Ungroup, entering groups, the Layers expander, `shapes()`, and paint and style application.

**Alternative:** the instance's frame as the placement, as for an image. Rejected: the selection box would be the symbol's artboard, not the content, and would drift from the content after the symbol is edited.

### 4. Keeping instances exact: re-expansion when an edit is recorded
`Project::refresh_instances()` re-expands every instance of every texture from its symbol and placement.
- It turns an instance whose symbol is missing (pasted from another project) into a group with the same children.
- `Project::relink()` ends with it, and `Workspace::record` calls `relink()`: symbol content is relinked first, then the instances are expanded from it.
- During a live gesture, the transform functions move the children along with the frame, as for a group, so the canvas follows the pointer. Once the change is recorded, re-expansion makes the children exact.
- Edits recorded in the symbol view therefore reach every instance in the same undo step.
- **Flips:** a flip is a resize by −1, so it composes a mirror into the placement, as any resize does.

**Cost:** expansion clones the symbol's objects once per instance per recorded edit, which is small for a fleet (tens of instances, tens of objects each). Unchanged instances keep their `Arc`, because the result is compared before it replaces the old one.

### 5. Convert to Symbol, Place, Detach, Duplicate, Delete
These are project operations in a new `tp-core/src/symbols.rs`:
- **`convert_to_symbol(ids, prefix)`:**
  1. detaches instances among `ids`;
  2. takes the selected objects in paint order and computes their axis-aligned bounds `B`;
  3. sets the artboard side `S` to the larger side of `B`, rounded up to a whole pixel;
  4. translates the content so that `B` is centered on the artboard;
  5. replaces the objects with one instance at the first object's stacking position and parent. Its placement translates the artboard so that the content lands where the objects were, so the expansion equals them.
- **`place_symbol(id, at)`:** one instance at 100%, centered at `at`.
- **`detach_instances(ids)`:** each instance becomes a `Group` with its current children (fresh ids), name, opacity, visibility and lock.
- **`duplicate_symbol(id)`:** "<name> copy" with fresh object ids.
- **`delete_symbol(id)`:** detaches every instance on every texture, then removes the symbol.
- **`rename_symbol`:** refuses empty or duplicate names.

### 6. The rest of the document
- **Brand kit:** `update_surfaces` walks the textures and the symbols' surfaces. Swatch edits and style propagation reach symbol content, then `refresh_instances` updates the instances. The `relink()` walk skips instance children (they are expansions) and checks symbol content instead.
- **Paint and style application:** paint and style operations (`map_selected_shapes`, `apply_*_style`, `selected_shapes`) skip instances, because `shapes()` treats an instance as a leaf and the paint operations filter out `Instance`.
- **Panels with only instances selected:** the Colors and Stroke panels show the `instance-look-in-symbol` message.
- **Command reasons:** Convert to Path, Create Outlines, Combine and Direct Selection get an `instance` reason in their enablement (`EditContext.selection_has_instance`).
- **Assets:** asset usage counts image objects in the textures (not inside instances) and in the symbols' content.
- **Text:** `relayout_all_texts` also lays out symbol content, then refreshes instances.

### 7. File format: optional additions to v1
- `FileProject.symbols: Vec<FileSymbol { id, name, size, guides, objects }>`, with `#[serde(default)]` and skipped when empty.
- `FileKind::Instance { symbol, placement: [f64; 6] }`. Instance children aren't written: `into_project` rebuilds them with `refresh_instances()` after reading.
- A file with symbols can't be read by an earlier build, because the unknown `Instance` variant fails to parse. That's accepted before the first release, and files without symbols are byte-for-byte unchanged in shape.
- Ids: symbol ids and object ids inside symbols join the `next_id` computation in `from_parts`.

### 8. Interface
- **Commands:**
  - Object menu: Convert to Symbol, Edit Symbol and Detach Instance, after Ungroup;
  - Escape and Done handling for the symbol view;
  - the canvas double-click on an instance calls Edit Symbol, before the text and group double-click behaviors.
- **Symbols panel** (`PanelKind::Symbols` after Styles, inserted collapsed by `sanitized()`):
  - rows with the symbol icon, name and "N instances";
  - Place and Edit buttons;
  - a context menu with Rename (inline), Duplicate and Delete. Delete asks for confirmation when the symbol is used;
  - rows drag onto the canvas, as Assets rows do (`dnd_set_drag_payload(SymbolId)`).
- **Symbol bar:** "Editing symbol <name>", with Done, drawn at the top of the canvas area while `editing_symbol` is set. The status bar shows "Symbol › <name>".
- **Layers panel:** instance rows use the symbol icon, with no expander.
- **Properties:** an Instance section with the symbol name, Edit Symbol and Detach Instance.

## Risks / Trade-offs

- **[A missed `is_group()` site walks into or edits instance children]** → Every site is reviewed in task 1.3, and the result recorded in the task. Tests cover the paths that matter: selection, paint, styles, Ungroup, Layers, Direct Selection and Combine. In any case re-expansion at record time overwrites changes to children.
- **[Expansion approximates when a non-uniform scale meets rotated content]** Rectangles and ellipses can't skew. → The approximation is the same as resizing a group today (`resize_one`), so instances look like a transformed group would. Paths take the exact transform.
- **[Ids of expanded children]** Selection and Layers state referencing them would break if they changed on every expansion. → Children are never selectable. Re-expansion reuses the previous children's ids by position in paint order, and takes fresh ids from the project counter only for added objects. Ids stay unique, and render caches keyed by id stay warm.
- **[Earlier builds can't open files with symbols]** → Accepted before release, and documented in the roadmap and the PR.
- **[Escape conflicts]** → The order is: end typing, cancel a gesture, clear the selection, then leave the symbol view.

## Migration Plan

Additive. Files without symbols are unchanged. Rollback is reverting the change.
