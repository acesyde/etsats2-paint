## MODIFIED Requirements

### Requirement: Export Mod dialog
Export › Export Mod… (Cmd/Ctrl+E) SHALL be available when a project is open, except while a symbol is being edited. It SHALL also be reached from the **Export…** button of the top bar, shown in every space (see the workspace-spaces capability). They SHALL be enabled and disabled together, with the same reason as a tooltip.

The dialog SHALL edit nothing: the mod settings and pictures are edited in the Project space (see the project-screen capability). It SHALL be titled "Export Mod" with the project's name and game below the title, and SHALL hold, from top to bottom:
- **a summary of the mod:** each vehicle of the project, with the main textures and accessories it paints. For a truck with several main textures, each main texture lists its cabins, and the cabins of textures that aren't painted are listed as "not painted";
- **the destination** (see Destination), under its label, with **Change…**;
- **the problems that block the export,** if any (see Checks before export), listed just above the buttons, each with an icon and its text, and a **Show** action;
- **the warnings about the textures,** if any (see Checks before export), after the problems, each with its own icon and its text;
- **Cancel** and **Export**, the primary action.

Show on a problem SHALL close the dialog, show the Project space and take the player to where the problem is fixed, as the project-screen capability describes (Problems before exporting): the setting's field, or the vehicle's card.

The dialog SHALL fit German labels without truncation: buttons have free widths and labels sit above their values.

Export SHALL be disabled while a problem remains; warnings SHALL NOT disable it. Cancel (or Escape) SHALL close the dialog. Opening the dialog, closing it, following a problem and exporting SHALL NOT change the project, its undo history or its save state.

#### Scenario: Opening the dialog
- **WHEN** a project named "ACE Logistics" holds the sample truck painting Standard cab, Chassis and Cab accessories, and the user presses Cmd/Ctrl+E
- **THEN** the Export Mod dialog opens with "ACE Logistics" and the game below its title, its summary shows the TruckPaint Sample Truck with Standard cab (cabin "standard"), High roof not painted, and the accessories Chassis and Cab accessories, and it shows the destination "ACE Logistics.scs"

#### Scenario: Export… in the top bar
- **WHEN** the Brand space is shown and the user clicks Export… in the top bar
- **THEN** the Export Mod dialog opens

#### Scenario: From the Project space
- **WHEN** the Project space is shown and the user clicks Export… in the top bar
- **THEN** the Export Mod dialog opens; the Mod information column has no button of its own for it

#### Scenario: Nothing to edit in the dialog
- **WHEN** the Export Mod dialog is open
- **THEN** it shows no field for the Name, Version, Author, Description, Price, Unlock level or internal name, and no picture

#### Scenario: Advanced is collapsed
- **WHEN** the user opens Export Mod… on a project whose internal name has no problem
- **THEN** the dialog has no Advanced section: the internal name is shown only under the collapsed Advanced section of the Project space's Mod information column

#### Scenario: A problem leads to its field
- **WHEN** the project's Price is 0, the user opens Export Mod… and clicks Show on the problem saying the price must be more than 0
- **THEN** the dialog closes, the Project space is shown and its Price field has keyboard focus

#### Scenario: Advanced opens on an internal name problem
- **WHEN** a project's internal name was set to "ace_logistic" (12 characters) while it held only the sample trailer, the sample truck (whose package has two main textures) was then added, and the user opens Export Mod…
- **THEN** the problem saying the internal name can have at most 10 characters is listed above the buttons and Export is disabled; clicking its Show closes the dialog and gives the internal name field, under the opened Advanced section of the Project space, keyboard focus

#### Scenario: Cancel keeps the settings
- **WHEN** the user opens Export Mod… on a saved project and presses Escape
- **THEN** the dialog closes, the status bar still shows "Saved" and the undo history has no new step

#### Scenario: Warnings don't block
- **WHEN** a project with no problem has an Empty texture and a texture To check, and the user opens Export Mod…
- **THEN** the dialog lists both warnings, and Export is enabled

### Requirement: Mod settings
The mod settings SHALL be part of the project and saved with it:

| Setting | Default | Used for |
|---|---|---|
| Name | the project name | the paint job's name in the shop and the mod's name in the Mod Manager |
| Version | `1.0` | the mod's version in the Mod Manager |
| Author | empty | the mod's author in the Mod Manager |
| Description | empty | the mod's description in the Mod Manager |
| Price | 5000 | the paint job's price in the game's currency |
| Unlock level | 0 | the player level that unlocks the paint job |
| Internal name | derived from the Name | the identifier of the paint job in the game's definitions and file paths |

The internal name SHALL be derived from the Name: lowercased, each run of characters other than `a`–`z` and `0`–`9` replaced by `_`, `_` trimmed at both ends, and cut to its maximum length (see Checks before export). A Name that gives nothing gives `paintjob`. While the player hasn't edited it, the internal name follows the Name and the vehicles of the project. Once edited, it is kept as typed.

