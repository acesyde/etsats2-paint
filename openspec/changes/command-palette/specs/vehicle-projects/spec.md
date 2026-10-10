## MODIFIED Requirements

### Requirement: Switching textures
The user SHALL switch textures from the Workshop's Textures tab, from the textures of the Project space (see the project-screen capability), from the command palette (see the command-palette capability), and with the keyboard: clicking a texture makes its surface active. Choosing a texture in the command palette SHALL make its surface active and show the Workshop, from any space. There are no texture tabs above the canvas. Vehicle › Next Texture (Cmd/Ctrl+], also Cmd/Ctrl+Page Down) and Vehicle › Previous Texture (Cmd/Ctrl+[, also Cmd/Ctrl+Page Up) SHALL make the following or preceding surface active, in project order and wrapping around, without changing the space shown; they are disabled for a project with one texture.

When the active surface changes:
- the canvas, rulers, Layers tab, inspector and breadcrumb SHALL show that surface. The breadcrumb names it as "<vehicle> › Main textures|Accessories › <texture>" (see the workspace-spaces capability);
- the selection SHALL be cleared;
- each surface SHALL keep its own view (zoom and scroll) for as long as the project's list of surfaces doesn't change. Adding or removing vehicles or textures fits the other surfaces to the window again.

Undo and redo SHALL apply to the whole project. Undoing a change made on another surface SHALL make that surface active.

#### Scenario: Paint on the chassis
- **WHEN** the user clicks "Chassis" in the Textures tab and draws a rectangle
- **THEN** the rectangle is on the Chassis surface and the Standard cab surface is unchanged

#### Scenario: Undo across textures
- **WHEN** the user draws on Standard cab, switches to Chassis, and presses Undo
- **THEN** Standard cab becomes active and the rectangle drawn on it is removed

#### Scenario: Next texture from the keyboard
- **WHEN** the active texture is the sample truck's High roof, followed by its Chassis, and the user presses Cmd/Ctrl+]
- **THEN** Chassis becomes active, and the breadcrumb shows "TruckPaint Sample Truck › Accessories › Chassis 4096²"

#### Scenario: Page keys kept
- **WHEN** the active texture is the sample truck's Chassis and the user presses Cmd/Ctrl+Page Up
- **THEN** High roof becomes active

#### Scenario: Switching from the palette
- **WHEN** a rectangle is selected on Standard cab at 200% zoom, the user opens the command palette, chooses Chassis, then chooses Standard cab the same way
- **THEN** Chassis becomes active with nothing selected, and Standard cab is shown again at 200% at the same place

### Requirement: Textures tab
The Workshop's **Textures** tab (see the workspace-layout capability) SHALL show the fleet as a tree, with an **Add Vehicle** (+) button in its header that runs Vehicle › Add Vehicle…, and above the tree a **Search textures** field showing the Command Palette shortcut at its right end in the platform's notation. Clicking the field, or giving it the keyboard focus and pressing Enter or typing, SHALL open the command palette limited to textures (see the command-palette capability), with what was typed in its field. The field itself SHALL never hold text nor filter the tree.

The tree:
- **each vehicle** is a group titled with its name, with its kind and package version below (the supported game versions on hover) and a **⋯** actions menu: **Textures…**, **Update Template…** (when a newer version is installed) and **Remove from Project** (disabled for the last vehicle). When a newer version is installed, an update button showing that version SHALL also appear next to the menu;
- **under each vehicle**, its textures in project order:
  - a **Main textures** heading over its chosen main textures (a truck's cabins, a trailer's base);
  - an **Accessories** heading over its chosen accessories, when it has any;
- **each texture** is one row with a thumbnail of its artwork, its name, its size and, at its right end, a marker of its state (see the texture-status capability): a hollow ring for Empty, a filled dot for Modified, and a warning icon in the signal color for To check, so that the states differ by shape and not only by color. Hovering the marker SHALL show the state's label, and for To check why ("Layout changed in <version>" or "Not in this version"); the row's name for assistive technologies SHALL end with the state. The thumbnail SHALL show the texture's current artwork without the template, and follow its edits. The active texture is highlighted without relying on color alone, and clicking a texture makes it active.

Vehicle groups can be collapsed; the active vehicle stays open.

The Project space SHALL offer the same vehicle actions (Add Vehicle…, Textures…, Update Template…, Remove from Project) on its vehicle cards, and the same texture switching (see the project-screen capability).

#### Scenario: Newer version available
- **WHEN** a project made with version 1.2.0 of a vehicle is open and version 1.3.0 of that vehicle is installed
- **THEN** that vehicle's group in the Textures tab shows an update button for 1.3.0, which opens Update Template

#### Scenario: Truck and trailer in the tree
- **WHEN** a project holds the sample truck with both cabins and the sample trailer, each with its accessories
- **THEN** the truck's group lists "Standard cab" and "High roof" under Main textures, then its accessories under Accessories, and the trailer's group lists "Base" under Main textures, then its accessories under Accessories

#### Scenario: Switching from the tree
- **WHEN** the user clicks "Mudflaps" under the trailer's Accessories in the Textures tab
- **THEN** that surface becomes active and is highlighted in the tree

#### Scenario: Thumbnail follows the artwork
- **WHEN** the user draws a red rectangle covering the Chassis texture
- **THEN** the Chassis row's thumbnail shows the red artwork and no template lines, and its marker changes from a hollow ring to a filled dot

#### Scenario: Flagged texture in the tree
- **WHEN** Update Template to version 1.3.0 flagged the Standard cab texture "Layout changed"
- **THEN** its row shows the warning icon in the signal color, and hovering it says "To check" and "Layout changed in 1.3.0"

#### Scenario: Search textures field
- **WHEN** the Textures tab is shown on macOS in English
- **THEN** a field reading "Search textures" with "⌘K" at its right end is shown above the tree, and clicking it opens the command palette listing only the project's textures
