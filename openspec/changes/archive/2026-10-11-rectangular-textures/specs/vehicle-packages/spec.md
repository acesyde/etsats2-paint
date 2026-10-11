## MODIFIED Requirements

### Requirement: Package format
A vehicle package SHALL be a ZIP file with the `.tpv` extension containing a `vehicle.json` manifest at its root and the template images it references. The manifest SHALL contain:

- `format`: the package format version (1);
- `id`: lowercase words separated by dots, at least two (`scs.volvo.fh16_2012`, `community.jdoe.mighty_hauler`), each word made of a–z, 0–9, `_` and `-`;
- `version`: a semantic version (`1.2.0`);
- `name`, `brand` and `kind` (`truck` or `trailer`);
- optionally `authors`, `license`, `homepage`, `description` and a `preview` image;
- `game`, what the package targets in the game:
  - `id`: `ets2` or `ats`;
  - `versions`: a version range of the game the package is made for (`>=1.50, <1.54`);
  - `path`: the vehicle's path in the game's definitions (`scania.r_2016`), as words of a–z, 0–9 and `_` separated by dots;
  - optionally `alt_uv` (the paint job uses the vehicle's alternate UV set, some ATS trucks) and `colour_picker` (the paint job lets the player pick a base color), both false by default;
  - optionally `requires`: mods the vehicle depends on, each with a name and an optional version range;
- `paint_job`, the vehicle's paint job as the game structures it, made of **parts**:
  - `main`: one or more main textures. A trailer, or a truck whose cabins share one layout, has one; a truck whose cabins have different layouts has one per layout;
  - optionally `accessories`: the accessory groups, shared by the whole vehicle.

Every part, main texture or accessory, SHALL have the same fields:
- an `id`, unique within the vehicle, and a `name`;
- `game_ids`: the names the game uses for what the part covers. For a main texture, the internal names of the cabins that share its layout (`["highline", "highline_8x4"]`); a main texture without game ids covers every cabin. For an accessory, the accessory ids (`["mirror.painted", "s_mirror.painted"]`);
- `texture`: its `size` in pixels as `[width, height]`, each a power of two from 64 to 8192 (`[4096, 4096]`, `[8192, 1024]`), its `template` image path inside the package (a PNG or SVG file), and its `layout_version`, a positive integer that the author increases whenever the texture's layout changes.

Unknown fields SHALL be ignored, so packages can carry data for later TruckPaint versions. A package SHALL contain only data: JSON and images.

#### Scenario: Minimal valid package
- **WHEN** a truck package declares one main texture "Cabin" without game ids, with a 4096 px template PNG in the package
- **THEN** it is accepted and listed as one vehicle with one main texture and no accessory, painted on every cabin

#### Scenario: Truck with cabins of different layouts
- **WHEN** a truck package declares the main textures "Standard cab" (game id `standard`) and "High roof" (game id `high_roof`)
- **THEN** it is accepted and listed with two main textures

#### Scenario: Trailer package
- **WHEN** a trailer package declares one main texture "Base" and two accessory groups "Body 13.6 m" and "Mudflaps"
- **THEN** it is accepted and listed as one trailer with one main texture and two accessories

#### Scenario: Data for later versions
- **WHEN** a package's manifest contains a field TruckPaint does not know, such as `mirror`
- **THEN** the package is accepted and the field is ignored

### Requirement: Package validation
Installing a package SHALL validate it completely and refuse it, with a message naming the first problem found, when:

- it is not a ZIP, has no `vehicle.json`, or the manifest is not valid JSON;
- a required field is missing or invalid: the id, version, game id, game-version range or game path, or an unknown kind;
- it declares a format version newer than this version of TruckPaint supports;
- the paint job has no main texture;
- two parts of the vehicle share an id;
- the paint job has two or more main textures and one of them has no game id, or an accessory has no game id;
- a game id is not words of a–z, 0–9 and `_` separated by dots, or appears twice among the main textures or twice among the accessories;
- a texture size is not a `[width, height]` pair of allowed sides (a single number is refused), or a template file is missing, is not a PNG or SVG, or cannot be read;
- an entry path is absolute or contains `..`;
- the package is larger than 512 MB uncompressed, or an image larger than 16384 px on a side.

Nothing SHALL be installed when validation fails.

#### Scenario: Missing template
- **WHEN** the user installs a package whose accessory "Chassis" references `templates/chassis.png` but the file is absent
- **THEN** installation is refused with a message saying the template of "Chassis" is missing, and the library is unchanged

#### Scenario: Unsafe path
- **WHEN** a package contains an entry named `../../evil.png`
- **THEN** installation is refused and no file is written outside the library

#### Scenario: Several main textures without game ids
- **WHEN** a package declares the main textures "Standard cab" and "High roof", and "High roof" has no game id
- **THEN** installation is refused with a message saying "High roof" has no game id

#### Scenario: Accessory without game id
- **WHEN** an accessory group "Mirrors" has an empty `game_ids` list
- **THEN** installation is refused with a message saying "Mirrors" has no game id

#### Scenario: Rectangular texture
- **WHEN** the user installs a package whose trailer base texture has the size `[8192, 1024]`
- **THEN** it is installed, and a project made from it has an 8192 × 1024 surface

#### Scenario: Single size refused
- **WHEN** a package gives a texture the size `4096`
- **THEN** installation is refused with a message saying the texture's size must be a width and a height
