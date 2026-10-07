# vehicle-packages Specification

## Purpose

Defines vehicle packages, the versioned files that describe one truck or trailer and its texture templates, and the local library where they are installed. Painters can then work on any official or community vehicle without TruckPaint shipping game assets.

## Requirements

### Requirement: Package format
A vehicle package SHALL be a ZIP file with the `.tpv` extension containing a `vehicle.json` manifest at its root and the template images it references. The manifest SHALL contain:

- `format`: the package format version (1);
- `id`: lowercase words separated by dots, at least two (`scs.volvo.fh16_2012`, `community.jdoe.mighty_hauler`), each word made of a–z, 0–9, `_` and `-`;
- `version`: a semantic version (`1.2.0`);
- `name`, `brand`, `kind` (`truck` or `trailer`), `game` (`ets2` or `ats`);
- `game_versions`: a version range of the game the package is made for (`>=1.50, <1.54`);
- optionally `authors`, `license`, `homepage`, `description`, a `preview` image, and `requires` (mods the vehicle depends on, each with a name and an optional version range);
- `variants`: at least one, each with an `id`, a `name` and at least one texture.

A texture SHALL have:
- an `id`, unique within its variant, and a `name`;
- a square `size` in pixels: 256, 512, 1024, 2048, 4096 or 8192;
- a `template` image path inside the package, a PNG or SVG file;
- a `layout_version`: a positive integer that the author increases whenever the texture's layout changes;
- an optional `export` object, kept as-is for the mod export.

Unknown fields SHALL be ignored, so packages can carry data for later TruckPaint versions. A package SHALL contain only data: JSON and images.

#### Scenario: Minimal valid package
- **WHEN** a package declares one variant "Standard cabin" with one 4096 px texture "Cabin" whose template is a PNG in the package
- **THEN** it is accepted and listed as one vehicle with one variant and one texture

#### Scenario: Data for later versions
- **WHEN** a package's manifest contains a field TruckPaint does not know, such as `mirror`
- **THEN** the package is accepted and the field is ignored

### Requirement: Package validation
Installing a package SHALL validate it completely and refuse it, with a message naming the first problem found, when:

- it is not a ZIP, has no `vehicle.json`, or the manifest is not valid JSON;
- a required field is missing or invalid: the id, version or game-version range, or an unknown kind or game;
- it declares a format version newer than this version of TruckPaint supports;
- a variant has no texture, or two textures of a variant share an id;
- a texture size is not allowed, or a template file is missing, is not a PNG or SVG, or cannot be read;
- an entry path is absolute or contains `..`;
- the package is larger than 512 MB uncompressed, or an image larger than 16384 px on a side.

Nothing SHALL be installed when validation fails.

#### Scenario: Missing template
- **WHEN** the user installs a package whose texture "Chassis" references `templates/chassis.png` but the file is absent
- **THEN** installation is refused with a message saying the template of "Chassis" is missing, and the library is unchanged

#### Scenario: Unsafe path
- **WHEN** a package contains an entry named `../../evil.png`
- **THEN** installation is refused and no file is written outside the library

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
- the newest installed version and its supported game versions;
- the variants, and the other installed versions.

It SHALL offer:
- **Install…**;
- **Remove** for a vehicle version, after confirmation. Removing never changes projects, which embed their templates.

The list SHALL be filterable by game and kind and searchable by name and brand. With no vehicle installed, it SHALL show an empty state explaining what packages are and offering Install….

#### Scenario: Remove a version
- **WHEN** the user removes version 1.2.0 of a vehicle that also has 1.3.0 installed
- **THEN** only 1.3.0 remains in the library, and an open project made with 1.2.0 still shows its templates

### Requirement: Built-in sample vehicle
TruckPaint SHALL include the newest version of the sample vehicle (see the vehicle-authoring capability). When no vehicle is installed, an **Install the sample vehicle** button SHALL appear in two places:
- the empty state of the Vehicle Library;
- the Vehicle step of New Project.

The button SHALL install the sample like any other package, with the same confirmation and the same errors. Once a vehicle is installed, the button SHALL no longer be shown. Installing the sample SHALL need no network access.

#### Scenario: First vehicle project
- **WHEN** a painter with no vehicle installed opens New Project and clicks Install the sample vehicle
- **THEN** "TruckPaint Sample Truck" appears in the list with its newest version, a confirmation names it, and it can be picked to create a project

#### Scenario: Library with vehicles
- **WHEN** at least one vehicle is installed
- **THEN** neither the Vehicle Library nor the Vehicle step shows Install the sample vehicle
