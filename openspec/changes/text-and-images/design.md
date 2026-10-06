## Context

After `editing-panels`: `Object { id, kind: ShapeKind (Copy: Rectangle | Ellipse | Group), name, frame, fill, stroke, opacity, visible, locked, children }` lives in a copy-on-write tree (`tp-core::document::tree`); history stores `Snapshot`s (object lists, palette, selection). The canvas draws `tree::draw_list()` leaves as **convex** polygons from a geometry cache keyed by object + zoom bucket (`tp-app/src/geometry_cache.rs`), so concave shapes cannot be drawn yet. Panels edit through `Workspace::live_edit` / `commit_pending`; keyboard shortcuts are suppressed while `typing` (text field focused, or focused last frame). There is no font handling beyond egui's UI fonts, no file I/O for documents and no image decoding. See proposal.md and specs/.

## Goals / Non-Goals

**Goals:**
- Text drawn from real glyph outlines, so canvas and future export match exactly.
- On-canvas text editing with familiar keyboard/pointer behavior.
- Images and SVGs as first-class objects; SVG resolution-independent.
- Keep `tp-core` free of font engines and decoders (it stores data; engines live in `tp-text` and `tp-app`).

**Non-Goals:**
- Rich text (mixed styles), text on path, wrapping boxes, kerning/OpenType feature UI, vertical text.
- Image editing (crop, masks, filters); converting SVG/text to editable paths.
- Saving fonts/assets to disk (`persistence`), export rendering (`export-and-preview`).

## Decisions

### D1. Data model
`ShapeKind` stays `Copy` and gains `Text` and `Image { asset: AssetId }` (`AssetId(u64)`). `Object` gains `text: Option<TextBlock>` with `TextBlock { content: String, style: CharStyle, layout_size: Size, scale: Vec2 }` and `CharStyle { family: String, weight: u16, italic: bool, size: f64, align: Align, letter_spacing: f64 /* 1/1000 em */, line_height: f64 /* % */ }`. A text's `frame.size` is `layout_size × scale`; resizing with handles changes `scale` (and size), never the font size. `layout_size` is a cache written by the app whenever content or style changes, so `tp-core` can compute bounds, hit tests and transforms without a font engine. `Project` gains `assets: BTreeMap<AssetId, Arc<Asset>>` with `Asset { id, name, kind: Raster | Svg, bytes: Arc<[u8]>, size: Size, hash: [u8; 32] }`; `Snapshot` includes the asset map (Arc-shared), so undoing an import removes the asset. Usage counts are computed by walking the tree.
*Alternative:* a separate `Node` enum per kind — rejected for the same reasons as groups (D1 of editing-panels).

