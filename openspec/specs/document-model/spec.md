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
Each surface SHALL hold an ordered list of objects; later objects are drawn above earlier ones. Every object SHALL have a unique, stable identifier that does not change when the object is moved, transformed or reordered.

#### Scenario: Stacking order
- **WHEN** a rectangle is created and then an ellipse overlapping it
- **THEN** the ellipse is drawn above the rectangle

#### Scenario: Identity is stable
- **WHEN** an object is moved, resized, rotated and brought forward
- **THEN** its identifier is unchanged

### Requirement: Object geometry and appearance
Rectangles and ellipses SHALL be defined by a frame (center point, width, height, rotation in degrees clockwise around the center), and SHALL remain vector objects: their geometry is never rasterized in the document. Rectangles SHALL support a corner radius (default 0). Every object SHALL have a solid fill color with alpha, an optional stroke (color and width in texture pixels), and an opacity from 0% to 100%. Width and height SHALL never be smaller than 1 texture pixel.

#### Scenario: Default appearance of a new shape
- **WHEN** a rectangle is created with a tool
- **THEN** it has the default fill color, no stroke, 100% opacity and a corner radius of 0

#### Scenario: Minimum size
- **WHEN** an object is resized so that its width would become 0 or negative
- **THEN** its width is clamped to at least 1 texture pixel

### Requirement: Objects may extend beyond the surface
Objects SHALL be allowed to lie partly or fully outside the surface bounds; they are kept in the document and remain selectable, and only the part inside the surface belongs to the texture.

#### Scenario: Object dragged off the artboard
- **WHEN** an object is moved so it lies entirely outside the artboard
- **THEN** it still exists, is drawn on the pasteboard and can be selected and moved back

### Requirement: Hit testing
The document SHALL determine which object is under a given point by testing the actual filled shape (including rotation and rounded corners), from topmost to bottommost. For objects whose drawn size on screen is very small, a tolerance of a few screen pixels SHALL apply so they remain clickable.

#### Scenario: Clicking the corner of an ellipse's bounding box
- **WHEN** the user clicks inside an ellipse's bounding box but outside the ellipse itself
- **THEN** the ellipse is not hit and the object below it (or nothing) is hit instead

#### Scenario: Topmost wins
- **WHEN** two objects overlap at the clicked point
- **THEN** the topmost of the two is hit
