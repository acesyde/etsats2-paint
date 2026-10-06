# path-tools Specification

## Purpose

Defines how freeform paths, straight lines and regular polygons or stars are created on the canvas with the Pen, Line and Polygon tools, and how polygon settings are edited.

## Requirements

### Requirement: Pen tool point placement
With the Pen tool active, each click on the canvas SHALL add an anchor point to the path being drawn. A click without dragging SHALL add a corner point. Pressing and dragging SHALL add a smooth point: its outgoing handle follows the pointer and its incoming handle mirrors it. While a path is being drawn, the canvas SHALL show its points, its segments so far, and a preview of the next segment from the last point to the pointer. The path SHALL be drawn with the current fill and stroke as it would be created. Holding Shift SHALL constrain the new point (relative to the previous point) and the dragged handle to the nearest multiple of 45°.

#### Scenario: Three clicks
- **WHEN** the Pen tool is active and the user clicks at (100, 100), (500, 100) and (500, 400), then presses Enter
- **THEN** an open path with three corner points and two straight segments is created and selected

#### Scenario: Drag makes a smooth point
- **WHEN** the user clicks at (100, 100), then presses at (500, 100) and drags to (600, 100) before releasing
- **THEN** the second point is smooth, with its outgoing handle at (600, 100) and its incoming handle at (400, 100)

#### Scenario: Shift constrains the next point
- **WHEN** after a point at (100, 100) the user clicks at (400, 130) while holding Shift
- **THEN** the new point is placed at (400, 100)

### Requirement: Finishing, closing and cancelling a pen path
Clicking the first point of the path being drawn (within a few screen pixels) SHALL close it and finish it. Pressing Enter or Escape, choosing another tool, or starting another command SHALL finish it as an open path. A finished path SHALL be added at the top of the active layer (same rule as the shape tools), selected, and recorded as one undo step; the Pen tool SHALL stay active. A path with fewer than 2 points SHALL be discarded when finished. Backspace or Undo (Cmd/Ctrl+Z) while drawing SHALL remove the last placed point instead of undoing a document change; removing the only point SHALL discard the path. The path being drawn SHALL NOT be part of the document until it is finished.

#### Scenario: Closing a triangle
- **WHEN** the user clicks three points and then clicks the first point again
- **THEN** a closed triangular path is created, filled with the current fill and selected

#### Scenario: Single point is discarded
- **WHEN** the user clicks once with the Pen tool and presses Enter
- **THEN** no object is created

#### Scenario: Backspace removes the last point
- **WHEN** the user has placed four points and presses Backspace, then Enter
- **THEN** the created path has three points

#### Scenario: Undo while drawing removes a point
- **WHEN** the user has placed three points and presses Cmd/Ctrl+Z
- **THEN** the path being drawn has two points and the document is unchanged

#### Scenario: Whole path is one undo step
- **WHEN** the user creates a five-point path and presses Cmd/Ctrl+Z
- **THEN** the whole path disappears

### Requirement: Line tool
With the Line tool active, dragging on the canvas SHALL draw a straight line from the press point to the pointer, with a live preview; releasing SHALL create an open path with two corner points named "Line", selected, at the top of the active layer, and keep the tool active. Holding Shift SHALL constrain the angle to the nearest multiple of 45°, and holding Alt/Option SHALL use the press point as the middle of the line. A drag shorter than 2 screen pixels SHALL NOT create a line. Escape during the drag SHALL cancel it. While dragging, the canvas SHALL show the line's length and angle. The created line's rotation SHALL be its own angle (folded into −90°..90°), so that its selection frame follows the line and the Transform panel shows that angle; setting the rotation to 0° SHALL make the line exactly horizontal.

#### Scenario: Nearly horizontal line
- **WHEN** the user drags a line from (500, 1000) to (3500, 1020)
- **THEN** the line's rotation is about 0.38°, its frame is 3000 px long and follows the line, and setting the rotation to 0° in the Transform panel makes both end points have the same y

#### Scenario: Horizontal line with Shift
- **WHEN** the user drags from (100, 100) to (500, 120) with the Line tool while holding Shift
- **THEN** a line from (100, 100) to (500, 100) is created

#### Scenario: Line from the middle
- **WHEN** the user drags from (500, 500) to (600, 500) while holding Alt/Option
- **THEN** a line from (400, 500) to (600, 500) is created

### Requirement: Appearance of new open paths
Open paths (pen paths that were not closed, and lines) SHALL be created with the current fill and stroke, like every other shape: their line is drawn in the fill color and the stroke (off by default) outlines it. They SHALL use the line width last set in the Properties panel, initially 8 texture pixels.

#### Scenario: Default line
- **WHEN** the current fill is red with no stroke and the user draws a line in a new project
- **THEN** the line is red, 8 texture pixels wide, and has no stroke

#### Scenario: Line uses the current stroke as an outline
- **WHEN** the current fill is white, the current stroke is black at 4 px, and the user draws a line
- **THEN** the line is white with a black outline

### Requirement: Line width
When every selected object is a path with at least one open subpath, the Properties panel SHALL show a Width field (texture pixels, from 0.5 to 1000) setting the width of the open subpaths' lines. Changes SHALL apply to every selected path, be one undo step per change, and show "Mixed" for differing values. The value set SHALL also become the width used for new open paths. The width SHALL NOT change when the path is resized.

#### Scenario: Thicker line
- **WHEN** a line is selected and the user sets Width to 40
- **THEN** the line is drawn 40 texture pixels wide and one Undo restores 8

#### Scenario: Next line uses the last width
- **WHEN** the user sets a line's Width to 40 and then draws another line
- **THEN** the new line is 40 texture pixels wide

### Requirement: Polygon tool
With the Polygon tool active, dragging on the canvas SHALL draw a polygon (or star) filling the dragged box, with a live preview; releasing SHALL create it, select it, place it at the top of the active layer and keep the tool active. The first vertex SHALL point straight up, and the polygon's bounds SHALL match the dragged box. Holding Shift SHALL keep the polygon regular (equal sides and angles), and holding Alt/Option SHALL draw from the press point as the center; both SHALL apply live. A drag shorter than 2 screen pixels SHALL NOT create a polygon, and Escape during the drag SHALL cancel it. New polygons SHALL use the current fill and stroke, and the sides and star settings last chosen in the Properties panel (initially 6 sides, not a star, 50% inner radius).

#### Scenario: Default hexagon
- **WHEN** the user drags a box from (100, 100) to (500, 500) with the Polygon tool in a new project
- **THEN** a six-sided polygon named "Polygon" is created whose bounds are (100, 100) to (500, 500), with a vertex at the top center (300, 100)

#### Scenario: Shift keeps it regular
- **WHEN** the user drags with Shift held
- **THEN** every side of the created polygon has the same length

### Requirement: Polygon settings
When every selected object is a polygon, the Properties panel SHALL show a Sides field (3 to 12), a Star toggle and, for stars, an Inner radius field (10% to 90% of the outer radius). Changes SHALL apply to every selected polygon, keep each polygon's bounds, center and rotation, and be one undo step per change; differing values SHALL show "Mixed". The values set SHALL also become the settings used for new polygons. A star with N points SHALL have N outer and N inner vertices, alternating.

#### Scenario: Make a five-point star
- **WHEN** a polygon is selected and the user sets Sides to 5 and turns Star on
- **THEN** the polygon becomes a five-point star with ten vertices in the same bounds, and one Undo restores the polygon

#### Scenario: Sides are clamped
- **WHEN** the user types 40 in the Sides field
- **THEN** the polygon has 12 sides
