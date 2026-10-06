# shape-tools Specification

## Purpose

Defines how rectangles and ellipses are drawn on the canvas with the Rectangle and Ellipse tools.

## Requirements

### Requirement: Drag to create
With the Rectangle or Ellipse tool active, pressing on the canvas and dragging SHALL show a live preview of the shape between the press point and the pointer; releasing SHALL create the shape on the active surface, select it, and keep the tool active. The preview and the created shape SHALL use the current fill and stroke. The shape SHALL be inserted at the top of the active layer: the group that is selected, or the group containing the selected objects when they share one, otherwise the top level. A drag shorter than 2 screen pixels SHALL NOT create a shape.

#### Scenario: Drawing a rectangle
- **WHEN** the Rectangle tool is active and the user drags from (100, 100) to (500, 300) in texture pixels
- **THEN** a rectangle with center (300, 200), width 400 and height 200 is created, selected, and the Rectangle tool stays active

#### Scenario: Click without drag
- **WHEN** the Ellipse tool is active and the user clicks without moving
- **THEN** no shape is created

#### Scenario: Drawing into the active layer
- **WHEN** the group "Branding" is selected and the user draws an ellipse
- **THEN** the ellipse becomes the topmost child of "Branding"

### Requirement: Modifier constraints
While drawing, holding Shift SHALL constrain the shape to a square or circle, and holding Alt/Option SHALL draw from the press point as the center. Both modifiers SHALL combine and SHALL apply live as they are pressed or released during the drag.

#### Scenario: Shift draws a circle
- **WHEN** the Ellipse tool is active and the user drags while holding Shift
- **THEN** the created ellipse has equal width and height

#### Scenario: Alt draws from the center
- **WHEN** the Rectangle tool is active and the user drags from (500, 500) to (600, 550) while holding Alt/Option
- **THEN** the rectangle is centered at (500, 500) with width 200 and height 100

### Requirement: Cancel drawing
Pressing Escape during a drag SHALL cancel the shape being drawn without modifying the document.

#### Scenario: Escape cancels
- **WHEN** the user is dragging a new rectangle and presses Escape
- **THEN** the preview disappears and no rectangle is created

### Requirement: Tool cursors
The canvas cursor SHALL show:

- a crosshair while the Rectangle, Ellipse, Polygon or Line tool is active;
- a pen cursor while the Pen tool is active;
- the default arrow while the Direct Selection tool is active;
- a text cursor (I-beam) while the Text tool is active.

Every tool of the tool bar SHALL be functional: no tool SHALL show a "not available yet" hint.

#### Scenario: Pen tool is available
- **WHEN** the Pen tool is active and the user clicks on the canvas
- **THEN** an anchor point is placed and no "not available yet" hint is shown
