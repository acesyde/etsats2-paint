# start-screen Specification

## Purpose

Gives the user an Adobe-style home screen to start a new livery project, open an existing one or resume recent work, as the entry point of the application.

## Requirements

### Requirement: Home screen on launch
When no project is open, the application SHALL show a home screen presenting the actions New Project, Open Project and the list of Recent Projects.

#### Scenario: First launch
- **WHEN** the application starts for the first time
- **THEN** the home screen is shown with New Project and Open Project actions and a Recent Projects area

#### Scenario: Returning to home
- **WHEN** the user closes the last open project
- **THEN** the home screen is shown again

### Requirement: Recent projects list
The Recent Projects area SHALL list previously opened projects, most recent first, each with its name, file location and last-opened date. Clicking an available entry SHALL open that project. When the list is empty, it SHALL show an explanatory empty state instead of a blank area. Entries whose file no longer exists SHALL be shown as unavailable, SHALL NOT open, and SHALL be removable from the list.

#### Scenario: Empty recent list
- **WHEN** no project has ever been opened
- **THEN** the Recent Projects area shows an empty state message inviting the user to create a new project

#### Scenario: Missing recent file
- **WHEN** a recent project's file has been deleted or moved
- **THEN** its entry is displayed as unavailable with a non-color indicator and offers a "Remove from list" action

#### Scenario: Open a recent project
- **WHEN** the user clicks an available recent entry
- **THEN** the editor opens that project

### Requirement: New Project dialog
Choosing New Project SHALL open a modal dialog in two steps:

1. **Vehicle:** the user picks an installed vehicle and chooses the textures to paint.
   - The step lists the installed vehicles (name, brand, kind, game, newest version). It is searchable by name and brand, filterable by game and kind, and offers Install… to add packages.
   - Picking a vehicle shows checkboxes:
     - under **Main textures**, one per main texture, with the first one checked; the last checked main texture (and so a single main texture) can't be unchecked;
     - under **Accessories**, one per accessory group, all checked.
   - Next SHALL be enabled only when a vehicle is chosen and at least one main texture is checked. With no vehicle installed, the step offers Install… and Install the sample vehicles.
2. **Name:** the user enters a project name, which defaults to the vehicle's name. The step lists the chosen textures with their sizes.

Confirming SHALL open the editor workspace with a new project for that vehicle and its chosen textures (see the vehicle-projects capability). Cancelling SHALL return to the home screen without side effects. The dialog SHALL be fully operable by keyboard (Tab to move, Space to toggle a checkbox, Enter to confirm, Escape to cancel).

#### Scenario: Creating a project
- **WHEN** the user opens New Project, picks the sample truck, keeps "Standard cab" and the accessories checked, clicks Next and then Create
- **THEN** the editor workspace opens with a project named "TruckPaint Sample Truck" on its Standard cab and accessory textures, and the window title shows the project name

#### Scenario: Cancelling
- **WHEN** the user presses Escape in the New Project dialog
- **THEN** the dialog closes and the home screen is unchanged

#### Scenario: Empty name
- **WHEN** the user leaves the name empty and confirms
- **THEN** the project is created with the vehicle's name

#### Scenario: Creating a project for a vehicle
- **WHEN** the user opens New Project, picks "Volvo FH16 2012", checks its "Globetrotter XL" main texture, unchecks every accessory, continues and clicks Create
- **THEN** the workspace opens with a project named "Volvo FH16 2012" with one surface, the Globetrotter XL main texture

#### Scenario: Creating a trailer project
- **WHEN** the user picks the sample trailer in the Vehicle step
- **THEN** Base is shown checked and can't be unchecked, the accessories are checked, and Next is enabled

#### Scenario: The last main texture stays checked
- **WHEN** the user picks the sample truck, checks "High roof" and unchecks "Standard cab"
- **THEN** "High roof" can't be unchecked, and Next stays enabled

#### Scenario: No vehicle installed
- **WHEN** the user opens New Project with no vehicle installed
- **THEN** Next is disabled, and Install… and Install the sample vehicles are offered

### Requirement: Dialog structure ready for vehicle steps
The New Project dialog SHALL be organized as a sequence of steps with a visible step indicator. Next and Back buttons move between steps, Enter confirms the current step, and Create is available on the last step.

#### Scenario: Step indicator
- **WHEN** the New Project dialog is open
- **THEN** a step indicator shows the current step and the total number of steps

#### Scenario: Back to the vehicle
- **WHEN** the user is on the Name step and clicks Back
- **THEN** the Vehicle step is shown with the previous vehicle and its textures still checked

### Requirement: Open Project action
The home screen's Open Project action SHALL open the same file dialog as File › Open… and open the chosen project.

#### Scenario: Open from the home screen
- **WHEN** the user clicks Open Project and selects a `.truckpaint` file
- **THEN** the editor opens that project

### Requirement: Recovered projects on the home screen
When recovery copies exist, the home screen SHALL show them above the recent projects, as described by crash recovery, until each one is restored or discarded.

#### Scenario: Recovery shown first
- **WHEN** the application starts with a recovered project
- **THEN** the home screen shows a "Recovered projects" section above Recent projects
