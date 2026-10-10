## 1. Usage (tp-core, pure)

- [x] 1.1 Add `tp-core/src/usage.rs` with `Count` and `Usage` and `Project::usage()` per design Decision 1 (symbol content swatch sets, per-object dedup, groups recurse, instance counts once for its symbol and for each swatch of its content, textures through per-surface sets); verify with unit tests: brand-impact "A swatch used on two textures", "Through a symbol", "An object linked twice", "Unused swatch", "Symbol usage", a gradient stop linked to a swatch, texts following a text style, and `usage().symbol(id).objects == instance_count(id)` on the sample fleet.
- [x] 1.2 Add `Project::set_graphic_style_look(id, look)` and rewrite `redefine_graphic_style` on top of it (design Decision 2); verify with unit tests: followers on two surfaces take the new stroke width and opacity and keep `style == Some(id)`, an instance whose symbol content follows the style is refreshed, and the existing `brand.rs` tests pass unchanged.

## 2. Editor model (tp-app)

- [x] 2.1 Add the usage cache to `Workspace` (`Workspace::usage()`, key of surface and symbol `Arc` pointers plus palette and style ids) and `brand_ops::usage_label` / `symbol_usage_label` with the fluent plural strings of design Decisions 1 and 6 in en/fr/de/es; verify with unit tests: brand-impact "Usage follows undo" (link one more rectangle, undo, redo), the cache is not recomputed on a frame without change, delete swatch then undo restores the count, and `cargo test -p tp-i18n --release` passes.
- [x] 2.2 Replace `SwatchEdit` with `BrandEdit` / `BrandEditTarget` / `EditValue` and `ws.panels.brand_edit` (design Decision 2): `start_swatch_edit`, `start_style_edit`, `set_edit_value` (rebuilds `preview` on a cloned project, only when the value changes), `apply_brand_edit`, `cancel_brand_edit`; remove `preview_swatch_color`, `finish_swatch_edit`, `cancel_swatch_edit`; verify with unit tests replacing `cancelled_swatch_edit_restores_and_records_nothing`: the document and `history.len()` are unchanged while a value is pending, Apply records one step and keeps links, Undo restores, Cancel records nothing, rename-only applies, unchanged Apply records nothing, empty name refuses to apply, and a style edit changes followers on two surfaces.
- [x] 2.3 Cancel the open edit when the space changes (`state.rs` space switch path); verify with a unit or kittest test: brand-space "Switching space cancels the editor".
- [x] 2.4 Measure `set_edit_value` on a 40-surface project (sample truck surfaces duplicated, one swatch linked on all) in a release-mode test; verify it stays under 5 ms per call, otherwise add the 50 ms rebuild throttle while the pointer is down (design Risks).

## 3. Preview thumbnails

- [x] 3.1 Extend `SurfaceThumbnails` with a texture-name prefix and an `only` surface filter (design Decision 3), add `Workspace.preview_thumbs` reset on edit start, updated each frame while an edit is open, and extend `tests/common::settle_renders` to wait on it; verify with unit tests: only `affected` surfaces are rendered (`renders` count), a new value re-renders only surfaces whose objects changed, and the workspace's own thumbnails are not re-rendered by a pending edit.

## 4. Editor UI

- [x] 4.1 Add `ui/workspace/brand_editor.rs` with the shared body of design Decision 4 (caption and name, swatch picker or style Fill/Stroke toggle + stroke width/None + opacity, Before/After, hex, impact box with the tiles grid and note, Cancel / Apply to Fleet), making the `colors.rs` picker widgets `pub(crate)`; host it in a right `Panel` of the Brand space and in a modal for the Workshop popover, replacing `edit_swatch_popup`; verify with kittest in `tests/brand_space.rs`: brand-impact "Edit the company red", "Nothing changes before Apply" (canvas pixel and `ws.project` unchanged, After block value), "Impact before applying" (impact text, tile count), "Cancel" (Escape), "Rename only", "Restyle the stripes", "Empty name"; color-panel "Edit the company red" from the popover (modal host), "Cancel an edit", "Edit Swatch from the Brand space" (panel host); shared-styles "Edit a graphic style" and "No editor for text styles"; brand-space "Edit Style from Brand", "Recolor from Brand", "Rename a swatch".
- [x] 4.2 Add Edit Style… to the graphic style context menus (Brand cards and Resources tab) with the `styles-edit` string; verify with kittest that it is offered for graphic styles only.

## 5. Usage in the lists

- [x] 5.1 Show the usage on palette, graphic style and text style cards, "N instances · N textures" on Brand symbol cards, the edited card marked by the selected fill, and the usage line in the color popover's swatch tooltip (design Decision 5); verify with kittest: brand-space "Usage on a swatch card", symbols "Instances and textures in Brand", brand-space "Create a symbol from Brand" ("1 instance · 1 texture"), color-panel "Usage on hover", and that symbols "Instance count in the Resources tab" still reads "6 instances".

## 6. Screenshots, checks and docs

- [x] 6.1 Add to `tests/screenshots.rs` the Brand space with the swatch editor open and with the style editor open, on the sample fleet, in English and German at 100 % and 200 %, plus the Workshop modal host; verify the renders show every German label whole, the tiles show the new color, and the layout matches the mockup's artboard 07 panel (340 points, Before/After, impact box, Apply to Fleet).
- [x] 6.2 Run the long-text check (every interface text 40 % longer) over the editor and the cards; verify no label is clipped or overlaps.
- [x] 6.3 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --workspace --release --locked`; verify all pass.
- [x] 6.4 At ship time, update `docs/roadmap.md`: remove row 2d `brand-edit-preview` from "Next changes" and add it at the top of "Shipped" (usage counts on swatches, styles and symbols; before/after editor for swatches and graphic styles with impact and live thumbnails, nothing applied before Apply to Fleet; text styles and the library relation left for later); verify `openspec validate brand-edit-preview --strict` passes.
