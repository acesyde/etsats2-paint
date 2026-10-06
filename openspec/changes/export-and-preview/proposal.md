## Why

Users can now design, save and reopen a livery, but they still cannot get the result out of TruckPaint: Export is disabled. A livery is only useful once it becomes a texture the game (or another tool) can load, so exporting PNG and DDS at full resolution is the next step toward a real paintjob.

## What Changes

- **Export Texture… dialog** (Export menu, Cmd/Ctrl+E): choose the format (PNG, or DDS for the games), the output size (the project's texture resolution by default, or a smaller power of two), and the background (opaque color, white by default, or transparent). A live preview shows exactly the image that will be written, with its pixel size, format and estimated file size. Confirming asks for a destination with a native save dialog.
- **Full-resolution renderer**: the active surface is rendered off-screen at the chosen size from the vector document — shapes, groups (with opacity), texts drawn from the same glyph outlines as the canvas, raster images resampled with high quality, and SVGs rendered directly from their vector source so they stay sharp at 8192 px. Hidden objects are not exported; locked objects are.
- **DDS for ETS2/ATS**: BC3 (DXT5) compression with alpha and a full mipmap chain, or uncompressed RGBA with mipmaps for maximum quality.
- **Non-blocking export**: rendering and encoding run in the background with progress in the dialog and a Cancel button; errors (disk full, unwritable location) are reported with the file name and reason. The project is not modified by exporting.
- Export Mod… stays disabled until vehicle templates exist.

Not in this change: game mod packaging (`.sii`, folders, `.scs`), vehicle templates, the 3D preview panel (still a placeholder until the 3D model source is decided), recent-project thumbnails, batch export of several surfaces.

## Capabilities

### New Capabilities

- `texture-export`: the Export Texture dialog and preview, output formats and sizes, background, what is rendered and how, DDS encoding and mipmaps, background progress, cancel and errors.

### Modified Capabilities

(none — the Export menu already shows unavailable items as disabled; enabling one does not change a requirement)

## Impact

- New crate `tp-render`: CPU renderer (tiny-skia, already a dependency through resvg) drawing a `Project` surface to an RGBA pixmap, using `tp-text` for glyph outlines and `resvg` for SVG assets; mipmap generation; PNG encoding (`image`) and DDS writing (header + BC3 blocks via `texpresso`).
- `tp-text`: glyph outline access is reused as is (layout + `GlyphCache`).
- `tp-app`: Export Texture command enabled (Cmd/Ctrl+E), export dialog with preview, background export job with progress and cancel, save dialog through `FileDialogs`.
- New dependency: `texpresso` (pure Rust BC1–BC5 encoder).
