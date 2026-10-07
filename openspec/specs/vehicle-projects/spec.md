# vehicle-projects Specification

## Purpose

Makes a project the paint job of a fleet: one or more vehicles of one game, each with its chosen variants, and one surface per texture of each variant with its template shown as a locked overlay. It records which package version each vehicle's templates come from, so each vehicle can move to a newer package version after a game update.

## Requirements

### Requirement: Project from a vehicle
Creating a project SHALL take one vehicle and one or more of its variants. It gives a project with:
- for each chosen variant, in package order, one surface per texture of the variant, in package order, each named after its texture and sized to its texture;
- the first texture of the first chosen variant active;
- each surface's template embedded in the project;
- a record of the vehicle (package id, version, name, brand, kind and game) and of each chosen variant (id and name).

Every surface SHALL belong to one texture of one variant of one vehicle of the project.

The project name SHALL default to the vehicle name.

#### Scenario: Truck with three textures
- **WHEN** the user creates a project from a variant with textures "Cabin" (4096 px), "Chassis" (2048 px) and "Accessories" (1024 px)
- **THEN** the project has three surfaces with those names and sizes, "Cabin" is active, and each shows its template

#### Scenario: Two variants at once
- **WHEN** the user creates a project from the sample truck with both "Standard cab" and "High roof" checked
- **THEN** the project has six surfaces, three per variant, and the Standard cab's Cabin is active

### Requirement: Switching textures
The sidebar's tree SHALL be the place to switch textures: clicking a texture makes its surface active. There are no texture tabs above the canvas. Vehicle › Next Texture (Cmd/Ctrl+Page Down) and Vehicle › Previous Texture (Cmd/Ctrl+Page Up) SHALL make the following or preceding surface active, in project order and wrapping around; they are disabled for a project with one texture.

When the active surface changes:
- the canvas, rulers, Layers panel, Properties panel and status bar SHALL show that surface. The status bar names it as "<vehicle> › <variant> › <texture>";
- the selection SHALL be cleared;
- each surface SHALL keep its own view (zoom and scroll) for as long as the project's list of surfaces doesn't change. Adding or removing vehicles or variants fits the other surfaces to the window again.

Undo and redo SHALL apply to the whole project. Undoing a change made on another surface SHALL make that surface active.

#### Scenario: Paint on the chassis
- **WHEN** the user clicks "Chassis" in the sidebar's tree and draws a rectangle
- **THEN** the rectangle is on the Chassis surface and the Cabin surface is unchanged

#### Scenario: Undo across textures
- **WHEN** the user draws on Cabin, switches to Chassis, and presses Undo
- **THEN** Cabin becomes active and the rectangle drawn on it is removed

#### Scenario: Next texture from the keyboard
- **WHEN** the active texture is the High roof's Chassis and the user presses Cmd/Ctrl+Page Down
- **THEN** the High roof's Accessories becomes active, and the status bar shows "TruckPaint Sample Truck › High roof › Accessories"

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
- **each vehicle** is a group titled with its name, with its kind and package version below (the supported game versions on hover) and a **⋯** actions menu: **Variants…**, **Update Template…** (when a newer version is installed) and **Remove from Project** (disabled for the last vehicle). When a newer version is installed, an update button showing that version SHALL also appear next to the menu;
- **under each vehicle:** its chosen variants, with a warning icon when one of their textures was flagged by an update;
- **under each variant:** its textures with their sizes and a warning icon when flagged. The active texture is highlighted, and clicking a texture makes it active.

Groups can be collapsed; the active vehicle and variant stay open. Vehicle › Vehicle Information SHALL open the sidebar.

#### Scenario: Newer version available
- **WHEN** a project made with version 1.2.0 of a vehicle is open and version 1.3.0 of that vehicle is installed
- **THEN** that vehicle's group shows an update button for 1.3.0, which opens Update Template

#### Scenario: Switching from the tree
- **WHEN** the user clicks "Side skirts" under the trailer's variant in the sidebar
- **THEN** that surface becomes active and is highlighted in the tree

### Requirement: Update Template
Update Template SHALL work on one vehicle of the project. It is reached from that vehicle's update button or ⋯ menu in the sidebar, or from Vehicle › Update Template…, which applies to the vehicle of the active surface. It SHALL be enabled when a newer version of that vehicle's package is installed.

