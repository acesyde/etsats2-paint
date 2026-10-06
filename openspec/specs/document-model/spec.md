# document-model Specification

## Purpose

Defines the vector document a livery is made of: texture surfaces in texture-pixel coordinates holding ordered, fully editable vector objects, independent of display and export resolution.

## Requirements

### Requirement: Surfaces in texture-pixel coordinates
A project SHALL contain one or more surfaces. Each surface represents one texture of the vehicle and has a name and a square size equal to the project resolution, expressed in texture pixels with the origin at the top-left corner, x to the right and y downward. A new project SHALL contain exactly one surface named "Main texture". Exactly one surface is active at a time.

#### Scenario: New project surface
- **WHEN** a 4096×4096 project is created
- **THEN** it contains one active surface named "Main texture" spanning (0, 0) to (4096, 4096)

### Requirement: Ordered vector objects
Each surface SHALL hold an ordered tree of objects: a top-level ordered list where any object may be a group holding its own ordered list of children (groups can nest). Within a list, later objects are drawn above earlier ones, and a group is drawn at its position with its children in order. Every object SHALL have a unique, stable identifier that does not change when the object is moved, transformed, reordered, grouped or ungrouped, and a name (default: "Rectangle", "Ellipse" or "Group").

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
Rectangles and ellipses SHALL be defined by a frame (center point, width, height, rotation in degrees clockwise around the center), and SHALL remain vector objects: their geometry is never rasterized in the document. Rectangles SHALL support a corner radius (default 0, at most half the smaller side). Every shape SHALL have a solid fill color with alpha, an optional stroke (color and width in texture pixels), and an opacity from 0% to 100%; a group SHALL have an opacity that multiplies its children's. A group's bounds are the union of its visible children's bounds. Width and height SHALL never be smaller than 1 texture pixel. New shapes created with a tool SHALL use the workspace's current fill and stroke (initially the default fill and no stroke).

#### Scenario: Default appearance of a new shape
- **WHEN** a rectangle is created with a tool in a new project
- **THEN** it has the default fill color, no stroke, 100% opacity and a corner radius of 0

#### Scenario: Minimum size
- **WHEN** an object is resized so that its width would become 0 or negative
- **THEN** its width is clamped to at least 1 texture pixel

#### Scenario: Current style applies to new shapes
- **WHEN** the current fill is set to white with a black 8 px stroke and the user draws an ellipse
- **THEN** the ellipse has a white fill and a black 8 px stroke

### Requirement: Objects may extend beyond the surface
Objects SHALL be allowed to lie partly or fully outside the surface bounds; they are kept in the document and remain selectable, and only the part inside the surface belongs to the texture.

#### Scenario: Object dragged off the artboard
- **WHEN** an object is moved so it lies entirely outside the artboard
- **THEN** it still exists, is drawn on the pasteboard and can be selected and moved back

### Requirement: Hit testing
The document SHALL determine which object is under a given point by testing the actual filled shape (including rotation and rounded corners), from topmost to bottommost, descending into groups. Hidden objects and locked objects (including children of hidden or locked groups) SHALL NOT be hit. For objects whose drawn size on screen is very small, a tolerance of a few screen pixels SHALL apply so they remain clickable.

#### Scenario: Clicking the corner of an ellipse's bounding box
- **WHEN** the user clicks inside an ellipse's bounding box but outside the ellipse itself
- **THEN** the ellipse is not hit and the object below it (or nothing) is hit instead

#### Scenario: Topmost wins
- **WHEN** two objects overlap at the clicked point
- **THEN** the topmost of the two is hit

#### Scenario: Hidden object is not hit
- **WHEN** a hidden rectangle lies above a visible ellipse at the clicked point
- **THEN** the ellipse is hit

### Requirement: Visibility and lock flags
Every object SHALL have a visible flag (default true) and a locked flag (default false). An object is effectively hidden or locked when it or any ancestor group is. Hidden objects SHALL NOT be drawn.

#### Scenario: Hidden group hides children
- **WHEN** a group is hidden
- **THEN** none of its children are drawn, even though their own visible flags are true

### Requirement: Project palette
A project SHALL hold an ordered palette of colors without duplicates, starting empty.

#### Scenario: No duplicate swatches
- **WHEN** the same color is added to the palette twice
- **THEN** the palette contains it once
