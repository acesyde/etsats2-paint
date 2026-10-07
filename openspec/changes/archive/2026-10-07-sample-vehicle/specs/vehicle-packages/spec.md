## ADDED Requirements

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
