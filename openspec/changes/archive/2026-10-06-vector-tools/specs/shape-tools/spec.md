## ADDED Requirements

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

## REMOVED Requirements

### Requirement: Tool feedback
**Reason**: Polygon, Pen, Line and Direct Selection are implemented, so no tool is unavailable any more; the cursor part moves to "Tool cursors".
**Migration**: See "Tool cursors" in this spec and the `path-tools` and `path-editing` capabilities.
