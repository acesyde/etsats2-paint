## 1. Document model (tp-core)

- [x] 1.1 Add `ShapeKind::Text` / `ShapeKind::Image { asset }`, `AssetId`, `TextBlock` (content, `CharStyle`, layout_size, scale) and default names ("Text", file stem for images); text frame = layout_size × scale; verify unit tests for defaults and text frames
- [x] 1.2 Add the project asset store (`Asset` with name, kind, bytes, size, blake3 hash), dedup on add, usage counts, rename/remove, assets in `Snapshot`; verify unit tests for dedup, usage after duplicate, and undo restoring a removed asset
- [x] 1.3 Hit testing and bounds for texts (laid-out box) and images (frame), resize of texts updating `scale`; verify unit tests for "Clicking between letters" and image frame hits

## 2. Text engine (tp-text)

- [x] 2.1 Create the `tp-text` crate with `cosmic-text` and matching `skrifa`; download bundled OFL fonts (Inter Bold/Italic, Barlow Condensed, Oswald, Bebas Neue, Montserrat) with licenses into `assets/fonts/`; verify the crate builds on the CI matrix and fonts load
- [x] 2.2 Implement `FontLibrary` (bundled first, system fonts loaded in a background thread, family list, nearest-weight resolution, Inter fallback with `missing` flag); verify unit tests for listing, weight fallback and missing family
- [x] 2.3 Implement `layout` (letter spacing, line height, alignment, anchor offsets) and `outline` → `BezPath` with a glyph cache; verify unit tests for growing width, two lines, centered lines, letter spacing and an "O" outline with a hole
- [x] 2.4 Implement `EditSession` (insert, delete, motions incl. Home/End and Shift selection, select word/all, clipboard text, caret/selection rects, in-session undo); verify unit tests for each operation

## 3. Canvas rendering

- [x] 3.1 Tessellate text outlines (fill + stroke) with `lyon_tessellation` into cached meshes and draw them; verify a screenshot with filled, stroked, rotated and scaled texts and a perf check of 50 texts while panning
- [x] 3.2 Add the image cache: raster decode with downscaled mipmapped textures, SVG rasterization by `resvg` per size bucket in a background thread, textured-quad drawing with rotation and opacity, placeholder while loading; verify screenshots of PNG and SVG images at 25% and 800% zoom
- [x] 3.3 Eyedropper picks image pixels; verify the "Pick a color from a logo" kittest

## 4. Text tool and editing

- [x] 4.1 Text tool: click creates a text with the current text style and fill in the active layer and starts a session; I-beam cursor; verify kittests for "Click and type" and "Empty text is removed"
- [x] 4.2 Enter/leave edit mode (double-click, Text tool click on a text, Enter command, Escape, click outside, tool change), anchor kept on relayout; verify kittests for "Escape leaves edit mode" and that the anchor stays fixed when typing
- [x] 4.3 Keyboard/pointer routing during sessions (typing suppresses shortcuts; Escape, Cmd+A, Cmd+Z, clipboard routed to the session; drag-select, double-click word); verify kittests for "Typing a tool letter while editing", "Select all and replace", "New line"
- [x] 4.4 Session commit as one "Create Text"/"Edit Text" step and in-session undo; verify the "Undo an editing session" kittest

## 5. Character settings

- [x] 5.1 Character section in Properties (family, weight, italic, size, alignment, letter spacing, line height; Mixed; nothing selected edits the text style) with relayout; verify kittests for "Change size of two texts", "Centered multi-line text" and "Outlined lettering"
- [x] 5.2 Font picker popup with search, keyboard navigation and outline previews; missing-font warning; verify kittests for "Search a font" and "Font not installed", and a screenshot of the picker

## 6. Images and assets

- [x] 6.1 Import pipeline (format sniffing, errors, placement size, dedup, one "Place" step) and Place command with `rfd`; verify kittests via the byte API for "Place a PNG", "Large image is scaled down", "Unsupported file", "Same logo twice", "Undo an import"
- [x] 6.2 Image tool (click opens Place and positions at click) and drag-and-drop of files; verify a kittest injecting `dropped_files` for "Drop two files"
- [x] 6.3 Image properties (asset name, kind, source size, Reset Size) and hiding fill/stroke/radius for image-only selections; verify kittests for "Reset a distorted image" and "Proportional resize with Shift"
- [x] 6.4 Assets panel (thumbnails, size or Vector, use counts, Place, Rename, Remove with disabled reason, drag to canvas, empty state with Place…); verify kittests for "Asset listed after import", "Remove an unused asset", "Remove is disabled while used"

## 7. Integration

- [x] 7.1 Enable the Text and Image tools and the Place / Edit Text commands; update the unavailable-tool list; verify the shortcut-collision test and the "Unavailable tool" kittest still pass
- [x] 7.2 Screenshot review of a livery with lettering and logos at 100% and 200% UI scale
- [ ] 7.3 Run `openspec validate text-and-images --strict` and `mise run ci`; verify both pass locally and the CI matrix is green on the PR
