## MODIFIED Requirements

### Requirement: New Project dialog
Choosing New Project SHALL open a modal dialog in two steps:

1. **Vehicle:** the user picks an installed vehicle and checks one or more of its variants.
   - The step lists the installed vehicles (name, brand, kind, game, newest version). It is searchable by name and brand, filterable by game and kind, and offers Install… to add packages.
   - Picking a vehicle shows its variants as checkboxes, with the first one checked. A vehicle with a single variant shows it checked.
   - Next SHALL be enabled only when a vehicle and at least one variant are chosen. With no vehicle installed, the step offers Install… and Install the sample vehicle.
2. **Name:** the user enters a project name, which defaults to the vehicle's name. The step lists the textures of each chosen variant with their sizes.

Confirming SHALL open the editor workspace with a new project for that vehicle and its chosen variants (see the vehicle-projects capability). Cancelling SHALL return to the home screen without side effects. The dialog SHALL be fully operable by keyboard (Tab to move, Space to toggle a checkbox, Enter to confirm, Escape to cancel).

#### Scenario: Creating a project
- **WHEN** the user opens New Project, picks the sample truck, keeps "Standard cab" checked, clicks Next and then Create
- **THEN** the editor workspace opens with a project named "TruckPaint Sample Truck" on its Standard cab textures, and the window title shows the project name

#### Scenario: Cancelling
- **WHEN** the user presses Escape in the New Project dialog
- **THEN** the dialog closes and the home screen is unchanged

#### Scenario: Empty name
- **WHEN** the user leaves the name empty and confirms
- **THEN** the project is created with the vehicle's name

#### Scenario: Creating a project for a vehicle
- **WHEN** the user opens New Project, picks "Volvo FH16 2012", checks its "Globetrotter XL" variant, continues and clicks Create
- **THEN** the workspace opens with a project named "Volvo FH16 2012" with one surface per texture of that variant

#### Scenario: No vehicle installed
- **WHEN** the user opens New Project with no vehicle installed
- **THEN** Next is disabled, and Install… and Install the sample vehicle are offered

### Requirement: Dialog structure ready for vehicle steps
The New Project dialog SHALL be organized as a sequence of steps with a visible step indicator. Next and Back buttons move between steps, Enter confirms the current step, and Create is available on the last step.

#### Scenario: Step indicator
- **WHEN** the New Project dialog is open
- **THEN** a step indicator shows the current step and the total number of steps

#### Scenario: Back to the vehicle
- **WHEN** the user is on the Name step and clicks Back
- **THEN** the Vehicle step is shown with the previous vehicle and variants still checked
