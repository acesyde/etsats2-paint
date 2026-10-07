## ADDED Requirements

### Requirement: One game per project
A project SHALL belong to exactly one game, ETS2 or ATS: the game of its first vehicle. Add Vehicle… SHALL list only vehicles of that game. A project file mixing games SHALL be refused as invalid.

#### Scenario: Adding to an ETS2 project
- **WHEN** the user opens Add Vehicle… in a project made for an ETS2 truck, with ETS2 and ATS packages installed
- **THEN** only the ETS2 vehicles are listed

### Requirement: Adding and removing vehicles and variants
The user SHALL be able to change which vehicles and variants a project covers. Each change is one undo step.

- **Add a vehicle:** Vehicle › Add Vehicle… opens a dialog listing the installed vehicles of the project's game that aren't in the project yet. It is searchable by name and brand and filterable by kind. Each vehicle shows its variants as checkboxes, and at least one must be checked. Confirming adds the vehicle at its newest installed version and adds one surface per texture of each checked variant, after the existing surfaces. The first new surface becomes active.
- **Change the variants of a vehicle:** the vehicle's **Variants…** action in the Vehicles sidebar shows checkboxes for the variants of the project's recorded version.
  - Checking a variant adds its surfaces.
  - Unchecking one removes its surfaces. If any of them holds artwork, a confirmation names the variant first.
  - At least one variant stays checked.
  - Adding a variant needs the recorded version to be installed. When it isn't, the action explains that the version is missing and offers Update Template… instead.
- **Remove a vehicle:** the vehicle's **Remove from Project** action removes the vehicle and its surfaces. If any of them holds artwork, a confirmation names the vehicle first. The last vehicle of a project can't be removed.

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

## MODIFIED Requirements

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
The user SHALL be able to make any surface active from the Vehicles sidebar tree.

When the active variant has several textures, tabs above the canvas SHALL list them in order and SHALL show which one is active. Clicking a tab SHALL make that surface active. The tabs only show textures of the active variant, and are hidden when it has a single texture.

When the active surface changes:
- the canvas, rulers, Layers panel and status bar SHALL show that surface. The status bar names it as "<vehicle> › <variant> › <texture>";
- the selection SHALL be cleared;
- each surface SHALL keep its own view (zoom and scroll) for as long as the project's list of surfaces doesn't change. Adding or removing vehicles or variants fits the other surfaces to the window again.

Undo and redo SHALL apply to the whole project. Undoing a change made on another surface SHALL make that surface active.

#### Scenario: Paint on the chassis
- **WHEN** the user clicks the "Chassis" tab and draws a rectangle
- **THEN** the rectangle is on the Chassis surface and the Cabin surface is unchanged

#### Scenario: Undo across textures
- **WHEN** the user draws on Cabin, switches to Chassis, and presses Undo
- **THEN** Cabin becomes active and the rectangle drawn on it is removed

#### Scenario: Tabs follow the active variant
- **WHEN** a project covers the sample truck's Standard cab and High roof, and the user clicks the High roof's Chassis in the Vehicles sidebar
- **THEN** the tabs list the High roof's textures with Chassis active, and the status bar shows "TruckPaint Sample Truck › High roof › Chassis"

### Requirement: Vehicle panel
The Vehicles sidebar, on the left of the canvas (see the workspace-layout capability), SHALL show the project's game and a tree of its vehicles:
- **for each vehicle:** its name, brand and kind, and its package version with its supported game versions;
- **under each vehicle:** its chosen variants;
- **under each variant:** its textures with their sizes. The active texture is highlighted, and clicking a texture makes it active.

Each vehicle SHALL offer **Variants…** and **Remove from Project**. The panel SHALL end with an **Add Vehicle…** action.

Below the tree, the sidebar SHALL show the active surface's template opacity and visibility.

Textures flagged by an update SHALL show a "Layout changed" badge with a Dismiss action, and textures not in the recorded version a "Not in this version" badge. When a newer version of a vehicle's package is installed, that vehicle SHALL show a notice with an **Update Template…** action.

Vehicle › Vehicle Information SHALL open the Vehicles sidebar.

#### Scenario: Newer version available
- **WHEN** a project made with version 1.2.0 of a vehicle is open and version 1.3.0 of that vehicle is installed
- **THEN** the Vehicles sidebar shows, on that vehicle, that version 1.3.0 is available with an Update Template… action

#### Scenario: Switching from the tree
- **WHEN** the user clicks "Side skirts" under the trailer's variant in the Vehicles sidebar
- **THEN** that surface becomes active and is highlighted in the tree

### Requirement: Update Template
Update Template SHALL work on one vehicle of the project. It is reached from that vehicle's notice in the Vehicles sidebar, or from Vehicle › Update Template…, which applies to the vehicle of the active surface. It SHALL be enabled when a newer version of that vehicle's package is installed.

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
A project SHALL open with its templates on any computer, whether or not its packages are installed. When a package isn't installed, the Vehicles sidebar SHALL still show the recorded vehicle, version, variant names and textures. Update Template and adding variants stay disabled for that vehicle.

#### Scenario: Opening without the package
- **WHEN** a project is opened on a computer where its packages are not installed
- **THEN** every surface shows its template, and the Vehicles sidebar shows each vehicle, its version and its variants as recorded in the project
