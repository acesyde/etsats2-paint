# document-model Specification

## Purpose

Defines the vector document a livery is made of: texture surfaces in texture-pixel coordinates holding ordered, fully editable vector objects, independent of display and export resolution.

## Requirements

### Requirement: Surfaces in texture-pixel coordinates
A project SHALL contain one or more surfaces. Each surface represents one texture of one of the project's vehicles: a main texture or an accessory. It has the texture's name and the texture's square size, in texture pixels, with the origin at the top-left corner, x to the right and y downward. Exactly one surface is active at a time.

Every surface SHALL have a template: a reference image of the texture's layout, with an opacity, a visibility, a layout version and a status, and the vehicle and texture it belongs to, with whether that texture is a main texture or an accessory. The template isn't part of the surface's artwork:
- it is not an object;
- it is never selected or exported;
- changing its opacity or visibility doesn't change the document's undo history.

A surface whose texture is no longer in its package version keeps its template record, marked "Not in this version", and is drawn without a template.

#### Scenario: New project surface
- **WHEN** a project is created from the sample truck's "Standard cab"
- **THEN** its active surface is the 4096 px "Standard cab" main texture, spanning (0, 0) to (4096, 4096)

#### Scenario: Surfaces of different sizes
- **WHEN** a vehicle project has a 4096 px "Standard cab" texture and a 1024 px "Cab accessories" texture
- **THEN** its Standard cab surface spans (0, 0) to (4096, 4096) and its Cab accessories surface spans (0, 0) to (1024, 1024)

### Requirement: Ordered vector objects
Each surface SHALL hold an ordered tree of objects: a top-level ordered list where any object may be a group holding its own ordered list of children (groups can nest). Within a list, later objects are drawn above earlier ones, and a group is drawn at its position with its children in order. Every object SHALL have a unique, stable identifier that does not change when the object is moved, transformed, reordered, grouped, ungrouped, point-edited or converted to a path. Every object SHALL also have a name; the default is "Rectangle", "Ellipse", "Polygon", "Path", "Line", "Group" or "Text", and for images, the imported file name without extension.

#### Scenario: Stacking order
- **WHEN** a rectangle is created and then an ellipse overlapping it
- **THEN** the ellipse is drawn above the rectangle

#### Scenario: Identity is stable
- **WHEN** an object is moved, resized, rotated and brought forward
- **THEN** its identifier is unchanged

#### Scenario: Group drawing order
- **WHEN** a group containing a red rectangle above a blue ellipse sits below a green rectangle
- **THEN** the drawing order from bottom to top is blue ellipse, red rectangle, green rectangle

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

### Requirement: Objects may extend beyond the surface
Objects SHALL be allowed to lie partly or fully outside the surface bounds; they are kept in the document and remain selectable, and only the part inside the surface belongs to the texture.

#### Scenario: Object dragged off the artboard
- **WHEN** an object is moved so it lies entirely outside the artboard
- **THEN** it still exists, is drawn on the pasteboard and can be selected and moved back

### Requirement: Hit testing
The document SHALL determine which object is under a given point by testing, from topmost to bottommost and descending into groups:

- for rectangles, ellipses, polygons and closed paths: the actual filled shape (including rotation, rounded corners, concave parts and holes), plus the stroke band when the object has a stroke;
- for open paths: the line, including its stroke, at least a few screen pixels wide;
- for texts: the laid-out text bounds;
- for images: the frame.

Hidden objects and locked objects (including children of hidden or locked groups) SHALL NOT be hit. For objects whose drawn size on screen is very small, a tolerance of a few screen pixels SHALL apply so they remain clickable. Marquee selection SHALL use the same shapes: an object is touched by a marquee when its shape intersects the marquee rectangle.

#### Scenario: Clicking the corner of an ellipse's bounding box
- **WHEN** the user clicks inside an ellipse's bounding box but outside the ellipse itself
- **THEN** the ellipse is not hit and the object below it (or nothing) is hit instead

#### Scenario: Topmost wins
- **WHEN** two objects overlap at the clicked point
- **THEN** the topmost of the two is hit

