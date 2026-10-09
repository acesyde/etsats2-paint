## Purpose

Organises an open project into three spaces, Project, Workshop and Brand, that follow how a livery is made, with a top bar to switch between them, an always-visible Export… action, and a breadcrumb that tells which texture is being painted.

## ADDED Requirements

### Requirement: Three spaces
An open project SHALL be shown in exactly one of three spaces at a time:
- **Project:** the fleet and the mod information (see the project-screen capability);
- **Workshop:** painting the active texture, with the tool rail, the left panel, the canvas and the inspector (see the workspace-layout capability);
- **Brand:** the project's palette, graphic styles, text styles, symbols and images (see the brand-space capability).

The top bar SHALL show a space switcher Project / Workshop / Brand in that order, drawn as tabs (see the ui-design-system capability). Clicking one of its options SHALL show that space. View › Project (Cmd/Ctrl+1), View › Workshop (Cmd/Ctrl+2) and View › Brand (Cmd/Ctrl+3) SHALL do the same. The active option SHALL be identifiable without relying on color alone (its fill differs from the other options), and each option SHALL be reachable by keyboard navigation and announce its name and selected state to assistive technologies.

The menu bar, the top bar and the status bar SHALL stay in every space. Dialogs opened from one space (Export Mod…, Vehicle Library, Add Vehicle…, Preferences) SHALL work the same in all three.

#### Scenario: Switching with the keyboard
- **WHEN** a project is open in the Workshop and the user presses Cmd/Ctrl+3
- **THEN** the Brand space is shown and Brand is the active option of the switcher

#### Scenario: Switching with the switcher
- **WHEN** the Brand space is shown and the user clicks Workshop in the top bar
- **THEN** the Workshop is shown with the active texture on the canvas

#### Scenario: Spaces in the View menu
- **WHEN** the user opens the View menu
- **THEN** it lists Project, Workshop and Brand with Cmd/Ctrl+1, Cmd/Ctrl+2 and Cmd/Ctrl+3, the shown space checked

### Requirement: Space shown when a project opens
Opening a project file, creating a project through New Project and restoring a recovered project SHALL show the **Project** space. The space SHALL NOT be saved in the project file nor remembered per project.

#### Scenario: Opening a file
- **WHEN** the user opens a project file that was last edited in the Workshop
- **THEN** the project is shown in the Project space

#### Scenario: New project
- **WHEN** the user creates a project from the sample truck
- **THEN** the project is shown in the Project space, with "Standard cab" as the active texture

#### Scenario: Recovered project
- **WHEN** the user restores a recovered project from the home screen
- **THEN** it is shown in the Project space, with "Unsaved changes" in the status bar

### Requirement: Opening a texture from the Project space
Clicking a texture in the Project space SHALL make that texture's surface active and show the Workshop. Choosing a texture in the Workshop's Textures tab SHALL make it active without leaving the Workshop.

#### Scenario: Clicking a texture in Project
- **WHEN** the Project space is shown and the user clicks the trailer's "Mudflaps"
- **THEN** the Workshop is shown with the Mudflaps surface active on the canvas and highlighted in the Textures tab

### Requirement: Switching spaces keeps the work
Switching spaces SHALL NOT change the document, the selection, the undo history, the save state, the active texture nor the view (zoom and scroll) of any texture. Undo and Redo SHALL work in every space; undoing a change made on a texture SHALL make that texture active without changing the space shown.

#### Scenario: Selection and zoom kept
- **WHEN** in the Workshop a rectangle is selected at 200% zoom, and the user presses Cmd/Ctrl+1 then Cmd/Ctrl+2
- **THEN** the Workshop shows the same texture at 200% at the same place, with the rectangle still selected

#### Scenario: Undo from the Brand space
- **WHEN** the user draws a rectangle in the Workshop, shows the Brand space and presses Cmd/Ctrl+Z
- **THEN** the rectangle is removed, and the Brand space stays shown

### Requirement: Top bar
Under the menu bar, every space SHALL show a top bar, 44 px high, with from left to right:
- the project name and a badge naming its game ("ETS2" or "ATS");
- the space switcher;
- in the Workshop, the breadcrumb of the active texture;
- the **Export…** button, at the right end.

Its texts SHALL be free width: no label is cut in any of the four languages; when the window is too narrow, the project name is shortened with an ellipsis first and shown in full on hover.

#### Scenario: Name and game
- **WHEN** a project named "ACE Logistics" made for ETS2 vehicles is open
- **THEN** the top bar shows "ACE Logistics" with the badge "ETS2"

#### Scenario: Long name in a narrow window
- **WHEN** the project name does not fit the top bar
- **THEN** it is shortened with an ellipsis, the switcher and Export… stay whole, and hovering the name shows it in full

### Requirement: Export button
The top bar SHALL always show an **Export…** button, styled as the primary action, in every space. It SHALL run the Export Mod… command (see the mod-export capability), which has the shortcut Cmd/Ctrl+E. It SHALL be enabled and disabled as Export › Export Mod… is, and when disabled it SHALL say why on hover.

#### Scenario: Export from the Project space
- **WHEN** the Project space is shown and the user clicks Export…
- **THEN** the Export Mod dialog opens

#### Scenario: Export with the keyboard
- **WHEN** the Workshop is shown and the user presses Cmd/Ctrl+E
- **THEN** the Export Mod dialog opens

#### Scenario: Disabled while editing a symbol
- **WHEN** a symbol is being edited
- **THEN** Export… is disabled and its tooltip says why

### Requirement: Breadcrumb
The Workshop SHALL tell which texture is being painted with a breadcrumb `<vehicle> › Main textures|Accessories › <texture>`, followed by the texture's size (`1024²`), grouped as in the Textures tab. It SHALL be shown in the top bar and inlaid in a corner of the canvas area, where it stays visible whatever left tab is shown and when the panels are hidden. It SHALL follow the active texture as soon as it changes. The inlaid breadcrumb SHALL NOT take pointer input from the canvas.

While a symbol is being edited, the breadcrumb SHALL read `Symbol › <name>` instead.

When the breadcrumb does not fit, it SHALL be shortened with an ellipsis and shown in full on hover.

#### Scenario: Accessory texture
- **WHEN** the sample truck's Side skirts (1024 px) is the active texture
- **THEN** the top bar and the canvas show "TruckPaint Sample Truck › Accessories › Side skirts 1024²"

#### Scenario: Following the active texture
- **WHEN** the user presses Cmd/Ctrl+] and the next texture is the sample truck's Chassis (4096 px)
- **THEN** the breadcrumb reads "TruckPaint Sample Truck › Accessories › Chassis 4096²"

#### Scenario: Main texture
- **WHEN** the sample truck's High roof is active
- **THEN** the breadcrumb reads "TruckPaint Sample Truck › Main textures › High roof 4096²"

#### Scenario: Editing a symbol
- **WHEN** the user edits the symbol "Logo"
- **THEN** the breadcrumb reads "Symbol › Logo"
