## MODIFIED Requirements

### Requirement: Tool bar
The workspace SHALL show a vertical tool bar on the left with, in order: Selection, Direct Selection, Move, Rectangle, Ellipse, Polygon, Pen, Line, Text, Image, Eyedropper, Gradient, Zoom, Hand. Exactly one tool SHALL be active at a time and the active tool SHALL be identifiable without relying on color alone. Clicking a tool SHALL make it active.

#### Scenario: Default tool
- **WHEN** a project is opened
- **THEN** the Selection tool is active

#### Scenario: Switching tools
- **WHEN** the user clicks the Rectangle tool
- **THEN** the Rectangle tool shows the active state and the previously active tool no longer does
