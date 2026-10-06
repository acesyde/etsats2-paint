## MODIFIED Requirements

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

## ADDED Requirements

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

## REMOVED Requirements

### Requirement: Open Project availability
**Reason**: The project file format now exists, so Open Project and recent entries are enabled.
**Migration**: Covered by "Open Project action" and "Recent projects list".