#### Scenario: Hidden object is not hit
- **WHEN** a hidden rectangle lies above a visible ellipse at the clicked point
- **THEN** the ellipse is hit

#### Scenario: Clicking between letters
- **WHEN** the user clicks in the gap between two letters of a text
- **THEN** the text is hit

#### Scenario: Inside a star's notch
- **WHEN** the user clicks between two points of a star, outside the star but inside its bounds
- **THEN** the star is not hit

#### Scenario: Clicking near a thin line
- **WHEN** the user clicks 2 screen pixels away from a 1 px line
- **THEN** the line is hit

### Requirement: Visibility and lock flags
Every object SHALL have a visible flag (default true) and a locked flag (default false). An object is effectively hidden or locked when it or any ancestor group is. Hidden objects SHALL NOT be drawn.

#### Scenario: Hidden group hides children
- **WHEN** a group is hidden
- **THEN** none of its children are drawn, even though their own visible flags are true

### Requirement: Project palette
A project SHALL hold an ordered palette of **swatches**, starting empty. Each swatch has:
- a stable identity;
- a name, which is not empty;
- a color.

Adding a color that a swatch already has SHALL add nothing. A new swatch is named "Color N", where N is the first number not yet used by such a name.

**Links:**
- A solid fill, a solid stroke and each gradient stop MAY be **linked** to a swatch. The color of a linked paint SHALL always equal its swatch's color.
- Changing a swatch's color SHALL change every color linked to it, on every surface and in every shared style, at once.
- A link SHALL be dropped when the color it links is changed to anything other than its swatch's color, or when its swatch is deleted.
- Deleting a swatch SHALL leave every color unchanged.
- Copying an object keeps its links. A link to a swatch the project doesn't have, such as on an object pasted from another project, SHALL be dropped.

#### Scenario: No duplicate swatches
- **WHEN** the same color is added to the palette twice
- **THEN** the palette contains it once

#### Scenario: A swatch recolors the fleet
- **WHEN** rectangles on two textures fill with the swatch "Company red", a gradient stop of a third object is linked to it, and its color is changed to dark red
- **THEN** both rectangles and that stop are dark red, and the other stops are unchanged

#### Scenario: Picking another color unlinks
- **WHEN** a rectangle linked to "Company red" gets a blue fill from the hex field
- **THEN** its fill is no longer linked, and editing "Company red" doesn't change it

#### Scenario: Deleting a swatch
- **WHEN** the swatch "Company red" is deleted while three objects are linked to it
- **THEN** the three objects stay red and are no longer linked

### Requirement: Text and image objects
A text object SHALL hold its content (one or more lines) and its character style; its frame is derived from its laid-out bounds and scale. An image object SHALL reference a project asset and keep its own frame; it has no fill or stroke. Both kinds SHALL support opacity, visibility, lock, grouping and all transforms.

A text or an image SHALL also have a mirrored state, off by default. A mirrored object SHALL be drawn reversed across its own vertical axis, within the same frame. A vertical mirror is a mirrored object turned by a half-turn. The mirrored state SHALL apply wherever the object is drawn or read:

- on the canvas, in texture and mod export, and inside symbol instances;
- for an image, when the eyedropper samples it: it SHALL take the pixel shown under the pointer;
- for a text, while editing it: the caret and the selected characters SHALL be placed on the mirrored glyphs, and clicking places the caret under the pointer;
- for a text, when it is outlined: Create Outlines SHALL give mirrored letters.

The mirrored state SHALL NOT change the object's bounds, hit testing, size or its links to swatches and styles.

#### Scenario: Text bounds follow content
- **WHEN** characters are added to a text
- **THEN** its width grows and the selection bounds follow

#### Scenario: A mirrored image is exported mirrored
- **WHEN** a mirrored image of a horse facing left on the canvas is exported with Export Texture
- **THEN** the exported texture shows the horse facing left, at the same place

