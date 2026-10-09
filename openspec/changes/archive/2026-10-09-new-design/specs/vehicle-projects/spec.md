## ADDED Requirements

### Requirement: Textures tab
The Workshop's **Textures** tab (see the workspace-layout capability) SHALL show the fleet as a tree, with an **Add Vehicle** (+) button in its header that runs Vehicle › Add Vehicle…:
- **each vehicle** is a group titled with its name, with its kind and package version below (the supported game versions on hover) and a **⋯** actions menu: **Textures…**, **Update Template…** (when a newer version is installed) and **Remove from Project** (disabled for the last vehicle). When a newer version is installed, an update button showing that version SHALL also appear next to the menu;
- **under each vehicle**, its textures in project order:
  - a **Main textures** heading over its chosen main textures (a truck's cabins, a trailer's base);
  - an **Accessories** heading over its chosen accessories, when it has any;
- **each texture** is one row with a thumbnail of its artwork, its name, its size and a warning icon when it was flagged by an update. The thumbnail SHALL show the texture's current artwork without the template, and follow its edits. The active texture is highlighted without relying on color alone, and clicking a texture makes it active.

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
- **THEN** the Chassis row's thumbnail shows the red artwork and no template lines

### Requirement: Template overlay as a view setting
A surface's template SHALL be drawn on the canvas above the artwork, at the surface's size, as a locked overlay.
- It cannot be selected, moved or edited, and it is not listed in the Layers tab.
- Its opacity (default 60%) and visibility SHALL be set per surface in the Workshop's status bar: a **Template** toggle and an opacity value, shown for the active surface. They are saved with the project and not recorded in the undo history.
- View › Show Template (G) SHALL toggle the visibility of the active surface's template. The single key G SHALL act only when no text field has keyboard focus, no text is being edited on the canvas and no modal dialog is open.
- The template SHALL never be exported. It is also left out of the eyedropper and of hit testing.

When an update flagged the active texture, the inspector SHALL say so when nothing is selected (see the properties-panel capability): "Layout changed" with a Dismiss action, or "Not in this version". A surface marked "Not in this version" has no template: the status bar's template settings are disabled for it.

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

## MODIFIED Requirements

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

## REMOVED Requirements

### Requirement: Vehicle panel
**Reason**: The Vehicles sidebar is removed from the workspace (see workspace-layout); its fleet tree moves to the Workshop's Textures tab and to the vehicle cards of the Project space.
**Migration**: The tree, its headings, the update button, the ⋯ menu (Textures…, Update Template…, Remove from Project) and the Add Vehicle (+) button are in the Textures tab requirement; the vehicle cards of the Project space offer the same actions (project-screen). Vehicle › Vehicle Information is removed: the Project space (Cmd/Ctrl+1) and the Textures tab (1) replace it.

### Requirement: Template overlay
**Reason**: Its settings move from the Properties panel, which is removed, to the Workshop's status bar, and Show Template moves from Shift+T to G.
**Migration**: Everything it required is kept in the Template overlay as a view setting requirement: locked overlay, per-surface opacity and visibility (now in the status bar), View › Show Template (now G), never exported, left out of the eyedropper and hit testing, and the "Layout changed" / "Not in this version" notices (now in the inspector with nothing selected).
