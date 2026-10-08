## Purpose

Turns a project into the deliverable of the app: one ready-to-install Euro Truck Simulator 2 or American Truck Simulator mod that holds the paint job of the whole fleet, with its definitions, textures, shop icon and Mod Manager entry.

## ADDED Requirements

### Requirement: Export Mod dialog
Export › Export Mod… (Cmd/Ctrl+Shift+E) SHALL be available when a project is open, except while a symbol is being edited. It SHALL open a dialog with:
- **the mod settings** (see Mod settings);
- **the images:** the shop icon and the Mod Manager image (see Mod images);
- **a summary of the mod:** each vehicle of the project, with the main textures and accessories it paints. For a truck with several main textures, each main texture lists its cabins, and the cabins of textures that aren't painted are listed as "not painted";
- **the problems that block the export,** if any (see Checks before export).

Export… SHALL be disabled while a problem remains. Cancel (or Escape) SHALL close the dialog and leave the project unchanged, mod settings included.

#### Scenario: Opening the dialog
- **WHEN** a project named "ACE Logistics" holds the sample truck painting Standard cab, Chassis and Cab accessories, and the user presses Cmd/Ctrl+Shift+E
- **THEN** the Export Mod dialog opens with the Name "ACE Logistics", and its summary shows the TruckPaint Sample Truck with Standard cab (cabin "standard"), High roof not painted, and the accessories Chassis and Cab accessories

#### Scenario: Cancel keeps the settings
- **WHEN** the user changes the Price to 9000 in the dialog and presses Escape
- **THEN** the dialog closes, the project's price is unchanged and the undo history has no new step

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

Confirming the dialog with changed settings SHALL record them as one undo step, "Edit Mod Settings", before the export starts. Undo SHALL restore the previous settings. Confirming with unchanged settings SHALL add no undo step.

#### Scenario: Default settings of a new project
- **WHEN** the user opens Export Mod… on a new project named "ACE Logistics" holding the sample trailer
- **THEN** the dialog shows the Name "ACE Logistics", the Version "1.0", an empty Author and Description, the Price 5000, the Unlock level 0 and the internal name "ace_logistic" (cut to 12 characters)

#### Scenario: Shorter internal name for a truck with several cabins
- **WHEN** the user opens Export Mod… on a new project named "ACE Logistics" holding the sample truck, whose package has two main textures
- **THEN** the internal name is "ace_logist" (cut to 10 characters)

#### Scenario: Settings saved with the project
- **WHEN** the user sets the Author to "Jane" and the Price to 8000, exports, saves the project, closes it and opens it again
- **THEN** Export Mod… shows the Author "Jane" and the Price 8000

#### Scenario: Edited internal name is kept
- **WHEN** the user changes the internal name to "acelog", then changes the Name to "ACE Freight"
- **THEN** the internal name stays "acelog"

#### Scenario: Undo the settings
- **WHEN** the user changes the Version to "1.1", exports, then presses Undo
- **THEN** the mod version is "1.0" again, and the exported file is unchanged

### Requirement: Mod images
The mod SHALL carry a shop icon of 256 × 64 px and a Mod Manager image of 276 × 162 px. By default, each one SHALL be generated from the project's first main texture, the first surface: its artwork, without the template, scaled to cover the image's size and centered, over a white background.

For each image, the dialog SHALL show a preview and offer:
- **Choose Image…:** pick a PNG or JPEG file. The chosen image is stored in the project and scaled to cover the size, centered;
- **Use Generated:** go back to the generated image. It is shown only when an image was chosen.

A chosen image SHALL be saved with the mod settings and travel with the project file.

#### Scenario: Generated images
- **WHEN** the user opens Export Mod… on a project whose first texture has a red rectangle covering it
- **THEN** both previews are red

#### Scenario: Chosen Mod Manager image
- **WHEN** the user chooses a 1280 × 720 JPEG for the Mod Manager image and exports
- **THEN** the mod's Mod Manager image is that picture scaled to 288 × 162 and cropped at both sides to 276 × 162

#### Scenario: Chosen image travels with the project
- **WHEN** the user chooses an icon image, exports, saves the project and opens it on another computer
- **THEN** Export Mod… shows the chosen icon