#### Scenario: Editing a mirrored text
- **WHEN** a mirrored text "TRANS" is double-clicked and the user types "PORT" at its end
- **THEN** the text reads "TRANSPORT", stays mirrored, and the new letters appear on the side where the text ends when mirrored

#### Scenario: Outlining a mirrored text
- **WHEN** a mirrored text is converted with Create Outlines
- **THEN** the letter paths are mirrored like the text was, at the same place

#### Scenario: Eyedropper on a mirrored image
- **WHEN** an image is red on its left half and blue on its right half, is mirrored, and the eyedropper is clicked on its left half
- **THEN** the picked color is blue

### Requirement: Project assets
A project SHALL hold an asset store: each asset has a stable identifier, a name, its kind (raster or SVG), its original file bytes and its pixel or declared size. Assets are shared by every image object referencing them and are deduplicated by content.

#### Scenario: Asset shared by two images
- **WHEN** an image is duplicated
- **THEN** both objects reference the same asset

### Requirement: Path objects
A path object SHALL be made of one or more subpaths. Each subpath is an ordered list of anchor points and is either open or closed.

- **Points**: each anchor point has a position and optional incoming and outgoing Bézier handles. A point is smooth when its handles are aligned (opposite directions), otherwise a corner.
- **Segments**: between two consecutive points, a segment is straight when neither of the facing handles is set, otherwise a cubic curve. A closed subpath also has a segment from its last point back to its first.
- **Filling**: closed subpaths SHALL be filled with the fill color using the non-zero winding rule, so that a subpath drawn in the opposite direction inside another makes a hole, and the stroke SHALL follow their outline.
- **Lines**: open subpaths SHALL be drawn as lines of the path's line width (default 8 texture pixels) in the fill color, with rounded ends; the area between the points of an open subpath is never filled. The stroke, when set, SHALL outline each line (centered on its edges, like a shape's outline), so a white line with a black stroke shows a black border around a white line.
- **Geometry**: path geometry SHALL stay vector. Moving, resizing, rotating and flipping a path SHALL transform its points and handles exactly. Flipping SHALL mirror the path rather than turn it.
- **Frame**: a path's frame (bounds, center and rotation, used by the selection overlay and the inspector's Layout section) SHALL always be the bounds of its geometry in its own rotation, at least 1 texture pixel on each side.

#### Scenario: Flipping an arrow
- **WHEN** a path shaped like an arrow pointing right is flipped by dragging its right handle past its left side
- **THEN** the arrow points left

#### Scenario: Hole in a closed path
- **WHEN** a path has an outer square subpath and a smaller inner square subpath drawn in the opposite direction
- **THEN** the inner square area is not filled and objects below show through it

#### Scenario: Open path is a line
- **WHEN** an open path with three points forming a "V" has a red fill, a line width of 20 px and no stroke
- **THEN** a red "V" line 20 px wide is drawn and the area between its arms is empty

#### Scenario: Outlined line
- **WHEN** that "V" also has a black 4 px stroke
- **THEN** the red line has a black border along both edges and around its rounded ends

### Requirement: Polygon objects
A polygon object SHALL be defined by a frame (like rectangles and ellipses), a number of sides from 3 to 12, and an optional star inner radius from 10% to 90%:

- Its vertices SHALL be those of a regular polygon (or star) with its first vertex pointing up, scaled so that the polygon's bounds fill the frame.
- A star with N points SHALL alternate N outer vertices and N inner vertices, the inner vertices at the inner radius.
- A polygon SHALL remain editable through its settings until it is converted to a path.

#### Scenario: Triangle fills its frame
- **WHEN** a 3-sided polygon has a frame of 300 × 200
- **THEN** its top vertex is at the top center of the frame and its two other vertices are at the bottom corners

### Requirement: Surface guides
Each surface SHALL hold an ordered list of guides. A guide is horizontal or vertical and has a position in texture pixels (y for horizontal guides, x for vertical guides); positions outside the surface are allowed. A new surface SHALL have no guides.

#### Scenario: New project has no guides
- **WHEN** a project is created
- **THEN** its surface has no guides

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
