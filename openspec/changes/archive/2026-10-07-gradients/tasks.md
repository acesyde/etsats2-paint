## 1. Paint model (tp-core)

- [x] 1.1 Add `document/paint.rs` with `Paint`, `Gradient`, `GradientKind`, `ColorStop`, `MAX_STOPS = 16`, sorted `stops()`/`set_stops()` (clamped offsets, 2..=16 stops), the linear and radial defaults from D2, `Paint::solid_color()` and `Paint::is_visible()`, and export them from `document/mod.rs`. Verify with unit tests: sort stability for equal offsets, clamping, and min/max stop counts.
- [x] 1.2 Implement `Gradient::color_at(t)` (straight-alpha sRGB interpolation, padding outside 0..1). Verify with unit tests: red→blue at 0.5 = (128, 0, 128), a fade to transparent keeps the RGB, and a hard edge at equal offsets.
- [x] 1.3 Implement `Gradient::to_document(&Frame) -> Option<Affine>` (linear perpendicular in document space, radial conjugate axes, `None` when degenerate), plus the Angle and Aspect getters and setters that rebuild `minor` perpendicular. Verify with unit tests: follows a rotation, a flip and a non-uniform resize; the "Elliptical glow" 200 × 100 case; a degenerate gradient gives `None`.
- [x] 1.4 Implement `Gradient::remap(from: &Frame, to: &Frame)`. Make `Object::refit` and `set_path_rotation` remap the fill and stroke gradients from the frame on entry. Verify with unit tests: a path point edit that grows the bounds leaves `D(start)` and `D(end)` unchanged.
- [x] 1.5 Change `Object.fill` to `Paint` and `StrokeStyle.color` to `StrokeStyle.paint: Paint`. Update `DEFAULT_FILL` users, `stroke_region` and tp-core tests. Verify with `cargo test -p tp-core`.

## 2. File format v3 (tp-file)

- [x] 2.1 Add a frozen `v3.rs` (copy of v2 with `FilePaint`/`FileStop`, `FileObject.fill: FilePaint`, `FileStroke.paint: FilePaint`), `v3::from_v2`, `FORMAT_VERSION = 3`, `pub use v3 as current`, and the v1 → v2 → v3 chain in `migrate`. Reading with fewer than 2 stops is `Damaged`. Verify that `cargo build -p tp-file` succeeds.
- [x] 2.2 Update `tests/format.rs`:
  - `rich_project` gains a linear fill, a radial fill and a gradient stroke on a line;
  - `format_3_is_newer` becomes `format_4_is_newer`;
  - add a v2-fixture-opens-as-solid-paints test and a gradient round-trip test.

  Regenerate the v3 fixture with `cargo test -p tp-file -- --ignored write_current_fixture`, keeping the committed v2 fixture. Verify with `cargo test -p tp-file`.

## 3. Export rendering (tp-render)

- [x] 3.1 Add a `shader(paint, frame, opacity, scale)` helper (solid color, `LinearGradient`, single-circle `RadialGradient`, stop alpha × opacity, solid last-stop fallback). Make `fill_with`, `stroke_with` and `draw_path`/text drawing take paints. Verify that `cargo test -p tp-render` passes for the existing tests.
- [x] 3.2 Add pixel tests in `tests/render.rs`:
  - the black→white 1024 px export (each column non-decreasing, middle column within 1 of 128);
  - a radial gradient's center and edge colors;
  - a gradient that follows a 90° rotation;
  - an Outside stroke with a gradient;
  - a degenerate gradient renders the last stop.

  Verify with `cargo test -p tp-render`.

## 4. Canvas drawing (tp-app)

- [x] 4.1 Make `geometry_cache::uses_mesh` also return true for gradient fill and stroke paints. Verify with a unit test: a gradient rectangle uses meshes and a solid one does not.
- [x] 4.2 Add a `GradientTextures` cache (linear 2048 × 1 ramp, radial 512 × 512, `ClampToEdge` + linear filtering, keyed by kind+stops, evicted when unused for a frame). Verify with unit tests: texel colors match `color_at` at texel centers, and a second request reuses the texture.
- [x] 4.3 Give `mesh_to_screen` a `MeshPaint` (`Solid` or `Ramp { texture, doc_to_uv, tint }`) using the UV formulas of D4. Use it in `draw_mesh_object` and `draw_text` for the fill, casing, line and stroke layers. Verify with a unit test of `doc_to_uv` (start → u at the first texel center, end → last).
- [x] 4.4 Draw gradient previews for the selection swatches, the Colors target swatches and the Properties swatches. Add `render_gradients` in `tests/screenshots.rs` (a gradient livery scene: linear cab fade, radial glow, gradient lettering with an Outside stroke, gradient dashed line) and check the images visually with `mise run screenshots`.

