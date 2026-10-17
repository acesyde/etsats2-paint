## MODIFIED Requirements

### Requirement: Official packages installed manually
Official generated ETS2 trucks and owned trailers SHALL use ids `official.ets2.<vehicle>`, including `official.ets2.daf.2021` and `official.ets2.krone.coolliner`. The application SHALL install and use them through the ordinary validated manual package workflow. It SHALL NOT embed a vehicle catalog, build packages from Carsmaniac folders, offer template pack downloads or retain credits used only by that removed chain. No migration of old Carsmaniac packages or placements SHALL be provided, and this change SHALL NOT automatically delete user packages or projects.

#### Scenario: Install an extracted DAF
- **WHEN** the player manually installs a valid `official.ets2.daf.2021` package
- **THEN** it appears among installed vehicles and can create a project and export a mod without a template pack folder

#### Scenario: Install an extracted trailer
- **WHEN** the player manually installs a valid `official.ets2.krone.coolliner` package
- **THEN** it appears among installed trailers and Add Vehicle can add it to an ETS2 project

#### Scenario: No implicit catalog offers
- **WHEN** no vehicle package is installed
- **THEN** no uninstalled official truck or trailer is offered by an embedded catalog
