## MODIFIED Requirements

### Requirement: Ordered vector objects
Each surface SHALL hold an ordered tree of objects: a top-level ordered list where any object may be a group holding its own ordered list of children (groups can nest). Within a list, later objects are drawn above earlier ones, and a group is drawn at its position with its children in order. Every object SHALL have a unique, stable identifier that does not change when the object is moved, transformed, reordered, grouped or ungrouped, and a name (default: "Rectangle", "Ellipse", "Group", "Text" or, for images, the imported file name without extension).

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
The document SHALL determine which object is under a given point by testing the actual filled shape (including rotation and rounded corners) for rectangles and ellipses, the laid-out text bounds for texts, and the frame for images, from topmost to bottommost, descending into groups. Hidden objects and locked objects (including children of hidden or locked groups) SHALL NOT be hit. For objects whose drawn size on screen is very small, a tolerance of a few screen pixels SHALL apply so they remain clickable.

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

## ADDED Requirements

### Requirement: Text and image objects
A text object SHALL hold its content (one or more lines) and its character style; its frame is derived from its laid-out bounds and scale. An image object SHALL reference a project asset and keep its own frame; it has no fill or stroke. Both kinds SHALL support opacity, visibility, lock, grouping and all transforms.

#### Scenario: Text bounds follow content
- **WHEN** characters are added to a text
- **THEN** its width grows and the selection bounds follow

### Requirement: Project assets
A project SHALL hold an asset store: each asset has a stable identifier, a name, its kind (raster or SVG), its original file bytes and its pixel or declared size. Assets are shared by every image object referencing them and are deduplicated by content.

#### Scenario: Asset shared by two images
- **WHEN** an image is duplicated
- **THEN** both objects reference the same asset