The settings SHALL be edited in the Mod information column of the Project space, each committed edit recorded as its own undo step, "Edit Mod Settings" (see the project-screen capability, Editing the mod settings). Undo SHALL restore the previous value. Exporting SHALL NOT record them: the export uses the project's settings as they are.

#### Scenario: Default settings of a new project
- **WHEN** a new project named "ACE Logistics" holding the sample trailer is shown in the Project space
- **THEN** the Mod information column shows the Name "ACE Logistics", the Version "1.0", an empty Author and Description, the Price 5000, the Unlock level 0 and, under Advanced, the internal name "ace_logistic" (cut to 12 characters)

#### Scenario: Shorter internal name for a truck with several cabins
- **WHEN** a new project named "ACE Logistics" holds the sample truck, whose package has two main textures
- **THEN** the internal name is "ace_logist" (cut to 10 characters)

#### Scenario: Settings saved with the project
- **WHEN** the user sets the Author to "Jane" and the Price to 8000 in the Project space, saves the project, closes it and opens it again
- **THEN** the Mod information column shows the Author "Jane" and the Price 8000

#### Scenario: Edited internal name is kept
- **WHEN** the user commits "acelog" in the internal name field, then changes the Name to "ACE Freight"
- **THEN** the internal name stays "acelog"

#### Scenario: Undo the settings
- **WHEN** the user commits the Version "1.1" in the Project space, exports, then presses Undo
- **THEN** the mod version is "1.0" again, and the exported file is unchanged

#### Scenario: Exporting records nothing
- **WHEN** the user commits the Version "1.1", exports, then presses Undo once
- **THEN** the step undone is "Edit Mod Settings" for the Version, the export having added no step

### Requirement: Mod images
The mod SHALL carry a shop icon of 256 × 64 px and a Mod Manager image of 276 × 162 px. By default, each one SHALL be generated from the project's first main texture, the first surface: its artwork, without the template, scaled to cover the image's size and centered, over a white background.

Each image SHALL be shown, chosen and reset to the generated one in the Mod information column of the Project space (see the project-screen capability, Mod pictures in the Project space), from a PNG or JPEG file picked with Choose Image… or dropped on its preview. The chosen image is stored in the project and scaled to cover the size, centered.

A chosen image SHALL be saved with the mod settings and travel with the project file.

#### Scenario: Generated images
- **WHEN** a project's first texture has a red rectangle covering it and the Project space is shown
- **THEN** both pictures of the Mod information column are red

#### Scenario: Chosen Mod Manager image
- **WHEN** the user chooses a 1280 × 720 JPEG for the Mod Manager image and exports
- **THEN** the mod's Mod Manager image is that picture scaled to 288 × 162 and cropped at both sides to 276 × 162

#### Scenario: Chosen image travels with the project
- **WHEN** the user chooses an icon image, saves the project and opens it on another computer
- **THEN** the Mod information column shows the chosen icon, and the exported mod uses it

### Requirement: Checks before export
The dialog SHALL list every problem that blocks the export, each naming what to fix, and update the list as the project changes:
- the Name is empty, or contains `"`, `\` or a line break;
- the Version or the Author contains `"`, `\` or a line break;
- the internal name is empty, contains a character other than `a`–`z`, `0`–`9` and `_`, or is too long: at most 12 characters, or at most 10 when a vehicle of the project has several main textures in its package;
- the Price is 0;
- two vehicles of the project have the same game path;
- a vehicle's game data is missing (see project-files), naming the vehicle and the package version to install;
- one of the project's Game versions isn't written like the game's versions: two to four numbers separated by dots, the last one possibly replaced by `*` (`1.56`, `1.56.*`, `1.56.2`, `1.56.2.*`), naming it;
- one of the project's Game versions isn't supported by a vehicle's package, naming the version, the vehicle and its range;
- the vehicles have no game version in common, naming two vehicles whose ranges don't overlap and their ranges, and asking to update or remove one of them.

The same problems SHALL be shown in the Project space, under their field and in Before exporting (see the project-screen capability, Problems before exporting).

The dialog SHALL also list the project's warnings about its textures, the textures To check and the Empty textures, as the texture-status capability describes (Before exporting warnings). They don't block the export: the Empty textures are exported transparent, and the textures not in their vehicle's version are left out of the mod, as before.

#### Scenario: Empty name
- **WHEN** the user clears the Name in the Project space and opens Export Mod…
- **THEN** the dialog says the mod needs a name, and Export is disabled

#### Scenario: Internal name too long for a truck with several cabins
- **WHEN** the project holds the sample truck, whose package has two main textures, and the internal name is "ace_logistic" (12 characters)
- **THEN** the dialog says the internal name can have at most 10 characters, and Export is disabled

#### Scenario: Missing game data
- **WHEN** a project made by a build without game data is opened on a computer that doesn't have its vehicle's package version 1.1.0, and the user opens Export Mod…
- **THEN** the dialog says that TruckPaint Sample Truck 1.1.0 must be installed to export the mod, and Export is disabled

