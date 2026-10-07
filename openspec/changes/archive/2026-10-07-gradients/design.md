## Context

**Model.** `Object.fill` is an `Rgba` (`tp-core/src/document/object.rs`), and so is `StrokeStyle.color`. Both `StrokeStyle` and the workspace `Style` are `Copy`, and many call sites rely on that (`o.stroke.map(|s| …)`, `ws.style.stroke()`).

**Canvas** (`tp-app/src/ui/workspace/canvas/paint.rs`):
- Rectangles and ellipses with a plain centered stroke are drawn as egui convex polygons with a uniform color.
- Everything else is drawn from document-space triangle meshes. That covers paths, polygons, styled strokes (`geometry_cache::uses_mesh`) and texts (`text_engine`). `mesh_to_screen` maps those meshes to screen vertices with one color and `WHITE_UV`.
- eframe runs on wgpu with 4× MSAA. Headless tests and screenshots use `egui_kittest` with its wgpu renderer.

**Export** (`tp-render`): tiny-skia 0.12 (via resvg), through `fill_with` and `stroke_with`, which take a `Color`.

**Files.** `tp-file` is at format v2, with a frozen-module and migration scheme (`v1`, `v2::from_v1`, `lib::migrate`) and committed fixtures. No release has been tagged.

**User decisions:**
- linear and radial gradients;
- gradients on fills and strokes;
- canvas handles plus panel fields;
- canvas drawn with ramp textures, chosen over a custom wgpu shader or vertex colors.

## Goals / Non-Goals

**Goals:**
- One `Paint` type used by every fill and stroke, rendered the same on the canvas and in the export.
- Gradients that follow their object through every transform and survive conversions (outlines, booleans, path edits).
- No new GPU pipeline, so behavior is identical in the app, kittests and screenshots.

**Non-Goals:**
- Conic or angular gradients, and repeat or reflect spread modes. Beyond the ends a gradient pads.
- Gradient swatches in the project palette, gradient presets, and gradient meshes or freeform gradients.
- Gradients on images or groups, and gradient opacity masks.
- Color spaces other than sRGB, and interpolation in linear light or OKLab.
- Draggable stop markers on the canvas. The canvas shows stop marks, and stops are moved in the panel.

## Decisions

### D1. `Paint` and `Gradient` in tp-core, `Copy`, at most 16 stops
New `document/paint.rs`:

```rust
pub enum Paint { Solid(Rgba), Gradient(Gradient) }
pub enum GradientKind { Linear, Radial }
pub struct ColorStop { pub offset: f32 /* 0..=1 */, pub color: Rgba }
pub struct Gradient {
    pub kind: GradientKind,
    /// Frame-unit coordinates: (0,0) = frame's local top-left, (1,1) = bottom-right.
    pub start: Point,   // linear start, radial center
    pub end: Point,     // linear end, radial radius point
    pub minor: Point,   // radial only: end of the perpendicular semi-axis
    stops: [ColorStop; MAX_STOPS], len: u8,
}
```

`Object.fill: Paint`, and `StrokeStyle.color` is renamed `StrokeStyle.paint: Paint`. Keeping `Paint` `Copy` with a fixed array (16 × 8 bytes) avoids rewriting every `stroke.map(|s| …)` and the `Style` copies. Sixteen stops is far more than liveries need, and the spec caps them. Stops are kept sorted by offset (stable, so equal offsets keep their order and give hard edges). They are accessed through `stops()` and `set_stops()`, which sort and clamp.

`Paint::solid_color() -> Option<Rgba>` and `Paint::is_visible()` (any alpha > 0) replace the `fill.a > 0` checks.

*Alternatives:*
- `Vec` or `Arc<[ColorStop]>`: unlimited stops, but `Paint` stops being `Copy`, which means a large mechanical churn across tp-app.
- Storing a full affine per gradient (Figma): more general, but no handle the user can see maps to it.

### D2. Gradient space and its mapping to the document
`Gradient::to_document(&self, frame: &Frame) -> Affine` maps **gradient space** to document space:
- **Linear:** (0,0) is the start and (1,0) the end. The y axis is the document-space perpendicular of start→end, so lines of equal color stay perpendicular to the visible vector.
- **Radial:** the unit circle maps to the ellipse whose conjugate semi-axes are `D(end) − D(center)` and `D(minor) − D(center)`. `D` is the frame's unit → document map, `frame.affine() * local_rect scaling`.

The parameter is `t = x` (linear) or `t = |p|` (radial). Because everything is stored in frame units, any frame change (move, resize including non-uniform, rotation, flip) carries the gradient along. An affine image of a circle is an ellipse, so radial gradients remain exact ellipses after any transform.

The Aspect shown in the panel is `|D(minor) − D(c)| / |D(end) − D(c)|`. Setting Aspect, Angle or dragging the aspect handle rebuilds `minor` perpendicular in document space. A degenerate mapping (start = end, or a zero minor axis) is detected by `to_document` returning `None`, and callers paint the last stop's color, as the spec requires.

