## MODIFIED Requirements

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

### Requirement: Tool feedback
The canvas cursor SHALL show a crosshair while a shape tool is active. While a tool whose feature is not available yet is active (Polygon, Pen, Line, Text, Image, Direct Selection), the canvas SHALL show a short, non-blocking hint that the tool is not available yet and SHALL NOT modify the document.

#### Scenario: Unavailable tool
- **WHEN** the Pen tool is active and the user clicks on the canvas
- **THEN** a hint states that the Pen tool is not available yet and the document is unchanged
