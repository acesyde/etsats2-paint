## MODIFIED Requirements

### Requirement: Vehicles sidebar
The workspace SHALL show a sidebar on the left, between the tool bar and the canvas area, open by default. It holds two collapsible sections:
- **Project:** the project's Name, Version and Game versions. The game is not listed: it can't change once chosen and titles the Vehicles section.
  - Name and Version are read only. The Version is the mod version set in Export Mod….
  - **Game versions** is editable: the game versions the mod is made for, typed as the game writes them and separated by commas (`1.56.*, 1.57.*`). It is empty for a new project. Pressing Enter or leaving the field records the change as one undo step, "Edit Game Versions"; Escape restores the previous value. The list is kept in the order typed, without blanks and empty entries. Export Mod copies it into the manifest (see mod-export).
  - Under the field, the section SHALL show the game versions every vehicle of the project supports, from the version ranges their packages give: their overlap written as a range (`>=1.56, <1.58`), "any version" when no vehicle limits them, "no common version" when the ranges don't overlap, and nothing when no vehicle has its game data.
- **Vehicles:** the fleet's navigation (see the vehicle-projects capability).

The Project header has a Hide Sidebar button that reduces the sidebar to a narrow strip, whose Show Sidebar button opens it again. View › Sidebar (F5) SHALL toggle it, and Vehicle › Vehicle Information SHALL open it. Its width SHALL be resizable by dragging its edge within sensible bounds. Whether it is open, and its width, SHALL be remembered across sessions; Reset Workspace SHALL open it at its default width.

#### Scenario: Hiding the sidebar
- **WHEN** the user clicks Hide Sidebar
- **THEN** the sidebar becomes a narrow strip with a Show Sidebar button, the canvas area widens, and the sidebar stays hidden after a restart

#### Scenario: Toggling with the keyboard
- **WHEN** the sidebar is hidden and the user presses F5
- **THEN** the sidebar is shown again

#### Scenario: Project properties
- **WHEN** a new project named "ACE Logistics" is open
- **THEN** the Project section shows the name "ACE Logistics" and the Version "1.0", read only, and an empty, editable Game versions field

#### Scenario: Version follows the mod settings
- **WHEN** the user sets the Version to "1.2" in Export Mod… and exports
- **THEN** the Project section shows the Version "1.2"

#### Scenario: Editing the game versions
- **WHEN** the user types `1.56.*, 1.57.*` in the Game versions field and presses Enter
- **THEN** the project's Game versions are `1.56.*` and `1.57.*`, the status bar shows "Unsaved changes", and Undo restores the empty field

#### Scenario: Versions supported by the fleet
- **WHEN** a project holds a vehicle supporting `>=1.53, <1.58` and another supporting `^1.56`
- **THEN** the Project section shows ">=1.56, <1.58" under the Game versions field

#### Scenario: Vehicles that never run together
- **WHEN** a project holds the sample truck (`>=1.56`) and a custom vehicle supporting `<1.55`
- **THEN** the Project section shows "no common version" under the Game versions field
