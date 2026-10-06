## ADDED Requirements

### Requirement: Path objects
A path object SHALL be made of one or more subpaths. Each subpath is an ordered list of anchor points and is either open or closed.

- **Points**: each anchor point has a position and optional incoming and outgoing Bézier handles. A point is smooth when its handles are aligned (opposite directions), otherwise a corner.
- **Segments**: between two consecutive points, a segment is straight when neither of the facing handles is set, otherwise a cubic curve. A closed subpath also has a segment from its last point back to its first.
- **Filling**: closed subpaths SHALL be filled with the non-zero winding rule, so that a subpath drawn in the opposite direction inside another makes a hole. Open subpaths SHALL never be filled. The stroke SHALL follow every subpath, open or closed, with rounded ends on open subpaths.
- **Geometry**: path geometry SHALL stay vector. Moving, resizing, rotating and flipping a path SHALL transform its points and handles exactly. Flipping SHALL mirror the path rather than turn it.
- **Frame**: a path's frame (bounds, center and rotation, used by the selection overlay and the Transform panel) SHALL always be the bounds of its geometry in its own rotation, at least 1 texture pixel on each side.

#### Scenario: Flipping an arrow
- **WHEN** a path shaped like an arrow pointing right is flipped by dragging its right handle past its left side
- **THEN** the arrow points left

#### Scenario: Hole in a closed path
- **WHEN** a path has an outer square subpath and a smaller inner square subpath drawn in the opposite direction
- **THEN** the inner square area is not filled and objects below show through it

#### Scenario: Open path is not filled
- **WHEN** an open path with three points forming a "V" has a red fill and a black stroke
- **THEN** only the black "V" stroke is drawn

### Requirement: Polygon objects
A polygon object SHALL be defined by a frame (like rectangles and ellipses), a number of sides from 3 to 12, and an optional star inner radius from 10% to 90%:

- Its vertices SHALL be those of a regular polygon (or star) with its first vertex pointing up, scaled so that the polygon's bounds fill the frame.
- A star with N points SHALL alternate N outer vertices and N inner vertices, the inner vertices at the inner radius.
- A polygon SHALL remain editable through its settings until it is converted to a path.

#### Scenario: Triangle fills its frame
- **WHEN** a 3-sided polygon has a frame of 300 × 200
- **THEN** its top vertex is at the top center of the frame and its two other vertices are at the bottom corners

## MODIFIED Requirements

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

### Requirement: Hit testing
The document SHALL determine which object is under a given point by testing, from topmost to bottommost and descending into groups:

- for rectangles, ellipses, polygons and closed paths: the actual filled shape (including rotation, rounded corners, concave parts and holes), plus the stroke band when the object has a stroke;
- for open paths: the stroke, at least a few screen pixels wide;
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
