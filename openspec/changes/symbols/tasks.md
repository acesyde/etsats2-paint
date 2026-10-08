## 1. Model (tp-core)

- [x] 1.1 Add `SymbolId`, `Symbol { id, name, surface }`, `Project.symbols` and `Project.editing_symbol` (design D1, D2):
  - `surface()` and `surface_mut()` return the edited symbol's surface;
  - symbols and `editing_symbol` are in `Snapshot`, `restore` and `same_document`;
  - symbol and content ids count in `from_parts`.

  Verify with unit tests: tools editing through `surface_mut()` change the symbol while it is edited and the texture otherwise; a snapshot taken while editing restores the editing state.
- [x] 1.2 Add `transform::apply_affine(objects, a)` (design D3) and make `vehicle_project::scale_objects` use it. Verify with unit tests: identity; translation; 2× scale; 90° rotation; non-uniform scale on a rotated child, which matches `resize`; a path under a mirror; and the Copy From Cabin tests still passing.
- [x] 1.3 Add `ShapeKind::Instance { symbol, placement }` and `Object::has_content()`. Review every `is_group()` and `ShapeKind::Group` site in `tp-core`, `tp-app`, `tp-render` and `tp-file`, and record here which ones became `has_content()`:
  - walking for drawing, bounds, hit testing, sampling and parent frames → `has_content()`;
  - Ungroup, entering groups, `shapes()`, paints and styles → groups only.

  Review result:
  - `has_content()`: `extent_points`, `refresh_group_frame`, `bounding_box`, `contains`, `local_path` (object.rs); `draw_list`, `sample`, `top_level_in_rect` (tree.rs); the group refresh after loading (v1.rs); the renderer's `Group | Instance` arm.
  - Unchanged (real groups only): `hit_test` (an instance is a leaf, hit through `contains`), insertion and `move_to` parents, `ungroup`, `add_to`, `active_layer`, `selection_has_group`, `editable_paths`, the Layers expander.
  - Instances excluded: `for_each_shape` and `shapes()` skip them; `takes_graphic_style` and the Styles panel exclude them; `operand_problem` returns `Instance`.
  - New arms: names, icons, undo labels, `FileKind::Instance` (children not written).

  The transform functions compose their transform into an instance's placement. Verify with unit tests: `draw_list`, `hit_test` and bounds see an instance's children; `shapes()` treats an instance as a leaf; moving, rotating, resizing and flipping update the placement to match the children.
- [x] 1.4 Implement expansion and `Project::refresh_instances()` (design D4):
  - children are the symbol's objects mapped by the placement;
  - ids are reused by paint order, with fresh ones only for added objects;
  - unchanged instances keep their `Arc`;
  - an instance of a missing symbol becomes a group.

  Verify with unit tests: an expansion at 100% equals the content; a rotated and mirrored instance matches `apply_affine`; re-expansion after a content edit keeps ids; a missing symbol gives a group.

## 2. Operations (tp-core)

- [x] 2.1 Add `convert_to_symbol`, `place_symbol`, `detach_instances`, `duplicate_symbol`, `delete_symbol` and `rename_symbol` (design D5), in `tp-core/src/symbols.rs`. Verify with unit tests, one per spec scenario of "Convert to Symbol", "Symbols panel" and "Detach Instance" that the model covers:
  - the instance shows the objects where they were;
  - conversion inside a group keeps the parent and the stacking position;
  - a selection holding an instance detaches it first;
  - Undo restores;
  - delete detaches every instance;
  - names are unique and not empty.
- [x] 2.2 Extend the brand kit, assets and text to symbols (design D6):
  - `update_surfaces` covers the symbols;
  - `relink` skips instance children;
  - asset usage counts symbol content and not instance children;
  - `refresh_instances` runs after swatch and style propagation.

  Verify with unit tests: the spec's "Recolor a swatch used in a symbol"; a style redefined from a texture updates a styled shape inside a symbol and its instances; an image in a symbol with three instances counts one use.

## 3. Files (tp-file)

