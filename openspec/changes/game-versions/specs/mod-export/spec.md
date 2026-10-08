## MODIFIED Requirements

### Requirement: Mod contents
The export SHALL write one `.scs` file: a ZIP archive holding the mod at its root. All texts in it SHALL be in English, whatever the interface language, except what the player typed. With `<id>` the internal name, `<type>` `truck` for a truck and `trailer_owned` for a trailer, and `<path>` the vehicle's game path, it SHALL contain:

**The mod:**
- `manifest.sii`: a `mod_package` with the Version, the Name, the Author, the category `paint_job`, `mp_mod_optional: true`, the icon `icon.jpg`, the description file `description.txt`, and a copy of the project's Game versions (see workspace-layout): one `compatible_versions[]` line per version, in the order listed, none when the list is empty;
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

### Requirement: Checks before export
The dialog SHALL list every problem that blocks the export, each naming what to fix, and update the list as the settings change:
- the Name is empty, or contains `"`, `\` or a line break;
- the Version or the Author contains `"`, `\` or a line break;
- the internal name is empty, contains a character other than `a`–`z`, `0`–`9` and `_`, or is too long: at most 12 characters, or at most 10 when a vehicle of the project has several main textures in its package;
- the Price is 0;
- two vehicles of the project have the same game path;
- a vehicle's game data is missing (see project-files), naming the vehicle and the package version to install;
- one of the project's Game versions isn't written like the game's versions: two to four numbers separated by dots, the last one possibly replaced by `*` (`1.56`, `1.56.*`, `1.56.2`, `1.56.2.*`), naming it;
- one of the project's Game versions isn't supported by a vehicle's package, naming the version, the vehicle and its range;
- the vehicles have no game version in common, naming two vehicles whose ranges don't overlap and their ranges, and asking to update or remove one of them.

#### Scenario: Empty name
- **WHEN** the user clears the Name
- **THEN** the dialog says the mod needs a name, and Export… is disabled

#### Scenario: Internal name too long for a truck with several cabins
- **WHEN** the project holds the sample truck, whose package has two main textures, and the internal name is "ace_logistic" (12 characters)
- **THEN** the dialog says the internal name can have at most 10 characters, and Export… is disabled

#### Scenario: Missing game data
- **WHEN** a project made by a build without game data is opened on a computer that doesn't have its vehicle's package version 1.1.0, and the user opens Export Mod…
- **THEN** the dialog says that TruckPaint Sample Truck 1.1.0 must be installed to export the mod, and Export… is disabled

#### Scenario: Badly written game version
- **WHEN** the project's Game versions are `1.56.x` and the user opens Export Mod…
- **THEN** the dialog says that 1.56.x isn't a game version and gives 1.56.* as an example, and Export… is disabled

#### Scenario: Game version a vehicle doesn't support
- **WHEN** the project holds the sample truck (`>=1.56`), its Game versions are `1.55.*, 1.56.*`, and the user opens Export Mod…
- **THEN** the dialog says that TruckPaint Sample Truck (>=1.56) doesn't support 1.55.*, and Export… is disabled

#### Scenario: No common game version
- **WHEN** the project holds the sample truck (`>=1.56`) and a custom vehicle "Old Hauler" supporting `<1.55`, and the user opens Export Mod…
- **THEN** the dialog says that TruckPaint Sample Truck (>=1.56) and Old Hauler (<1.55) have no game version in common and that one of them must be updated or removed, and Export… is disabled
