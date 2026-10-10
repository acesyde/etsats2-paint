# vehicle-projects Specification

## Purpose

Makes a project the paint job of a fleet: one or more vehicles of one game, each painting the main textures and accessories chosen for it, with one surface per texture and its template shown as a locked overlay. It records which package version each vehicle's templates come from, so each vehicle can move to a newer package version after a game update.

## Requirements

### Requirement: Project from a vehicle's paint job
Creating a project SHALL take one vehicle and the textures to paint:
- **its main textures:** one or more of them. A vehicle with a single main texture (a trailer, or a truck whose cabins share one layout) always includes it; a truck whose cabins have different layouts has one main texture per layout, and the player checks which ones to paint;
- **its accessories:** any of them.

It gives a project with:
- one surface per chosen texture, in package order: main textures, then accessories. Each surface is named after its texture and sized to it;
- the first surface active;
- each surface's template embedded in the project;
- a record of the vehicle (package id, version, name, brand, kind and game).

Every surface SHALL belong to one texture of one vehicle of the project, as a main texture or an accessory.

The project name SHALL default to the vehicle name.

#### Scenario: Truck with two cabins
- **WHEN** the user creates a project from sample truck 1.1.0 with both main textures and every accessory checked
- **THEN** the project has five surfaces, "Standard cab" (4096), "High roof" (4096), "Chassis" (4096), "Cab accessories" (1024) and "Side skirts" (1024), "Standard cab" is active, and each shows its template

#### Scenario: Trailer
- **WHEN** the user creates a project from the sample trailer with every accessory checked
- **THEN** the project has the surfaces "Base", "Curtain body 13.6 m", "Curtain body 10.5 m" and "Mudflaps", and "Base" is active

#### Scenario: Accessory left out
- **WHEN** the user creates a project from the sample truck's Standard cab and unchecks "Cab accessories"
- **THEN** the project has no "Cab accessories" surface

### Requirement: Switching textures
The user SHALL switch textures from the Workshop's Textures tab, from the textures of the Project space (see the project-screen capability), and with the keyboard: clicking a texture makes its surface active. There are no texture tabs above the canvas. Vehicle › Next Texture (Cmd/Ctrl+], also Cmd/Ctrl+Page Down) and Vehicle › Previous Texture (Cmd/Ctrl+[, also Cmd/Ctrl+Page Up) SHALL make the following or preceding surface active, in project order and wrapping around, without changing the space shown; they are disabled for a project with one texture.

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

### Requirement: Update Template
Update Template SHALL work on one vehicle of the project. It is reached from that vehicle's update button or ⋯ menu in the Textures tab, from its vehicle card in the Project space (see the project-screen capability), or from Vehicle › Update Template…, which applies to the vehicle of the active surface. It SHALL be enabled when a newer version of that vehicle's package is installed.

It SHALL show what will change for every texture of the vehicle in the project. It SHALL also list the textures that are new in that version: new accessories as checked checkboxes and new main textures as unchecked ones. When the new version has a single main texture and the vehicle doesn't paint it yet, that texture is always added. On confirmation, it switches the vehicle to that version as one undo step:

- **Same texture id:** the template is replaced, the artwork is kept, and the size is kept.
  - If the texture's size changed, the artwork is scaled to the new size.
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

### Requirement: Templates travel with the project
A project SHALL open with its templates on any computer, whether or not its packages are installed. When a package isn't installed, the Textures tab and the Project space SHALL still show the recorded vehicle, its version and its textures. Update Template and adding textures stay disabled for that vehicle.

#### Scenario: Opening without the package
- **WHEN** a project is opened on a computer where its packages are not installed
- **THEN** every surface shows its template, and the Project space and the Textures tab show each vehicle, its version and its textures as recorded in the project

### Requirement: One game per project
A project SHALL belong to exactly one game, ETS2 or ATS: the game of its first vehicle. Add Vehicle… SHALL list only vehicles of that game. A project file mixing games SHALL be refused as invalid.

#### Scenario: Adding to an ETS2 project
- **WHEN** the user opens Add Vehicle… in a project made for an ETS2 truck, with ETS2 and ATS packages installed
- **THEN** only the ETS2 vehicles are listed

### Requirement: Adding and removing vehicles and textures
The user SHALL be able to change which vehicles and textures a project covers. Each change is one undo step.

- **Add a vehicle:** Vehicle › Add Vehicle…, also reached from the Add Vehicle button of the Textures tab and of the Project space, opens a dialog listing the installed vehicles of the project's game that aren't in the project yet. It is searchable by name and brand and filterable by kind. Picking a vehicle shows its main textures and accessories as checkboxes, as in New Project, and at least one main texture must be checked. Confirming adds the vehicle at its newest installed version and adds one surface per chosen texture, after the existing surfaces. The first new surface becomes active.
  - The dialog SHALL also offer **Custom vehicle…**, which opens the Custom Vehicle dialog with the project's game, locked (see the custom-vehicles capability). A vehicle created there SHALL be listed and selected when Add Vehicle… is shown again, with its textures checked as for any picked vehicle; it is not added to the project until the user confirms Add.