### D2. `tp-text` crate (layout, outlines, editing)
New crate depending on `tp-core`, `cosmic-text` (shaping, bidi, line breaking, `Editor` for cursor/selection, `fontdb` database) and `skrifa` at the version cosmic-text uses (glyph outlines). API:
- `FontLibrary`: loads bundled fonts first, then system fonts in a background thread (`fontdb::Database::load_system_fonts`) and swaps them in when ready; lists families (bundled first, deduplicated); resolves `CharStyle` to a face with nearest-weight fallback and Inter fallback for missing families (`resolved.missing == true`).
- `layout(&TextBlock) -> TextLayout` (size, line metrics, positioned glyphs) with letter spacing and line height applied; alignment relative to the anchor.
- `outline(&TextLayout) -> BezPath` in local coordinates (glyph paths via skrifa's `OutlinePen` into `kurbo`), cached per (font, glyph, size bucket).
- `EditSession` wrapping `cosmic_text::Editor`: insert text, delete, motions, selection, select word/all, clipboard text, caret and selection rectangles in local coordinates, and a small in-session undo stack of content states.
*Alternatives:* `parley` (newer, less complete editing support); egui's own text layout (would differ from export and lacks outlines).

### D3. Text anchor
The anchor is the first line's baseline at the left, center or right depending on alignment, stored implicitly through `frame` and `layout_size`: when content or style changes, the app recomputes `layout_size` and adjusts `frame.center` so the anchor stays fixed in document space (rotation included). Creating a text at a click puts the anchor at the click point.

### D4. Rendering concave geometry
Text outlines (with holes) are tessellated with `lyon_tessellation` (non-zero fill; stroke tessellation for text strokes) into triangles in document space, cached like flattened outlines (object `Arc` pointer + zoom bucket) and drawn as `egui::Mesh`. Rectangles/ellipses keep the existing convex fast path.

### D5. Images on the canvas
- Raster: decoded once with `image` (png, jpeg features) on import to validate and read the size; for display, decoded into an `egui::ColorImage` downscaled to at most 2048 px on its longest side and uploaded as a mipmapped texture, cached per `AssetId` in the app (`ImageCache`), and kept CPU-side for eyedropper sampling.
- SVG: parsed with `usvg` on import (validation, declared size); rasterized with `resvg` into a `tiny_skia::Pixmap` at the object's on-screen size rounded up to a power-of-two bucket (max 4096 px), in a background worker thread; the previous raster is shown until the new one arrives (`request_repaint` on completion).
- Drawing: a textured quad (`egui::Mesh` with UVs) on the four frame corners, so rotation works; opacity in vertex color alpha. Missing textures (still decoding) draw a neutral placeholder with a spinner.

### D6. Import flow
`import::read_files(paths) -> Vec<Result<ImportedFile, ImportError>>` reads bytes and sniffs the format (magic bytes for PNG/JPEG, XML `<svg` for SVG); `Workspace::place_assets(files, at: Option<Point>, now)` adds assets (dedup by `blake3` hash), creates image objects (natural size, scaled to fit half the surface), offsets multiple placements by 40 px, selects them, records one "Place" step, and returns per-file errors which the app shows as a canvas hint. File › Place… uses `rfd::FileDialog` (blocking, filtered to png/jpg/jpeg/svg, multiple); drag-and-drop reads `RawInput::dropped_files` paths when hovering the canvas. Tests call the byte-level API directly.

### D7. Edit mode integration
`Workspace.text_session: Option<TextSession { id, session: EditSession, before: Snapshot, created: bool }>`. While a session exists, `AppState` treats the frame as `typing` (so shortcuts like V, Delete, arrows are not commands) except Escape/Undo/Redo/Copy/Cut/Paste/Select All, which are routed to the session. Text events (`Event::Text`, `Event::Key`, `Event::Paste`, `Event::Copy`, `Event::Cut`) are consumed by the canvas and applied to the session; each change updates the object live (`live_edit` without recording). Ending a session (Escape, click outside, tool change, selection change) commits one "Create Text"/"Edit Text" step, or deletes an empty text (no step for a created-then-empty text). The caret blinks at 1 Hz (`request_repaint_after`), and caret/selection rectangles are mapped through the object's frame and scale.

### D8. Character section and font picker
The Properties panel gains a Character section (family button, weight dropdown limited to available weights, italic toggle, size, alignment segmented control, letter spacing, line height), editing selected texts through `live_edit` ("Change Font", "Change Text Size", …) and relayouting them (D3), or the workspace `text_style` when nothing is selected. The family picker is a popup with a search field and a virtualized list; previews are drawn with `tp-text` outlines tessellated to small meshes (cached per family), so no egui font atlas rebuild is needed. Missing families show a warning icon and tooltip.

### D9. Bundled fonts
`assets/fonts/` gains OFL fonts with their licenses: Barlow Condensed (Regular, SemiBold, Bold, Italic, Bold Italic), Oswald (variable), Bebas Neue (Regular), Montserrat (variable + italic variable), alongside Inter (Regular, SemiBold already; Bold and Italic added). Embedded with `include_bytes!` in `tp-text`.

### D10. Commands and tools
Text (T) and Image (Shift+I) tools become available; new commands: Place… (Cmd/Ctrl+Shift+P, File menu), Edit Text (Enter, enabled with exactly one unlocked visible text selected). The canvas cursor shows an I-beam for the Text tool and over texts while editing.

### D11. Testing
- `tp-text`: layout sizes grow with content, multi-line and alignment offsets, letter spacing effect, outline non-empty with holes for "O", missing-family fallback flag, edit session operations (insert, motions, select all, word select, in-session undo).
- `tp-core`: asset dedup, usage counts, snapshot restore of assets, text frame from layout_size × scale, image hit testing.
- `tp-app` kittests: click-and-type, Escape ends editing, empty text removed, tool letter typed while editing, select-all-and-replace, new line, session undo; Character section changes and Mixed; Place via byte API (generated PNG, inline SVG), placement size scaling, unsupported file message, dedup, Assets panel list/remove/place, eyedropper on an image pixel; screenshots of text and images at several zooms.

## Risks / Trade-offs

- [System font enumeration can take hundreds of milliseconds] → bundled fonts are available immediately; system fonts load in a background thread and appear when ready.
- [`cosmic-text` and `skrifa` versions must match] → `tp-text` uses the `skrifa` re-exported/required by `cosmic-text`, pinned in the workspace; CI catches drift.
- [Large or complex SVGs are slow to rasterize] → background rendering with size buckets, max 4096 px, previous raster shown meanwhile.
- [Memory for large images] → display copies are downscaled to 2048 px; original bytes stay in the asset store only once (dedup).
- [Text layout differs slightly between platforms if system fonts differ] → projects using bundled fonts are identical everywhere; missing fonts are flagged visibly.
- [Edit-mode keyboard routing interacting with global shortcuts] → one explicit routing table in `AppState` (D7), covered by kittests.

## Migration Plan

No saved documents exist yet. New `Object` fields default to `None`; existing tests keep constructing shapes unchanged.

## Open Questions

- Whether to offer a "paragraph text" (fixed-width, wrapping) mode later — independent of this change.
