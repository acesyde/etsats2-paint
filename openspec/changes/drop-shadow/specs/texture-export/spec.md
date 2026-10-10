## MODIFIED Requirements

### Requirement: Rendering fidelity
The exported image SHALL be rendered from the vector document at the chosen size, covering exactly the surface (texture pixel (0,0) to (size,size)), with anti-aliased edges.

It SHALL include every visible object in stacking order, drawn with:

- fill, stroke, opacity (including group opacity), rotation and rounded corners;
- drop shadows, drawn as on the canvas (see the drop-shadow capability);
- solid colors and linear and radial gradients, for fills and strokes alike, matching the canvas;
- polygons, stars and paths with their exact curves, the same as on the canvas: closed subpaths filled with the non-zero rule (holes included), open subpaths drawn as lines of their width in the fill color with rounded ends, outlined by the stroke when set;
- text drawn from the same glyph outlines as the canvas.

Raster images SHALL be resampled smoothly; SVG images SHALL be rendered from their vector source at the output resolution. Hidden objects (and children of hidden groups) SHALL NOT be rendered; locked objects SHALL be rendered. Parts of objects outside the surface SHALL be clipped.

#### Scenario: Hidden layer is not exported
- **WHEN** a group is hidden in the Layers panel and the texture is exported
- **THEN** none of its objects appear in the image

#### Scenario: Sharp SVG at 8K
- **WHEN** an SVG logo placed at 512 × 256 texture pixels on an 8192 project is exported at full size
- **THEN** its edges are as sharp as its other vector shapes, not an upscaled bitmap

#### Scenario: Stacking and opacity
- **WHEN** a red rectangle at 50% opacity lies above a blue ellipse
- **THEN** the overlapping pixels are the 50% blend of red over blue

#### Scenario: Open path in the export
- **WHEN** a red open "V" path with a line width of 20 px and no stroke is exported
- **THEN** the image shows a red "V" line 20 px wide, with no red area between its arms

#### Scenario: Gradient in the export
- **WHEN** a 1024 × 1024 project has a full-surface rectangle with a linear fill gradient from opaque black at the left edge to opaque white at the right edge, and it is exported at 1024 px
- **THEN** each pixel column's gray level increases from 0 to 255 from left to right, and the middle column is within one step of 128

#### Scenario: Shadow in the export
- **WHEN** a white square with a hard black shadow offset 10 / 10 lies on a red background and the texture is exported as PNG at full size
- **THEN** the image shows the black shadow 10 px right and below the square, under it, as on the canvas
