## MODIFIED Requirements

### Requirement: Default tool shortcuts
The application SHALL provide default single-key shortcuts for tools: Selection `V`, Direct Selection `A`, Move `M`, Rectangle `R`, Ellipse `E`, Polygon `Y`, Pen `P`, Line `\`, Text `T`, Image `Shift+I`, Eyedropper `I`, Gradient `G`, Zoom `Z`, Hand `H`. Holding Space SHALL temporarily activate the Hand tool and releasing it SHALL restore the previous tool.

#### Scenario: Selecting a tool by key
- **WHEN** the canvas area has focus and the user presses `E`
- **THEN** the Ellipse tool becomes the active tool

#### Scenario: Temporary hand tool
- **WHEN** the Rectangle tool is active and the user holds Space then releases it
- **THEN** the Hand tool is active while Space is held and the Rectangle tool is active again after release

#### Scenario: Gradient tool by key
- **WHEN** the canvas area has focus and the user presses `G`
- **THEN** the Gradient tool becomes the active tool
