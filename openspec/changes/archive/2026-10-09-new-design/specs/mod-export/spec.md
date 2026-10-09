## MODIFIED Requirements

### Requirement: Export Mod dialog
Export › Export Mod… (Cmd/Ctrl+E) SHALL be available when a project is open, except while a symbol is being edited. It SHALL also be reached from:
- the **Export…** button of the top bar, shown in every space (see the workspace-spaces capability);
- **Edit in Export Mod…** in the Mod information column of the Project space (see the project-screen capability), the place where the mod settings are shown when the dialog is closed.

They SHALL be enabled and disabled together, with the same reason as a tooltip.

The dialog SHALL be titled "Export Mod" with the project's name and game below the title, and SHALL hold, from top to bottom:
- **the mod settings** (see Mod settings), each with its label above its field: Name, Version, Author, Description, Price and Unlock level;
- **Advanced**, a collapsed section holding the internal name and its help. It SHALL open by itself while a problem concerns the internal name;
- **the images:** the shop icon and the Mod Manager image (see Mod images);
- **a summary of the mod:** each vehicle of the project, with the main textures and accessories it paints. For a truck with several main textures, each main texture lists its cabins, and the cabins of textures that aren't painted are listed as "not painted";
- **the problems that block the export,** if any (see Checks before export), listed just above the buttons, each with an icon and its text;
- **Cancel** and **Export…**, the primary action.

The dialog SHALL fit German labels without truncation: buttons have free widths and labels sit above their fields.

Export… SHALL be disabled while a problem remains. Cancel (or Escape) SHALL close the dialog and leave the project unchanged, mod settings included.

#### Scenario: Opening the dialog
- **WHEN** a project named "ACE Logistics" holds the sample truck painting Standard cab, Chassis and Cab accessories, and the user presses Cmd/Ctrl+E
- **THEN** the Export Mod dialog opens with the Name "ACE Logistics", and its summary shows the TruckPaint Sample Truck with Standard cab (cabin "standard"), High roof not painted, and the accessories Chassis and Cab accessories

#### Scenario: Export… in the top bar
- **WHEN** the Brand space is shown and the user clicks Export… in the top bar
- **THEN** the Export Mod dialog opens

#### Scenario: From the Project space
- **WHEN** the user clicks Edit in Export Mod… in the Project space's Mod information column
- **THEN** the Export Mod dialog opens with the project's mod settings

#### Scenario: Advanced is collapsed
- **WHEN** the user opens Export Mod… on a project whose internal name has no problem
- **THEN** the Advanced section is collapsed and the internal name field is hidden

#### Scenario: Advanced opens on an internal name problem
- **WHEN** a project's internal name was set to "ace_logistic" (12 characters) while it held only the sample trailer, the sample truck (whose package has two main textures) was then added, and the user opens Export Mod…
- **THEN** the Advanced section is open, the problem saying the internal name can have at most 10 characters is listed above the buttons, and Export… is disabled

#### Scenario: Cancel keeps the settings
- **WHEN** the user changes the Price to 9000 in the dialog and presses Escape
- **THEN** the dialog closes, the project's price is unchanged and the undo history has no new step

### Requirement: Mod contents
The export SHALL write one `.scs` file: a ZIP archive holding the mod at its root. All texts in it SHALL be in English, whatever the interface language, except what the player typed. With `<id>` the internal name, `<type>` `truck` for a truck and `trailer_owned` for a trailer, and `<path>` the vehicle's game path, it SHALL contain:

**The mod:**
- `manifest.sii`: a `mod_package` with the Version, the Name, the Author, the category `paint_job`, `mp_mod_optional: true`, the icon `icon.jpg`, the description file `description.txt`, and a copy of the project's Game versions (see the project-screen capability): one `compatible_versions[]` line per version, in the order listed, none when the list is empty;
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

#### Scenario: Game versions in the manifest
- **WHEN** a project whose Game versions are `1.56.*, 1.57.*` is exported
- **THEN** its `manifest.sii` holds `compatible_versions[]: "1.56.*"` then `compatible_versions[]: "1.57.*"`

#### Scenario: No game versions listed
- **WHEN** a project whose Game versions are empty is exported
- **THEN** its `manifest.sii` has no `compatible_versions`

