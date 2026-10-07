## MODIFIED Requirements

### Requirement: Object geometry and appearance
Rectangles and ellipses SHALL be defined by a frame (center point, width, height, rotation in degrees clockwise around the center), and SHALL remain vector objects: their geometry is never rasterized in the document. Rectangles SHALL support a corner radius (default 0, at most half the smaller side). Every shape SHALL have a fill paint, an optional stroke (paint and width in texture pixels), and an opacity from 0% to 100%. A paint SHALL be either a solid color with alpha or a linear or radial gradient (see the gradients capability). A group SHALL have an opacity that multiplies its children's. A group's bounds are the union of its visible children's bounds. Width and height SHALL never be smaller than 1 texture pixel. New shapes created with a tool SHALL use the workspace's current fill and stroke (initially the default fill and no stroke).

#### Scenario: Default appearance of a new shape
- **WHEN** a rectangle is created with a tool in a new project
- **THEN** it has the default fill color, no stroke, 100% opacity and a corner radius of 0

#### Scenario: Minimum size
- **WHEN** an object is resized so that its width would become 0 or negative
- **THEN** its width is clamped to at least 1 texture pixel

#### Scenario: Current style applies to new shapes
- **WHEN** the current fill is set to white with a black 8 px stroke and the user draws an ellipse
- **THEN** the ellipse has a white fill and a black 8 px stroke

#### Scenario: Gradient fill with a solid stroke
- **WHEN** a rectangle's fill is set to a linear gradient while its stroke stays solid black
- **THEN** the rectangle has the gradient fill and the solid black stroke, and its other properties are unchanged

### Requirement: Stroke style
A stroke SHALL have, besides its paint (a solid color or a gradient) and its width:

- an **alignment**: Center (straddles the edge, default), Inside (within the filled area) or Outside (outside it);
- an optional **dash pattern**: a dash length (0 or more) and a gap length (more than 0), in texture pixels, starting at the beginning of each subpath;
- a **cap**: Butt, Round (default) or Square, which applies to the ends of open paths and of each dash;
- a **join**: Miter (default) with a miter limit (default 4), Round or Bevel.

The defaults SHALL render exactly like strokes did before these options existed. The stroke width SHALL be the visible thickness whatever the alignment.

Alignment by kind:
- **Shapes and texts:** Inside and Outside SHALL be measured against the object's filled area, including holes. An Inside stroke never paints outside the filled area, and an Outside stroke never paints inside it.
- **Lines (open paths):** Inside puts the outline within the line's body, and Outside puts it around the body.

Lines have their own dash pattern, cap and join (defaults: solid, round caps, miter joins with limit 4), applied to the line's body. A line's stroke (outline) follows the line's dashes, caps and joins; the stroke's own dash, cap and join SHALL NOT apply to lines.

#### Scenario: Outside outline on lettering
- **WHEN** a text with a 20 px Outside stroke is drawn
- **THEN** the letters keep their full shape and the outline is 20 px wide around them, including around the inner edge of the "O"'s counter

#### Scenario: Inside border
- **WHEN** a 400 px square has a 30 px Inside stroke
- **THEN** its outer edge is still at 400 px and the border covers the outer 30 px of the square

#### Scenario: Dotted line
- **WHEN** a 200 px line has a 10 px stroke with dash 0, gap 20 and round caps
- **THEN** it is drawn as round dots 20 px apart

#### Scenario: Dashed pinstripe without outline
- **WHEN** a red 12 px line without stroke is set to Dashed 40/20
- **THEN** it is drawn as red dashes 40 px long separated by 20 px gaps

#### Scenario: Old projects unchanged
- **WHEN** a project saved before stroke options existed is opened
- **THEN** every stroke is drawn exactly as before (centered, solid, round caps, miter joins)
