## Why

A livery is mostly lettering and logos: company names, truck numbers, phone numbers, brand marks. After `editing-panels`, users can draw and organize shapes but cannot write text or bring in a logo, which blocks almost every real livery. This change adds a real text tool edited directly on the canvas, and imports PNG, JPG and SVG artwork that stays sharp at any export resolution.

## What Changes

- **Text objects**: a new object kind holding the text content and its character style (font family, weight, italic, size in texture pixels, alignment, letter spacing, line height). Text is laid out and converted to vector glyph outlines, so it renders exactly the same on the canvas and, later, in exports; it supports the fill and stroke of the Colors and Stroke panels and every transform.
- **Text tool (T) with on-canvas editing**: click to create a text at the pointer and start typing; double-click a text (or press Enter with one text selected) to edit it. While editing, a blinking caret and the text selection are drawn on the canvas; arrows, Home/End, Shift-selection, double-click word selection, Cmd/Ctrl+A, Cut/Copy/Paste of text, Backspace/Delete and Enter for new lines work; Escape or clicking outside ends editing. One editing session is one undo step; an empty text is removed when editing ends.
- **Character settings in the Properties panel** when texts are selected: font family picker with search (bundled fonts plus the fonts installed on the computer), weight, italic, size, alignment, letter spacing and line height, with "Mixed" values for multi-selection.
- **Bundled fonts** (SIL OFL) suited to liveries, so projects look the same on every computer: Inter, Barlow Condensed, Oswald, Bebas Neue and Montserrat.
- **Image import**: File › Place… (Cmd/Ctrl+Shift+P), the Image tool (Shift+I) and drag-and-drop of files onto the window import PNG, JPG and SVG files. Imported files become project assets (deduplicated by content) and are placed as image objects at their natural aspect ratio, centered on the click or the view. Images can be moved, resized (proportions kept by default), rotated, hidden, locked and grouped like any object; invalid or unsupported files show a clear error.
- **SVG stays vector**: SVG images are re-rendered from their source at the resolution needed for the current zoom (and, later, for export) instead of being stored as a bitmap.
- **Assets panel**: lists the project's imported files with thumbnail, name, pixel size or "Vector", and how many objects use each; supports placing an asset again, renaming and removing unused assets.
- **Eyedropper on images**: picking over an image takes the color of the image pixel under the pointer (useful to match logo colors).

Not in this change: text on a path, paragraph boxes with wrapping width, per-character styling (one style per text object), converting text or SVG to editable paths, image cropping/masks/filters, embedding font files and asset bytes in saved projects (`persistence`), export rasterization (`export-and-preview`).

## Capabilities

### New Capabilities

- `text-tool`: creating text, entering and leaving edit mode, on-canvas caret, selection and keyboard editing, undo behavior of editing sessions.
- `text-style`: character settings (family, weight, italic, size, alignment, letter spacing, line height), font sources (bundled and system), font picker, missing-font fallback.
- `image-import`: importing PNG/JPG/SVG through Place, the Image tool and drag-and-drop; asset deduplication; placement; image object behavior; SVG resolution-independent rendering; errors.
- `assets-panel`: the Assets panel listing, placing, renaming and removing project assets.

### Modified Capabilities

- `document-model`: new object kinds Text and Image (default names "Text" and "Image"), project asset store; hit testing of texts by their laid-out bounds and of images by their frame.
- `shape-tools`: the Text and Image tools are no longer listed as unavailable.
- `color-panel`: the Eyedropper takes the pixel color when picking over an image.
- `properties-panel`: adds character settings for texts and image information (source size, Reset Size).
- `undo-history`: text editing sessions, character style changes and imports are undoable.

## Impact

- `tp-core`: `ShapeKind::Text` / `ShapeKind::Image`, text content and `CharStyle` on objects, `AssetId` and an `Assets` store on `Project` (bytes, kind, name, pixel size), text outline geometry (glyph outlines → `kurbo::BezPath`), image pixel sampling.
- New crate `tp-text` (or module) wrapping `cosmic-text` (layout, shaping, editing, font database incl. system fonts) and `skrifa` (glyph outlines); bundled OFL fonts in `assets/fonts/`.
- Canvas: concave/holed glyph outlines require tessellation (`lyon_tessellation`) into egui meshes; images drawn as textured quads with egui textures (downscaled for display, mipmapped); SVG rasterized with `resvg` per zoom bucket in a background thread.
- `tp-app`: Text tool and edit mode, Image tool, Place command (`rfd` file dialog), drag-and-drop of files, Assets panel, character section in Properties.
- New dependencies: `cosmic-text`, `skrifa`, `lyon_tessellation`, `image` (png, jpeg), `resvg`/`usvg`, `rfd`, `blake3`.
- Binary size grows by the bundled fonts (≈2–3 MB).
