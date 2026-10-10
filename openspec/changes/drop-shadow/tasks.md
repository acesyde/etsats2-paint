## 1. Model (tp-core)

- [x] 1.1 Add `Shadow` (with `DEFAULT` and `clamped()`), `Object.shadow`, `Object::takes_shadow()` and `Look.shadow`, wired through `Look::of`, `matches`, `apply_to`, `recolor`, `unlink`, `set_swatch_color`, `relink` and `delete_swatch` (design Decision 1); verify with unit tests: drop-shadow "Shadow color follows its swatch", picking another color unlinks, deleting the swatch keeps the color unlinked, a group and an instance never take a shadow, shared-styles "A style carries the shadow" (New Style from Selection and apply), "Changing the shadow detaches", Redefine from Selection takes the shadow, and `clamped()` bounds every field.
- [x] 1.2 Count shadow links in `usage.rs` and bring them through `import.rs` (closure and remap) for library and paste; verify with unit tests: brand-impact "Through a shadow", shared-library "A shadow's swatch comes along" (Add to Library of a style), import of a symbol whose text's shadow links to a swatch reuses an equal swatch and remaps the link.

## 2. Rendering (tp-render)

- [x] 2.1 Add `shadow.rs` with the separable 3-pass box blur and `shadow_layer()` (bounds, silhouette through the existing draw functions, tint, blur), and draw it in `render` before each shadowed object (design Decision 2); verify with unit tests: the blur of a single opaque pixel approximates a Gaussian within 2 levels, blur 0 leaves the tinted silhouette, drop-shadow "Hard shadow" (pixel checks), "Opacity applies to the shadow" (group opacity too), a hidden object draws no shadow, an image's shadow follows its alpha, and texture-export "Shadow in the export".
- [x] 2.2 Check that Export Mod's textures go through the same `render` (no separate path) with a test exporting a mod whose texture has a shadowed text and comparing one shadow pixel with `render`'s.
- [x] 2.3 Measure in a release test: the sample truck at 4096 with 20 shadowed texts (blur 8) and one lettering at blur 200; record the timings in the test's output, and verify a full export stays under 3 s on this Mac (otherwise report before going further).

## 3. Canvas (tp-app)

- [x] 3.1 Add `shadow_cache.rs` (position-independent key, zoom-tied scale capped at 2048 px, worker thread, stale texture reuse, pruning) and draw the layers in `paint.rs` under each object (design Decision 3); extend `tests/common::settle_renders` to wait for it; verify with unit tests (moving an object reuses its entry, changing the blur or the zoom bucket renders again, at most one job at a time) and kittest: drop-shadow "Clicking the shadow" (a click on the shadow alone selects nothing), selection bounds and snapping ignore the shadow, and "Same in the export" (a canvas pixel at 100 % zoom against the export's pixel, within anti-aliasing).
- [x] 3.2 Measure the worker's render time for a 4096 px lettering at blur 200 at 100 % and 25 % zoom; verify each stays under 30 ms in release, otherwise lower the cap and note it in design.md.

## 4. Inspector

- [x] 4.1 Add `shadow_ops.rs` (`add_shadow`, `remove_shadow`, `set_shadow` through live edits) and the Appearance row: + Add a shadow, Shadow row with swatch, summary or "Mixed" and remove button, left out for groups and instances; the popover with the color widgets (palette links), opacity, offset X/Y and blur; strings of design Decision 6 in en/fr/de/es (design Decision 4); verify with kittest: drop-shadow "Add a shadow" (default values, summary, one Undo), "Edit the blur by dragging" (one undo step), "Mixed values", "Remove the shadow", "No shadow on a group", properties-panel "Sections of an image" (+ Add a shadow, no Fill/Stroke), Escape closes the popover, a palette click links the shadow color.

## 5. Files

- [x] 5.1 Add `FileShadow` as an optional field of `FileObject`, `FileGraphicStyle` and `FileTextStyle`, clamped on load (design Decision 5); verify with tp-file tests: project-files "Shadow saved and reopened" (link kept), "File without shadows" (an existing fixture loads unchanged and saves without a `shadow` key), and an out-of-range blur in a file loads clamped to 200.

## 6. Screenshots, checks and docs

- [x] 6.1 Add to `tests/screenshots.rs` a Workshop with shadowed lettering and a logo image (EN and DE, 100 % and 200 %), the inspector with the Shadow row and its popover open, and a selection with + Add a shadow; verify the renders against the mockup's Appearance rows (artboard 05) and that the shadow sits under its object.
- [x] 6.2 Run the long-text check (every interface text 40 % longer) over the Shadow row and popover; verify no label is clipped or overlaps.
- [x] 6.3 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --workspace --release --locked`; verify all pass.
- [x] 6.4 At ship time, update `docs/roadmap.md`: remove row 2f `drop-shadow` from "Next changes" and add it at the top of "Shipped" (one shadow per shape, path, text or image: color with swatch link, opacity, offset, blur 0–200 px; styles carry it; same on canvas and in exports; format 1 with an optional field; not in the Brand space's Edit Style… nor for new objects); verify `openspec validate drop-shadow --strict` passes.
