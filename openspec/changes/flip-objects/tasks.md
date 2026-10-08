## 1. Model and transforms (tp-core)

- [x] 1.1 Add `Object.mirrored: bool` (false in `Object::new`, `from_path`, `group` and every constructor) and `Object::content_affine()`, which is `frame.affine()` followed by `scale(−1, 1)` when mirrored. Verify with `cargo build --workspace` and a unit test: `content_affine` maps local (10, 0) to the left of the center for a mirrored, unrotated image.
- [x] 1.2 In `transform::resize_one`, when `sx · sy < 0`, toggle `mirrored` on texts and images and take their rotation from the transformed local "up", as for polygons (design: Mirror detection). Verify with unit tests:
  - dragging the right handle of an image past its left side mirrors it and keeps rotation 0;
  - a vertical handle flip gives `mirrored` and 180°;
  - a positive resize leaves `mirrored` unchanged;
  - the existing flip tests (paths, gradients, triangles) still pass.
- [x] 1.3 Add `FlipAxis { Horizontal, Vertical }` and `transform::flip(objects, axis)`: a reflection through the center of `selection_frame`, along the texture axes, through `resize_one` with bounds rotation 0. Verify with unit tests:
  - a rotated rectangle at (400, 400) and 30° gives −30° at the same center;
  - two objects swap ends and keep the union bounds;
  - an arrow path flipped twice equals the original (1e-9);
  - an image flipped horizontally then vertically is not mirrored and is turned 180°;
  - an instance's placement gets a negative determinant, and its image child is mirrored after `refresh_instances`;
  - fills linked to swatches and objects following styles stay linked after `relink`.

## 2. Drawing and reading (tp-text, tp-render, tp-app)

- [x] 2.1 Apply the mirror in `tp_text::layout_to_doc`. Verify with a tp-render test: each glyph of a mirrored text is the unmirrored glyph reflected across the frame's vertical axis (same height, x bounds reflected about the center), and the first glyph lies at the right.
- [x] 2.2 Use `content_affine()` in `tp_render::draw_image`'s `place`. Verify with a tp-render test: an image that is red on its left half renders red on the right when mirrored.
- [x] 2.3 Canvas: swap the mesh U coordinates in `canvas/paint.rs::draw_image` when mirrored. In the eyedropper (`canvas/mod.rs`), use `content_affine().inverse()` so the sampled UV is the pixel shown. Then grep `frame.affine()` over the text and image paths in tp-app and tp-render, and list here any other reader fixed. Verify with a UI test: the eyedropper on the left half of a mirrored red|blue image picks blue.

  Review result: the other `frame.affine()` readers in tp-app and tp-render act on paths (snapping, point editing, Direct Selection), not on texts or images. Also fixed: the text mesh cache (`text_engine::mesh`) now compares `mirrored` too, so a reused `Arc` address can't serve a stale mesh. The 3D preview is still a placeholder; export goes through tp-render.
- [x] 2.4 Check that text editing and Create Outlines follow the mirror through `layout_to_doc`, and fix `text_engine::place` / `anchor` if they don't. (`place` rebuilt the frame center without the mirror and now applies it; `anchor`, caret clicks and outlines already went through `layout_to_doc`.) Verify with UI tests:
  - typing "PORT" at the end of a mirrored "TRANS" gives "TRANSPORT", still mirrored, with its anchor (right edge on screen) unchanged; clicking near its right edge puts the caret at the start;
  - Create Outlines on a mirrored text gives letter paths whose bounds match the text's, with the first letter on the right.

## 3. File format (tp-file)

- [x] 3.1 Add `FileObject.mirrored` with `#[serde(default, skip_serializing_if = …)]`, written by `object_to_file` and read by `object_from_file`. Verify with tests:
  - a round trip of a mirrored image, a vertically flipped text and an unmirrored image restores each flag and rotation;
  - the `v1.truckpaint` fixture still loads and writes byte-for-byte as before (no `mirrored` key for unmirrored objects).

## 4. Commands and UI (tp-app, tp-ui, tp-i18n)

- [x] 4.1 Add `CommandId::Flip(FlipAxis)` with metadata:
  - `op-flip-horizontal` with Shift+H, and `op-flip-vertical` with Shift+V (one label for the command and its undo step, as for Align);
  - icons `FLIP_HORIZONTAL` and `FLIP_VERTICAL` in `tp_ui::icons`;
  - enabled when `editable_selection` holds and no text is being edited, with the existing select-an-object reason;
  - included in `CommandId::all()` and the shortcut conflict check. It is not in `changes_shapes()`: flipping an instance only changes its placement, and the spec requires instances to flip.

  Verify that the shortcut conflict test and the command tests pass.
- [x] 4.2 Add `Workspace::flip_selection(axis, now)`. It applies `transform::flip` to the selected top-level objects and records `op-flip-horizontal` or `op-flip-vertical`. Dispatch it from `state.rs`. Verify with a unit test: one undo step restores the objects.
- [x] 4.3 Add both commands to:
  - the Object menu, in their own group after Align;
  - the canvas context menu, after Send Backward;
  - the Transform panel, as two icon buttons at the end of the distribute row after a gap (the align row would overflow the column's default width and widen it).

  Verify with UI tests: Shift+H on a selected image mirrors it, and Undo restores it; Object › Flip Vertical works from the menu; the Transform panel's Flip Horizontal button runs the command; both menu items are disabled with an empty selection.
- [x] 4.4 Remove `CommandId::MirrorToOtherSide`: the variant, its entry in `ALL`, its metadata, the Object menu item, the `SOON_VEHICLES` constant, and the strings `cmd-mirror-to-other-side` and `reason-soon-vehicles` in en/fr/de/es. Verify that `grep -rn "MirrorToOtherSide\|mirror-to-other-side\|soon-vehicles" crates` returns nothing and that the tp-i18n and command tests pass.
- [x] 4.5 Add the strings in en/fr/de/es: the commands and undo labels (one key each; the disabled reason reuses `reason-no-selection`). Verify with the tp-i18n key-parity test and `cargo test -p tp-i18n`.

## 5. Docs and checks

- [x] 5.1 Add a `render_flip` screenshot test: a logo image and a text, with their mirrored copies side by side. Run it with `cargo test -p tp-app --test screenshots render_flip -- --ignored`, then inspect `target/screenshots/`.
- [x] 5.2 Update `docs/roadmap.md`: list `flip-objects` under Shipped, with the decisions (texts and images mirror; axis through the selection center; Mirror to Other Side removed, and to come back only as a change that adds left/right mapping data to packages). Verify by reading the file.
- [x] 5.3 Run `mise run ci` (fmt:check, clippy `-D warnings`, tests, release build) and `openspec validate flip-objects --strict`. Both pass.
