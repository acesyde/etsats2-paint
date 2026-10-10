## MODIFIED Requirements

### Requirement: Menu bar
The workspace SHALL show a menu bar with the menus File, Edit, Object, Layer, View, Vehicle, Export and Help, in every space. Each menu item SHALL display its shortcut (when it has one) right-aligned in the platform's notation. Items whose feature is not yet available SHALL be shown disabled.

The View menu SHALL list, in groups: Command Palette… (see the command-palette capability); the spaces Project, Workshop and Brand; the left tabs Textures, Layers and Resources; Hide Panels; then Show Template, Show Grid, Show Guides, Clear Guides and Snapping; then Zoom In, Zoom Out, Fit to Screen and Actual Size; then Reset Workspace. Toggles SHALL show their state with a check mark. The Vehicle menu SHALL NOT have a Vehicle Information item, and the View menu SHALL NOT have Sidebar nor panel items: the spaces and the left tabs replace them.

The menus' structure (their items, submenus and groups) SHALL be the one the command palette uses to show each command's menu path, so that a command's path in the palette always names the menu, and submenu, where the command is.

#### Scenario: Menus present
- **WHEN** a project is open
- **THEN** the menu bar shows exactly File, Edit, Object, Layer, View, Vehicle, Export, Help in that order

#### Scenario: Closing a project from the File menu
- **WHEN** the user chooses File > Close
- **THEN** the project closes and the home screen is shown

#### Scenario: View menu
- **WHEN** the user opens the View menu in the Workshop with the Layers tab shown
- **THEN** it lists Command Palette… with its shortcut, then Project, Workshop (checked) and Brand, then Textures, Layers (checked) and Resources, then Hide Panels, then the template, grid, guides and snapping items, then the zoom commands, then Reset Workspace

#### Scenario: Command Palette from the View menu on the home screen
- **WHEN** no project is open and the user chooses View › Command Palette…
- **THEN** the command palette opens

#### Scenario: Palette path follows the menu
- **WHEN** the user types "align left" in the command palette
- **THEN** the Align Left row's path reads "Object › Align", the menu and submenu where Align Left is

#### Scenario: No sidebar item
- **WHEN** the user opens the Vehicle menu
- **THEN** it has no Vehicle Information item, and pressing F5 does nothing
