## MODIFIED Requirements

### Requirement: Update Template
Update Template SHALL work on one vehicle of the project. It is reached from that vehicle's update button or ⋯ menu in the Textures tab, from its vehicle card in the Project space (see the project-screen capability), or from Vehicle › Update Template…, which applies to the vehicle of the active surface. It SHALL be enabled when a newer version of that vehicle's package is installed.

It SHALL show what will change for every texture of the vehicle in the project. It SHALL also list the textures that are new in that version: new accessories as checked checkboxes and new main textures as unchecked ones. When the new version has a single main texture and the vehicle doesn't paint it yet, that texture is always added. On confirmation, it switches the vehicle to that version as one undo step:

- **Same texture id:** the template is replaced, the artwork is kept, and the size is kept.
  - If the texture's size changed, the artwork is scaled to the new size: horizontally by the ratio of the widths and vertically by the ratio of the heights.
  - If its layout version changed, the surface is flagged "Layout changed".
- **Checked new texture, or an always-added single main texture:** a new empty surface is added with its template, in package order among the vehicle's surfaces.
- **Texture of the project missing from the new version:** the surface and its artwork are kept, marked "Not in this version" and drawn without a template.
- The vehicle's recorded package version becomes the new one. The project's other vehicles are unchanged.

Undo SHALL restore the previous templates, surfaces and recorded version.

#### Scenario: Patch that changed the cabin layout
- **WHEN** a vehicle in a project uses version 1.2.0 (Standard cab layout 1, Chassis layout 1) and the user updates it to 1.3.0 (Standard cab layout 2, Chassis layout 1)
- **THEN** both templates are replaced, the artwork is unchanged, Standard cab is flagged "Layout changed", Chassis is not, and the vehicle records version 1.3.0

#### Scenario: New accessory in the new version
- **WHEN** the user updates a project's sample truck from 1.0.0 to 1.1.0
- **THEN** Update Template lists "Side skirts" as a new texture, checked, and confirming adds a Side skirts surface after the vehicle's other accessories

#### Scenario: Undo an update
- **WHEN** the user presses Undo right after updating a vehicle to 1.3.0
- **THEN** the vehicle's templates and recorded version are those of 1.2.0 again

#### Scenario: Only one vehicle is updated
- **WHEN** a project holds the sample truck 1.0.0 and the sample trailer, and the user updates the sample truck to 1.1.0
- **THEN** the truck's surfaces are updated and the trailer's surfaces and recorded version are unchanged

#### Scenario: Update from the Project space
- **WHEN** the Project space is shown and the user starts Update Template from the sample truck's vehicle card
- **THEN** the same Update Template dialog opens for the sample truck

### Requirement: Copy from cabin
**Vehicle › Copy From Cabin…** SHALL copy the artwork of another main texture of the same truck onto the active texture. The cabin layouts of a truck share most of their texture, so the same coordinates fit.

It SHALL be enabled only when:
- the active texture is a main texture of a vehicle;
- the project paints at least one other main texture of that vehicle.

Otherwise, a tooltip says why. Trailers and accessories have no other main texture to copy from.

It SHALL open a dialog listing the vehicle's other main textures in the project, each with the number of objects it holds. The first texture that holds objects is chosen by default. Confirming SHALL:
- copy every object of the chosen texture, in stacking order, including hidden and locked ones, onto the active texture, above its objects;
- keep each copy's position in texture pixels. When the two textures differ in size, the copies are scaled by the ratio of the widths horizontally and of the heights vertically, as Update Template scales artwork;
- keep each copy's name, look, links to swatches and styles, visibility and lock;
- select the copies;
- record one undo step.

The chosen texture SHALL be unchanged. Guides are not copied. When the chosen texture holds no object, the dialog SHALL say so and Copy is disabled.

#### Scenario: Copy the standard cab onto the high roof
- **WHEN** the sample truck's Standard cab holds a logo and lettering, the High roof is active and empty, and the user chooses Copy From Cabin… with Standard cab
- **THEN** the High roof holds copies of the logo and the lettering at the same positions, selected, and one Undo removes them

#### Scenario: Different sizes
- **WHEN** a 2048 px main texture holds a rectangle at (100, 100), 200 × 50, and it is copied onto a 4096 px main texture of the same truck
- **THEN** the copy is at (200, 200), 400 × 100

#### Scenario: Not offered for accessories
- **WHEN** the active texture is the sample truck's Chassis
- **THEN** Copy From Cabin… is disabled, with a tooltip saying it copies between main textures

#### Scenario: Links kept
- **WHEN** a text following the text style "Lettering" is copied from cabin
- **THEN** the copy follows "Lettering" too

### Requirement: Textures tab
The Workshop's **Textures** tab (see the workspace-layout capability) SHALL show the fleet as a tree, with an **Add Vehicle** (+) button in its header that runs Vehicle › Add Vehicle…, and above the tree a **Search textures** field showing the Command Palette shortcut at its right end in the platform's notation. Clicking the field, or giving it the keyboard focus and pressing Enter or typing, SHALL open the command palette limited to textures (see the command-palette capability), with what was typed in its field. The field itself SHALL never hold text nor filter the tree.

The tree:
- **each vehicle** is a group titled with its name, with its kind and package version below (the supported game versions on hover) and a **⋯** actions menu: **Textures…**, **Update Template…** (when a newer version is installed) and **Remove from Project** (disabled for the last vehicle). When a newer version is installed, an update button showing that version SHALL also appear next to the menu;
- **under each vehicle**, its textures in project order:
  - a **Main textures** heading over its chosen main textures (a truck's cabins, a trailer's base);
  - an **Accessories** heading over its chosen accessories, when it has any;
- **each texture** is one row with a thumbnail of its artwork, its name, its size ("4096²", or "8192 × 1024" when its sides differ) and, at its right end, a marker of its state (see the texture-status capability): a hollow ring for Empty, a filled dot for Modified, and a warning icon in the signal color for To check, so that the states differ by shape and not only by color. Hovering the marker SHALL show the state's label, and for To check why ("Layout changed in <version>" or "Not in this version"); the row's name for assistive technologies SHALL end with the state. The thumbnail SHALL show the texture's current artwork without the template, and follow its edits. The active texture is highlighted without relying on color alone, and clicking a texture makes it active.

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
