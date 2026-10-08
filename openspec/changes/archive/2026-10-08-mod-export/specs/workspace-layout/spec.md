## MODIFIED Requirements

### Requirement: Vehicles sidebar
The workspace SHALL show a sidebar on the left, between the tool bar and the canvas area, open by default. It holds two collapsible sections:
- **Project:** the project's Name, Version and Game versions, read only. The Version is the mod version set in Export Mod…. Game versions is empty until a later feature sets it. The game is not listed: it can't change once chosen and titles the Vehicles section;
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
- **THEN** the Project section shows the name "ACE Logistics", the Version "1.0" and an empty Game versions field, none of them editable

#### Scenario: Version follows the mod settings
- **WHEN** the user sets the Version to "1.2" in Export Mod… and exports
- **THEN** the Project section shows the Version "1.2"
