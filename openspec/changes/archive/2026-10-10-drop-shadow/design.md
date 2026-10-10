## Context

**Model** (`tp-core`):
- `Object` has `fill`, `fill_swatch`, `stroke: Option<StrokeStyle>` (with `paint` and `swatch`), `opacity`, `style`, and `text.style_id`.
- `brand.rs::Look { fill, fill_swatch, stroke, opacity }` is the look of graphic and text styles. `Look::of`, `matches`, `apply_to`, `recolor` and `unlink` drive style following, swatch edits and relinking.
- `usage.rs` counts swatch links for the Brand space.
- `import.rs::closure` collects the swatches an element links to (`paint(&o.fill, o.fill_swatch)`, …) and `remap_paint` rewrites them on import and paste.

**Export rendering** (`tp-render/src/render.rs::render`):
- tiny-skia;
- walks `tree::draw_list(&surface.objects)`, which yields `(object, effective opacity)` in stacking order;
- draws each object straight into the output `Pixmap`: `draw_path` for shapes, paths and texts (glyph outlines), `draw_image` through a layer.
- tiny-skia has no blur.

**Canvas** (`ui/workspace/canvas/paint.rs`):
- egui meshes from `geometry_cache` (tessellated outlines per zoom bucket);
- images and gradients as egui textures (`image_cache`, `gradient_textures`);
- no offscreen raster and no blur.

**Inspector** (`ui/workspace/inspector.rs`): Appearance rows for Fill and Stroke, each with a popover beside the inspector (`Popover`, closed with Escape). The color popover's picker, palette and hex widgets live in `panels/colors.rs`.

**File** (`tp-file/src/v1.rs`): serde structs. `FileObject`, `FileGraphicStyle` and `FileTextStyle` already use `#[serde(default, skip_serializing_if = …)]` for optional fields.

**Mockup** (artboard 05): Appearance lists Fill, Stroke and a muted "+ Ajouter une ombre" (12 px) under them. "Remplissage, contour, ombre : une ligne compacte chacune."

**Decisions taken in /opsx:propose:**
- stay in file format 1 with an optional field;
- shadows on shapes, paths, texts and images only.

## Goals / Non-Goals

**Goals:**
- One `Shadow` value in the model, carried by objects and style looks, linked to swatches like a fill.
- One rasterizer for the shadow layer in tp-render, used by both the export and the canvas, so they match.
- Canvas cost bounded: cached per object, independent of the object's position, at a resolution tied to the zoom.

**Non-Goals:**
- Inner shadow, glow, spread, several shadows, blend modes, shadows on groups or instances.
- A shadow for new objects (the "new objects" look).
- Editing the shadow in the Brand space's Edit Style… editor: it keeps the style's shadow unchanged. Redefine from Selection sets it.
- Using the shadow color as an eyedropper target (the popover has its own picker).

## Decisions

### 1. Model

```rust
pub struct Shadow {
    pub color: Rgba,
    pub swatch: Option<SwatchId>,
    pub opacity: f32,   // 0..=1
    pub offset: Vec2,   // texture px, each in -1000..=1000
    pub blur: f64,      // texture px, 0..=200
}
impl Shadow { pub const DEFAULT: Shadow = /* black, 0.5, (8, 8), 8 */; pub fn clamped(self) -> Self }
```

- `Object.shadow: Option<Shadow>`. `ShapeKind::Group` and `Instance` never hold one: the setters skip them, and `Object::takes_shadow()` decides.
- `Look.shadow: Option<Shadow>`:
  - `Look::of` copies it;
  - `matches` compares it (exact equality);
  - `apply_to` sets it;
  - `recolor` and `unlink` treat `shadow.color` / `shadow.swatch` like a solid paint.
- An image can't follow a graphic style, which is unchanged: its shadow is its own.
- Swatch paths:
  - `set_swatch_color` reaches shadows through the same `update_surfaces` walk (a `recolor_shadow` helper);
  - `relink` unlinks shadows whose color no longer matches;
  - `delete_swatch` keeps the color and unlinks.
