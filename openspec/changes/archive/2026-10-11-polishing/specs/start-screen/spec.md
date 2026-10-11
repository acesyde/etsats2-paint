## MODIFIED Requirements

### Requirement: Recent projects list
The Recent projects area SHALL list previously opened projects, most recent first, as **cards**. Each card SHALL show a thumbnail of the project, its name, its file location and its last-opened date. The thumbnail is the artwork of the project's first main texture drawn over its template; while it is being read, and when the file can't be read, the card SHALL show a placeholder icon instead.

Clicking an available card SHALL open that project. When the list is empty, it SHALL show an explanatory empty state instead of a blank area: centered on the whole width of the list (from the left of the Recent projects heading to the right of Preferences), in a frame standing where the cards will be, with a **Create Your First Project** button that does what New Project does. Entries whose file no longer exists SHALL be shown as unavailable, SHALL NOT open, and SHALL be removable from the list.

#### Scenario: Empty recent list
- **WHEN** no project has ever been opened
- **THEN** the Recent projects area shows an empty state message inviting the user to create a new project

#### Scenario: Missing recent file
- **WHEN** a recent project's file has been deleted or moved
- **THEN** its card is displayed as unavailable with a non-color indicator, shows the placeholder instead of a thumbnail, and offers a "Remove from list" action

#### Scenario: Open a recent project
- **WHEN** the user clicks an available recent card
- **THEN** the editor opens that project

#### Scenario: Thumbnail of a recent project
- **WHEN** a recent project's first main texture is covered by a red rectangle
- **THEN** its card's thumbnail is red

#### Scenario: Empty list centered with a create button
- **WHEN** the home screen is shown with no recent project
- **THEN** the empty state is centered between the left edge of the Recent projects heading and the right edge of Preferences, and clicking Create Your First Project opens New Project
