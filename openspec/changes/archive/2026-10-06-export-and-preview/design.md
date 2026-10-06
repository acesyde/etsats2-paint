## Context

After `persistence`, a `Project` (tp-core) holds surfaces with Arc-shared object trees (rectangles with corner radius, ellipses, groups, texts with `TextBlock`, images referencing `Asset { kind: Raster | Svg, bytes, size }`), and `tree::draw_list()` yields visible leaves in paint order with accumulated group opacity. The canvas draws shapes as convex polygons (egui), texts as lyon meshes of `tp-text` glyph outlines, and images as textured quads from `ImageCache` (raster capped at 2048 px, SVG rasterized by resvg per zoom bucket). None of that is suitable for a full-resolution export: egui renders on the GPU at screen size, and the canvas caches are display-oriented. `resvg` (with `tiny-skia` 0.12) and `image` (png, jpeg) are already dependencies; `FileDialogs` (persistence) abstracts native dialogs; `Saver` shows the pattern for background work with `request_repaint`. Export › Export PNG… / Export Mod… exist as `NotYet` commands. User decisions: DDS defaults to **BC3/DXT5 with mipmaps** (uncompressed RGBA option); "preview" means the **export dialog preview only** (3D panel stays a placeholder).

## Goals / Non-Goals

**Goals:**
- One renderer, independent of the GPU and the screen, that produces the exact pixels written to disk, usable at any size (preview 512 px → 8192 px) and testable headlessly.
- SVGs and text stay vector up to the export resolution.
- Exports of 8K textures without freezing the UI, cancellable.

**Non-Goals:**
- Mod packaging, vehicle templates, 3D preview, thumbnails, multiple surfaces at once, color management (sRGB is assumed throughout).

## Decisions

### D1. CPU renderer on tiny-skia in a new `tp-render` crate
`tp-render::render(project, surface_index, options) -> Pixmap` (premultiplied RGBA) with `RenderOptions { size: u32, background: Option<Rgba> }`. The surface is mapped with `Transform::from_scale(size / surface.size)`. For each leaf of `draw_list` (paint order, accumulated opacity):
- **Rectangle / ellipse:** `Object::path()` (kurbo, rotation and rounded corners included) converted to a `tiny_skia::Path`, filled (non-zero, anti-aliased) then stroked (miter join, width in texture pixels) — same order as the canvas.
- **Text:** `tp-text` layout + `GlyphCache::outline` mapped by the object's layout-to-document transform (the same function as the canvas, moved from `tp-app::text_engine` to `tp-text` so both use it), filled then stroked.
- **Raster image:** decoded once per export with `image`, wrapped as a pixmap, drawn with `draw_pixmap` through the frame's affine transform using `FilterQuality::Bicubic`; a pre-downscale (`image::imageops::resize` Lanczos3) is applied when the target is less than half the source size to avoid aliasing.
- **SVG image:** `resvg::render(tree, transform, &mut pixmap)` with the frame's affine transform composed with a scale from the SVG viewbox to the frame size — rendered directly at output resolution, so it stays vector.
- **Opacity:** shapes and texts use paint alpha; images (raster and SVG) are rendered into a temporary layer the size of their on-texture bounding box and composited with `draw_pixmap` at the object's opacity, so overlapping parts inside an SVG are not double-blended.
Clipping to the surface is implicit (the pixmap is the surface). Fonts come from a `FontLibrary` passed in (the app passes its own, so system fonts used by texts render).
*Alternatives:* GPU offscreen render via egui/wgpu (results depend on driver and screen pipeline, limited texture sizes, hard to test); `vello` (GPU compute, heavier dependency, not needed for one-off renders).

### D2. Background
`background: Some(color)` fills the pixmap first (default white, opaque); `None` keeps it transparent. PNG keeps alpha; DDS BC3 keeps alpha.

