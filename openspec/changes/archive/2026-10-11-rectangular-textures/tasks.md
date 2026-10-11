## 1. Size type and project files

- [x] 1.1 Change `Surface.size` to `kurbo::Size` with `bounds()`, `max_side()` and `size_label()` ("4096²" / "8192 × 1024"), and fix every compile error across the workspace through these helpers (design Decisions 1–2); verify `cargo build --workspace` and the existing test suite pass unchanged for square surfaces, and unit tests for `size_label` and `bounds` (document-model "Rectangular texture").
- [x] 1.2 Save a surface's size as `[w, h]` in `tp-file` and mark a numeric surface `size` as a development format in `is_development_v1` (design Decision 7); verify with tp-file tests: a rectangular surface round-trips, project-files "File from before rectangular textures" (development-format message), and the existing format tests updated to `[w, h]`.

## 2. Packages and Custom vehicle

- [x] 2.1 Replace `Texture.size: u32` with `TextureSize { width, height }` serialized as `[w, h]`, validated per side (powers of two 64..=8192), and rewrite `sample.rs` with pairs (design Decisions 1, 6); verify with tp-vehicles tests: vehicle-packages "Rectangular texture" (an `[8192, 1024]` base installs and a project gets an 8192 × 1024 surface) and "Single size refused", existing validation tests updated.
- [x] 2.2 Custom vehicle (`tp-pack/src/custom.rs` and its dialog): width and height from the image, rounded per side, warning only when scaled, Width and Height selectors, strings in en/fr/de/es (design Decision 6); verify with tests: custom-vehicles "Drop the game's templates" (4096², 1024²), "Not square" (4096 × 2048, no warning), "Size that isn't a power of two" (4000×1000 → 4096 × 1024 with the warning), and `tpv check` printing "8192 × 1024".
- [x] 2.3 Update `docs/vehicle-package-format.md` (the `size` field as `[w, h]`, examples, limits, no more "make them square") and the `tpv` help text; verify the doc's example manifest validates with `tpv check` in a test.

## 3. Rendering and exports

- [x] 3.1 Give `RenderOptions` a width and a height and render rectangular surfaces (design Decision 3), including the shadow layer's surface clip; verify with tp-render tests: an 8192 × 1024 surface renders a 8192 × 1024 pixmap with an object at its right end in place, and a shadow near the bottom edge is clipped at 1024.
- [x] 3.2 Export Texture dialog: size choices dividing both sides and labelled "W × H"; PNG and DDS of the real shape (design Decision 3); verify with tests: texture-export "Rectangular texture" (1/4 of 8192 × 1024 is 2048 × 256), "Rectangular DDS" (DXT5, 8192 × 1024, 14 levels), and the existing square scenarios unchanged.
- [x] 3.3 Export Mod writes each texture at its own size; verify with a mod-export test: "Rectangular texture in the mod" (the base DDS header reads 8192 × 1024 and 14 mipmap levels).

## 4. Canvas, thumbnails and labels

- [x] 4.1 Canvas: `ScreenMap::fit`, artboard, template rectangle, rulers, grid, guides limits, snapping to the artboard edges and center, placement in the middle of the view, image fitting, following the real shape (design Decision 2 list); verify with kittest on an 8192 × 1024 surface: workspace-layout "Rectangular artboard" (centered, fully visible, eight times wider than tall), rulers' last labels near 8192 and 1024, a grid line at y = 1024 and none beyond, snapping to the artboard's bottom edge at 1024, Place puts an image in the middle of the visible part.
- [x] 4.2 Thumbnails keep the shape: render with the longest side at 128 and draw aspect-fit in the Textures tab, the Project space, recent projects and the Brand impact tiles (design Decision 4); verify with unit tests (an 8192 × 1024 thumbnail is 128 × 16) and a kittest that the Textures tab's thumbnail rect of that texture is 8 times wider than tall.
- [x] 4.3 Size labels through `size_label` in the breadcrumb, the Textures tab, the Project space, the status bar and the inspector's texture summary; verify with kittest: workspace-spaces breadcrumb "… 8192 × 1024" for a rectangular texture and "… 1024²" for the sample's Side skirts, vehicle-projects Textures tab row size.

## 5. Scaling across texture sizes

- [x] 5.1 Make `scale_objects` non-uniform through the resize logic of `transform.rs` and use it in Update Template and Copy from cabin (design Decision 5); verify with tests: vehicle-projects "Different sizes" unchanged for squares; a rectangle at (100, 100), 200 × 50 copied from 4096² onto 8192 × 2048 lands at (200, 50), 400 × 25; Update Template from 4096 × 4096 to 4096 × 2048 halves heights and y positions and keeps widths; a text keeps following its style after the scaling.

## 6. Screenshots, checks and docs

- [x] 6.1 Add to `tests/screenshots.rs` a project with a rectangular trailer texture (8192 × 1024) and a rectangular accessory (2048 × 256): the Workshop fitted, the Textures tab, the Project space's texture rows, and the Export Texture dialog, in EN and DE at 100 %; verify the renders show undistorted shapes and the "W × H" labels whole.
- [x] 6.2 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --workspace --release --locked`; verify all pass.
- [x] 6.3 At ship time, update `docs/roadmap.md`: replace rows 3 `paintjob-importer` and 5 `marketplace` per the `official-vehicles` proposal (rectangular-textures shipped, official-vehicles next, marketplace postponed, first-run no longer depending on it), and add `rectangular-textures` at the top of "Shipped" (textures of any power-of-two width and height from 64 to 8192; old packages and project files refused); verify `openspec validate rectangular-textures --strict` passes.
