# project-screen Specification

## Purpose
The Project space shows what a project paints and what it delivers: the fleet with its vehicles, cabins and textures, package updates, and the mod information with the editable game versions. It is where a project opens, and clicking a texture there opens it in the Workshop.

## Requirements

### Requirement: Project space layout
The Project space SHALL use the full width under the top bar (see the workspace-spaces capability), with two areas:
- **Vehicles**, on the left: a header and one card per vehicle of the project, in project order, scrolling when they don't fit;
- **Mod information**, a column on the right (see Mod information column).

The Vehicles header SHALL show the title "Vehicles", the number of vehicles and the number of textures of the project ("2 vehicles · 9 textures", with the plural forms of the interface language), and an **Add Vehicle…** button that runs Vehicle › Add Vehicle… (see the vehicle-projects capability). The header SHALL NOT show counts of texture states.

The space SHALL never show an empty panel: a project always holds at least one vehicle.

#### Scenario: Header counts
- **WHEN** a project holds the sample truck with five textures and the sample trailer with four textures, and the Project space is shown
- **THEN** the header reads "Vehicles", "2 vehicles · 9 textures", and offers Add Vehicle…

#### Scenario: One vehicle
- **WHEN** a project holds only the sample trailer with Base and Mudflaps
- **THEN** the header reads "1 vehicle · 2 textures"

#### Scenario: Adding a vehicle from the Project space
- **WHEN** the user clicks Add Vehicle… in the Vehicles header
- **THEN** the Add Vehicle dialog opens, listing only the vehicles of the project's game that aren't in the project

### Requirement: Vehicle cards
Each vehicle of the project SHALL be shown as a card holding:
- a **thumbnail**: the artwork of the vehicle's first painted main texture, drawn over its template, updated when that artwork changes;
- its **name**, and below it its **kind and package version** ("Truck · package 1.1.0"). Hovering it SHALL show the game versions the recorded package version supports, or that the package isn't installed;
- **Cabins**, for a truck: the names of the main textures it paints. Hovering them SHALL show their cabins' internal names when the package gives them; the internal names are not shown otherwise. A trailer shows no Cabins line;
- **Main texture**: the main texture mode, read only, from the package: "One per cabin layout" when the package has several main textures, "One for every cabin" for a truck with a single main texture, "Single main texture" for a trailer;
- when a newer version of the vehicle's package is installed, an "Update <version> available" notice in the signal color, with a text label and an icon (never color alone), and an **Update Template…** button that opens Update Template for that vehicle;
- a **Textures…** button, which opens the vehicle's Textures dialog (see the vehicle-projects capability);
- a **⋯** actions menu with **Textures…**, **Update Template…** (only when a newer version is installed) and **Remove from Project**. Remove from Project SHALL be disabled for the last vehicle of the project, with a tooltip saying why;
- the list of the vehicle's textures (see Texture rows).

When the vehicle's recorded package version isn't installed, the card SHALL still show the recorded name, kind, version and textures. Update Template is then offered only when a newer version is installed, and Textures… explains that the version is missing when the user tries to add a texture, as the vehicle-projects capability describes.

The actions SHALL be named for assistive technologies with the vehicle's name (for example "Actions for TruckPaint Sample Truck", "Update TruckPaint Sample Truck").

#### Scenario: Truck card
- **WHEN** a project holds the sample truck 1.1.0 painting Standard cab and High roof, and no newer version is installed
- **THEN** its card shows "TruckPaint Sample Truck", "Truck · package 1.1.0", Cabins "Standard cab, High roof" (hovering it shows "standard" and "high_roof"), Main texture "One per cabin layout", and no update notice

#### Scenario: Trailer card
- **WHEN** a project holds the sample trailer
- **THEN** its card shows "Trailer · package" with its version, no Cabins line, and Main texture "Single main texture"

#### Scenario: Newer version available
- **WHEN** a project made with version 1.2.0 of a vehicle is open and version 1.3.0 of that vehicle is installed
- **THEN** that vehicle's card shows "Update 1.3.0 available" with an Update Template… button, which opens Update Template for that vehicle

#### Scenario: Removing a vehicle
- **WHEN** a project holds a truck and a trailer, and the user chooses Remove from Project in the trailer's ⋯ menu
- **THEN** the trailer is removed as described by the vehicle-projects capability (with a confirmation when its textures hold artwork), and the header counts are updated

#### Scenario: The last vehicle stays
- **WHEN** a project has a single vehicle
- **THEN** Remove from Project in its ⋯ menu is disabled, with a tooltip saying the last vehicle of a project can't be removed

#### Scenario: Package not installed
- **WHEN** a project is opened on a computer where its vehicle's package is not installed
- **THEN** the vehicle's card shows its recorded name, version and textures, and hovering its version says the package isn't installed

### Requirement: Texture rows
Each vehicle card SHALL list the vehicle's textures in project order, under a **Main textures** heading for its painted main textures and an **Accessories** heading for its accessories (the heading is left out when the vehicle paints no accessory). Each texture row SHALL show:
- a **thumbnail** of the texture's artwork drawn over its template, updated when the artwork changes;
- its **name**;
- its **kind**, "Main" or "Accessory", and its **size** in pixels ("Main · 4096", "Accessory · 1024");
- when an update flagged the texture, a warning icon with the text "Layout changed" or "Not in this version".