## 5. Workspace state and commands (tp-app)

- [x] 5.1 Make `Style.fill: Paint` and `Style.stroke.paint`. Route `apply_color` to the selected stop when the target is a gradient. Add `apply_paint` and `edit_gradient(target, f, commit)`, which records one undo step per gesture with labels like "Change Fill Gradient" and "Change Stroke Gradient". Update `swap_fill_stroke` and the D reset. Verify with unit tests in `workspace.rs`.
- [x] 5.2 Add `PanelState.gradient_stop` and the kind change rules (solid → gradient defaults, linear ↔ radial keeps stops, gradient → solid uses the first stop). Verify with unit tests.
- [x] 5.3 Fix every remaining `fill`/`stroke.color` user (`combine.rs` with remap, `outline_text.rs` with per-letter remap from the text frame, `path_edit.rs`, the eyedropper copying whole paints, `canvas/mod.rs`, `home.rs`, `dialogs.rs`). Verify with `mise run lint`, plus unit tests for union and outlined rotated text keeping gradient positions.

## 6. Colors panel and GradientBar (tp-ui, tp-app)

- [x] 6.1 Add a `tp_ui::widgets::GradientBar` with checkerboard and ramp, markers, click-to-add, select, drag, drag-off delete (> 24 px), focusable markers with ←/→ ±1% and Delete/Backspace, a 16-stop limit and a 2-stop minimum, returning `GradientBarEvent`s with live/commit semantics. Add the Solid/Linear/Radial and Gradient tool icons. Verify with kittests of the widget.
- [x] 6.2 Update the Colors panel with:
  - the kind `SegmentedControl` ("Solid paint", "Linear gradient", "Radial gradient"; Mixed when kinds differ);
  - the `GradientBar`;
  - the Location, Angle and Aspect fields and Reverse;
  - the picker, hex, recent colors and palette bound to the selected stop.

  Verify with kittests in a new `tests/gradients.rs`:
  - "Make a fill linear" plus Undo;
  - "Add a middle stop";
  - "Recolor a stop with the hex field";
  - "Two stops minimum";
  - "New shapes use the current gradient";
  - a recent color on a stop;
  - "Swap a gradient fill with a solid stroke".

## 7. Gradient tool (tp-app)

- [x] 7.1 Add `Tool::Gradient` after Eyedropper with shortcut G, tool bar icon and name, and a crosshair cursor. Verify with kittests: pressing G activates it and the tool bar order matches the spec.
- [x] 7.2 Add `canvas/gradient_tool.rs`:
  - handle drawing (vector line, start/end or center/radius/aspect handles, stop ticks), with the transform handles hidden;
  - handle drags with Shift 45° snapping;
  - drag elsewhere sets a new vector on every selected visible, unlocked object, converting solid paints first;
  - click selects.

  Verify with kittests: "Drag a gradient across a rectangle" plus Undo, "Move the end handle", "Constrained angle", and a locked object left unchanged.

## 8. Integration and verification

- [x] 8.1 Kittest the eyedropper gradient pick ("Pick a gradient") and the persistence round trip of a gradient project through Save, then Open, in `tests/persistence.rs`. Verify with `cargo test -p tp-app`.
- [x] 8.2 Check canvas/export agreement: render a gradient scene with the wgpu harness and with `tp-render` at the same size, and check that the sampled interior pixels differ by at most 2 levels per channel. Verify with a test in `tests/screenshots.rs` or a dedicated wgpu test, skipped without a GPU like the other wgpu tests.
- [x] 8.3 Run `mise run fmt:check`, `mise run lint` and `mise run test`, plus the stress run (4 parallel × 5 rounds of panels, gradients, vector_tools and stroke_options), and confirm that everything passes. Benchmark panning with 50 gradient-filled texts, using the `text_images.rs` helper, and note the per-frame time.
