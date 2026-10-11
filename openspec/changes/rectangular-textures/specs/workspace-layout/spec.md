## MODIFIED Requirements

### Requirement: Canvas area
The canvas area SHALL occupy, in the Workshop, all space not used by the bars, the tool rail, the left panel and the inspector, SHALL show rulers along its top and left edges (see `canvas-guides`), and SHALL display the project's active texture surface as an artboard framed against a neutral pasteboard background, with the breadcrumb inlaid (see the workspace-spaces capability). When a project is opened, the view SHALL be fitted so the whole artboard is centered and visible; afterwards the view is controlled by the user (see `canvas-navigation`). Resizing the canvas area, including by resizing, hiding or showing the left panel or the inspector, SHALL keep the document point at the center of the canvas area fixed and SHALL NOT change the zoom level, unless the user has not navigated since the last fit, in which case the view is refitted.

#### Scenario: Artboard visible
- **WHEN** a new 4096×4096 project is opened and the Workshop is shown
- **THEN** a square artboard is displayed centered in the canvas area and entirely visible

#### Scenario: Window resize
- **WHEN** the window is resized and the user has not zoomed or panned since the project was opened or last fitted
- **THEN** the view is refitted so the artboard remains centered and fully visible

#### Scenario: Window resize after navigating
- **WHEN** the user has zoomed to 200% and then resizes the window
- **THEN** the zoom stays at 200% and the document point that was at the center of the canvas area stays at its center

#### Scenario: Hiding the panels after navigating
- **WHEN** the user has zoomed to 200% and then presses Tab to hide the panels
- **THEN** the canvas area widens, the zoom stays at 200% and the document point that was at its center stays at its center

#### Scenario: Rectangular artboard
- **WHEN** a project whose active texture is 8192 × 1024 is opened in the Workshop
- **THEN** the artboard is eight times wider than tall, centered and entirely visible, and the rulers run from 0 to 8192 across and from 0 to 1024 down
