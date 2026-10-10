## MODIFIED Requirements

### Requirement: Menu bar
The application SHALL offer the menus File, Edit, Object, Layer, View, Vehicle and Help, in that order, on the home screen and in every space. Where they are shown depends on the platform (see the window-title-bar capability):
- **macOS:** in the system menu bar at the top of the screen, after the application menu **TruckPaint**, and followed by the system's **Window** menu; nothing is drawn in the window;
- **Windows and Linux:** in the window's title bar, or in a row inside the window with the system title bar option.

Each menu item SHALL display its shortcut (when it has one) in the platform's notation, be enabled and disabled as its command is, and show its check mark when it is a toggle, wherever the menu is shown. Items whose feature is not yet available SHALL be shown disabled. Changing the language SHALL relabel the menus at once, including the macOS menu bar.

The File menu SHALL end with **Export Texture…** and **Export Mod…** before Quit: there is no Export menu. Shortcuts don't change.

On macOS:
- the application menu SHALL hold **About TruckPaint**, **Settings…** (`Cmd+,`, the Preferences command), the system's Services, Hide TruckPaint, Hide Others and Show All, and **Quit TruckPaint** (`Cmd+Q`); About and Preferences SHALL NOT be repeated in Help and Edit, nor Quit in File;
- Quit SHALL ask to save unsaved changes as closing the window does;
- a shortcut shown in the menu SHALL run its command once, whether the key or the menu item is used;
- while a text is being typed in a field or on the canvas, Edit › Undo, Redo, Cut, Copy, Paste and Select All and their shortcuts SHALL act on that text, as they do without the system menu.

The View menu SHALL list, in groups: Command Palette… (see the command-palette capability); the spaces Project, Workshop and Brand; the left tabs Textures, Layers and Resources; Hide Panels; then Show Template, Show Grid, Show Guides, Clear Guides and Snapping; then Zoom In, Zoom Out, Fit to Screen and Actual Size; then Reset Workspace. Toggles SHALL show their state with a check mark. The Vehicle menu SHALL NOT have a Vehicle Information item, and the View menu SHALL NOT have Sidebar nor panel items: the spaces and the left tabs replace them.

The menus' structure (their items, submenus and groups) SHALL be the one the command palette uses to show each command's menu path, so that a command's path in the palette always names the menu, and submenu, where the command is.

#### Scenario: Menus present
- **WHEN** a project is open
- **THEN** the menus are File, Edit, Object, Layer, View, Vehicle, Help in that order, with no Export menu

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

#### Scenario: Export items in File
- **WHEN** a project is open and the user opens the File menu
- **THEN** it lists Export Texture… with Shift+Cmd/Ctrl+E and Export Mod… with Cmd/Ctrl+E, before Quit

#### Scenario: System menu bar on macOS
- **WHEN** a project is open on macOS
- **THEN** the system menu bar shows TruckPaint, File, Edit, Object, Layer, View, Vehicle, Window and Help, and the window draws no menu row

#### Scenario: Shortcut runs once on macOS
- **WHEN** a rectangle is selected on macOS and the user presses Cmd+D
- **THEN** exactly one duplicate is made

#### Scenario: Disabled state follows the command
- **WHEN** nothing is selected on macOS
- **THEN** Edit › Duplicate is disabled in the system menu bar, and pressing Cmd+D does nothing

#### Scenario: Copy in a text field on macOS
- **WHEN** the user selects the text of the project name field in the Project space and presses Cmd+C, then Cmd+V in another field
- **THEN** the name is pasted in the other field, and no object of the canvas is copied

#### Scenario: Quit with unsaved changes on macOS
- **WHEN** a project has unsaved changes and the user chooses TruckPaint › Quit TruckPaint
- **THEN** the save prompt is shown, and Cancel keeps the application open

#### Scenario: Language switch relabels the system menu
- **WHEN** the user picks "Deutsch" in Preferences on macOS
- **THEN** the system menu bar reads Datei, Bearbeiten, Objekt, Ebene, Ansicht, Fahrzeug and Hilfe at once

### Requirement: Workspace frame
The window of an open project SHALL be arranged, from top to bottom, as: the window's title bar holding the top bar (see the window-title-bar and workspace-spaces capabilities; with the system title bar option, the menu row and then the top bar), in the Workshop the tool options bar, then the space's content, then the status bar.

In the Workshop, the content SHALL be, from left to right: the tool rail, the left panel, the canvas area and the inspector. The Project and Brand spaces SHALL use the full width between the top bar and the status bar, without tool rail, tool options bar, left panel nor inspector.

View › Reset Workspace SHALL show the left panel and the inspector at their default widths, with the Textures tab.

#### Scenario: Workshop frame
- **WHEN** the Workshop is shown
- **THEN** the tool rail, the left panel, the canvas area and the inspector are shown from left to right, between the tool options bar and the status bar

#### Scenario: Project frame
- **WHEN** the Project space is shown
- **THEN** no tool rail, tool options bar, left panel nor inspector is shown, and the status bar shows only the save state

#### Scenario: Reset Workspace
- **WHEN** the panels are hidden, the Layers tab is shown and the inspector was widened, and the user chooses View › Reset Workspace
- **THEN** the left panel and the inspector are shown at their default widths and the Textures tab is shown
