## 1. Renderer (tp-render)

- [x] 1.1 Move the text layout-to-document transform from `tp-app::text_engine` into `tp-text` (shared by canvas and export) and create the `tp-render` crate (tp-core, tp-text, resvg/tiny-skia, image); verify the workspace builds and existing text tests still pass
- [x] 1.2 Render backgrounds, rectangles (rounded, rotated), ellipses, fills and strokes, and accumulated opacity through `draw_list`, clipped to the surface; verify pixel tests for background/transparency, edge positions at 1:1 and ½ scale, rotated rounded rectangle, 50 % red over blue, hidden group excluded, locked object included, clipping
- [x] 1.3 Render texts from glyph outlines (fill + stroke) with the shared transform; verify a pixel test that the counter of an "O" is not filled and the stroke covers the outline
- [x] 1.4 Render raster images (bicubic, Lanczos pre-downscale) and SVG images (resvg at output resolution) through layers composited with the object's opacity; verify pixel tests for the image center color, SVG edge sharpness at 8K, and image opacity

## 2. Encoding

- [x] 2.1 Implement mipmap generation (premultiplied box filter) and PNG encoding (straight RGBA); verify tests for the mip chain sizes and a PNG round trip through `image`
- [x] 2.2 Implement DDS writing (header, BC3 via `texpresso`, uncompressed BGRA, mip levels, padding below 4×4); verify tests for header fields (DXT5, 12 levels at 2048) and decoding with `image` (`dds` feature) within tolerance, exact for uncompressed

## 3. Export in the app

- [x] 3.1 Add the background `ExportJob` (render → encode → atomic write, per-mille progress, cancel flag, repaint) and `FileDialogs::save_export`; verify unit tests for a completed job, a cancelled job leaving no file, and a failing destination
- [x] 3.2 Rename `ExportPng` to `ExportTexture` (Cmd/Ctrl+E) and add the Export Texture dialog (format, size, background, DDS encoding, remembered settings, estimated size, progress, Cancel, confirmation hint, error message); verify kittests "Default export settings", "Export to PNG", "Export does not change the project", "Unwritable destination" and cancelling during export
- [x] 3.3 Add the live preview (background render at ≤ 512 px on settings change, checkerboard under transparency); verify the "Transparent background preview" kittest and that changing the size updates the shown pixel size

## 4. Integration

- [x] 4.1 Screenshot review of the export dialog (PNG and DDS, transparent background) at 100 % and 200 % UI scale, and of an exported 4K PNG of the lettering scene compared with the canvas; perf check of a 4K and an 8K export in release
- [x] 4.2 Run `openspec validate export-and-preview --strict` and `mise run ci`; verify both pass locally and the CI matrix is green on the PR
