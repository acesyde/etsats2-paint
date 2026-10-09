# gradients Specification

## Purpose

Lets fills and strokes be painted with linear and radial color gradients, edited in the color popover of the inspector and directly on the canvas, so liveries can use fades, metallic bands and glows without stacking shapes.

## Requirements

### Requirement: Gradient paints
A fill or a stroke SHALL be painted with either a solid color or a gradient. A gradient SHALL be either:

- **Linear:** defined by a start point and an end point. The color varies along the line from start to end and is constant along lines perpendicular to it.
- **Radial:** defined by a center, a radius point and an aspect ratio. The colors form concentric ellipses around the center. The distance from the center to the radius point is the main radius, along the direction of that point. The aspect ratio, from 1% to 1000% with a default of 100%, sets the perpendicular radius relative to the main radius.

A gradient SHALL have from two to sixteen color stops. Each stop has a color with alpha and a location from 0% (start or center) to 100% (end or radius). Between two stops, the color SHALL be interpolated linearly, channel by channel, in sRGB with straight (non-premultiplied) alpha. Two stops MAY share a location, which gives a hard edge. Before the first stop and beyond the last one, the color of that end stop SHALL be used.

A gradient's points SHALL be stored relative to the object's frame. The gradient therefore follows every move, resize, rotation and flip of its object, including non-uniform resizes, which stretch it. A gradient whose start and end (or center and radius point) coincide SHALL be painted with its last stop's color.

#### Scenario: Horizontal fade
- **WHEN** a 400 × 100 rectangle is filled with a linear gradient from opaque red at 0% to opaque blue at 100%, from its left edge's midpoint to its right edge's midpoint
- **THEN** its left edge is red, its right edge is blue and its center is the 50% mix (128, 0, 128)

#### Scenario: Fade to transparent
- **WHEN** a rectangle has a linear gradient from opaque white to fully transparent white
- **THEN** it goes from white to see-through without darkening in the middle

#### Scenario: Beyond the ends
- **WHEN** a linear gradient's start and end points both lie inside a rectangle, at 25% and 75% of its width
- **THEN** the part left of the start is the first stop's color and the part right of the end is the last stop's color

#### Scenario: Gradient follows the object
- **WHEN** an object with a left-to-right gradient is rotated by 90° and then resized to twice its width
- **THEN** the gradient runs top to bottom and still spans the whole object

#### Scenario: Elliptical glow
- **WHEN** a radial gradient has its radius point 100 px right of its center and an aspect ratio of 50%
- **THEN** each color forms an ellipse 200 px wide and 100 px tall around the center

### Requirement: Where gradients apply
Gradients SHALL be available for the fill and the stroke of rectangles, ellipses, polygons, stars, paths and texts. On lines (open paths), the fill paint SHALL paint the line's body and the stroke paint SHALL paint its outline. Gradients SHALL work with every stroke alignment, dash pattern, cap and join. Opacity SHALL multiply the gradient's alpha like it does for solid colors. Images and groups have no fill or stroke and are unchanged.

#### Scenario: Gradient lettering with an outline
- **WHEN** a text has a vertical gold-to-brown fill gradient and a solid black 10 px Outside stroke
- **THEN** the letters are shaded from gold at the top to brown at the bottom, outlined in black

#### Scenario: Gradient on a dashed line
- **WHEN** a dashed 20 px line has a linear fill gradient from yellow at its start to red at its end
- **THEN** each dash takes the gradient color of its position along the line

### Requirement: Choosing a paint kind
The Colors panel SHALL show a Solid / Linear / Radial control for the current target (fill or stroke). It SHALL show "Mixed" (no segment selected) when the selection's paints differ in kind. Choosing a kind SHALL apply it to every selected object, as one undo step:

- **Solid → gradient:** the new gradient goes from the current color to the same color at 0% alpha. A linear gradient spans the object from the middle of its left edge to the middle of its right edge. A radial gradient is centered with a radius reaching the right edge, so its ellipse fits the object's frame.
- **Gradient → gradient:** switching between linear and radial keeps the stops.
- **Gradient → solid:** the paint becomes the color of the first stop.

With nothing selected, the choice applies to the current style, and new shapes get that paint.

#### Scenario: Make a fill linear
- **WHEN** a red rectangle is selected and the user chooses Linear for the fill
- **THEN** the rectangle fades from opaque red at its left edge to transparent at its right edge, and one Undo makes it solid red again

#### Scenario: New shapes use the current gradient
- **WHEN** nothing is selected, the user chooses Radial for the fill and draws an ellipse
- **THEN** the new ellipse has a radial fill gradient

