# vehicle-packages Specification

## Purpose

Defines vehicle packages, the versioned files that describe one truck or trailer and its texture templates, and the local library where they are installed. Painters can then work on any official or community vehicle without TruckPaint shipping game assets.

## Requirements

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
- `texture`: its `size` in pixels (256, 512, 1024, 2048, 4096 or 8192, square), its `template` image path inside the package (a PNG or SVG file), and its `layout_version`, a positive integer that the author increases whenever the texture's layout changes.

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
- a texture size is not allowed, or a template file is missing, is not a PNG or SVG, or cannot be read;
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

### Requirement: Installing packages
The user SHALL be able to install a package:
- from Vehicle › Vehicle Library… › Install…, choosing one or more `.tpv` files;
- from the Vehicle step of New Project › Install…;
- by dropping `.tpv` files on the home screen or the workspace.

Installed packages SHALL persist across sessions in the application's data folder.

Installing the same version again SHALL replace it. A different version of an installed vehicle SHALL be installed alongside the others. A confirmation SHALL name each vehicle and version installed.

#### Scenario: Install from the Vehicle Library
- **WHEN** the user installs `volvo_fh16-1.2.0.tpv` from the Vehicle Library and restarts the application
- **THEN** "Volvo FH16 2012" version 1.2.0 is listed in the library

#### Scenario: Two versions side by side
- **WHEN** versions 1.2.0 and 1.3.0 of the same vehicle are installed
- **THEN** the library lists the vehicle once, as version 1.3.0, with 1.2.0 shown as an older installed version

### Requirement: Vehicle Library dialog
Vehicle › Vehicle Library… SHALL open a dialog listing the installed vehicles. For each vehicle, it shows:
- the name, brand, kind and game;
- the newest installed version and its supported game versions, or "any version" when the package sets no limit;
- its paint job: the names of its main textures and the number of accessories;
- the other installed versions.

It SHALL offer:
- **Install…**;
- **Custom Vehicle…**, which opens the Custom Vehicle dialog (see the custom-vehicles capability);
- **New Version…** for a custom vehicle (id starting with `custom.`), which opens the Custom Vehicle dialog filled in from its newest version;
- **Export…** for a vehicle version. It saves that version's package, unchanged, as a `.tpv` file where the player chooses, proposing `<id>-<version>.tpv`. The save dialog SHALL remind the player that templates from the base games belong to SCS Software, and to check that they may share them;
- **Remove** for a vehicle version, after confirmation. Removing never changes projects, which embed their templates.

The list SHALL be filterable by game and kind and searchable by name and brand. With no vehicle installed, it SHALL show an empty state explaining what packages are and offering Install… and Custom Vehicle….

#### Scenario: Remove a version
- **WHEN** the user removes version 1.2.0 of a vehicle that also has 1.3.0 installed
- **THEN** only 1.3.0 remains in the library, and an open project made with 1.2.0 still shows its templates

#### Scenario: Paint job summary
- **WHEN** the sample truck 1.1.0 is installed and the Vehicle Library is open
- **THEN** its entry lists the main textures "Standard cab" and "High roof" and three accessories

#### Scenario: Export a version
- **WHEN** the user clicks Export… on version 1.0.0 of `custom.scania.r_2024` and chooses a folder
- **THEN** `custom.scania.r_2024-1.0.0.tpv` is written there with the same bytes as the installed package

#### Scenario: Any game version
- **WHEN** a custom vehicle was created with no game versions
- **THEN** its entry shows "any version" as its supported game versions

#### Scenario: Custom vehicle from the library
- **WHEN** the user clicks Custom Vehicle… in the Vehicle Library and creates a vehicle
- **THEN** the Vehicle Library is shown again and lists the new vehicle

### Requirement: Built-in sample vehicle
TruckPaint SHALL include the newest versions of the sample truck and the sample trailer (see the vehicle-authoring capability). When no vehicle is installed, an **Install the sample vehicles** button SHALL appear in two places:
- the empty state of the Vehicle Library;
- the Vehicle step of New Project.

The button SHALL install both samples like any other package, with the same confirmation and the same errors. Once a vehicle is installed, the button SHALL no longer be shown. Installing the samples SHALL need no network access.

#### Scenario: First vehicle project
- **WHEN** a painter with no vehicle installed opens New Project and clicks Install the sample vehicles
- **THEN** "TruckPaint Sample Truck" and "TruckPaint Sample Trailer" appear in the list with their newest versions, a confirmation names them, and either can be picked to create a project

#### Scenario: Library with vehicles
- **WHEN** at least one vehicle is installed
- **THEN** neither the Vehicle Library nor the Vehicle step shows Install the sample vehicles
