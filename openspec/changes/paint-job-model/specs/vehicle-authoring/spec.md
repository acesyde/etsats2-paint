## MODIFIED Requirements

### Requirement: Checking a package
The `tpv check <file.tpv>` command SHALL validate a package as the application does when installing. On success it SHALL print a summary:
- the id, version, name, kind, game, game path and supported game versions;
- the paint job: each main texture, then each accessory, with its game ids and its texture size.

On failure it SHALL print the reason. The exit status SHALL be zero only for a valid package.

#### Scenario: Check a downloaded package
- **WHEN** the author runs `tpv check volvo_fh16-1.3.0.tpv` on a valid package
- **THEN** its id, version, game path, main textures, accessories and texture sizes are printed and the command exits with status 0

### Requirement: Sample vehicle
TruckPaint SHALL ship a sample truck, "TruckPaint Sample Truck", made only of original artwork and no game asset. Its id is `community.truckpaint.sample_truck`, it targets ETS2, and its game path is fictional.

Its templates SHALL be vector drawings that look like a truck's UV layout:
- labelled panels for the cab sides, front, rear and roof;
- the doors, the chassis rails, the fuel tanks and the accessories.

Two versions SHALL be provided:
- **1.0.0:** the main textures **Standard cab** and **High roof** (4096 each, one per cabin layout), and the accessories **Chassis** (2048) and **Cab accessories** (1024), all at layout version 1.
- **1.1.0:** a newer supported game range. The Standard cab's layout changes (layout version 2), the Chassis becomes 4096, and a **Side skirts** accessory (1024) is added.

Each version's sources SHALL be kept in the repository next to the package built from them. A test SHALL fail when a committed package does not match what packing its sources produces.

#### Scenario: Try Update Template end to end
- **WHEN** a painter creates a project from the Standard cab of sample truck 1.0.0 with its accessories, paints on it, installs sample truck 1.1.0 and runs Update Template, keeping Side skirts checked
- **THEN** the Standard cab texture is flagged "Layout changed", the Chassis artwork is scaled to 4096, and a new Side skirts texture appears

#### Scenario: Stale sample package
- **WHEN** a template source of a sample is edited and its package is not rebuilt
- **THEN** the test suite fails and names the out-of-date package

## ADDED Requirements

### Requirement: Sample trailer
TruckPaint SHALL ship a sample owned trailer, "TruckPaint Sample Trailer", made only of original artwork and no game asset. Its id is `community.truckpaint.sample_trailer`, it targets ETS2, and its game path is fictional.

Its version 1.0.0 SHALL have:
- the main texture **Base** (2048): the chassis frame, the landing gear and the front wall;
- the accessories **Curtain body 13.6 m** (4096), **Curtain body 10.5 m** (4096) and **Mudflaps** (512), all at layout version 1.

Its sources and built package SHALL be kept and checked like the sample truck's.

#### Scenario: A trailer project needs no choice
- **WHEN** a painter creates a project from the sample trailer
- **THEN** the project has the Base texture, always included, and the three accessories