It SHALL show what will change for every chosen variant of the vehicle. On confirmation, it switches the vehicle to that version as one undo step:

- **Same variant and texture id:** the template is replaced, the artwork is kept, and the size is kept.
  - If the texture's size changed, the artwork is scaled to the new size.
  - If its layout version changed, the surface is flagged "Layout changed".
- **New texture in a chosen variant:** a new empty surface is added with its template, after that variant's surfaces.
- **Removed texture, or a chosen variant missing from the new version:** the surface and its artwork are kept, marked "Not in this version" and drawn without a template.
- The vehicle's recorded package version becomes the new one. The project's other vehicles are unchanged.

Undo SHALL restore the previous templates, surfaces and recorded version.

#### Scenario: Patch that changed the cabin layout
- **WHEN** a vehicle in a project uses version 1.2.0 (Cabin layout 1, Chassis layout 1) and the user updates it to 1.3.0 (Cabin layout 2, Chassis layout 1)
- **THEN** both templates are replaced, the artwork is unchanged, Cabin is flagged "Layout changed", Chassis is not, and the vehicle records version 1.3.0

#### Scenario: Undo an update
- **WHEN** the user presses Undo right after updating a vehicle to 1.3.0
- **THEN** the vehicle's templates and recorded version are those of 1.2.0 again

#### Scenario: Only one vehicle is updated
- **WHEN** a project holds the sample truck 1.0.0 and a trailer, and the user updates the sample truck to 1.1.0
- **THEN** the truck's surfaces are updated and the trailer's surfaces and recorded version are unchanged

### Requirement: Templates travel with the project
A project SHALL open with its templates on any computer, whether or not its packages are installed. When a package isn't installed, the sidebar SHALL still show the recorded vehicle, version, variant names and textures. Update Template and adding variants stay disabled for that vehicle.

#### Scenario: Opening without the package
- **WHEN** a project is opened on a computer where its packages are not installed
- **THEN** every surface shows its template, and the sidebar shows each vehicle, its version and its variants as recorded in the project

### Requirement: One game per project
A project SHALL belong to exactly one game, ETS2 or ATS: the game of its first vehicle. Add Vehicle… SHALL list only vehicles of that game. A project file mixing games SHALL be refused as invalid.

#### Scenario: Adding to an ETS2 project
- **WHEN** the user opens Add Vehicle… in a project made for an ETS2 truck, with ETS2 and ATS packages installed
- **THEN** only the ETS2 vehicles are listed

### Requirement: Adding and removing vehicles and variants
The user SHALL be able to change which vehicles and variants a project covers. Each change is one undo step.

- **Add a vehicle:** Vehicle › Add Vehicle… opens a dialog listing the installed vehicles of the project's game that aren't in the project yet. It is searchable by name and brand and filterable by kind. Each vehicle shows its variants as checkboxes, and at least one must be checked. Confirming adds the vehicle at its newest installed version and adds one surface per texture of each checked variant, after the existing surfaces. The first new surface becomes active.
- **Change the variants of a vehicle:** the vehicle's **Variants…** action in the sidebar (⋯ menu) shows checkboxes for the variants of the project's recorded version.
  - Checking a variant adds its surfaces.
  - Unchecking one removes its surfaces. If any of them holds artwork, a confirmation names the variant first.
  - At least one variant stays checked.
  - Adding a variant needs the recorded version to be installed. When it isn't, the action explains that the version is missing and offers Update Template… instead.
- **Remove a vehicle:** the vehicle's **Remove from Project** action (⋯ menu) removes the vehicle and its surfaces. If any of them holds artwork, a confirmation names the vehicle first. The last vehicle of a project can't be removed.

Template images no longer used by any surface SHALL be dropped from the project's assets. Undo SHALL restore them with the surfaces.

#### Scenario: A fleet of two trucks and a trailer
- **WHEN** a project made for the "Standard cab" of the sample truck gets a second truck with two variants and a trailer with one variant through Add Vehicle…
- **THEN** the project lists the three vehicles, each with its chosen variants and their textures

#### Scenario: Removing a variant with artwork
- **WHEN** the user unchecks a variant whose Cabin texture holds a rectangle and confirms
- **THEN** its surfaces are removed, and Undo brings them back with the rectangle

#### Scenario: The last vehicle stays
- **WHEN** a project has a single vehicle
- **THEN** its Remove from Project action is disabled