### Requirement: Checks before export
The dialog SHALL list every problem that blocks the export, each naming what to fix, and update the list as the settings change:
- the Name is empty, or contains `"`, `\` or a line break;
- the Version or the Author contains `"`, `\` or a line break;
- the internal name is empty, contains a character other than `a`–`z`, `0`–`9` and `_`, or is too long: at most 12 characters, or at most 10 when a vehicle of the project has several main textures in its package;
- the Price is 0;
- two vehicles of the project have the same game path;
- a vehicle's game data is missing (see project-files), naming the vehicle and the package version to install.

#### Scenario: Empty name
- **WHEN** the user clears the Name
- **THEN** the dialog says the mod needs a name, and Export… is disabled

#### Scenario: Internal name too long for a truck with several cabins
- **WHEN** the project holds the sample truck, whose package has two main textures, and the internal name is "ace_logistic" (12 characters)
- **THEN** the dialog says the internal name can have at most 10 characters, and Export… is disabled

#### Scenario: Missing game data
- **WHEN** a project made by a build without game data is opened on a computer that doesn't have its vehicle's package version 1.1.0, and the user opens Export Mod…
- **THEN** the dialog says that TruckPaint Sample Truck 1.1.0 must be installed to export the mod, and Export… is disabled

### Requirement: Destination
Export… SHALL ask for the destination with a native save dialog, proposing "<Name>.scs", with the characters that can't be used in file names replaced by `-`. The dialog SHALL open in the game's mod folder when that folder exists:

| | Euro Truck Simulator 2 | American Truck Simulator |
|---|---|---|
| Windows | `Documents\Euro Truck Simulator 2\mod` | `Documents\American Truck Simulator\mod` |
| macOS | `~/Library/Application Support/Euro Truck Simulator 2/mod` | `~/Library/Application Support/American Truck Simulator/mod` |
| Linux | `~/.local/share/Euro Truck Simulator 2/mod` | `~/.local/share/American Truck Simulator/mod` |

Otherwise it SHALL open in the folder of the last mod export of the session, or the system's default. Cancelling the save dialog SHALL write nothing. The mod settings are still recorded, since they were confirmed.

#### Scenario: Exporting into the game
- **WHEN** an ETS2 project named "ACE Logistics" is exported on Windows, with the folder `Documents\Euro Truck Simulator 2\mod` present
- **THEN** the save dialog opens in that folder and proposes "ACE Logistics.scs"

#### Scenario: No game folder
- **WHEN** no ATS mod folder exists and the user exports an ATS project for the first time in the session
- **THEN** the save dialog opens in the system's default folder

### Requirement: Mod contents
The export SHALL write one `.scs` file: a ZIP archive holding the mod at its root. All texts in it SHALL be in English, whatever the interface language, except what the player typed. With `<id>` the internal name, `<type>` `truck` for a truck and `trailer_owned` for a trailer, and `<path>` the vehicle's game path, it SHALL contain:

**The mod:**
- `manifest.sii`: a `mod_package` with the Version, the Name, the Author, the category `paint_job`, `mp_mod_optional: true`, the icon `icon.jpg` and the description file `description.txt`;
- `icon.jpg`: the Mod Manager image;
- `description.txt`: the Description, followed by "Vehicles supported:" and the name of each vehicle. When vehicles require other mods, it also has "Requires:" and the name and version of each one;
- `material/ui/accessory/<id>_icon.mat`, `.tobj` and `.dds`: the shop icon.

**For each vehicle:**
- `def/vehicle/<type>/<path>/paint_job/<id>_settings.sui`, shared by the vehicle's paint jobs. It holds:
  - the Name, the Price, the Unlock level, the icon `<id>_icon` and `airbrush: true`;
  - `alternate_uvset: true` when the package sets `alt_uv`;
  - `base_color_locked: false` when the package sets `colour_picker`.
- one paint job per painted main texture, in `def/vehicle/<type>/<path>/paint_job/<unit>.sii`. It is an `accessory_paint_job_data` named `<unit>.<path>.paint_job` that includes the settings and points to the texture:
  - `<unit>` is `<id>` when the vehicle's package has one main texture;
  - otherwise it is `<id>_` followed by the main texture's letter in package order (`a` for the first, `b` for the second…);
  - the paint job lists a `suitable_for[]` entry `<cabin>.<path>.cabin` for each cabin (game id) of the main texture. A main texture without game ids has no `suitable_for`, so it is painted on every cabin;
  - main textures the project doesn't paint get no paint job.