- Usage (`usage.rs`): an object's swatch set includes `shadow.swatch`, and so does a symbol's content set.
- Import (`import.rs`): `closure` adds the shadow swatch of objects and looks, and `remap_*` rewrites it.

### 2. Rasterizing the shadow layer (`tp-render/src/shadow.rs`)

```rust
pub struct ShadowLayer { pub pixmap: Pixmap, pub origin: Point /* doc px of pixmap (0,0) */, pub scale: f64 }
pub struct DrawCache { /* glyph outlines and decoded assets, Send, kept between renders */ }
pub fn shadow_layer(object: &Object, opacity: f32, scale: f64, clip: Option<Rect>, asset: Option<&Asset>,
                    fonts: &mut FontLibrary, cache: &mut DrawCache) -> Option<ShadowLayer>
```

1. The silhouette's bounds are the object's drawn bounds (the outline bounds grown by the stroke and line widths times the miter limit, or the image's frame), moved by the offset and grown by the blur's reach (the sum of the three box radii, about 1.5 × blur). With a `clip` (the surface), the drawn area stops where the blur can no longer reach into it.
2. The object is drawn into a transparent `Pixmap` of those bounds at `scale`, with the existing `draw_path` / `draw_image` at opacity 1, so text, strokes, open paths and image alpha are exactly the object's.
3. **Blur:** a separable box blur, 3 passes horizontal and vertical with the radius split per the usual "boxes for Gaussian" sizes (σ ≈ blur / 2 in output pixels), on the alpha channel in 16 bits. Blur 0 skips it.
4. **Tint:** every pixel becomes `shadow.color` with `alpha = a × shadow.opacity × shadow.color.a × opacity` (premultiplied), and the layer is cut to the clip.

`render` calls it before each object that has a shadow and draws the layer with `draw_pixmap` at `origin × scale`, so it lands under the object and above what came before. A hidden object never reaches the draw list. Clipping to the surface is the layer's own cut (so the blur still sees what lies past the edge) and the output pixmap's.

*Alternative considered:* the `image`/`imageproc` Gaussian blur. Rejected: it adds a dependency and is not faster than three box passes. The box blur is about 30 lines and unit-tested against a reference Gaussian within 2 levels.

### 3. Canvas: cached shadow textures (`tp-app/src/shadow_cache.rs`)

- **Key:**
  - the object with `frame.center` zeroed (`Arc` content, compared through a hash of the fields that shape the silhouette: kind, frame size and rotation, path, text and layout, image asset, fill, stroke, line width, shadow);
  - the effective opacity;
  - the zoom bucket from `geometry_cache`;
  - the fonts generation.

  Moving an object reuses its shadow, and changing its look or the zoom bucket renders it again.
- **Resolution:** `scale = min(1, bucket_scale)` texture px per screen px, with the layer's longest side capped at 2048 (the scale is lowered to fit). Blurred shadows don't need more.
- **Rendering on a worker thread,** like `surface_thumbnails`:
  - while a key is missing, the previous texture of the same object, if any, is drawn, moved to the new position (stale for a few frames during a blur drag);
  - at most one job runs at a time, and it batches the missing keys of visible objects;
  - repaint on completion.
- **Drawing:** `paint.rs` draws each cached layer as a textured rectangle (`origin + center`) just before the object, in the same order as the draw list. So a shadow is under its object and above the objects below. Off-screen objects are skipped.
- **Pruning:** entries unused for 2 s are pruned with the geometry cache's prune.

*Alternative considered:* draw the shadow as an offset mesh with a soft feathered edge. Rejected: the result diverges from the export for blurred text and images, and the spec asks that they match.

### 4. Inspector row and popover

`ui/workspace/inspector.rs`, Appearance, after Stroke:
- **No shadow in the selection:** a muted `+ Add a shadow` row (12 px, `TEXT_MUTED`) gives `Shadow::DEFAULT` to every selected object that takes one. One undo step, `undo-add-shadow`.
- **Shadow row:** a full-width `PaintRow` like Fill and Stroke: a swatch of the color, the summary (`"8 / 8 · 8"` in mono: offset x / y · blur, the same in every language, or `inspector-mixed` when the selection differs) with the full values as its tooltip (`inspector-shadow-summary-full`, "Offset 8 / 8, blur 8"), and the remove button inside the row at its right end (`PaintRow::action`, `icons::CLOSE`, tooltip "Remove shadow"; one step, `undo-remove-shadow`). The first version wrote "8 / 8 · blur 8" with the remove button beside the row; the German "Unschärfe" was then cut off at the default inspector width, so the word moved to the tooltip and the button into the row.
- **The row opens `shadow_popover()`** (an `Id` like `stroke_popover()`):
  - the colors widgets from `panels/colors.rs` (picker, palette with links, hex), driven by a `ShadowColor` target;
  - Opacity (%), Offset X, Offset Y, Blur (`DragValue`s with the inspector's field style, units "px").
  - Edits go through `live_edit("undo-change-shadow", …)` and `commit_pending` on release, like the stroke width, so a drag is one undo step.
  - A swatch click links, and any other color unlinks (relink rules).
- **When the row is shown:** for groups and instances only, there's no row. Mixed selections apply to the objects that take a shadow.
- `Workspace::set_shadow(f)`, `add_shadow()` and `remove_shadow()` live in a small `shadow_ops.rs`. Each goes through `map_selected_shapes`, extended to images, so a style follower detaches as with other look changes. `relink` and `detach` already compare `Look`.

### 5. File format (format 1, optional)

```rust
#[derive(Serialize, Deserialize)] pub struct FileShadow { color: FileColor, swatch: Option<u64>, opacity: f32, offset: [f64; 2], blur: f64 }
#[serde(default, skip_serializing_if = "Option::is_none")] pub shadow: Option<FileShadow>
```

- This is on `FileObject`, `FileGraphicStyle` and `FileTextStyle`. Older files load with `None`.
- Values are clamped on load (`Shadow::clamped`), so a hand-edited file can't ask for a 10 000 px blur.
- The format number doesn't change, because no release exists yet. Older builds ignore the field, and that is accepted.

### 6. Strings

New keys in en/fr/de/es: `inspector-add-shadow` ("+ Add a shadow"), `inspector-shadow`, `inspector-shadow-summary` ("{ $x } / { $y } · { $blur }"), `inspector-shadow-summary-full` ("Offset { $x } / { $y }, blur { $blur }"), `inspector-shadow-remove`, `shadow-opacity`, `shadow-offset-x`, `shadow-offset-y`, `shadow-blur`, `undo-add-shadow`, `undo-change-shadow`, `undo-remove-shadow`.

## Risks / Trade-offs

- **Export cost.** A 200 px blur on a 4096 px lettering means a layer of about 4700² px and 6 box passes, around 100 M operations, roughly 0.2–0.5 s in release. It runs in the export's background job, with progress already per object. Measured in task 2.3 on the sample truck at 4096 with 20 shadowed texts (blur 8) and a lettering at blur 200, on the development Mac in release: render 47 ms (10 ms without shadows), render + DDS 168 ms, whole Export Mod 190 ms (`mod_export::tests::shadowed_export_performance`, ignored by default).
- **Canvas latency while dragging the blur value.** The layer re-renders on a worker, so the shadow lags a few frames behind and stays in place in the meantime. A 2048 px cap and the zoom-tied scale keep each render under about 30 ms. Measured in task 3.2 for a 4096 px lettering at blur 200 on the development Mac in release: about 15 ms at 100 % (a 2041 × 430 layer) and 2 ms at 25 % (`shadow_cache::tests::worker_render_time`, ignored by default).
- **Canvas versus export.** The canvas layer is rendered at a lower scale when zoomed out, so a hard shadow (blur 0) may look slightly soft when zoomed out. At 100 % zoom it is rendered at full scale and matches the export, which is what the spec checks.
- **Format 1 without a version bump.** A build from before this change opens a file with shadows and drops them on save. Accepted, since there is no public release.
- **Brand space's Edit Style…** keeps the style's shadow but can't edit it. Players use Redefine from Selection. This could be added to the editor later.
