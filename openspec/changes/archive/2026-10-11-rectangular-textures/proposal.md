## Why

TruckPaint only knows **square** textures: a package declares one `size`, a surface has one `size`, and Custom vehicle takes an image's width as its size and stretches the template to a square. But the game's textures are often not square. In the official template pack for ETS2:
- the Krone Cool Liner's main texture is 8192 × 1024;
- the DAF NGD's sun visor is 2048 × 256;
- trailer chassis parts go down to 1024 × 128 or 256 × 512.

Stretched to a square, the canvas lies. A lettering that looks right on an 8192 × 8192 canvas is squashed eight times in the game, because the game maps the texture by its UV coordinates, whatever its pixel size. Every trailer and many accessories are affected, including the ones players make today with Custom vehicle.

This is the prerequisite of `official-vehicles`, which brings the official trucks and trailers from that pack. Texture sizes come from the game, so the editor must follow them.

## What Changes

- **A texture has a width and a height.** Each is a power of two from 64 to 8192 px, so 4096 × 4096, 8192 × 1024 and 2048 × 256 are all valid. A square is just a texture whose two sides are equal, with no special case.
- **Packages:** a texture's size is always `[width, height]`. Packing and validation check both sides. **BREAKING:** a package with a single `size` number is refused; the sample truck and trailer are rewritten.
- **Custom vehicle:** an image's size is taken as it is, width and height. The "not square, will be stretched" warning goes away. Each side is rounded to the nearest allowed power of two, and a warning appears only when the image isn't one, saying the template will be scaled to the texture's size.
- **Projects:** a surface stores its width and height. **BREAKING:** project files saved before this change are refused, with the usual message for invalid files. No version has been released, so no player is affected. The canvas, the artboard, the rulers, the grid, the guides, Fit to Screen, snapping to the artboard and the template are drawn at the real shape.
- **Thumbnails:** the texture thumbnails (Textures tab, Project space, Brand previews) keep the texture's shape inside their tile, without stretching.
- **Exports:**
  - Export Texture writes the surface at its real width and height, at full size or at 1/2 or 1/4;
  - DDS (BC3/DXT5 and RGBA) supports rectangular power-of-two sizes and their mipmaps;
  - Export Mod writes each texture at its own size.
- **Symbols** keep their square artboard. That's unchanged.

Non-goals: sizes that are not powers of two; resizing a project's textures; changing the official vehicles' data (that is `official-vehicles`).

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `document-model`: a surface has the texture's width and height instead of a square size.
- `vehicle-packages`: a texture's size is `[width, height]`, each a power of two from 64 to 8192; validation checks both; a single number is no longer accepted.
- `project-files`: a surface's size is saved as width and height; older files are refused.
- `custom-vehicles`: the image's width and height are kept; the warning is about non-power-of-two sides, not about non-square images.
- `texture-export`: the output covers the surface's width and height; the size choices scale both; rectangular DDS with mipmaps.
- `mod-export`: each texture is written at its own width and height.
- `vehicle-projects`: the Textures tab shows both sides when they differ; Update Template and Copy from cabin scale horizontally and vertically by their own ratios.
- `workspace-layout`: the canvas area shows the artboard at the texture's shape.
- `properties-panel`: the inspector's texture summary shows the size as "4096²" or "8192 × 1024".
- `workspace-spaces`: the breadcrumb shows "8192 × 1024" when the sides differ.

## Impact

- **tp-core:**
  - `Surface.size: f64` becomes a size (`width`, `height`). Every use of `surface.size` is reviewed: artboard bounds, clipping, placement in the middle of the view, snapping to the artboard, guides, `ScreenMap` fitting;
  - symbols keep `Size` with equal sides.
- **tp-vehicles / tp-pack:** manifest parsing of `[w, h]` only, validation, packing, `tpv check`; Custom vehicle sizing in `custom.rs`; the sample packages and `docs/vehicle-package-format.md` rewritten.
- **tp-render:** `render` and `RenderOptions` take a width and a height; DDS encoding (`dds.rs`) for rectangular sizes and mipmaps.
- **tp-file:** a surface's size saved as width and height, with no reading of the old square size (older files are refused).
- **tp-app:**
  - canvas (`viewport.rs`, `canvas/aids.rs`), thumbnails (`surface_thumbnails.rs`), Export Texture dialog, mod export, labels;
  - strings in en/fr/de/es where sizes are shown.
- **Risk:** `surface.size` is used in many places, so the change is wide but mechanical. Tests on a 8192 × 1024 surface (canvas fit, export pixels, DDS dimensions, mod export) guard it.
