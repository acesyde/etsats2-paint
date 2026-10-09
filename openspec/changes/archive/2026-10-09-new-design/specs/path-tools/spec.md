## MODIFIED Requirements

### Requirement: Line tool
With the Line tool active, dragging on the canvas SHALL draw a straight line from the press point to the pointer, with a live preview; releasing SHALL create an open path with two corner points named "Line", selected, at the top of the active layer, and keep the tool active. Holding Shift SHALL constrain the angle to the nearest multiple of 45°, and holding Alt/Option SHALL use the press point as the middle of the line. A drag shorter than 2 screen pixels SHALL NOT create a line. Escape during the drag SHALL cancel it. While dragging, the canvas SHALL show the line's length and angle. The created line's rotation SHALL be its own angle (folded into −90°..90°), so that its selection frame follows the line and the inspector's Layout section shows that angle; setting the rotation to 0° SHALL make the line exactly horizontal.

#### Scenario: Nearly horizontal line
- **WHEN** the user drags a line from (500, 1000) to (3500, 1020)
- **THEN** the line's rotation is about 0.38°, its frame is 3000 px long and follows the line, and setting the rotation to 0° in the inspector's Layout section makes both end points have the same y

#### Scenario: Horizontal line with Shift
- **WHEN** the user drags from (100, 100) to (500, 120) with the Line tool while holding Shift
- **THEN** a line from (100, 100) to (500, 100) is created

#### Scenario: Line from the middle
- **WHEN** the user drags from (500, 500) to (600, 500) while holding Alt/Option
- **THEN** a line from (400, 500) to (600, 500) is created

### Requirement: Appearance of new open paths
Open paths (pen paths that were not closed, and lines) SHALL be created with the current fill and stroke, like every other shape: their line is drawn in the fill color and the stroke (off by default) outlines it. They SHALL use the line width last set in the Width field of the inspector's Appearance section, initially 8 texture pixels.

#### Scenario: Default line
- **WHEN** the current fill is red with no stroke and the user draws a line in a new project
- **THEN** the line is red, 8 texture pixels wide, and has no stroke

#### Scenario: Line uses the current stroke as an outline
- **WHEN** the current fill is white, the current stroke is black at 4 px, and the user draws a line
- **THEN** the line is white with a black outline

### Requirement: Line width
When every selected object is a path with at least one open subpath, the inspector's Appearance section SHALL show a Width field (texture pixels, from 0.5 to 1000) setting the width of the open subpaths' lines (see the properties-panel capability, Line settings). Changes SHALL apply to every selected path, be one undo step per change, and show "Mixed" for differing values. The value set SHALL also become the width used for new open paths. The width SHALL NOT change when the path is resized.

#### Scenario: Thicker line
- **WHEN** a line is selected and the user sets Width to 40 in the inspector
- **THEN** the line is drawn 40 texture pixels wide and one Undo restores 8

#### Scenario: Next line uses the last width
- **WHEN** the user sets a line's Width to 40 and then draws another line
- **THEN** the new line is 40 texture pixels wide

### Requirement: Polygon tool
With the Polygon tool active, dragging on the canvas SHALL draw a polygon (or star) filling the dragged box, with a live preview; releasing SHALL create it, select it, place it at the top of the active layer and keep the tool active. The first vertex SHALL point straight up, and the polygon's bounds SHALL match the dragged box. Holding Shift SHALL keep the polygon regular (equal sides and angles), and holding Alt/Option SHALL draw from the press point as the center; both SHALL apply live. A drag shorter than 2 screen pixels SHALL NOT create a polygon, and Escape during the drag SHALL cancel it. New polygons SHALL use the current fill and stroke, and the sides and star settings last chosen in the Polygon tool's options bar or the inspector's Polygon section (initially 6 sides, not a star, 50% inner radius).

#### Scenario: Default hexagon
- **WHEN** the user drags a box from (100, 100) to (500, 500) with the Polygon tool in a new project
- **THEN** a six-sided polygon named "Polygon" is created whose bounds are (100, 100) to (500, 500), with a vertex at the top center (300, 100)

#### Scenario: Shift keeps it regular
- **WHEN** the user drags with Shift held
- **THEN** every side of the created polygon has the same length

### Requirement: Polygon settings
The polygon settings are a Sides field (3 to 12), a Star toggle and, for stars, an Inner radius field (10% to 90% of the outer radius). They SHALL be shown in two places:
- the **tool options bar** of the Polygon tool (see the workspace-layout capability), where they set the settings used for new polygons and, when every selected object is a polygon, also edit the selected polygons;
- an inspector **Polygon** section, shown when every selected object is a polygon, whatever the active tool (see the properties-panel capability).

Changes to selected polygons SHALL apply to every selected polygon, keep each polygon's bounds, center and rotation, and be one undo step per change; differing values SHALL show "Mixed". The values set, in either place, SHALL also become the settings used for new polygons. A star with N points SHALL have N outer and N inner vertices, alternating.

#### Scenario: Make a five-point star
- **WHEN** a polygon is selected and the user sets Sides to 5 and turns Star on in the inspector's Polygon section
- **THEN** the polygon becomes a five-point star with ten vertices in the same bounds, and one Undo restores the polygon

#### Scenario: Sides are clamped
- **WHEN** the user types 40 in the Sides field
- **THEN** the polygon has 12 sides

### Requirement: Line dashes, caps and joins
When every selected object is a path with open subpaths, the line settings popover opened from the Width field of the inspector's Appearance section (see the properties-panel capability, Line settings) SHALL show:
- a Dashed toggle with Dash and Gap fields and the presets of the stroke popover (see the stroke-panel capability);
- a Cap control (Butt, Round, Square);
- a Join control (Miter, Round, Bevel), with a Miter limit field when the join is Miter.

They SHALL set the line's own dash pattern, cap and join, apply to every selected line, show "Mixed" for differing values, and be one undo step per change. New lines SHALL use the values last set, initially solid with round caps and miter joins.

#### Scenario: Dashed line from the Properties panel
- **WHEN** a line is selected and the user opens its line settings in the inspector and turns Dashed on with Dash 30 and Gap 15
- **THEN** the line is drawn as 30 px dashes separated by 15 px gaps, without needing a stroke

#### Scenario: Butt ends
- **WHEN** a line with round ends is set to the Butt cap
- **THEN** its ends stop exactly at its end points
