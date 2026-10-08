## 1. Model: swatches and links (tp-core)

- [x] 1.1 Add the swatch model (design D1):
  - `SwatchId` and `Swatch { id, name, color }`, with `Project.palette: Vec<Swatch>`;
  - the link fields `Object.fill_swatch`, `StrokeStyle.swatch` and `ColorStop.swatch`;
  - `add_swatch(color) -> SwatchId`, which adds no duplicate color and names the swatch with the first unused "Color N";
  - `rename_swatch` and `delete_swatch`;
  - the palette in `Snapshot`, `restore` and `same_document`.

  Update the existing `add_to_palette` callers. Verify with unit tests: no duplicate; the names "Color 1" and "Color 2", then "Color 2" again once it is deleted; Undo restores a deleted swatch.
- [x] 1.2 Implement `set_swatch_color(id, color)` (design D4). It updates every linked solid fill, solid stroke and gradient stop on every surface, inside groups, and in the graphic styles. Verify with unit tests:
  - two surfaces and a nested group are recolored;
  - only the linked stop of a gradient changes;
  - unlinked objects of the same color are unchanged;
  - unchanged objects keep their `Arc` (`Arc::ptr_eq`).
- [x] 1.3 Implement `Project::relink()` (design D2) for swatch links. It drops a link whose color differs from its swatch's, whose paint is no longer solid, or whose swatch is missing. Verify with unit tests: a recolored fill is unlinked; a fill turned into a gradient is unlinked; a pasted object linked to an unknown swatch is unlinked; a linked object that is moved, rotated, duplicated, grouped or converted to a path stays linked.

## 2. Model: shared styles (tp-core)

- [x] 2.1 Add the style model:
  - `StyleId`;
  - `GraphicStyle { id, name, fill, fill_swatch, stroke, opacity }` and `TextStyle { id, name, style }`;
  - `Project.graphic_styles` and `Project.text_styles`, in the snapshot;
  - the links `Object.style` and `TextBlock.style_id`;
  - `GraphicStyle::matches(&Object)`, which ignores gradient points (design D3).

  Verify with unit tests of `matches`: an equal look; a different opacity; a gradient with moved points still matches; a different stop color doesn't.
- [x] 2.2 Implement the style operations (design D4):
  - `new_graphic_style(from)` and `new_text_style(from)`, named "Style N" and "Text style N", which link the source object;
  - `apply_graphic_style(id, ids)` and `apply_text_style(id, ids)`, which walk groups, skip images and non-texts, and keep the object's own gradient points;
  - `redefine_*`, `rename_style`, which refuses empty and duplicate names, and `delete_style`.

  Verify with unit tests, one per spec scenario of the "Graphic and text styles", "Following a style" and "Styles panel" requirements that the model covers:
  - a group applies to its children;
  - redefining changes users on two surfaces;
  - deleting keeps the looks;
  - the swatch-linked style follows `set_swatch_color`.
- [x] 2.3 Extend `relink()` to style links (a look that no longer matches, or a missing style). Verify with unit tests:
  - setting a text's size detaches it from its text style;
  - rotating an object with a gradient keeps its graphic style;
  - changing opacity detaches it;
  - the links survive `apply_*`, `redefine_*` and `set_swatch_color` without being dropped.

## 3. Files (tp-file)

- [x] 3.1 Add the optional v1 fields (design D5):
  - `swatches`, `graphic_styles` and `text_styles` on the project;
  - `fill_swatch` and `style` on objects;
  - `swatch` on strokes and stops;
  - `style_id` on texts.

  Read a legacy `palette` as "Color N" swatches when `swatches` is empty, and call `relink()` after reading. Verify with `tests/format.rs`:
  - the spec's "Brand kit round trip";
  - a file with an inconsistent link opens with the link dropped.
- [x] 3.2 Fixtures:
  - copy the current `fixtures/v1.truckpaint` to `fixtures/v1-palette-colors.truckpaint`, and test the spec's "Palette of an earlier file" with it;
  - add swatches, styles and links to `rich_project`, and regenerate `v1.truckpaint` with `write_current_fixture`.

  Verify that `cargo test -p tp-file` passes.

## 4. Workspace (tp-app)

- [x] 4.1 Call `project.relink()` in `Workspace::record`, before the after-snapshot. Call `relayout_all_texts()` after text style operations. Verify with workspace unit tests:
  - picking a color through `edit` unlinks;
  - a live swatch edit then `cancel_pending` restores the colors and adds no undo step;
  - redefining a text style's size changes the users' `layout_size`.