The texture that is active in the Workshop SHALL be highlighted, without relying on color alone; no texture is highlighted while a symbol is being edited.

Clicking a texture row, or pressing Enter or Space while it has keyboard focus, SHALL make that texture the active surface and show the Workshop space. Each row SHALL be named for assistive technologies as "<vehicle> › <texture>".

#### Scenario: Rows of a truck
- **WHEN** a project holds the sample truck 1.1.0 with both main textures and every accessory
- **THEN** its card lists "Standard cab" and "High roof" (each "Main · 4096") under Main textures, then "Chassis" ("Accessory · 4096"), "Cab accessories" and "Side skirts" ("Accessory · 1024") under Accessories

#### Scenario: Opening a texture in the Workshop
- **WHEN** the user clicks "Mudflaps" in the sample trailer's card
- **THEN** the Mudflaps surface becomes active, the Workshop space is shown with it on the canvas, and the selection is cleared

#### Scenario: Thumbnail follows the artwork
- **WHEN** the user draws a red rectangle covering the Chassis texture in the Workshop and shows the Project space
- **THEN** the Chassis row's thumbnail is red

#### Scenario: Flagged texture
- **WHEN** Update Template flagged the Standard cab texture "Layout changed"
- **THEN** its row shows a warning icon with "Layout changed"

### Requirement: Mod information column
The Project space SHALL show a **Mod information** column holding, from top to bottom:
- the **shop icon** (256 × 64) and the **Mod Manager image** (276 × 162) as previews, generated or chosen as described by the mod-export capability, and updated when the artwork or the chosen pictures change. A generated picture of a project whose first texture holds no artwork SHALL be shown as a placeholder (a dashed outline saying it is generated from the first main texture) instead of a blank picture;
- the mod's **Name**, **Author**, **Version** and **Description**, read only, each with its label above it and shown as plain text (not as a field). An empty Author or Description SHALL be shown as "Not set" in the secondary text color, not as a blank;
- an **Edit in Export Mod…** button that opens the Export Mod dialog (see the mod-export capability). It SHALL be disabled when Export Mod… is, with the same reason as a tooltip;
- the editable **Game versions** field and its supported-versions hint (see Game versions field).

The project's game is not listed: it can't change once chosen and is shown in the top bar.

#### Scenario: New project
- **WHEN** a new project named "ACE Logistics" is open in the Project space
- **THEN** the Mod information column shows the Name "ACE Logistics", the Version "1.0", Author and Description "Not set", both pictures as placeholders saying they are generated from the first main texture (it holds no artwork yet), and an empty, editable Game versions field

#### Scenario: Version follows the mod settings
- **WHEN** the user clicks Edit in Export Mod…, sets the Version to "1.2" and exports
- **THEN** the Mod information column shows the Version "1.2"

#### Scenario: Mod settings are read only here
- **WHEN** the user clicks the Name shown in the Mod information column
- **THEN** no text cursor appears and the Name can't be typed in

#### Scenario: Chosen Mod Manager image
- **WHEN** the user chose a picture as the Mod Manager image in Export Mod… and exported
- **THEN** the Mod information column previews that picture as the Mod Manager image

### Requirement: Game versions field
The Mod information column SHALL hold a **Game versions** field: the game versions the mod is made for, typed as the game writes them and separated by commas (`1.56.*, 1.57.*`). It is empty for a new project, with the hint `1.56.*, 1.57.*`.
- Pressing Enter or leaving the field SHALL record the change as one undo step, "Edit Game Versions". Escape SHALL restore the previous value and record nothing.
- The list SHALL be kept in the order typed, without blanks and empty entries.
- Export Mod copies it into the manifest (see the mod-export capability).

Under the field, the column SHALL show the game versions every vehicle of the project supports, from the version ranges their packages give:
- their overlap written as a range ("Supported by every vehicle: >=1.56, <1.58");
- "any version" when no vehicle limits them;
- "No game version is supported by every vehicle" when the ranges don't overlap;
- nothing when no vehicle has its game data.

#### Scenario: Editing the game versions
- **WHEN** the user types `1.56.*, 1.57.*` in the Game versions field and presses Enter
- **THEN** the project's Game versions are `1.56.*` and `1.57.*`, the status bar shows "Unsaved changes", and Undo restores the empty field

#### Scenario: Blanks and empty entries dropped
- **WHEN** the user types ` 1.57.* ,, 1.56.*` and leaves the field
- **THEN** the project's Game versions are `1.57.*` then `1.56.*`, and the field shows `1.57.*, 1.56.*`

#### Scenario: Escape restores
- **WHEN** the project's Game versions are `1.56.*`, and the user types `1.57.*` in the field and presses Escape
- **THEN** the field shows `1.56.*` again and the undo history has no new step

#### Scenario: Versions supported by the fleet
- **WHEN** a project holds a vehicle supporting `>=1.53, <1.58` and another supporting `^1.56`
- **THEN** the column shows ">=1.56, <1.58" under the Game versions field

#### Scenario: Vehicles that never run together
- **WHEN** a project holds the sample truck (`>=1.56`) and a custom vehicle supporting `<1.55`
- **THEN** the column shows that no game version is supported by every vehicle under the Game versions field

#### Scenario: No vehicle limits the versions
- **WHEN** a project holds only a custom vehicle created with no game versions
- **THEN** the column shows "any version" under the Game versions field
