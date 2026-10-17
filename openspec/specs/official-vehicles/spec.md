# official-vehicles Specification

## Purpose
Lets players paint the games' official vehicles without typing any game data: official ETS2 truck packages are generated from the installed game with `tpv extract` (see the ets2-game-extraction capability) and installed manually, like any other vehicle package.

## Requirements

### Requirement: Official packages installed manually
Official generated ETS2 trucks SHALL use ids `official.ets2.<vehicle>`, including `official.ets2.daf.2021`. The application SHALL install and use them through the ordinary validated manual package workflow. It SHALL NOT embed a vehicle catalog, build packages from Carsmaniac folders, offer template pack downloads or retain credits used only by that removed chain. No migration of old Carsmaniac packages or placements SHALL be provided, and this change SHALL NOT automatically delete user packages or projects.

#### Scenario: Install an extracted DAF
- **WHEN** the player manually installs a valid `official.ets2.daf.2021` package
- **THEN** it appears among installed vehicles and can create a project and export a mod without a template pack folder

#### Scenario: No implicit catalog offers
- **WHEN** no vehicle package is installed
- **THEN** no uninstalled official truck or trailer is offered by an embedded catalog