- **Change the textures of a vehicle:** the vehicle's **Textures…** action (its ⋯ menu in the Textures tab, or its vehicle card in the Project space) shows checkboxes for the main textures and accessories of the project's recorded version. A single main texture is shown checked and can't be unchecked.
  - Checking a texture adds its surface, in package order among the vehicle's surfaces.
  - Unchecking one removes its surface. If any removed surface holds artwork, a confirmation names those textures first.
  - At least one main texture stays checked.
  - Adding a texture needs the recorded version to be installed. When it isn't, the action explains that the version is missing and offers Update Template… instead.
- **Remove a vehicle:** the vehicle's **Remove from Project** action (its ⋯ menu in the Textures tab, or its vehicle card in the Project space) removes the vehicle and its surfaces. If any of them holds artwork, a confirmation names the vehicle first. The last vehicle of a project can't be removed.

Template images no longer used by any surface SHALL be dropped from the project's assets. Undo SHALL restore them with the surfaces.

#### Scenario: A truck and a trailer
- **WHEN** a project made for the sample truck's Standard cab gets the sample trailer through Add Vehicle…, with every accessory checked
- **THEN** the project lists both vehicles, the trailer with Base and its three accessories, and the trailer's Base is active

#### Scenario: Removing a texture with artwork
- **WHEN** the user unchecks the High roof main texture, whose surface holds a rectangle, in Textures… and confirms
- **THEN** its surface is removed, and Undo brings it back with the rectangle

#### Scenario: Adding an accessory later
- **WHEN** a project was created without the sample truck's "Cab accessories", and the user checks it in Textures…
- **THEN** a Cab accessories surface is added between the Chassis and Side skirts surfaces

#### Scenario: The last main texture stays
- **WHEN** a truck in a project paints a single one of its main textures
- **THEN** that texture's checkbox in Textures… can't be unchecked

#### Scenario: The last vehicle stays
- **WHEN** a project has a single vehicle
- **THEN** its Remove from Project action is disabled, in the Textures tab and in the Project space

#### Scenario: Custom trailer added to a fleet
- **WHEN** in an ETS2 project, the user opens Add Vehicle…, clicks Custom vehicle…, creates a trailer from the template `base.png`, then clicks Add
- **THEN** the trailer is added to the project with its "base" texture active, and the Custom Vehicle dialog showed ETS2 without a way to change it

### Requirement: Copy from cabin
**Vehicle › Copy From Cabin…** SHALL copy the artwork of another main texture of the same truck onto the active texture. The cabin layouts of a truck share most of their texture, so the same coordinates fit.

It SHALL be enabled only when:
- the active texture is a main texture of a vehicle;
- the project paints at least one other main texture of that vehicle.

Otherwise, a tooltip says why. Trailers and accessories have no other main texture to copy from.

It SHALL open a dialog listing the vehicle's other main textures in the project, each with the number of objects it holds. The first texture that holds objects is chosen by default. Confirming SHALL:
- copy every object of the chosen texture, in stacking order, including hidden and locked ones, onto the active texture, above its objects;
- keep each copy's position in texture pixels. When the two textures differ in size, the copies are scaled by the ratio of the sizes, as Update Template scales artwork;
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

### Requirement: Game data recorded with each vehicle
Each vehicle of a project SHALL record the game data of its package version, which the mod export needs:
- its game path, supported game versions, whether it uses the alternate UV set and the colour picker, and the mods it requires;
- for each of its textures in the project, the texture's game ids: the cabins of a main texture, or the accessory ids of an accessory.

This data SHALL be recorded when the vehicle enters the project, through New Project or Add Vehicle…, and when Textures… adds textures. Update Template SHALL replace it with the data of the new version, and undoing the update SHALL restore the previous data. A texture marked "Not in this version" keeps the game ids it had.

#### Scenario: Recorded on creation
- **WHEN** the user creates a project from the sample truck 1.1.0 painting Standard cab and Side skirts
- **THEN** the vehicle records the game path "truckpaint.sample", Standard cab records the game id "standard" and Side skirts records "sideskirt.sample"

#### Scenario: Replaced by Update Template
- **WHEN** a package's version 1.3.0 changes an accessory's game ids and the user updates a project's vehicle from 1.2.0 to 1.3.0
- **THEN** that accessory records the game ids of 1.3.0, and Undo restores those of 1.2.0

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
