## ADDED Requirements

### Requirement: Texture filter
The Vehicles header SHALL hold a segmented filter with three options:
- **All:** every texture;
- **To do:** the textures that are Empty or To check (see the texture-status capability);
- **To check:** the textures that are To check.

A filter SHALL hide texture rows, never vehicle cards:
- a vehicle card lists only its matching textures, and leaves out the Main textures or Accessories heading that would list none;
- a vehicle with no matching texture keeps its card header and shows "Nothing to do" in place of its textures.

The filter SHALL NOT change the header counts. A texture that stops matching (for example after Mark as Checked) SHALL leave the list at once. The filter is All when a project opens; it SHALL be kept while switching spaces and textures, and is not saved with the project nor in the preferences.

#### Scenario: Only what is left to do
- **WHEN** the sample truck has Standard cab and High roof Empty and its accessories Modified, and the user chooses To do
- **THEN** the truck's card lists Standard cab and High roof under Main textures and shows no Accessories heading

#### Scenario: Nothing to do on a vehicle
- **WHEN** every texture of the sample trailer is Modified and the user chooses To check
- **THEN** the trailer's card shows its header and "Nothing to do", and the header counts are unchanged

#### Scenario: Filter kept across spaces
- **WHEN** the user chooses To check, opens a texture in the Workshop and shows the Project space again
- **THEN** the filter is still To check

## MODIFIED Requirements

### Requirement: Project space layout
The Project space SHALL use the full width under the top bar (see the workspace-spaces capability), with two areas:
- **Vehicles**, on the left: a header and one card per vehicle of the project, in project order, scrolling when they don't fit;
- **Mod information**, a column on the right (see Mod information column).

The Vehicles header SHALL show the title "Vehicles"; the number of vehicles and the number of textures of the project, followed by the number of Modified and To check textures (see the texture-status capability), each left out when it is zero ("2 vehicles · 9 textures · 4 modified · 1 to check", with the plural forms of the interface language); the texture filter (see Texture filter); and an **Add Vehicle…** button that runs Vehicle › Add Vehicle… (see the vehicle-projects capability).

The space SHALL never show an empty panel: a project always holds at least one vehicle.

#### Scenario: Header counts
- **WHEN** a project holds the sample truck with five textures and the sample trailer with four textures, four of them Modified and one To check, and the Project space is shown
- **THEN** the header reads "Vehicles", "2 vehicles · 9 textures · 4 modified · 1 to check", and offers the All / To do / To check filter and Add Vehicle…

#### Scenario: One vehicle
- **WHEN** a project holds only the sample trailer with Base and Mudflaps, both Empty
- **THEN** the header reads "1 vehicle · 2 textures"

#### Scenario: Counts follow the work
- **WHEN** a new project's header reads "1 vehicle · 5 textures" and the user draws a rectangle on Chassis and shows the Project space
- **THEN** the header reads "1 vehicle · 5 textures · 1 modified"

#### Scenario: Adding a vehicle from the Project space
- **WHEN** the user clicks Add Vehicle… in the Vehicles header
- **THEN** the Add Vehicle dialog opens, listing only the vehicles of the project's game that aren't in the project

### Requirement: Texture rows
Each vehicle card SHALL list the vehicle's textures in project order, under a **Main textures** heading for its painted main textures and an **Accessories** heading for its accessories (the heading is left out when the vehicle paints no accessory, or when the texture filter hides all of them). Each texture row SHALL show:
- a **thumbnail** of the texture's artwork drawn over its template, updated when the artwork changes;
- its **name**;
- its **kind**, "Main" or "Accessory", and its **size** in pixels ("Main · 4096", "Accessory · 1024");
- its **state** (see the texture-status capability): a small dot followed by the state's label, "Empty", "Modified" or "To check". The label and dot of Empty SHALL be muted, those of Modified in the primary text color, and those of To check in the signal color. Hovering a To check state SHALL say why: "Layout changed in <version>" or "Not in this version".

The texture that is active in the Workshop SHALL be highlighted, without relying on color alone; no texture is highlighted while a symbol is being edited.

