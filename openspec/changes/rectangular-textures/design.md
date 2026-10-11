## Context

**Model:**
- `tp_core::project::Surface` holds `size: f64`, documented as "Side length in texture pixels" of "a square artboard".
- About 30 source files read `surface.size` / `surface().size`: canvas fit and painting, grid, snapping to the artboard, placement in the middle of the view, thumbnails, export, mod export, mod previews, Copy from cabin, Update Template, breadcrumb, Textures tab, status bar, inspector.
- Symbols already use a `kurbo::Size` artboard, kept square.

**Packages** (`tp-vehicles/src/lib.rs`):
- `Texture { size: u32, … }`, validated against `SIZES` (256…8192);
- the sample truck and trailer are built in code (`sample.rs`, `"size": t.size`);
- Custom vehicle (`tp-pack/src/custom.rs`) sets the size from the image's width and warns when the image is not square.

**Rendering:**
- `tp_render::render` takes a square `RenderOptions::size`;
- the DDS encoder (`encode.rs::encode_dds`) already works on any `Pixmap` width and height and builds its mip chain per level, so it needs no change beyond being fed a rectangular pixmap.

**Files** (`tp-file/src/v1.rs`):
- `FileSurface { size: f64 }`;
- `lib.rs::is_development_v1` already recognizes formats written by development builds and refuses them with the "development format" message.

**Scaling:**
- `vehicle_project::scale_objects(objects, old, new)` scales uniformly (Update Template, Copy from cabin);
- `document/transform.rs::resize` already scales a selection non-uniformly the way the resize handles do (frames, paths, texts, strokes).

**Decision taken with the user:** no backward compatibility. Old packages and old project files are refused, and samples and docs are rewritten.

## Goals / Non-Goals

**Goals:**
- One representation of a texture's size, width and height, from the package to the file to the export. A square is not a special case.
- Every view and output follows the real shape: canvas, rulers, grid, thumbnails, Export Texture, DDS and the mod.
- Scaling between textures of different sizes is correct in the game: horizontal by the width ratio, vertical by the height ratio.

**Non-Goals:**
- Non-power-of-two sizes. Custom vehicle rounds each side and the template is scaled.
- Resizing an existing project's textures.
- Rectangular symbol artboards.
- Reading old packages or project files.

## Decisions

### 1. One size type

- In `tp-core`, `Surface.size` becomes a `kurbo::Size` (f64 width and height), with:
  - `Surface::bounds() -> Rect`, so the many `Rect::new(0, 0, size, size)` call sites go through one helper;
  - `Surface::size_label()` returning "4096²" or "8192 × 1024".
- Packages use `TextureSize { width: u32, height: u32 }`. It is serialized as a JSON array `[w, h]` and validated by `TextureSize::is_allowed()` (each side a power of two in 64..=8192).
- `SIZES` is replaced by `MIN_SIDE` and `MAX_SIDE`.

*Alternative considered:* keeping `size` for squares plus an optional `height`. Rejected, because there is no compatibility to keep, and one path is simpler to test.

### 2. A compiler-driven sweep, with helpers first

The change is wide but mechanical:
1. Change the type, then fix every compile error.
2. Each fix uses `bounds()`, `size.width` or `size.height` explicitly. Where the code means "the longest side" (thumbnail scale, zoom-to-fit), it says `max_side()`.

The places where a square was assumed in arithmetic, not just in the type, are listed for review in tasks.md:
- `viewport::ScreenMap::fit(size, …)`;
- `canvas/aids.rs::paint_grid(…, side, …)`;
- snapping to the artboard edges and center (`snap.rs`);
- `placement.rs` (the middle of the view);
- the thumbnails (`surface_thumbnails.rs`, `recent_thumbnails.rs`, `mod_previews.rs`);
- the template drawing rectangle (`canvas/paint.rs`) and image fitting (`image_cache.rs`);
- the shadow layer clip (`tp-render`, the surface rect).