### D3. Encoding
- **PNG:** demultiply to straight RGBA8, encode with `image` (`PngEncoder`, default compression).
- **Mipmaps:** box filter (2×2 average) on premultiplied RGBA from the full image down to 1×1, then demultiplied per level; sizes are powers of two (project resolutions and the offered fractions are), so every level halves exactly.
- **DDS:** hand-written 128-byte DDS header (`DDSD_CAPS|HEIGHT|WIDTH|PIXELFORMAT|MIPMAPCOUNT|LINEARSIZE`, `DDSCAPS_TEXTURE|MIPMAP|COMPLEX`) followed by each level. BC3 levels are encoded with `texpresso::Format::Bc3` (`Params { algorithm: RangeFit }` for 8K speed, `ClusterFit` when the size ≤ 2048); levels smaller than 4×4 are padded to one block. Uncompressed uses the `DDPF_RGB|ALPHAPIXELS` 32-bit BGRA layout games read.
Tests decode exported DDS files with `image`'s `dds` feature (dev-dependency) and compare against the rendered image within BC3 tolerance.

### D4. Export job and progress
`tp-app::export::ExportJob` runs on a worker thread: render (reporting progress per object: `rendered / total`), encode (per mip level), write atomically through `tp_file::write_atomic` (temp file + rename, so a cancelled or failed export leaves nothing at the destination). Progress is an `Arc<AtomicU32>` (per-mille) plus a cancel `Arc<AtomicBool>` checked between objects and mip levels; the worker calls `ctx.request_repaint()` on progress. The project is captured by cloning (`Project` is Arc-shared, so it is cheap), so the user can keep editing while exporting, and the export reflects the moment it was started.

### D5. Export dialog and preview
`Modal::Export(ExportDialog { format, size_divisor: 1|2|4, background: Option<Rgba>, dds_encoding, preview, job })` with `ExportSettings` remembered in `AppState` for the session. The preview is rendered by the same `tp-render::render` at 512 px (or the output size if smaller) on a background thread whenever settings change (debounced to the latest request), uploaded as an egui texture and drawn over a checkerboard. Estimated size: PNG ≈ 0.5 × w × h × 4 bytes (shown with "about"), DDS BC3 = exact (sum of levels × 1 byte per pixel), uncompressed = exact (4 bytes per pixel × 4/3). Export: `FileDialogs::save_export(suggested, format)` (new method; scripted in tests), then the job starts and the dialog shows progress and Cancel; on success the dialog closes and the canvas hint shows "Exported ace.png"; on error a `Modal::Message`.

### D6. Commands
`ExportPng` is renamed to `ExportTexture` ("Export Texture…", Cmd/Ctrl+E, enabled with a project and no gesture); `ExportMod` stays `NotYet`. Command tests (shortcut collisions) cover the rename.

### D7. Testing
- `tp-render` (headless, exact pixels): background fill and transparency; rectangle edge positions at 1:1 and ½ scale; rotated rounded rectangle coverage; 50 % red over blue blend; hidden group excluded; locked object included; text "O" has an unfilled counter pixel; raster image pixel colors at its center; SVG at 8K has a sharp edge (adjacent pixels jump from fully inside to fully outside within 2 px); clipping outside the surface.
- Encoding: PNG round trip through `image`; mip chain count and sizes; DDS header fields (DXT5 FourCC, 12 levels at 2048) and decode via `image` within tolerance; uncompressed DDS exact.
- `tp-app` kittests with scripted dialogs and temp dirs: default settings shown, export PNG writes a correctly sized file, transparent background preview flag, export does not change save state/history, unwritable destination message and no file, cancel leaves no file.

## Risks / Trade-offs

- [CPU rendering of 8K textures takes seconds] → background thread with progress; tiny-skia is SIMD-optimized; images use cached decodes for the duration of a job.
- [Canvas and export could drift] → texts share outlines and transform code; shapes share `Object::path()`; screenshot tests compare a canvas capture and a preview render of the same scene visually during review.
- [BC3 quality on gradients] → ClusterFit for ≤ 2048, uncompressed option offered.
- [Memory at 8K] → one 256 MB RGBA pixmap plus mip levels (~340 MB peak); acceptable on desktop; the pixmap is dropped as soon as encoding finishes.

## Migration Plan

None (new feature). The `ExportPng` command id is renamed; it is not persisted anywhere.

## Open Questions

- None blocking. Which DDS variants specific game versions prefer (e.g. BC7 on recent ETS2) can be added later as another encoding option.
