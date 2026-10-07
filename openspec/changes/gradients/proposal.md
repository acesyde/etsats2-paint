## Why

Liveries rely on gradients: color fades along the cab sides, metallic bands, glows behind logos and lettering. Today every fill and stroke is a single solid color, so the only workaround is stacking many slightly different shapes. Gradients belong in the editor before vehicle templates and mod export, because they change the file format and every place that draws an object, which is cheaper to do before the first release.

## What Changes

- **Paints instead of colors.** An object's fill and its stroke each become a *paint*: a solid color (as today), a **linear gradient** or a **radial gradient**. A gradient has two or more color stops (color with alpha, location 0–100%) and a geometry stored relative to the object's frame. The gradient therefore moves, rotates, scales and flips with the object. The geometry is a start and an end point for linear gradients, and a center, radius point and aspect ratio for radial ones (elliptical gradients). Beyond its ends, a gradient keeps the color of the end stop.
- **Everywhere a color is painted.** Gradients work on fills and strokes of rectangles, ellipses, polygons, stars, paths and texts, including the body and outline of lines (open paths) and Inside, Outside and dashed strokes. Images and groups are unchanged.
- **Colors panel.** A Solid / Linear / Radial control picks the kind of paint for the current target (fill or stroke). In gradient mode the panel shows:
  - a gradient bar with stop markers: add, select, drag, delete;
  - the Location field and the Angle field;
  - a Reverse button.

  The existing picker, sliders, hex field, recent colors and palette edit the selected stop. With nothing selected, the current style can hold a gradient, which new shapes get.
- **Gradient tool (G).** Shows on-canvas handles for the selected object's fill or stroke gradient: start and end, or center, radius and aspect. It also draws a new gradient vector by dragging across the selection, with Shift constraining to 45° steps.
- **Canvas.** Gradients are drawn on the canvas with small gradient-ramp textures. The texture coordinates follow the gradient, so the canvas matches the export to within one 8-bit step. Objects with a gradient are drawn from their triangle meshes, like paths and styled strokes already are.
- **Export.** PNG and DDS exports render gradients with tiny-skia's native linear and radial gradients.
- **Related behavior:**
  - The Eyedropper copies the whole paint, gradient included.
  - Shift+X swaps the fill and stroke paints.
  - D resets to solid defaults.
  - Boolean operations and Convert to Outlines keep a gradient's appearance.
- **BREAKING (file format):** project files move to format **v3**: fills and stroke colors are stored as paints. v1 and v2 files still open, migrated to v3, and are written as v3 at the next save.

## Capabilities

### New Capabilities
- `gradients`: linear and radial gradient paints (stops, geometry relative to the frame, interpolation, behavior beyond the ends), the gradient editor in the Colors panel, and the Gradient tool with its canvas handles.

### Modified Capabilities
- `document-model`: an object's fill and its stroke color become a paint (solid, linear or radial gradient) instead of a solid color.
- `color-panel`:
  - the target swatches preview gradients;
  - the picker edits the selected stop;
  - the Eyedropper copies whole paints;
  - Shift+X and D handle paints.
- `texture-export`: rendering fidelity includes gradient fills and strokes.
- `workspace-layout`: the tool bar gains the Gradient tool after the Eyedropper.
- `command-system`: default shortcut G for the Gradient tool.

## Impact

- **tp-core:**
  - new `document/paint.rs` with `Paint`, `Gradient`, `GradientKind`, `ColorStop`, color interpolation and the gradient-space ↔ document mapping;
  - `Object.fill` and `StrokeStyle.color` become `Paint`;
  - gradient remapping for boolean operations and outlines.
- **tp-render:** gradient shaders in `fill_with` and `stroke_with`, plus new pixel tests.
- **tp-app:**
  - `geometry_cache::uses_mesh` also returns true for gradient paints;
  - canvas `paint.rs` draws textured meshes, using a new gradient-ramp texture cache;
  - `text_engine`;
  - workspace `Style`, `apply_color`, `swap_fill_stroke`, the eyedropper and the new Gradient tool;
  - panels `colors.rs` and `properties.rs`, where selection swatches show gradients;
  - `combine.rs` and `outline_text.rs`;
  - commands, kittests and screenshots.
- **tp-ui:** new `GradientBar` widget and gradient swatch drawing, plus a Gradient tool icon.
- **tp-file:**
  - new frozen `v3` module and `FORMAT_VERSION = 3`;
  - v2 → v3 migration;
  - a new `v3` fixture, with the v1 and v2 fixtures kept for migration tests.
- **Dependencies:** none new. tiny-skia already provides gradients, and egui meshes already support textures.
