## MODIFIED Requirements

### Requirement: Rendering fidelity
The exported image SHALL be rendered from the vector document at the chosen size, covering exactly the surface (texture pixel (0,0) to (size,size)), with anti-aliased edges.

It SHALL include every visible object in stacking order, drawn with:

- fill, stroke, opacity (including group opacity), rotation and rounded corners;
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
