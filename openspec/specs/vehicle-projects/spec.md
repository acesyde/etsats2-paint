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
The sidebar's tree SHALL be the place to switch textures: clicking a texture makes its surface active. There are no texture tabs above the canvas. Vehicle › Next Texture (Cmd/Ctrl+Page Down) and Vehicle › Previous Texture (Cmd/Ctrl+Page Up) SHALL make the following or preceding surface active, in project order and wrapping around; they are disabled for a project with one texture.

When the active surface changes:
- the canvas, rulers, Layers panel, Properties panel and status bar SHALL show that surface. The status bar names it as "<vehicle> › <texture>";
- the selection SHALL be cleared;
- each surface SHALL keep its own view (zoom and scroll) for as long as the project's list of surfaces doesn't change. Adding or removing vehicles or textures fits the other surfaces to the window again.

Undo and redo SHALL apply to the whole project. Undoing a change made on another surface SHALL make that surface active.

#### Scenario: Paint on the chassis
- **WHEN** the user clicks "Chassis" in the sidebar's tree and draws a rectangle
- **THEN** the rectangle is on the Chassis surface and the Standard cab surface is unchanged

#### Scenario: Undo across textures
- **WHEN** the user draws on Standard cab, switches to Chassis, and presses Undo
- **THEN** Standard cab becomes active and the rectangle drawn on it is removed

#### Scenario: Next texture from the keyboard
- **WHEN** the active texture is the sample truck's High roof, followed by its Chassis, and the user presses Cmd/Ctrl+Page Down
- **THEN** Chassis becomes active, and the status bar shows "TruckPaint Sample Truck › Chassis"

### Requirement: Template overlay
A surface's template SHALL be drawn on the canvas above the artwork, at the surface's size, as a locked overlay.
- It cannot be selected, moved or edited, and it is not listed in the Layers panel.
- Its opacity (default 60%) and visibility SHALL be set per surface in the Properties panel when nothing is selected, saved with the project, and not recorded in the undo history.
- View › Show Template (Shift+T) SHALL toggle the visibility of the active surface's template.
- The template SHALL never be exported. It is also left out of the eyedropper and of hit testing.

When an update flagged the active texture, the Properties panel SHALL say so: "Layout changed" with a Dismiss action, or "Not in this version" (no template settings then).

#### Scenario: Hide the template
- **WHEN** the user presses Shift+T on a vehicle project
- **THEN** the active surface's template is hidden, the artwork is unchanged, and pressing Shift+T again shows it

#### Scenario: Template settings in Properties
- **WHEN** nothing is selected and the user unchecks Show Template in the Properties panel
- **THEN** the active surface's template is hidden, and no undo step is recorded

#### Scenario: Template is not exported
- **WHEN** a vehicle project with its template visible is exported to PNG
- **THEN** the image contains only the artwork

### Requirement: Vehicle panel
The sidebar, on the left of the canvas (see the workspace-layout capability), SHALL show a **Vehicles** section titled with the project's game ("Vehicles · ETS2"), with an **Add Vehicle** (+) button, and a tree of the project's vehicles:
- **each vehicle** is a group titled with its name, with its kind and package version below (the supported game versions on hover) and a **⋯** actions menu: **Textures…**, **Update Template…** (when a newer version is installed) and **Remove from Project** (disabled for the last vehicle). When a newer version is installed, an update button showing that version SHALL also appear next to the menu;
- **under each vehicle**, its textures in project order:
  - a **Main textures** heading over its chosen main textures (a truck's cabins, a trailer's base);
  - an **Accessories** heading over its chosen accessories, when it has any;
- **each texture** is one row with its name, its size and a warning icon when it was flagged by an update. The active texture is highlighted, and clicking a texture makes it active.

Vehicle groups can be collapsed; the active vehicle stays open. Vehicle › Vehicle Information SHALL open the sidebar.