### 3. Rendering and export

- `RenderOptions` takes `width` and `height`. `render` builds a `Pixmap` of that size, and the scale is `width / surface.width` (equal to `height / surface.height`, since the output keeps the surface's shape).
- The Export Texture dialog offers full, 1/2 and 1/4. Each choice divides both sides, never below 1 px, and is labelled "W × H".
- PNG and DDS take the pixmap as it is. DDS mipmaps halve each side to a minimum of 1, as `mip_chain` already does: 8192 × 1024 gives 14 levels, the last 1 × 1.
- Export Mod renders each texture at its own width and height.

### 4. Thumbnails keep the shape

- Surface thumbnails are rendered with their longest side at `SIDE` (128 px), so 8192 × 1024 gives 128 × 16.
- Tiles draw them without stretching (`tp_ui::widgets::thumbnail_place`): a picture at least as wide as tall fills the tile's width, shown whole and centered when it is wider than the tile, and showing its middle when it is taller (how a square already looked in the landscape tiles of the Project space and the home screen, so those are unchanged); a taller-than-wide texture is fitted whole. In square tiles (Textures tab, Brand impact) this is plain aspect-fit.
- No stretching anywhere.

### 5. Scaling between textures: non-uniform, like the resize handles

- `scale_objects(objects, old: Size, new: Size)` applies `sx = new.w / old.w` and `sy = new.h / old.h` through the same per-object logic as `transform::resize`, about the origin (0, 0). Frames, paths, texts (through their `scale`), gradients and stroke widths then behave exactly as when the player stretches a selection with the handles.
- When `sx == sy` this is today's uniform scaling, so existing tests keep passing.
- Update Template and Copy from cabin both call it.

### 6. Packages and Custom vehicle

- The manifest's `texture.size` is `[w, h]`. A number fails validation with "the texture's size must be a width and a height".
- `sample.rs` writes `[4096, 4096]`, `[1024, 1024]`, and so on. The sample trailer keeps its square base; a rectangular texture is covered by tests, not by the sample.
- Custom vehicle reads the image's width and height. Each side rounds to the nearest power of two in 64..=8192, with ties going up. The row warns only when the rounded size differs from the image. The row's size editor becomes two selectors, Width and Height.
- `docs/vehicle-package-format.md`: the `size` field, the examples, and the end of the "make them square" advice.
- `tpv check` prints sizes with `size_label` rules.

### 7. Project files

- `FileSurface.size` becomes `[f64; 2]`.
- In `is_development_v1`, a surface whose `size` is a JSON number marks the file as a development format, so the existing message is shown: "uses a development format that this version cannot open". There's no migration.
- The format number stays 1, since no release exists.

### 8. Strings

- A size label goes through one function (`size_label`). "²" and "×" are the same in every language, so no new strings are needed except in Custom vehicle: "Width", "Height", and the warning "The template will be scaled to W × H" (`custom-template-scaled`), in en/fr/de/es.
- The old "not square" warning string is removed.

## Risks / Trade-offs

- **Missed square assumptions.** A place that keeps using one side for both would draw or export wrong only on rectangular textures. Mitigation:
  - the sweep list in Decision 2;
  - tests on an 8192 × 1024 surface for each output: canvas fit and rulers (kittest), grid and snapping, thumbnail shape, PNG and DDS dimensions and pixels, mod DDS header, Copy from cabin and Update Template across sizes;
  - a screenshot of a rectangular texture in the Workshop and the Project space.
- **Non-uniform scaling distorts artwork.** It does, by design: it is the correct result in the game, since the game maps by UV. It only happens when a texture's shape changes between versions or cabins, which is rare.
- **Breaking old files and packages.** This was accepted: there's no release. Development projects and custom vehicles must be recreated. The message says so.
- **Very wide canvases** (8192 × 128 accessories): Fit to Screen gives a thin strip. The zoom and navigation already allow zooming in; no special case.