Defaults:
- Linear: start (0, 0.5), end (1, 0.5).
- Radial: center (0.5, 0.5), end (1, 0.5), minor (0.5, 1), so the ellipse fits the frame.
- Solid → gradient: stops `[color @0, color with a = 0 @1]`.

### D3. Color interpolation shared by every consumer
`Gradient::color_at(t) -> Rgba` interpolates the straight (unpremultiplied) sRGB channels, padding outside [0, 1]. That is the same convention tiny-skia uses (it interpolates unpremultiplied, then premultiplies). The canvas ramps and the panel's "add a stop with the color there" use this function, and the export uses tiny-skia's native shader, so all three agree. Fading to `color @ a=0` therefore never darkens.

### D4. Canvas: textured meshes with gradient ramp textures
- **Routing:** `uses_mesh` also returns true when the fill or stroke paint is a gradient, so rectangles and ellipses with gradients go through `shape_mesh`. That path already serves styled strokes. The geometry cache key is unchanged, because paint is not geometry.
- **Ramp textures:** a new `GradientTextures` cache in tp-app (next to `ImageCache`), keyed by `(kind, stops)` and evicted when unused for a frame, like the geometry cache.
  - **Linear:** a 2048 × 1 RGBA ramp. Texel `i` holds `color_at((i + 0.5) / 2048)`.
  - **Radial:** a 512 × 512 texture over gradient space [−1, 1]². Each texel holds `color_at(min(|p|, 1))`.

  Textures are created with `TextureOptions { magnification: Linear, minification: Linear, wrap_mode: ClampToEdge }`. Clamping gives the pad behavior: outside [−1, 1]², the radial texture clamps to edge texels whose `t ≥ 1`.
- **UVs:** `mesh_to_screen` gains a `fill: MeshPaint` argument, either `Solid(Color32)` or `Ramp { texture, doc_to_uv: Affine, tint }`.
  - For each vertex, `uv = doc_to_uv * p`. `doc_to_uv` is `to_document⁻¹` followed by the gradient-space → texel-center UV map.
  - Linear: `u = (0.5 + t·(W−1))/W`, `v = 0.5`.
  - Radial: `u, v = (0.5 + (p+1)/2·(N−1))/N`.
  - UVs are affine in position, so the GPU interpolates them exactly. Linear gradients are exact up to ramp quantization. Radial gradients are exact up to bilinear sampling of a smooth function.
- **Opacity** is the vertex tint `Color32::from_white_alpha(α)`, which is premultiplied `(α, α, α, α)`.
- **Text** goes through the same path: `draw_text` passes the paint, and its meshes are already in document space.
- **Selection swatches, the target swatches and the gradient bar** draw the gradient with `tp_ui::widgets::GradientPreview`, a vertex-colored strip (linear) or rings (radial). tp-ui doesn't depend on tp-core, and a piecewise-linear strip is exact for a left-to-right preview.

*Alternatives (decided with the user):*
- Custom wgpu shader through paint callbacks: per-pixel exact, but needs the target format and MSAA count plumbed in, plus a fallback for headless tests.
- Per-vertex colors: wrong for radial gradients and needs subdivision.

### D5. Export: tiny-skia gradient shaders
`fill_with` and `stroke_with` take a `&Paint`, the object's frame and opacity instead of a `Color`. A helper `shader(paint, frame, opacity, scale) -> Option<tiny_skia::Shader>` builds the shader:
- **Solid:** a color.
- **Linear:** `LinearGradient::new((0,0), (1,0), stops, SpreadMode::Pad, T)`.
- **Radial:** `RadialGradient::new((0,0), (0,0), 1.0, stops, Pad, T)`, a two-point conical gradient degenerated to one circle.

`T = scale × to_document`. Stop alphas are multiplied by the opacity.

If tiny-skia refuses a shader (degenerate transform), `shader` falls back to the solid last-stop color. That matches D2 and the canvas, where `GradientTextures` maps degenerate gradients to a solid mesh.

The region paths (casing, Inside and Outside strokes, dashes) are already plain fills, so gradients work for every stroke style with no extra code.

### D6. Gradient remapping for conversions
`Gradient::remap(&self, from: &Frame, to: &Frame) -> Gradient` maps `start`, `end` and `minor` through `D_from` then `D_to⁻¹`. For linear gradients, `minor` is recomputed as the document perpendicular. It is used by:
- **Convert to Outlines** (`outline_text.rs`): each letter is remapped from the text's frame, so the gradient spans the word, not each letter.
- **Boolean operations** (`combine.rs`): from the frame of the object whose style is taken.
- **Path edits that refit the frame** (`Object::refit`): `refit` itself remaps the fill and stroke gradients, so every caller (point drags, handle edits, adding or deleting points) keeps them in place. `set_path_rotation` changes the rotation before calling `refit`, so it remaps from the frame it had on entry.
- **The Eyedropper:** no remap. The spec copies the gradient relative to the target's frame.