- [x] 4.2 Copy from cabin (design D6):
  - extract `scale_objects(objects, old, new)` from `scale_surface`;
  - add `Workspace::copy_from_texture(source, now)`;
  - add a `CommandContext` count of the other main textures of the active vehicle.

  Verify with unit tests:
  - a 2048 → 4096 copy puts a rectangle at (100, 100), 200 × 50, at (200, 200), 400 × 100;
  - the copies are on top, selected, with fresh ids and the same links;
  - the source is unchanged;
  - Undo removes them;
  - an accessory or a trailer gives a count of 0.

## 5. Interface (tp-app)

- [x] 5.1 In the Colors panel palette:
  - swatch names as hover text and accessible names;
  - clicking a swatch applies the color and the link in one edit, and Add to palette links the target;
  - the linked swatch gets a ring and a name line;
  - the context menu has Edit Swatch… and Delete Swatch.

  Verify with UI tests: "Add and apply a palette color"; "Linked swatch marked"; "Picking another color" (no swatch marked after a recent color).
- [x] 5.2 Add the Edit Swatch popup: the name, the picker and hex, with live propagation through `live_edit`, OK through `commit_pending`, Cancel and Escape through `cancel_pending`, and a refused empty name. Verify with UI tests: "Edit the company red" across two textures, with one Undo; "Cancel an edit".
- [x] 5.3 Add the Styles panel (`PanelKind::Styles` after Colors, inserted collapsed by `sanitized()`):
  - Graphic styles and Text styles sections, each with an explanation when empty;
  - + New Style from Selection, with its disabled reasons;
  - preview chips, and text style names drawn in their own font;
  - the marks of the selection's styles;
  - click to apply;
  - the context menu: Rename inline, Redefine from Selection, Select Users on This Texture, Delete.

  Verify with UI tests for the spec scenarios "Restyle the fleet's lettering", "Apply by clicking", "New style from a selected object", "Delete keeps the looks" and "Text style needs a text", and the layout unit test "Layout from before the Styles panel".
- [x] 5.4 Add the Copy From Cabin… command (Vehicle menu, after Previous Texture) and its dialog: the other main textures with their object counts, the first one with objects chosen, and Copy disabled for an empty texture. Verify with UI tests: "Copy the standard cab onto the high roof" with the sample truck; "Not offered for accessories", with its tooltip.

## 6. Text, docs and checks

- [x] 6.1 Add every new message to `tp-i18n` in en, fr, de and es:
  - the swatch names "Color N" and Edit Swatch…;
  - the Styles panel: its title, sections, empty states, actions, reasons and default names;
  - Copy From Cabin… and its dialog;
  - the undo labels.

  Verify that `cargo test -p tp-app --test localization` passes and the keys match across languages.
- [x] 6.2 Update `docs/roadmap.md`:
  - record the decisions: linked palette and styles, symbols as their own change, a symbol edited in its own view;
  - add `symbols` as the next change after `brand-kit` and renumber;
  - move `brand-kit` to Shipped.

  Verify by reading the page against the specs.
- [x] 6.3 Add screenshots to `tests/screenshots.rs` (en and de): the palette with a linked swatch marked, the Edit Swatch popup, the Styles panel with both sections, and the Copy From Cabin dialog. Run `mise run screenshots` and check them visually. Run `mise run ci`. Verify that every step passes with no warnings.

## 7. Text styles carry the whole lettering (after trying the first version)

- [x] 7.1 Give `TextStyle` a `look` (fill, fill swatch, stroke, opacity), shared with `GraphicStyle` as `Look`. New, apply and redefine copy it, keeping gradient positions. `set_swatch_color` and `relink` cover it, and a text follows one style. Verify with unit tests: the spec's "A text style carries the whole lettering" and "Recoloring a lettering detaches it"; the swatch-linked text style recolors its users.
- [x] 7.2 Store the text style's look in `FileTextStyle` as optional fields (a branch file without them opens with the default look). Verify with the brand kit round trip and a text style without look fields.
- [x] 7.3 End the text being typed before every Styles panel operation. Verify with a workspace test of the spec's "Applying a style while typing": one Undo restores the look and keeps the text.
- [x] 7.4 Show the text style's fill and stroke chip in the Styles panel. Update the UI tests and screenshots. Run `mise run ci`.