#### Scenario: Badly written game version
- **WHEN** the project's Game versions are `1.56.x` and the user opens Export Mod…
- **THEN** the dialog says that 1.56.x isn't a game version and gives 1.56.* as an example, and Export is disabled

#### Scenario: Game version a vehicle doesn't support
- **WHEN** the project holds the sample truck (`>=1.56`), its Game versions are `1.55.*, 1.56.*`, and the user opens Export Mod…
- **THEN** the dialog says that TruckPaint Sample Truck (>=1.56) doesn't support 1.55.*, and Export is disabled

#### Scenario: No common game version
- **WHEN** the project holds the sample truck (`>=1.56`) and a custom vehicle "Old Hauler" supporting `<1.55`, and the user opens Export Mod…
- **THEN** the dialog says that TruckPaint Sample Truck (>=1.56) and Old Hauler (<1.55) have no game version in common and that one of them must be updated or removed, and Export is disabled

#### Scenario: Empty textures warned
- **WHEN** a project's High roof texture is Empty and every other texture is Modified, and the user opens Export Mod…
- **THEN** the dialog warns that High roof is empty and exported with the game's color, and the exported mod still holds a transparent High roof texture

#### Scenario: Texture to check warned
- **WHEN** a project's Curtain body 13.6 m texture is To check because its layout changed, and the user opens Export Mod…
- **THEN** the dialog warns "Curtain body 13.6 m: layout changed", and Export stays enabled when there is no problem

### Requirement: Destination
The dialog SHALL show the destination: the full path of the file the mod will be written to, in the monospace face, shortened from its start with an ellipsis when it doesn't fit (so the file name stays visible), and shown in full on hover. When the dialog opens, it SHALL propose "<Name>.scs", with the characters that can't be used in file names replaced by `-`, in the game's mod folder when that folder exists:

| | Euro Truck Simulator 2 | American Truck Simulator |
|---|---|---|
| Windows | `Documents\Euro Truck Simulator 2\mod` | `Documents\American Truck Simulator\mod` |
| macOS | `~/Library/Application Support/Euro Truck Simulator 2/mod` | `~/Library/Application Support/American Truck Simulator/mod` |
| Linux | `~/.local/share/Euro Truck Simulator 2/mod` | `~/.local/share/American Truck Simulator/mod` |

Otherwise it SHALL propose the folder of the last mod export of the session, or else the user's Documents folder (their home folder when there is none).

**Change…** SHALL open the native save dialog in the destination's folder, proposing its file name. Choosing a file SHALL make it the destination (with `.scs` added when the user left it out); cancelling SHALL keep the destination.

**Export** SHALL write to the destination. When a file already exists there, the dialog SHALL first ask whether to replace it, naming the file, with **Replace** and **Cancel**; Cancel SHALL write nothing and keep the dialog open. A file chosen with Change… whose replacement the native save dialog already confirmed SHALL NOT be asked about again.

#### Scenario: Exporting into the game
- **WHEN** the user opens Export Mod… on an ETS2 project named "ACE Logistics" on Windows, with the folder `Documents\Euro Truck Simulator 2\mod` present
- **THEN** the dialog shows the destination `Documents\Euro Truck Simulator 2\mod\ACE Logistics.scs`, and Export writes the mod there

#### Scenario: No game folder
- **WHEN** no ATS mod folder exists and the user opens Export Mod… on an ATS project named "Blue Line" for the first time in the session
- **THEN** the destination is "Blue Line.scs" in the user's Documents folder

#### Scenario: Changing the destination
- **WHEN** the user clicks Change… and chooses `ace` in another folder
- **THEN** the destination shows `ace.scs` in that folder, and Export writes the mod there

#### Scenario: Cancelling Change…
- **WHEN** the user clicks Change… and cancels the save dialog
- **THEN** the destination is unchanged and nothing is written

#### Scenario: Replacing an existing file
- **WHEN** the proposed destination "ACE Logistics.scs" already exists and the user clicks Export
- **THEN** the dialog asks whether to replace "ACE Logistics.scs"; Cancel writes nothing and keeps the dialog open, Replace starts the export

### Requirement: Background export with progress
Rendering, encoding and writing SHALL run without freezing the editor. While they run, the dialog SHALL show the progress over all the mod's textures and a Cancel button. Cancelling SHALL stop the export and leave no file at the destination, and an existing file there SHALL be left unchanged.

When the export finishes, the dialog SHALL close and a confirmation naming the file SHALL be shown. When it fails, a message SHALL name the file and the reason.

Exporting SHALL NOT modify the project, its undo history or its save state.

#### Scenario: Cancel during export
- **WHEN** the user cancels while the third of five textures is being rendered
- **THEN** the export stops and no `.scs` file is created

#### Scenario: Replacing an earlier export
- **WHEN** the user exports again to "ACE Logistics.scs", which already exists, and confirms the replacement
- **THEN** the file is replaced only once the new mod is complete

#### Scenario: Export of a saved project with unchanged settings
- **WHEN** a saved project is exported
- **THEN** the status bar still shows "Saved" and the undo history is unchanged
