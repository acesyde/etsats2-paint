## MODIFIED Requirements

### Requirement: Mod information column
The Project space SHALL show a **Mod information** column holding, from top to bottom:
- the **shop icon** (256 × 64) and the **Mod Manager image** (276 × 162), each under its label, as a preview generated or chosen as described by the mod-export capability, and updated when the artwork or the chosen pictures change. A generated picture of a project whose first texture holds no artwork SHALL be shown as a placeholder (a dashed outline saying it is generated from the first main texture) instead of a blank picture. Under each picture, its controls (see Mod pictures in the Project space);
- the mod settings of the mod-export capability as editable fields, each with its label above it: **Name**; **Author** and **Version** side by side; the **Game versions** (see Game versions field); **Description**, on several lines; **Price** and **Unlock level** side by side. The Version, the Price and the Unlock level SHALL be shown in the monospace face;
- **Advanced**, a collapsed section holding the **internal name** and its help. It SHALL open by itself while a problem concerns the internal name;
- **Before exporting** (see Problems before exporting): the problems that block the export, then the project's warnings (see the texture-status capability, Before exporting warnings), each warning with a dot in its state's color and its text. A To check line SHALL offer **Open**, which makes that texture active and shows the Workshop. The Empty line SHALL expand to list the Empty textures, each with Open; when there is only one, its Open is on the line itself.

How a field is edited and recorded is described by Editing the mod settings.

The project's game is not listed: it can't change once chosen and is shown in the top bar. The column SHALL scroll when its content is taller than the window.

#### Scenario: New project
- **WHEN** a new project named "ACE Logistics" is open in the Project space
- **THEN** the Mod information column shows both pictures as placeholders saying they are generated from the first main texture (it holds no artwork yet), the Name field "ACE Logistics", an empty Author field, the Version field "1.0", no game version and + Add, an empty Description field, the Price 5000, the Unlock level 0, a collapsed Advanced section, and under Before exporting the line saying its textures are empty, exported with the game's color

#### Scenario: Mod settings are edited here
- **WHEN** the user clicks the Name field of the Mod information column, types " Freight" at its end and presses Enter
- **THEN** the mod's Name is "ACE Logistics Freight"

#### Scenario: No Edit in Export Mod
- **WHEN** the Project space is shown
- **THEN** the Mod information column has no Edit in Export Mod… button, and Export… stays in the top bar

#### Scenario: Version follows the mod settings
- **WHEN** the user types "1.2" in the Version field, presses Enter and exports the mod
- **THEN** the Version field still shows "1.2" and the mod's manifest has the version "1.2"

#### Scenario: Mod settings are read only here
- **WHEN** the user clicks the supported-versions hint or a picture's size in the Mod information column
- **THEN** no text cursor appears: only the settings are fields, and the hint and sizes are read only

#### Scenario: Chosen Mod Manager image
- **WHEN** the user chose a picture as the Mod Manager image in the Mod information column
- **THEN** the column previews that picture as the Mod Manager image, and the export uses it

#### Scenario: Internal name under Advanced
- **WHEN** the user opens Advanced in the Mod information column of a new project named "ACE Logistics" holding the sample trailer
- **THEN** the internal name field shows "ace_logistic" with its help

#### Scenario: Opening a texture to check
- **WHEN** Before exporting lists "Curtain body 13.6 m: layout changed" and the user clicks its Open
- **THEN** Curtain body 13.6 m becomes the active texture and the Workshop space is shown

#### Scenario: Opening an empty texture
- **WHEN** Before exporting lists "4 textures empty, exported with the game's color", and the user expands it and clicks Open next to High roof
- **THEN** High roof becomes the active texture and the Workshop space is shown

### Requirement: Game versions field
The Mod information column SHALL hold a **Game versions** field: the game versions the mod is made for, as the game writes them (`1.56.*`), shown as one chip per version in the order listed, followed by **+ Add**. A new project has none.
- Each chip SHALL show its version in the monospace face and a remove button, named for assistive technologies "Remove <version>". Removing it SHALL record one undo step, "Edit Game Versions".
- **+ Add** SHALL turn into a short text field with the hint `1.56.*`. Pressing Enter or leaving it SHALL add what was typed at the end of the list as one undo step, "Edit Game Versions"; several versions separated by commas are added in the order typed, without blanks and empty entries. A version already listed is not added again, and nothing typed adds nothing and records nothing. Escape SHALL close the field and record nothing.
- A chip whose version isn't written like the game's versions (see Checks before export in the mod-export capability) SHALL be marked with the error color and a warning icon, never color alone, and hovering it SHALL say why.
- Export Mod copies the list into the manifest (see the mod-export capability).

