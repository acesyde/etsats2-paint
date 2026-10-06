## MODIFIED Requirements

### Requirement: Canvas area
The canvas area SHALL occupy all space not used by bars and panels and SHALL display the project's active texture surface as an artboard framed against a neutral pasteboard background. When a project is opened, the view SHALL be fitted so the whole artboard is centered and visible; afterwards the view is controlled by the user (see `canvas-navigation`). Resizing the canvas area SHALL keep the document point at the center of the canvas area fixed and SHALL NOT change the zoom level, unless the user has not navigated since the last fit, in which case the view is refitted.

#### Scenario: Artboard visible
- **WHEN** a new 4096×4096 project is opened
- **THEN** a square artboard is displayed centered in the canvas area and entirely visible

#### Scenario: Window resize
- **WHEN** the window is resized and the user has not zoomed or panned since the project was opened or last fitted
- **THEN** the view is refitted so the artboard remains centered and fully visible

#### Scenario: Window resize after navigating
- **WHEN** the user has zoomed to 200% and then resizes the window
- **THEN** the zoom stays at 200% and the document point that was at the center of the canvas area stays at its center
