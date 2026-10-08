## MODIFIED Requirements

### Requirement: Persisted preferences
The application SHALL persist, in the operating system's standard per-user configuration directory, at least: UI scale, text size, panel visibility / collapsed state / order, panel column width, window size/position/maximized state, the recent projects list, the interface language when the user has chosen one, whether the grid and guides are shown, whether snapping is on, and the grid spacing.

A preferences file written by a build with the 3D preview, which also stores a view mode and the width of the 3D panel, SHALL load with all its other preferences. The view mode and the panel width are ignored.

#### Scenario: Preferences survive restart
- **WHEN** the user sets the UI scale to 125%, closes the Assets panel, quits and restarts the application
- **THEN** the UI scale is 125% and the Assets panel is closed

#### Scenario: Grid and snapping settings survive restart
- **WHEN** the user shows the grid, sets the grid spacing to 128, turns snapping off, quits and restarts
- **THEN** the grid is shown with a 128 px spacing and snapping is off

#### Scenario: Language survives restart
- **WHEN** the user picks "Deutsch" in Preferences, quits and restarts the application
- **THEN** the interface is in German, whatever the system language

#### Scenario: Preferences from a build with the 3D preview
- **WHEN** the application starts with a preferences file that stores the UI scale 125%, the 3D view mode and a 3D panel width
- **THEN** the UI scale is 125%, the canvas is shown, no backup of the file is made, and no message about the preferences appears