Under the chips, the column SHALL show the game versions every vehicle of the project supports, from the version ranges their packages give:
- their overlap written as a range ("Supported by every vehicle: >=1.56, <1.58");
- "any version" when no vehicle limits them;
- "No game version is supported by every vehicle" when the ranges don't overlap;
- nothing when no vehicle has its game data.

#### Scenario: Editing the game versions
- **WHEN** a new project is open, the user clicks + Add, types `1.56.*, 1.57.*` and presses Enter
- **THEN** the project's Game versions are `1.56.*` then `1.57.*`, shown as two chips, the status bar shows "Unsaved changes", and one Undo removes both

#### Scenario: Blanks and empty entries dropped
- **WHEN** the user clicks + Add, types ` 1.57.* ,, 1.56.*` and leaves the field
- **THEN** the chips read `1.57.*` then `1.56.*`

#### Scenario: Removing a game version
- **WHEN** the project's Game versions are `1.56.*, 1.57.*` and the user clicks the remove button of `1.56.*`
- **THEN** the project's Game versions are `1.57.*`, and Undo brings `1.56.*` back in first place

#### Scenario: Escape restores
- **WHEN** the project's Game versions are `1.56.*`, and the user clicks + Add, types `1.57.*` and presses Escape
- **THEN** the only chip is `1.56.*` and the undo history has no new step

#### Scenario: A version already listed
- **WHEN** the project's Game versions are `1.56.*` and the user adds `1.56.*`
- **THEN** the only chip is `1.56.*` and the undo history has no new step

#### Scenario: Badly written version marked
- **WHEN** the user adds `1.56.x`
- **THEN** the chip `1.56.x` is marked with the error color and a warning icon, and hovering it says that 1.56.x isn't a game version and gives 1.56.* as an example

#### Scenario: Versions supported by the fleet
- **WHEN** a project holds a vehicle supporting `>=1.53, <1.58` and another supporting `^1.56`
- **THEN** the column shows ">=1.56, <1.58" under the Game versions chips

#### Scenario: Vehicles that never run together
- **WHEN** a project holds the sample truck (`>=1.56`) and a custom vehicle supporting `<1.55`
- **THEN** the column shows that no game version is supported by every vehicle under the Game versions chips

#### Scenario: No vehicle limits the versions
- **WHEN** a project holds only a custom vehicle created with no game versions
- **THEN** the column shows "any version" under the Game versions chips

## ADDED Requirements

### Requirement: Editing the mod settings
Each field of the Mod information column (Name, Author, Version, Description, Price, Unlock level and the internal name) SHALL change the project's mod settings when its edit is committed:
- pressing Enter in a single-line field, or leaving any field (clicking elsewhere, Tab, or opening a dialog such as Export Mod…), SHALL commit it. The Description is on several lines: Enter starts a new line, and it is committed when it is left;
- a committed change SHALL be recorded as one undo step, "Edit Mod Settings", and mark the project as having unsaved changes. Committing a value equal to the current one SHALL record nothing;
- Escape SHALL restore the field's value from before the edit, leave the field and record nothing;
- Undo and Redo SHALL show the restored value in the field at once;
- the Price SHALL take a whole number from 0 to 100 000 000 and the Unlock level from 0 to 1000; a value that isn't a number SHALL restore the previous one.

The internal name SHALL follow the Name (as the mod-export capability derives it) until a value is committed in its field; from then on it is kept as typed.

The fields SHALL stay editable while a symbol is being edited.

#### Scenario: One step per field
- **WHEN** the user types "Jane" in the Author field and presses Enter, then types 8000 in the Price field and presses Tab
- **THEN** the undo history has two new "Edit Mod Settings" steps, the first Undo restores the Price 5000 and the second an empty Author

#### Scenario: Escape restores the field
- **WHEN** the Version is "1.0", and the user types "1.1" in the Version field and presses Escape
- **THEN** the field shows "1.0", the mod version is "1.0" and the undo history has no new step

#### Scenario: Unchanged value records nothing
- **WHEN** the user clicks the Name field and leaves it without typing
- **THEN** the undo history has no new step and the save state is unchanged

#### Scenario: Description on several lines
- **WHEN** the user types "Fleet colors", presses Enter, types "Truck and trailer" in the Description field and clicks elsewhere
- **THEN** the mod's Description is the two lines, recorded as one "Edit Mod Settings" step

#### Scenario: Internal name follows the Name until edited
- **WHEN** a project holding the sample trailer has the Name "ACE Logistics" and its internal name was never edited, and the user changes the Name to "Blue Line"
- **THEN** the internal name is "blue_line"; once the user commits "acelog" in the internal name field and changes the Name to "ACE Freight", it stays "acelog"

#### Scenario: Undo shows in the field
- **WHEN** the user commits the Version "1.2" and presses Cmd/Ctrl+Z
- **THEN** the Version field shows "1.0"

