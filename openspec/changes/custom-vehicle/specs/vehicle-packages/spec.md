## MODIFIED Requirements

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
