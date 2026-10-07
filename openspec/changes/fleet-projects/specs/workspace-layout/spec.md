## ADDED Requirements

### Requirement: Vehicles sidebar
The workspace SHALL show a **Vehicles** sidebar on the left, between the tool bar and the canvas area. It holds the fleet's navigation (see the vehicle-projects capability) and is open by default.

- Its header has a Hide Vehicles button that reduces it to a narrow strip. The strip's Show Vehicles button opens it again.
- View › Vehicles (F5) SHALL toggle it, and Vehicle › Vehicle Information SHALL open it.
- Its width SHALL be resizable by dragging its edge within sensible bounds.
- Whether it is open, and its width, SHALL be remembered across sessions. Reset Workspace SHALL open it at its default width.

#### Scenario: Hiding the sidebar
- **WHEN** the user clicks Hide Vehicles
- **THEN** the sidebar becomes a narrow strip with a Show Vehicles button, the canvas area widens, and the sidebar stays hidden after a restart

#### Scenario: Toggling with the keyboard
- **WHEN** the sidebar is hidden and the user presses F5
- **THEN** the sidebar is shown again

## MODIFIED Requirements

### Requirement: View modes
The workspace SHALL offer three view modes — 2D, 3D and Split — selectable from a segmented control and from the View menu with shortcuts. 2D shows only the canvas area, 3D shows only the 3D preview area, Split shows the canvas area and the 3D preview side by side with a draggable divider.

Opening or creating a project SHALL show its canvas: when the remembered view mode is 3D, the project opens in 2D. 2D and Split are kept.

#### Scenario: Switching to split view
- **WHEN** the user selects Split
- **THEN** the canvas area is shown on the left and the 3D preview on the right, separated by a draggable divider

#### Scenario: Opening a project after using the 3D view
- **WHEN** the user last used the 3D view and opens a project
- **THEN** the project opens in 2D with its canvas visible

### Requirement: Right panel stack
The workspace SHALL show a right-hand panel column containing the panels Properties, Layers, Colors, Stroke, Transform and Assets. Each panel SHALL have a header with its title, can be collapsed/expanded by clicking its header, and can be closed. Closed panels SHALL be reopenable from the View menu. The column width SHALL be resizable by dragging its edge within sensible bounds. Panels whose feature is not yet available SHALL show an explicit empty state.

#### Scenario: Collapsing a panel
- **WHEN** the user clicks the header of the Layers panel
- **THEN** the panel body collapses, the header remains visible with a collapsed indicator, and other panels reflow

#### Scenario: Closing and reopening a panel
- **WHEN** the user closes the Colors panel and then chooses View > Colors
- **THEN** the Colors panel is shown again at its previous position in the stack

#### Scenario: Resizing the panel column
- **WHEN** the user drags the left edge of the panel column
- **THEN** the column width changes live and is clamped between a minimum and a maximum width
