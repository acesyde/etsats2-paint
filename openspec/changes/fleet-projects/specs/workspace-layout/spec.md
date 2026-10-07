## MODIFIED Requirements

### Requirement: Right panel stack
The workspace SHALL show a right-hand panel column containing the panels Properties, Layers, Colors, Stroke, Transform, Assets and Vehicles. Each panel SHALL have a header with its title, can be collapsed/expanded by clicking its header, and can be closed. Closed panels SHALL be reopenable from the View menu. The column width SHALL be resizable by dragging its edge within sensible bounds. Panels whose feature is not yet available SHALL show an explicit empty state.

#### Scenario: Collapsing a panel
- **WHEN** the user clicks the header of the Layers panel
- **THEN** the panel body collapses, the header remains visible with a collapsed indicator, and other panels reflow

#### Scenario: Closing and reopening a panel
- **WHEN** the user closes the Colors panel and then chooses View > Colors
- **THEN** the Colors panel is shown again at its previous position in the stack

#### Scenario: Resizing the panel column
- **WHEN** the user drags the left edge of the panel column
- **THEN** the column width changes live and is clamped between a minimum and a maximum width
