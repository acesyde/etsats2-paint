# vehicle-authoring Specification

## Purpose

Defines how vehicle packages are authored: a command-line packer that builds and checks `.tpv` files, including from the game's DDS templates, and the sample truck and trailer shipped with TruckPaint as reference examples and for end-to-end testing.

## Requirements

### Requirement: Packing a folder
The `tpv pack <folder>` command SHALL build a vehicle package from a folder holding a `vehicle.json` manifest and the files it references. It writes the result to the path given with `-o`, or, by default, to `<id>-<version>.tpv` in the current directory.

The package SHALL contain:
- the manifest;
- every template it references;
- its preview, if any.

Other files in the folder SHALL be left out, and the packer SHALL list them as ignored.

Before writing anything, the packer SHALL validate the result exactly as the application does when installing. On failure, it SHALL print the reason and the file concerned, write no file, and exit with a non-zero status.

Packing the same folder twice SHALL produce identical bytes.

#### Scenario: Pack a valid folder
- **WHEN** the author runs `tpv pack my-truck/` on a folder with a valid manifest and its templates
- **THEN** `community.jdoe.my_truck-1.0.0.tpv` is written, and the application installs it without error

#### Scenario: Missing template
- **WHEN** the manifest references `templates/cabin.png` and the folder has no such file
- **THEN** the packer reports that the Cabin template is missing, writes no file and exits with a non-zero status

#### Scenario: Unrelated files
- **WHEN** the folder also holds `notes.txt` and `cabin.psd`
- **THEN** the package does not contain them and the packer lists them as ignored

### Requirement: DDS templates
The packer SHALL accept templates in DDS format, as SCS distributes them: uncompressed, or block-compressed BC1, BC2 or BC3. It SHALL convert each one to a PNG of the same pixels and update the template's path in the packaged manifest to that PNG. The source folder SHALL NOT be modified.

A DDS file the packer cannot decode SHALL fail packing, naming the texture concerned.

#### Scenario: Pack SCS templates
- **WHEN** the manifest references `templates/cabin.dds`, a BC3 DDS file of 4096×4096
- **THEN** the package holds `templates/cabin.png`, a 4096×4096 PNG, and its manifest references that file

#### Scenario: Unsupported DDS
- **WHEN** a referenced DDS file uses a compression the packer cannot decode
- **THEN** packing fails with a message naming the texture, and no file is written

### Requirement: Checking a package
The `tpv check <file.tpv>` command SHALL validate a package as the application does when installing. On success it SHALL print a summary: the id, version, name, game, supported game versions, and each variant with its textures and sizes. On failure it SHALL print the reason. The exit status SHALL be zero only for a valid package.

#### Scenario: Check a downloaded package
- **WHEN** the author runs `tpv check volvo_fh16-1.3.0.tpv` on a valid package
- **THEN** its id, version, variants and textures are printed and the command exits with status 0

### Requirement: Sample vehicle
TruckPaint SHALL ship a sample vehicle, "TruckPaint Sample Truck", made only of original artwork and no game asset. Its id is `community.truckpaint.sample_truck`, and it targets ETS2.

Its templates SHALL be vector drawings that look like a truck's UV layout:
- labelled panels for the cab sides, front, rear and roof;
- the doors, the chassis rails, the fuel tanks and the accessories.

Two versions SHALL be provided:
- **1.0.0:** variants **Standard cab** and **High roof**, each with the textures Cabin (4096), Chassis (2048) and Accessories (1024), all at layout version 1.
- **1.1.0:** a newer supported game range. The Standard cab's Cabin layout changes (layout version 2), the Chassis becomes 4096 in both variants, and a **Side skirts** texture (1024) is added to both variants.

Each version's sources SHALL be kept in the repository next to the package built from them. A test SHALL fail when a committed package does not match what packing its sources produces.

#### Scenario: Try Update Template end to end
- **WHEN** a painter creates a project from the Standard cab of sample 1.0.0, paints on it, installs sample 1.1.0 and runs Update Template
- **THEN** the Cabin texture is flagged "Layout changed", the Chassis artwork is scaled to 4096, and a new Side skirts texture appears

#### Scenario: Stale sample package
- **WHEN** a template source of the sample is edited and its package is not rebuilt
- **THEN** the test suite fails and names the out-of-date package
