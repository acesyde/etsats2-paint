## MODIFIED Requirements

### Requirement: Tool feedback
The canvas cursor SHALL show a crosshair while a shape tool is active and a text cursor (I-beam) while the Text tool is active. While a tool whose feature is not available yet is active (Polygon, Pen, Line, Direct Selection), the canvas SHALL show a short, non-blocking hint that the tool is not available yet and SHALL NOT modify the document.

#### Scenario: Unavailable tool
- **WHEN** the Pen tool is active and the user clicks on the canvas
- **THEN** a hint states that the Pen tool is not available yet and the document is unchanged
