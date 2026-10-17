## REMOVED Requirements

### Requirement: Official vehicle catalog
**Reason**: The embedded catalog and Carsmaniac/Paintjob Packer source chain are retired.
**Migration**: Generate supported ETS2 truck packages with `tpv extract` and install them manually; use existing manual packages or Custom Vehicle for other vehicles. Old packages have no migration.

### Requirement: Template pack
**Reason**: The embedded catalog and Carsmaniac/Paintjob Packer source chain are retired.
**Migration**: Generate supported ETS2 truck packages with `tpv extract` and install them manually; use existing manual packages or Custom Vehicle for other vehicles. Old packages have no migration.

### Requirement: Installing an official vehicle
**Reason**: The embedded catalog and Carsmaniac/Paintjob Packer source chain are retired.
**Migration**: Generate supported ETS2 truck packages with `tpv extract` and install them manually; use existing manual packages or Custom Vehicle for other vehicles. Old packages have no migration.

### Requirement: Keeping the catalog up to date
**Reason**: The embedded catalog and Carsmaniac/Paintjob Packer source chain are retired.
**Migration**: Generate supported ETS2 truck packages with `tpv extract` and install them manually; use existing manual packages or Custom Vehicle for other vehicles. Old packages have no migration.

### Requirement: Checking the catalog against a pack
**Reason**: The embedded catalog and Carsmaniac/Paintjob Packer source chain are retired.
**Migration**: Generate supported ETS2 truck packages with `tpv extract` and install them manually; use existing manual packages or Custom Vehicle for other vehicles. Old packages have no migration.

## ADDED Requirements

### Requirement: Official packages installed manually
Official generated ETS2 trucks SHALL use ids `official.ets2.<vehicle>`, including `official.ets2.daf.2021`. The application SHALL install and use them through the ordinary validated manual package workflow. It SHALL NOT embed a vehicle catalog, build packages from Carsmaniac folders, offer template pack downloads or retain credits used only by that removed chain. No migration of old Carsmaniac packages or placements SHALL be provided, and this change SHALL NOT automatically delete user packages or projects.

#### Scenario: Install an extracted DAF
- **WHEN** the player manually installs a valid `official.ets2.daf.2021` package
- **THEN** it appears among installed vehicles and can create a project and export a mod without a template pack folder

#### Scenario: No implicit catalog offers
- **WHEN** no vehicle package is installed
- **THEN** no uninstalled official truck or trailer is offered by an embedded catalog