Duplicate and paste copy the object with its frame, so the gradient already stays in place.

### D7. Workspace and panel state
- **`Style`:**
  - `fill: Paint`, and `stroke.paint`;
  - `apply_color(target, Rgba)` stays the entry point for color edits. When the target paint is a gradient, it sets the selected stop's color.
  - New `apply_paint(target, Paint, label)` for kind changes, Eyedropper, Reverse and the like.
  - New `edit_gradient(target, impl Fn(&mut Gradient), commit)` for stop, angle, aspect and handle edits. It records one undo step per gesture, like `LineEdit`'s `Change::Apply { commit }`.
- **`PanelState`:**
  - `gradient_stop: usize`, the selected stop index of the current target, clamped on use.
  - There is no remembered "last gradient": the spec defines Solid → gradient as always "color → transparent".
- **Mixed selections:**
  - The kind control uses `common(kind)`.
  - The bar shows the first selected object's gradient.
  - An edit computes the new gradient from that one and writes it to every selected object.
- **`swap_fill_stroke`** swaps paints. Stroke-less objects gain a stroke only if the fill paint is visible, as today.
- **`D` reset** sets `Paint::Solid` defaults.

### D8. Colors panel layout
Top to bottom:
1. Target swatches (gradient previews).
2. Kind control `SegmentedControl` (Solid / Linear / Radial icons, accessible names "Solid paint", "Linear gradient", "Radial gradient"), shown when the target is gradient-capable. The stroke target shows it only when a stroke exists.
3. In gradient mode, a new **`tp_ui::widgets::GradientBar`**: a checkerboard plus a ramp rect, with markers below. It handles click-to-add, select, drag, drag-off-to-delete (more than 24 px vertically) and keyboard (Tab focus, ←/→ ±1%, Delete). It returns a `GradientBarEvent`. Under it go Location %, Angle °, Aspect % (radial) and Reverse.
4. The existing picker, models, hex, recent colors and palette, bound to the selected stop's color.

### D9. Gradient tool
- **Registration:** `Tool::Gradient` goes after Eyedropper, with shortcut `G` (plain G is free; ⌘G is Group) and a Phosphor `GRADIENT` icon.
- **Module:** a new canvas module `canvas/gradient_tool.rs` with handle hit testing, using the same handle radius as path editing.
- **Drag handles:** start/end, center/end/minor. Shift snaps the angle in document space.
- **Drag elsewhere:** sets a new vector for every selected, unlocked and visible object, using `edit_gradient` and converting solid paints first. With radial gradients, the minor axis is rebuilt perpendicular at the current aspect.
- **Click without drag:** selects, like the Select tool.
- **Paint layer:** the tool draws the vector line, handles and stop ticks with the selection style, and hides the transform handles while active.

### D10. File format v3
- **New frozen `v3.rs`:**
  - a copy of v2, with `FilePaint` = `Solid([u8; 4]) | Linear { start, end, stops } | Radial { center, end, minor, stops }`;
  - `FileStop { offset: f32, color: [u8; 4] }`;
  - `FileObject.fill: FilePaint` and `FileStroke.paint: FilePaint`.
- **Migration:** `v3::from_v2` wraps colors in `Solid`, and `migrate` chains v1 → v2 → v3.
- **Version:** `FORMAT_VERSION = 3`.
- **Reading:** stop counts are clamped to 2..=16 and offsets to 0..=1. Fewer than 2 stops makes the file `Damaged`.
- **Fixtures:** the committed v2 fixture stays as a migration test, and a v3 fixture is written by the ignored `write_current_fixture` test, with `rich_project` gaining gradient objects. `format_3_is_newer` becomes `format_4_is_newer`.

The type of an existing field changes, so a version bump is required (persistence D2). The version bump is also why this lands before any release.

## Risks / Trade-offs

- **Ramp resolution.** Hard stops in a linear gradient blur over 1/2048 of its length on the canvas. That is about 4 px for an 8192 px-long gradient at 100% zoom. Radial hard stops blur over about 1/256 of the radius. The export is exact, and the canvas difference is visible only at high zoom on hard stops. If needed later, the ramp size can be chosen from the on-screen gradient length.
- **sRGB texture filtering.** egui textures are sRGB, so bilinear filtering between adjacent texels happens in linear light. Adjacent texels differ by at most a few levels, so the error is below one 8-bit step.
- **Fixed 16-stop limit.** Visible in the UI (adding is disabled when full). It can move to a heap representation later without a format change, since `FilePaint` stores a `Vec`.
- **`StrokeStyle.color` → `paint` rename.** Touches many tests and the screenshot scenes. It is mechanical, and the compiler finds every site.
- **Refit remapping.** `Object::refit` remaps gradients, so any code that sets a path's frame some other way would skip it. The `refit` doc says it is the only way to change a path's frame after edits, and a unit test covers this.
- **Texture memory.** Each radial ramp is 1 MiB. The cache drops textures unused for a frame, so memory is bounded by the gradients visible at once.