#### Scenario: Newer version available
- **WHEN** a project made with version 1.2.0 of a vehicle is open and version 1.3.0 of that vehicle is installed
- **THEN** that vehicle's group shows an update button for 1.3.0, which opens Update Template

#### Scenario: Truck and trailer in the tree
- **WHEN** a project holds the sample truck with both cabins and the sample trailer, each with its accessories
- **THEN** the truck's group lists "Standard cab" and "High roof" under Main textures, then its accessories under Accessories, and the trailer's group lists "Base" under Main textures, then its accessories under Accessories

#### Scenario: Switching from the tree
- **WHEN** the user clicks "Mudflaps" under the trailer's Accessories in the sidebar
- **THEN** that surface becomes active and is highlighted in the tree

### Requirement: Update Template
Update Template SHALL work on one vehicle of the project. It is reached from that vehicle's update button or ⋯ menu in the sidebar, or from Vehicle › Update Template…, which applies to the vehicle of the active surface. It SHALL be enabled when a newer version of that vehicle's package is installed.

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

### Requirement: Templates travel with the project
A project SHALL open with its templates on any computer, whether or not its packages are installed. When a package isn't installed, the sidebar SHALL still show the recorded vehicle, its version and its textures. Update Template and adding textures stay disabled for that vehicle.

#### Scenario: Opening without the package
- **WHEN** a project is opened on a computer where its packages are not installed
- **THEN** every surface shows its template, and the sidebar shows each vehicle, its version and its textures as recorded in the project

### Requirement: One game per project
A project SHALL belong to exactly one game, ETS2 or ATS: the game of its first vehicle. Add Vehicle… SHALL list only vehicles of that game. A project file mixing games SHALL be refused as invalid.

#### Scenario: Adding to an ETS2 project
- **WHEN** the user opens Add Vehicle… in a project made for an ETS2 truck, with ETS2 and ATS packages installed
- **THEN** only the ETS2 vehicles are listed

### Requirement: Adding and removing vehicles and textures
The user SHALL be able to change which vehicles and textures a project covers. Each change is one undo step.

- **Add a vehicle:** Vehicle › Add Vehicle… opens a dialog listing the installed vehicles of the project's game that aren't in the project yet. It is searchable by name and brand and filterable by kind. Picking a vehicle shows its main textures and accessories as checkboxes, as in New Project, and at least one main texture must be checked. Confirming adds the vehicle at its newest installed version and adds one surface per chosen texture, after the existing surfaces. The first new surface becomes active.
  - The dialog SHALL also offer **Custom vehicle…**, which opens the Custom Vehicle dialog with the project's game, locked (see the custom-vehicles capability). A vehicle created there SHALL be listed and selected when Add Vehicle… is shown again, with its textures checked as for any picked vehicle; it is not added to the project until the user confirms Add.
- **Change the textures of a vehicle:** the vehicle's **Textures…** action in the sidebar (⋯ menu) shows checkboxes for the main textures and accessories of the project's recorded version. A single main texture is shown checked and can't be unchecked.
  - Checking a texture adds its surface, in package order among the vehicle's surfaces.
  - Unchecking one removes its surface. If any removed surface holds artwork, a confirmation names those textures first.
  - At least one main texture stays checked.
  - Adding a texture needs the recorded version to be installed. When it isn't, the action explains that the version is missing and offers Update Template… instead.
- **Remove a vehicle:** the vehicle's **Remove from Project** action (⋯ menu) removes the vehicle and its surfaces. If any of them holds artwork, a confirmation names the vehicle first. The last vehicle of a project can't be removed.

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
- **THEN** its Remove from Project action is disabled

#### Scenario: Custom trailer added to a fleet
- **WHEN** in an ETS2 project, the user opens Add Vehicle…, clicks Custom vehicle…, creates a trailer from the template `base.png`, then clicks Add
- **THEN** the trailer is added to the project with its "base" texture active, and the Custom Vehicle dialog showed ETS2 without a way to change it
