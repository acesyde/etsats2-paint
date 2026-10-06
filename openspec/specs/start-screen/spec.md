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
The Recent Projects area SHALL list previously opened projects, most recent first, each with its name, file location and last-opened date. When the list is empty, it SHALL show an explanatory empty state instead of a blank area. Entries whose file no longer exists SHALL be shown as unavailable and SHALL be removable from the list.

#### Scenario: Empty recent list
- **WHEN** no project has ever been opened
- **THEN** the Recent Projects area shows an empty state message inviting the user to create a new project

#### Scenario: Missing recent file
- **WHEN** a recent project's file has been deleted or moved
- **THEN** its entry is displayed as unavailable with a non-color indicator and offers a "Remove from list" action

### Requirement: Open Project availability
Until the project file format exists, the Open Project action and recent entries SHALL be visibly disabled with a tooltip stating that opening projects is not available yet.

#### Scenario: Clicking Open Project before persistence exists
- **WHEN** the user hovers the disabled Open Project action
- **THEN** a tooltip explains that opening projects is not available yet and clicking does nothing

### Requirement: New Project dialog
Choosing New Project SHALL open a modal dialog where the user enters a project name and chooses a texture resolution among 2048×2048, 4096×4096 and 8192×8192 (default 4096×4096). Confirming SHALL open the editor workspace with a new, empty, untitled-or-named project of that resolution. Cancelling SHALL return to the home screen without side effects. The dialog SHALL be fully operable by keyboard (Tab to move, Enter to confirm, Escape to cancel).

#### Scenario: Creating a project
- **WHEN** the user opens New Project, keeps the default resolution and clicks Create
- **THEN** the editor workspace opens with an empty 4096×4096 project and the window title shows the project name

#### Scenario: Cancelling
- **WHEN** the user presses Escape in the New Project dialog
- **THEN** the dialog closes and the home screen is unchanged

#### Scenario: Empty name
- **WHEN** the user leaves the name empty and confirms
- **THEN** the project is created with the default name "Untitled"

### Requirement: Dialog structure ready for vehicle steps
The New Project dialog SHALL be organized as a sequence of steps with a visible step indicator, so that game, manufacturer, vehicle and cabin selection steps can be inserted before the resolution step without changing the dialog's interaction model.

#### Scenario: Step indicator
- **WHEN** the New Project dialog is open
- **THEN** a step indicator shows the current step and the total number of steps
