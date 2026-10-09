## MODIFIED Requirements

### Requirement: View commands
The application SHALL provide Zoom In (Cmd/Ctrl+=), Zoom Out (Cmd/Ctrl+-), Fit to Screen (Shift+Cmd/Ctrl+0) and Actual Size 100% (Cmd/Ctrl+0). Zoom In/Out SHALL step through preset levels (for example 25%, 33%, 50%, 67%, 100%, 150%, 200%, …) centered on the canvas area. Fit to Screen SHALL show the whole artboard centered with a margin. These shortcuts SHALL zoom the canvas, not the application interface. Cmd/Ctrl+1 SHALL NOT change the zoom: it shows the Project space.

#### Scenario: Fit to screen
- **WHEN** the user is zoomed in on a corner and presses Shift+Cmd/Ctrl+0
- **THEN** the whole artboard is visible and centered

#### Scenario: Actual size
- **WHEN** the user presses Cmd/Ctrl+0
- **THEN** the zoom becomes 100% centered on the current view center

#### Scenario: Cmd/Ctrl+1 no longer zooms
- **WHEN** the Workshop is shown at 50% and the user presses Cmd/Ctrl+1
- **THEN** the Project space is shown, and the zoom is still 50% when the user returns to the Workshop

#### Scenario: Interface does not zoom
- **WHEN** the user presses Cmd/Ctrl+=
- **THEN** the canvas zooms in and menus, panels and text keep their size
