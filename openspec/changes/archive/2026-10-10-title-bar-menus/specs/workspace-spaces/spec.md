## MODIFIED Requirements

### Requirement: Three spaces
An open project SHALL be shown in exactly one of three spaces at a time:
- **Project:** the fleet and the mod information (see the project-screen capability);
- **Workshop:** painting the active texture, with the tool rail, the left panel, the canvas and the inspector (see the workspace-layout capability);
- **Brand:** the project's palette, graphic styles, text styles, symbols and images (see the brand-space capability).

The top bar SHALL show a space switcher Project / Workshop / Brand in that order, drawn as tabs (see the ui-design-system capability). Clicking one of its options SHALL show that space. View › Project (Cmd/Ctrl+1), View › Workshop (Cmd/Ctrl+2) and View › Brand (Cmd/Ctrl+3) SHALL do the same. The active option SHALL be identifiable without relying on color alone (its fill differs from the other options), and each option SHALL be reachable by keyboard navigation and announce its name and selected state to assistive technologies.

The menus, the top bar and the status bar SHALL stay in every space. Dialogs opened from one space (Export Mod…, Vehicle Library, Add Vehicle…, Preferences) SHALL work the same in all three.

#### Scenario: Switching with the keyboard
- **WHEN** a project is open in the Workshop and the user presses Cmd/Ctrl+3
- **THEN** the Brand space is shown and Brand is the active option of the switcher

#### Scenario: Switching with the switcher
- **WHEN** the Brand space is shown and the user clicks Workshop in the top bar
- **THEN** the Workshop is shown with the active texture on the canvas

#### Scenario: Spaces in the View menu
- **WHEN** the user opens the View menu
- **THEN** it lists Project, Workshop and Brand with Cmd/Ctrl+1, Cmd/Ctrl+2 and Cmd/Ctrl+3, the shown space checked

### Requirement: Top bar
Every space SHALL show a top bar, 40 px high, merged with the window's title bar (see the window-title-bar capability), with:
- on macOS and with the system title bar option, from left to right: the project name and a badge naming its game ("ETS2" or "ATS"), the space switcher, then the **Export…** button at the right end;
- on Windows and Linux with the drawn title bar: the space switcher centered and the **Export…** button, between the menus and the window controls; the project name is the window's title.

The breadcrumb is not in the top bar: it is inlaid in the canvas area (see Breadcrumb).

Its texts SHALL be free width: no label is cut in any of the four languages; when the window is too narrow, the project name is shortened with an ellipsis first and shown in full on hover; with the drawn title bar, the switcher moves off center before anything is shortened.

#### Scenario: Name and game
- **WHEN** a project named "ACE Logistics" made for ETS2 vehicles is open
- **THEN** on macOS the top bar shows "ACE Logistics" with the badge "ETS2", and on every platform the window's title reads "ACE Logistics — TruckPaint"

#### Scenario: Long name in a narrow window
- **WHEN** the project name does not fit the top bar
- **THEN** it is shortened with an ellipsis, the switcher and Export… stay whole, and hovering the name shows it in full

#### Scenario: German at the minimum width
- **WHEN** the window is 960 px wide on Windows with the language German and a project open
- **THEN** the title bar shows every menu, the switcher, "Exportieren…" and the window controls whole, without overlap

### Requirement: Export button
The top bar SHALL always show an **Export…** button, styled as the primary action, in every space. It SHALL run the Export Mod… command (see the mod-export capability), which has the shortcut Cmd/Ctrl+E. It SHALL be enabled and disabled as File › Export Mod… is, and when disabled it SHALL say why on hover.

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
The Workshop SHALL tell which texture is being painted with a breadcrumb `<vehicle> › Main textures|Accessories › <texture>`, followed by the texture's size (`1024²`), grouped as in the Textures tab. It SHALL be inlaid in a corner of the canvas area, where it stays visible whatever left tab is shown and when the panels are hidden. It SHALL follow the active texture as soon as it changes. The inlaid breadcrumb SHALL NOT take pointer input from the canvas.

While a symbol is being edited, the breadcrumb SHALL read `Symbol › <name>` instead.

When the breadcrumb does not fit, it SHALL be shortened with an ellipsis and shown in full on hover.

#### Scenario: Accessory texture
- **WHEN** the sample truck's Side skirts (1024 px) is the active texture
- **THEN** the canvas shows "TruckPaint Sample Truck › Accessories › Side skirts 1024²"

#### Scenario: Following the active texture
- **WHEN** the user presses Cmd/Ctrl+] and the next texture is the sample truck's Chassis (4096 px)
- **THEN** the breadcrumb reads "TruckPaint Sample Truck › Accessories › Chassis 4096²"

#### Scenario: Main texture
- **WHEN** the sample truck's High roof is active
- **THEN** the breadcrumb reads "TruckPaint Sample Truck › Main textures › High roof 4096²"

#### Scenario: Editing a symbol
- **WHEN** the user edits the symbol "Logo"
- **THEN** the breadcrumb reads "Symbol › Logo"