### Requirement: Gradient bar and stops
In gradient mode, the panel SHALL show a gradient bar previewing the gradient over a checkerboard, with one marker per stop. Exactly one stop is the selected stop. The panel SHALL support these operations:

- **Select a stop:** clicking a marker selects it. The color picker, sliders, hex field, recent colors and palette then edit and apply to the selected stop.
- **Move a stop:** dragging a marker moves its stop along the bar. A Location field (0–100%) sets its location exactly. Stops may pass each other.
- **Add a stop:** clicking the bar away from markers adds a stop at that location, with the color the gradient has there, and selects it. Nothing is added when the gradient already has sixteen stops.
- **Delete a stop:** Delete or Backspace on a focused marker, or dragging a marker well away from the bar, removes its stop. A gradient SHALL keep at least two stops.
- **Reverse:** a Reverse button mirrors every stop's location.
- **Angle (both kinds):** an Angle field shows the direction from start to end (linear) or center to radius point (radial), in degrees on the document. Setting it rotates the gradient around its midpoint (linear) or center (radial), keeping its length.
- **Aspect (radial only):** an Aspect field sets the aspect ratio.

Each drag SHALL be one undo step, and each field change SHALL be one undo step. Markers SHALL be keyboard-focusable, and Left and Right arrows move the focused stop by 1%. When the selected objects have different gradients, the bar SHALL show the first selected object's gradient, and an edit SHALL apply that whole gradient to every selected object.

#### Scenario: Add a middle stop
- **WHEN** a red-to-blue gradient is edited and the user clicks the middle of the bar
- **THEN** a third stop is added at 50% with color (128, 0, 128) and is selected

#### Scenario: Recolor a stop with the hex field
- **WHEN** the second stop is selected and the user types "#00ff00" and presses Enter
- **THEN** only that stop becomes green

#### Scenario: Two stops minimum
- **WHEN** a gradient has two stops and the user presses Delete on one of them
- **THEN** both stops remain

#### Scenario: Hard-edged band
- **WHEN** the user sets two adjacent stops, white and red, to the same location 50%
- **THEN** the gradient switches from white to red sharply at the middle

### Requirement: Gradient tool
The Gradient tool SHALL edit the gradient of the current target (fill or stroke) on the canvas.

When exactly one object is selected and its target paint is a gradient, the tool SHALL show handles over the object:

- a start handle and an end handle joined by a line (linear);
- a center handle, a radius handle and an aspect handle on the perpendicular axis (radial);
- the stop locations as marks along the line.

Dragging a handle SHALL move that point, and Shift SHALL constrain the direction to 45° steps around the opposite point (or around the center). Dragging the aspect handle SHALL change only the aspect ratio. Each drag SHALL be one undo step.

Dragging elsewhere on the canvas while objects are selected SHALL set a new gradient vector for the target of every selected object, from the press point to the release point (start to end, or center to radius point), keeping the stops and aspect ratio. Shift SHALL constrain the direction to 45° steps. Objects whose target is solid SHALL first get a linear gradient as if Linear had been chosen. Clicking without dragging SHALL select the object under the pointer, as the Selection tool does.

Hidden and locked objects SHALL NOT be edited. The cursor SHALL show a crosshair.

#### Scenario: Drag a gradient across a rectangle
- **WHEN** a solid blue rectangle is selected, the Gradient tool is active, and the user drags from its top edge to its bottom edge
- **THEN** the rectangle fades from opaque blue at the top to transparent at the bottom, and one Undo restores the solid blue fill

#### Scenario: Move the end handle
- **WHEN** a rectangle with a linear fill gradient is selected with the Gradient tool, and the user drags the end handle halfway toward the start
- **THEN** the gradient ends at the new point and the far part of the rectangle shows the last stop's color

#### Scenario: Constrained angle
- **WHEN** the user drags a new gradient vector with Shift held, at about 40° from horizontal
- **THEN** the gradient vector is at exactly 45°

### Requirement: Gradients survive geometry conversions
Converting text to outlines, combining shapes with boolean operations, editing a path's points or handles, and duplicating or copying and pasting objects SHALL keep each resulting object's fill and stroke gradients at the same place in the document, even when the resulting object's frame differs from the original's.

#### Scenario: Outlined gradient text
- **WHEN** a rotated text with a linear fill gradient is converted to outlines
- **THEN** the resulting path looks the same, with the gradient at the same position and angle

#### Scenario: Unite keeps the gradient
- **WHEN** two overlapping shapes are combined with Unite and the resulting shape takes the topmost shape's gradient fill
- **THEN** the gradient sits where it was on the topmost shape

#### Scenario: Editing a path's points
- **WHEN** a path with a linear fill gradient has one of its points moved with the Direct Selection tool, which enlarges its bounds
- **THEN** the gradient's start and end stay at the same place in the document
