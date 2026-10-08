## Context

Today, flipping only happens inside `transform::resize_one`, when a resize handle is dragged past the opposite side. The scale factors are then negative, and each kind of object handles it in its own way:

- **Rectangles and ellipses** are symmetric: only their rotation is folded.
- **Paths** take the exact transform in their points.
- **Polygons** take their rotation from where their local "up" goes.
- **Gradients** are remapped exactly.
- **Groups** recurse into their children.
- **Instances** compose the transform into their placement (`place_by`). Their content is then rebuilt with `apply_affine`, which goes through `resize_one`.

A `Frame` (center, size, rotation) cannot express a mirror. Texts and images are drawn straight from their frame: `tp_text::layout_to_doc` for texts, and the `place` affine of `tp_render::draw_image` and the mesh UVs of the canvas `draw_image` for images. So they move with a flip but are never mirrored. That is the gap this change closes (see proposal.md).

## Goals / Non-Goals

**Goals:**
- One mirrored state, used in the same way by flip commands, handle flips and instance placements.
- Every reader of a text or an image (render, canvas, caret hit testing, glyph outlines, eyedropper) honours the mirror through a small number of functions.
- Flip commands built on the existing transform code, without a second transform path.

**Non-Goals:**
- Mirroring in `Frame` itself. Frames, handles, snapping and the Transform panel stay mirror-free.
- A mirrored state on shapes and paths: their geometry already carries it.

## Decisions

### `Object.mirrored: bool`, meaningful for texts and images
A mirrored object is drawn as `frame.affine() · scale(−1, 1)`, a reflection across its local vertical axis, inside the same frame. A vertical flip is that reflection plus a half-turn (`scale(1, −1) = rotate(180°) · scale(−1, 1)`), so a single flag covers both axes.

The flag is kept false on every other kind. Groups and instances carry no flag: their children do.

- *Alternative: a flag in `Frame`.* Rejected. `Frame::affine` is used for handles, snapping, hit testing and the Transform panel. A mirror there would swap handle positions and change many call sites for no visible gain.
- *Alternative: a negative `TextBlock.scale.x` for texts.* Rejected. `sync_text_scale` recomputes the scale from the frame, and images have no scale.

### Mirror detection in `resize_one`
When the transform applied to an object has a negative determinant (`sx · sy < 0`), a text or an image toggles `mirrored`. Its rotation is then taken from where its local "up" goes, exactly as for polygons. Under a reflection, "up" is preserved by `scale(−1, 1)`, so the up direction alone fixes the rotation:

- A horizontal flip of an object at θ gives −θ.
- A vertical flip gives 180° − θ.

Because handle flips, flip commands and `apply_affine` (instance content) all go through `resize_one`, they all get it. A mirrored image in a symbol shows un-mirrored in an instance whose placement mirrors it, since the two reflections cancel.

### `transform::flip(objects, axis)`
`flip` takes the selection frame of the objects. With several objects, the selection frame is axis-aligned, so it is computed from their union. For one object it is that object's own frame: the reflection still uses the texture axes through its center, with bounds rotation 0, so a rotated object's rotation is reversed (30° becomes −30°).

`flip` builds the reflection `T(c) · S · T(−c)`, where `S = scale(−1, 1)` for horizontal and `scale(1, −1)` for vertical. It then calls `resize_one(o, transform, sx, sy, 0.0)` on each object. A new `FlipAxis { Horizontal, Vertical }` names the axis. The guide `Axis` in `project.rs` describes a line's orientation, and reusing it would read backwards.

- *Alternative: `apply_affine` with the reflection.* That works, but it decomposes the reflection into a vertical scale and a half-turn: a detour that is harder to read.

### Readers
- **Texts:** `tp_text::layout_to_doc` appends the mirror after `frame.affine()`. Every text reader in the code base goes through it: tp-render, canvas paint and session highlight, `text_engine` (anchor, glyph outlines, caret hit testing) and `text_session`. So drawing, editing and Create Outlines follow with no further change.
- **Images:** a helper `Object::content_affine()` returns `frame.affine()` with the mirror. tp-render `draw_image` uses it in `place`. The canvas `draw_image` swaps the U coordinates of its mesh when mirrored. The eyedropper reads `u` as `1 − u` when mirrored.

### Commands and UI
`CommandId::Flip(FlipAxis)` covers both commands.

- **Shortcuts and enabling:** Shift+H and Shift+V, both free today. Enabled by `editable_selection`, with the existing "select an object" reason, and disabled while editing text, like Align.
- **Workspace:** `Workspace::flip_selection(axis, now)` applies `flip` to the selected top-level objects and records the undo label `op-flip-horizontal` or `op-flip-vertical`. Recording relinks, which also refreshes instances.
- **Where the commands appear:**
  - the Object menu, in a new group after Align;
  - the canvas context menu, after Bring Forward and Send Backward;
  - the Transform panel, as two icon buttons at the end of the distribute row after a gap. The align row, with its Align to selector, is already full at the column's default width (280 px); adding them there would widen the column and shrink the canvas.
- **Icons:** `ph::FLIP_HORIZONTAL` and `ph::FLIP_VERTICAL`.
- **Strings:** en/fr/de/es locale strings for the commands and undo labels.

### Removing Mirror to Other Side
`CommandId::MirrorToOtherSide` goes away: the variant, its entry in `CommandId::ALL`, its metadata, the Object menu item, the `SOON_VEHICLES` constant (used by nothing else) and the strings `cmd-mirror-to-other-side` and `reason-soon-vehicles` in en/fr/de/es. No spec mentions the command, so no spec changes. Projects and preferences never stored it, so nothing needs migrating.

- *Alternative: keep it disabled until packages have mirror data.* Rejected. No change on the roadmap plans that data, and a permanently disabled "Mirror…" item next to the Flip commands would invite confusion.

### File format
`FileObject.mirrored: bool` uses `#[serde(default, skip_serializing_if = "is_false")]`. Unmirrored objects write nothing, so existing files and the `v1.truckpaint` fixture are unchanged and the format version stays 1. That matches how `fill_swatch` and `style` were added.

## Risks / Trade-offs

- [An older TruckPaint opening a file with mirrored objects ignores the field and shows them unmirrored.] → Accepted, as with earlier optional fields. No forward-compatibility rule exists for v1.
- [Text relayout keeps the anchor through `layout_to_doc`. Typing in a mirrored text therefore grows it on the mirrored side.] → This is the specified behavior. A UI test types into a mirrored text and checks that the anchor stays put.
- [Code that reads `frame.affine()` directly for a text or an image would miss the mirror.] → A grep of `frame.affine()` in the text and image paths is part of the tasks. Tests cover render, eyedropper, outlines and caret placement.
- [Floating-point drift after repeated flips.] → `flip` maps exact reflections. A test checks that two flips restore the frame to within 1e-9 and the flag to its starting value.