Clicking a texture row, or pressing Enter or Space while it has keyboard focus, SHALL make that texture the active surface and show the Workshop space. Each row SHALL be named for assistive technologies as "<vehicle> › <texture>", followed by its state.

#### Scenario: Rows of a truck
- **WHEN** a project holds the sample truck 1.1.0 with both main textures and every accessory, nothing drawn yet
- **THEN** its card lists "Standard cab" and "High roof" (each "Main · 4096") under Main textures, then "Chassis" ("Accessory · 4096"), "Cab accessories" and "Side skirts" ("Accessory · 1024") under Accessories, each with the state "Empty"

#### Scenario: Opening a texture in the Workshop
- **WHEN** the user clicks "Mudflaps" in the sample trailer's card
- **THEN** the Mudflaps surface becomes active, the Workshop space is shown with it on the canvas, and the selection is cleared

#### Scenario: Thumbnail follows the artwork
- **WHEN** the user draws a red rectangle covering the Chassis texture in the Workshop and shows the Project space
- **THEN** the Chassis row's thumbnail is red and its state reads "Modified"

#### Scenario: Flagged texture
- **WHEN** Update Template to version 1.3.0 flagged the Standard cab texture "Layout changed"
- **THEN** its row's state reads "To check" in the signal color, hovering it says "Layout changed in 1.3.0", and its accessible name ends with "To check"

### Requirement: Mod information column
The Project space SHALL show a **Mod information** column holding, from top to bottom:
- the **shop icon** (256 × 64) and the **Mod Manager image** (276 × 162) as previews, generated or chosen as described by the mod-export capability, and updated when the artwork or the chosen pictures change. A generated picture of a project whose first texture holds no artwork SHALL be shown as a placeholder (a dashed outline saying it is generated from the first main texture) instead of a blank picture;
- the mod's **Name**, **Author**, **Version** and **Description**, read only, each with its label above it and shown as plain text (not as a field). An empty Author or Description SHALL be shown as "Not set" in the secondary text color, not as a blank;
- an **Edit in Export Mod…** button that opens the Export Mod dialog (see the mod-export capability). It SHALL be disabled when Export Mod… is, with the same reason as a tooltip;
- the editable **Game versions** field and its supported-versions hint (see Game versions field);
- **Before exporting:** the project's warnings (see the texture-status capability, Before exporting warnings), each with a dot in its state's color and its text. A To check line SHALL offer **Open**, which makes that texture active and shows the Workshop. The Empty line SHALL expand to list the Empty textures, each with Open; when there is only one, its Open is on the line itself.

The project's game is not listed: it can't change once chosen and is shown in the top bar.

The problems that block the export are not listed here: they concern the mod settings, edited in Export Mod…, which lists them.

#### Scenario: New project
- **WHEN** a new project named "ACE Logistics" is open in the Project space
- **THEN** the Mod information column shows the Name "ACE Logistics", the Version "1.0", Author and Description "Not set", both pictures as placeholders saying they are generated from the first main texture (it holds no artwork yet), an empty, editable Game versions field, and under Before exporting the line saying its textures are empty, exported with the game's color

#### Scenario: Version follows the mod settings
- **WHEN** the user clicks Edit in Export Mod…, sets the Version to "1.2" and exports
- **THEN** the Mod information column shows the Version "1.2"

#### Scenario: Mod settings are read only here
- **WHEN** the user clicks the Name shown in the Mod information column
- **THEN** no text cursor appears and the Name can't be typed in

#### Scenario: Chosen Mod Manager image
- **WHEN** the user chose a picture as the Mod Manager image in Export Mod… and exported
- **THEN** the Mod information column previews that picture as the Mod Manager image

#### Scenario: Opening a texture to check
- **WHEN** Before exporting lists "Curtain body 13.6 m: layout changed" and the user clicks its Open
- **THEN** Curtain body 13.6 m becomes the active texture and the Workshop space is shown

#### Scenario: Opening an empty texture
- **WHEN** Before exporting lists "4 textures empty, exported with the game's color", and the user expands it and clicks Open next to High roof
- **THEN** High roof becomes the active texture and the Workshop space is shown