- [x] 3.1 Add `FileProject.symbols` (optional) and `FileKind::Instance { symbol, placement }`. Instance children aren't written, and `refresh_instances()` rebuilds them after reading (design D7). Verify with `tests/format.rs`: the spec's "Symbols round trip" (a rotated and mirrored instance); a file without symbols serializes without a `symbols` field.
- [x] 3.2 Add a symbol and two instances to `rich_project` and regenerate `fixtures/v1.truckpaint`. Verify that `cargo test -p tp-file` passes, including `v1_fixture_opens`.

## 4. Workspace (tp-app)

- [x] 4.1 Call `project.refresh_instances()` in `Workspace::record`, before `relink()`. Add the workspace's symbol operations (convert, place in the view center, detach, duplicate, delete, rename) as undo steps. Verify with workspace unit tests: a change recorded while editing a symbol updates its instances in the same undo step; moving an instance re-expands it exactly at record time; pasting an instance whose symbol is missing gives a group.
- [x] 4.2 Editing a symbol:
  - `edit_symbol(id)` and `finish_symbol_edit()`, with per-symbol viewports (fitted the first time);
  - choosing a texture ends the edit;
  - `restore` follows `editing_symbol` for Undo across views;
  - the Escape order: typing, then gesture, then selection, then leaving.

  Verify with unit tests: the spec's "Undo across views"; Escape with a selection clears it first; choosing a texture ends the edit.
- [x] 4.3 Commands and enablement:
  - Convert to Symbol, Edit Symbol and Detach Instance, with their reasons;
  - `EditContext.editing_symbol` and `selection_has_instance`;
  - `reason-editing-symbol` on Copy From Cabin, Next and Previous Texture, Update Template, Show Template and Export Texture;
  - the instance reasons on Convert to Path, Create Outlines, Combine and Direct Selection;
  - paints and styles skip instances.

  Verify with unit tests of `disabled_reason_for`, and a test that applying a color to a selection of a rectangle and an instance changes only the rectangle.

## 5. Interface (tp-app)

- [x] 5.1 Add the Symbols panel (`PanelKind::Symbols` after Styles, inserted collapsed by `sanitized()`):
  - rows with the icon, name and instance count, an empty state, Place and Edit;
  - drag onto the canvas;
  - the context menu: Rename inline, Duplicate, and Delete with confirmation when the symbol is used.

  Verify with UI tests: "Convert a logo" (the panel lists "Symbol 1" with 1 instance); "Place on another texture"; "Delete a used symbol" with Undo; "Duplicate for a variant"; and the layout test "Layout from before the Symbols panel".
- [x] 5.2 Add the symbol view's interface:
  - the "Editing symbol <name>" bar with Done;
  - the status bar "Symbol › <name>";
  - no texture highlighted in the sidebar;
  - the canvas double-click on an instance;
  - the Object menu entries.

  Verify with UI tests: "Edit from an instance" (double-click, recolor, Done, every instance updated); the Escape order.
- [x] 5.3 Adjust the other panels:
  - Layers: instance row with the symbol icon and no expander, and rename;
  - Properties: an Instance section with the symbol name, Edit Symbol, Detach Instance and opacity;
  - Colors and Stroke: the "edited in the symbol" message when only instances are selected.

  Verify with UI tests: "Colors don't apply to an instance"; "A one-off variant" (Detach from Properties, then recolor); "Resize an instance" (corner drag doubles the size; the content keeps its look).

## 6. Text, docs and checks

- [x] 6.1 Add every new message to `tp-i18n` in en, fr, de and es:
  - commands, reasons and undo labels;
  - the Symbols panel: title, empty state, actions, confirmation and "N instances" plural;
  - the symbol bar and the status bar;
  - the "Symbol N" and "<name> copy" names;
  - the instance messages in Colors, Stroke and Properties.

  Verify that `cargo test -p tp-app --test localization` passes and the keys match across languages.
- [x] 6.2 Update `docs/roadmap.md`:
  - record the decisions: no instance overrides, no nesting, deleting detaches, editing in its own view;
  - move `symbols` to Shipped and renumber the next changes;
  - note that files with symbols don't open in earlier builds.

  Verify by reading it against the specs.
- [x] 6.3 Add screenshots (en and fr): the Symbols panel with instances on a texture, the symbol view with its bar, and Properties for an instance. Run `mise run screenshots` and check them visually. Run `mise run ci`. Verify that every step passes with no warnings.