- when the vehicle paints accessories, `def/vehicle/<type>/<path>/paint_job/accessory/<unit>.sii` for each of its paint jobs. It holds one `simple_paint_job_data` per painted accessory, pointing to its texture and listing each of its game ids in `acc_list[]`;
- for each painted texture, `vehicle/<type>/upgrade/paintjob/<id>/<path>/<texture id>.dds` and a `.tobj` beside it that points to it.

**The textures:**
- each texture SHALL be rendered at its full size with the same renderer and fidelity as Export Texture;
- each one SHALL be written as DDS, BC3 (DXT5) with a full mipmap chain;
- areas without artwork SHALL stay transparent;
- templates SHALL never be included.

#### Scenario: Truck with two cabin layouts
- **WHEN** a project with internal name "ace" paints the sample truck's Standard cab and High roof, the Chassis and the Side skirts, and is exported
- **THEN** the mod has:
  - `def/vehicle/truck/truckpaint.sample/paint_job/ace_a.sii`, with `accessory_paint_job_data: ace_a.truckpaint.sample.paint_job` and `suitable_for[]: "standard.truckpaint.sample.cabin"`;
  - `ace_b.sii` for `high_roof`;
  - `accessory/ace_a.sii` and `accessory/ace_b.sii`, each with a `simple_paint_job_data` listing `chassis.sample` and another listing `sideskirt.sample`;
  - the textures `vehicle/truck/upgrade/paintjob/ace/truckpaint.sample/standard.dds`, `high_roof.dds`, `chassis.dds` and `side_skirts.dds`, each with its `.tobj`.

#### Scenario: Unpainted cabin
- **WHEN** the project paints only the sample truck's High roof among its main textures
- **THEN** the mod has `ace_b.sii` and no `ace_a.sii`, and no texture for Standard cab

#### Scenario: Trailer
- **WHEN** a project with internal name "ace" holds the sample trailer painting Base and Mudflaps
- **THEN** the mod has `def/vehicle/trailer_owned/truckpaint.sample_trailer/paint_job/ace.sii` without `suitable_for`, and `accessory/ace.sii` listing `r_mudflap.sample`

#### Scenario: Fleet in one mod
- **WHEN** a project holds the sample truck and the sample trailer and is exported
- **THEN** one `.scs` file holds the paint jobs of both vehicles, one `manifest.sii`, and a description listing "TruckPaint Sample Truck" and "TruckPaint Sample Trailer"

#### Scenario: Texture size and format
- **WHEN** a project painting the sample truck's 1024 px Side skirts is exported
- **THEN** `side_skirts.dds` declares DXT5, 1024 × 1024 and 11 mipmap levels

#### Scenario: Alternate UV set
- **WHEN** a vehicle whose package sets `alt_uv` is exported
- **THEN** its `<id>_settings.sui` contains `alternate_uvset: true`

### Requirement: Background export with progress
Rendering, encoding and writing SHALL run without freezing the editor. While they run, the dialog SHALL show the progress over all the mod's textures and a Cancel button. Cancelling SHALL stop the export and leave no file at the destination, and an existing file there SHALL be left unchanged.

When the export finishes, the dialog SHALL close and a confirmation naming the file SHALL be shown. When it fails, a message SHALL name the file and the reason.

Apart from recording the mod settings, exporting SHALL NOT modify the project or its save state.

#### Scenario: Cancel during export
- **WHEN** the user cancels while the third of five textures is being rendered
- **THEN** the export stops and no `.scs` file is created

#### Scenario: Replacing an earlier export
- **WHEN** the user exports again to "ACE Logistics.scs", which already exists, and confirms the replacement
- **THEN** the file is replaced only once the new mod is complete

#### Scenario: Export of a saved project with unchanged settings
- **WHEN** a saved project is exported without changing its mod settings
- **THEN** the status bar still shows "Saved" and the undo history is unchanged
