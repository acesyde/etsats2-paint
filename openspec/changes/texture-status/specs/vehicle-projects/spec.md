## MODIFIED Requirements

### Requirement: Textures tab
The Workshop's **Textures** tab (see the workspace-layout capability) SHALL show the fleet as a tree, with an **Add Vehicle** (+) button in its header that runs Vehicle › Add Vehicle…:
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

### Requirement: Template overlay as a view setting
A surface's template SHALL be drawn on the canvas above the artwork, at the surface's size, as a locked overlay.
- It cannot be selected, moved or edited, and it is not listed in the Layers tab.
- Its opacity (default 60%) and visibility SHALL be set per surface in the Workshop's status bar: a **Template** toggle and an opacity value, shown for the active surface. They are saved with the project and not recorded in the undo history.
- View › Show Template (G) SHALL toggle the visibility of the active surface's template. The single key G SHALL act only when no text field has keyboard focus, no text is being edited on the canvas and no modal dialog is open.
- The template SHALL never be exported. It is also left out of the eyedropper and of hit testing.

When an update flagged the active texture, the inspector SHALL say so when nothing is selected (see the properties-panel capability): the texture is To check (see the texture-status capability), because its layout changed, with a **Mark as Checked** action, or because it is not in this version. A surface marked "Not in this version" has no template: the status bar's template settings are disabled for it.

#### Scenario: Hide the template
- **WHEN** the user presses G on a vehicle project in the Workshop
- **THEN** the active surface's template is hidden, the artwork is unchanged, the status bar's Template toggle shows its off state, and pressing G again shows it

#### Scenario: Template settings in the status bar
- **WHEN** the user sets the template opacity to 35% in the status bar
- **THEN** the active surface's template is drawn at 35%, other surfaces keep their opacity, and no undo step is recorded

#### Scenario: G while typing
- **WHEN** the user types "G" in a text being edited on the canvas
- **THEN** the letter is typed and the template's visibility does not change

#### Scenario: Template is not exported
- **WHEN** a vehicle project with its template visible is exported to PNG
- **THEN** the image contains only the artwork

#### Scenario: Hiding the template doesn't check the texture
- **WHEN** the active texture is To check because its layout changed, and the user hides its template with G
- **THEN** it is still To check