#### Scenario: Export while typing
- **WHEN** the user types "ACE Freight" in the Name field and presses Cmd/Ctrl+E without leaving the field
- **THEN** the Name is committed as one "Edit Mod Settings" step and the Export Mod dialog proposes "ACE Freight.scs"

### Requirement: Mod pictures in the Project space
Under each picture of the Mod information column, the column SHALL show its size in pixels and offer:
- **Choose Image…** (for the shop icon, **Choose Icon…**): pick a PNG or JPEG file with the native open dialog;
- **Use Generated Image** (for the shop icon, **Use Generated Icon**): go back to the generated picture. It SHALL be shown only while a picture is chosen.

A PNG or JPEG file dropped from the operating system on a picture's preview SHALL become that picture. While files are dragged over the window, each preview SHALL show that it accepts a drop, and the preview under the pointer SHALL be highlighted. Files dropped elsewhere SHALL keep their current behavior.

Choosing, dropping or going back to the generated picture SHALL be recorded as one undo step, "Edit Mod Settings", and the preview SHALL update. A file that can't be read as a PNG or JPEG SHALL change nothing, and a message under that picture SHALL name the file and the reason, until the next picture change.

#### Scenario: Choosing a Mod Manager image
- **WHEN** the user clicks Choose Image… under the Mod Manager image and picks a red 1280 × 720 PNG
- **THEN** the Mod Manager image preview is red, Use Generated Image is shown under it, and one Undo goes back to the generated picture

#### Scenario: Dropping an icon
- **WHEN** the user drops a JPEG file on the shop icon's preview
- **THEN** the shop icon is that picture, recorded as one "Edit Mod Settings" step, and no image is placed on the active texture

#### Scenario: Back to the generated picture
- **WHEN** a Mod Manager image was chosen and the user clicks Use Generated Image
- **THEN** the preview shows the picture generated from the first main texture and Use Generated Image is hidden

#### Scenario: Unreadable file
- **WHEN** the user drops a text file renamed "logo.png" on the Mod Manager image's preview
- **THEN** the picture is unchanged, the undo history has no new step, and a message under the picture says logo.png can't be used and why

### Requirement: Problems before exporting
Each problem that blocks the export (see Checks before export in the mod-export capability) SHALL be shown in the Project space, with the same text as in the Export Mod dialog, an icon and the error color, never color alone, and SHALL disappear as soon as it is fixed:
- **under its field**, for a problem about a setting: the Name (empty, or a forbidden character), the Author, the Version, the internal name (Advanced then opens by itself), the Price, and the Game versions (a badly written version, or one a vehicle doesn't support);
- **in Before exporting**, for every problem, listed before the warnings and apart from them. Each line SHALL offer **Show**, named for assistive technologies after what it shows, which scrolls the column or the Vehicles area to where the problem is fixed and, for a field, gives it keyboard focus:

| Problem | Where it is fixed |
|---|---|
| Name empty or with a forbidden character | the Name field |
| Version or Author with a forbidden character | its field |
| internal name empty, with a forbidden character or too long | the internal name field, Advanced opened |
| Price 0 | the Price field |
| a badly written game version, or one a vehicle doesn't support | the Game versions field (its + Add) |
| two vehicles with the same game path | the card of the second one |
| a vehicle's game data missing | that vehicle's card |
| no game version in common | the card of the first vehicle named |

When the project has neither problem nor warning, Before exporting SHALL say "Nothing to check".

#### Scenario: Empty name shown under its field
- **WHEN** the user clears the Name field and presses Enter
- **THEN** the message saying the mod needs a name is shown under the Name field and first in Before exporting, and the top bar's Export… stays enabled

#### Scenario: Show leads to the field
- **WHEN** the Price is 0 and the user clicks Show on its line in Before exporting
- **THEN** the Price field is scrolled into view and has keyboard focus

#### Scenario: Internal name problem opens Advanced
- **WHEN** a project's internal name was set to "ace_logistic" while it held only the sample trailer, and the sample truck (whose package has two main textures) is then added
- **THEN** Advanced is open in the Mod information column, the message saying the internal name can have at most 10 characters is shown under the internal name field and in Before exporting

#### Scenario: Problem about a vehicle
- **WHEN** a project's vehicle lacks its game data and the user clicks Show on that problem's line in Before exporting
- **THEN** that vehicle's card is scrolled into view

#### Scenario: Problems before warnings
- **WHEN** a project's Price is 0 and its High roof texture is Empty
- **THEN** Before exporting lists the Price problem with the error icon, then the High roof warning with its dot

#### Scenario: Nothing to check
- **WHEN** a project has no problem and every texture is Modified
- **THEN** Before exporting says "Nothing to check"
