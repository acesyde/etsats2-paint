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

1. **Vehicle:** the user picks an installed vehicle and its variant, or **Blank texture**. Blank texture is selected by default when no vehicle is installed. The step lists the installed vehicles (name, brand, kind, game, newest version), is searchable by name and brand, filterable by game and kind, and offers Install… to add packages. Picking a vehicle with several variants shows its variants to choose from.
2. **Name & resolution:** the user enters a project name, which defaults to the vehicle's name for a vehicle project. For a blank texture, the user also chooses a texture resolution among 2048×2048, 4096×4096 and 8192×8192 (default 4096×4096). For a vehicle, this step lists its textures and their sizes instead.

Confirming SHALL open the editor workspace with a new project: an empty blank texture of the chosen resolution, or a vehicle project (see the vehicle-projects capability). Cancelling SHALL return to the home screen without side effects. The dialog SHALL be fully operable by keyboard (Tab to move, Enter to confirm, Escape to cancel).

#### Scenario: Creating a project
- **WHEN** the user opens New Project, keeps Blank texture and the default resolution and clicks Create
- **THEN** the editor workspace opens with an empty 4096×4096 project and the window title shows the project name

#### Scenario: Cancelling
- **WHEN** the user presses Escape in the New Project dialog
- **THEN** the dialog closes and the home screen is unchanged

#### Scenario: Empty name
- **WHEN** the user leaves the name empty and confirms
- **THEN** the project is created with the default name "Untitled"

#### Scenario: Creating a project for a vehicle
- **WHEN** the user opens New Project, picks "Volvo FH16 2012" and its "Globetrotter XL" variant, continues and clicks Create
- **THEN** the workspace opens with a project named "Volvo FH16 2012" with one surface per texture of that variant

### Requirement: Dialog structure ready for vehicle steps
The New Project dialog SHALL be organized as a sequence of steps with a visible step indicator. Next and Back buttons move between steps, Enter confirms the current step, and Create is available on the last step.

#### Scenario: Step indicator
- **WHEN** the New Project dialog is open
- **THEN** a step indicator shows the current step and the total number of steps

#### Scenario: Back to the vehicle
- **WHEN** the user is on the Name & resolution step and clicks Back
- **THEN** the Vehicle step is shown with the previous choice still selected

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
