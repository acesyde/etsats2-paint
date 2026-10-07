# vehicle-projects Specification

## Purpose

Lets a project be made for a vehicle: one surface per texture of the chosen variant, each with its template shown as a locked overlay. It records which package version the templates come from, so a project can move to a newer package version after a game update.

## Requirements

### Requirement: Project from a vehicle
Creating a project from a vehicle variant SHALL give a project with:
- one surface per texture of the variant, in package order, each named after its texture and sized to its texture;
- the variant's first texture active;
- each surface's template embedded in the project;
- a record of the package id, version, variant id and vehicle name.

The project name SHALL default to the vehicle name.

#### Scenario: Truck with three textures
- **WHEN** the user creates a project from a variant with textures "Cabin" (4096 px), "Chassis" (2048 px) and "Accessories" (1024 px)
- **THEN** the project has three surfaces with those names and sizes, "Cabin" is active, and each shows its template

### Requirement: Switching textures
When a project has several surfaces, tabs above the canvas SHALL list them in order and SHALL show which one is active. Clicking a tab SHALL make that surface active.
- The canvas, rulers, Layers panel and status bar SHALL then show that surface.
- The selection SHALL be cleared.
- Each surface SHALL keep its own view (zoom and scroll).

Undo and redo SHALL apply to the whole project. Undoing a change made on another surface SHALL make that surface active. Projects with one surface SHALL show no tabs.

#### Scenario: Paint on the chassis
- **WHEN** the user clicks the "Chassis" tab and draws a rectangle
- **THEN** the rectangle is on the Chassis surface and the Cabin surface is unchanged

#### Scenario: Undo across textures
- **WHEN** the user draws on Cabin, switches to Chassis, and presses Undo
- **THEN** Cabin becomes active and the rectangle drawn on it is removed

### Requirement: Template overlay
A surface's template SHALL be drawn on the canvas above the artwork, at the surface's size, as a locked overlay.
- It cannot be selected, moved or edited, and it is not listed in the Layers panel.
- Its opacity (default 60%) and visibility SHALL be set per surface in the Vehicle panel, saved with the project, and not recorded in the undo history.
- View › Show Template (Shift+T) SHALL toggle the visibility of the active surface's template.
- The template SHALL never be exported. It is also left out of the eyedropper and of hit testing.

#### Scenario: Hide the template
- **WHEN** the user presses Shift+T on a vehicle project
- **THEN** the active surface's template is hidden, the artwork is unchanged, and pressing Shift+T again shows it

#### Scenario: Template is not exported
- **WHEN** a vehicle project with its template visible is exported to PNG
- **THEN** the image contains only the artwork

### Requirement: Vehicle panel
For a vehicle project, the Vehicle panel SHALL show:
- the vehicle name, brand, game and variant;
- the package version and its supported game versions;
- the textures, each with its size, where clicking one makes it active;
- the active surface's template opacity and visibility.

It SHALL show any texture flagged by an update with a "Layout changed" badge and a Dismiss action. When a newer version of the project's package is installed, it SHALL show a notice with an **Update Template…** action.

For a blank-texture project, it SHALL say the project has no vehicle.

Vehicle › Vehicle Information SHALL reveal the Vehicle panel.

#### Scenario: Newer version available
- **WHEN** a project made with version 1.2.0 is open and version 1.3.0 of its vehicle is installed
- **THEN** the Vehicle panel shows that version 1.3.0 is available with an Update Template… action

### Requirement: Update Template
Update Template… (Vehicle menu, or the Vehicle panel notice) SHALL be enabled when a newer installed version of the project's package has the project's variant. It SHALL show what will change, and on confirmation switch the project to that version as one undo step:

- **Same texture id:** the template is replaced, the artwork is kept, and the size is kept.
  - If the texture's size changed, the artwork is scaled to the new size.
  - If its layout version changed, the surface is flagged "Layout changed".
- **New texture:** a new empty surface is added with its template.
- **Removed texture:** the surface and its artwork are kept, marked "Not in this version" and drawn without a template.
- The recorded package version becomes the new one.

Undo SHALL restore the previous templates, surfaces and recorded version.

#### Scenario: Patch that changed the cabin layout
- **WHEN** a project uses version 1.2.0 (Cabin layout 1, Chassis layout 1) and the user updates to 1.3.0 (Cabin layout 2, Chassis layout 1)
- **THEN** both templates are replaced, the artwork is unchanged, Cabin is flagged "Layout changed", Chassis is not, and the project records version 1.3.0

#### Scenario: Undo an update
- **WHEN** the user presses Undo right after updating a project to 1.3.0
- **THEN** the project's templates and recorded version are those of 1.2.0 again

### Requirement: Templates travel with the project
A vehicle project SHALL open with its templates on any computer, whether or not its package is installed. When the package is not installed, the Vehicle panel SHALL still show the recorded vehicle and version, and Update Template stays disabled.

#### Scenario: Opening without the package
- **WHEN** a vehicle project is opened on a computer where its package is not installed
- **THEN** every surface shows its template, and the Vehicle panel shows the vehicle and version recorded in the project
