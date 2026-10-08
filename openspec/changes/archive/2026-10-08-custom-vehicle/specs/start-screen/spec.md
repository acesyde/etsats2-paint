## MODIFIED Requirements

### Requirement: New Project dialog
Choosing New Project SHALL open a modal dialog in two steps:

1. **Vehicle:** the user picks an installed vehicle and chooses the textures to paint.
   - The step lists the installed vehicles (name, brand, kind, game, newest version). It is searchable by name and brand, filterable by game and kind, and offers Install… to add packages and **Custom vehicle…** to create a vehicle from template files (see the custom-vehicles capability).
   - Picking a vehicle shows checkboxes:
     - under **Main textures**, one per main texture, with the first one checked; the last checked main texture (and so a single main texture) can't be unchecked;
     - under **Accessories**, one per accessory group, all checked.
   - Next SHALL be enabled only when a vehicle is chosen and at least one main texture is checked. With no vehicle installed, the step offers Install…, Install the sample vehicles and Custom vehicle….
   - A vehicle created with Custom vehicle… SHALL be selected when the step is shown again, with its textures checked as for any picked vehicle. The step's search and filters SHALL let it be seen. Cancelling the Custom Vehicle dialog SHALL show the step as it was.
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
- **THEN** Next is disabled, and Install…, Install the sample vehicles and Custom vehicle… are offered

#### Scenario: Project from a custom vehicle
- **WHEN** the user, in the Vehicle step with no vehicle installed, clicks Custom vehicle…, creates a truck from one template file, then clicks Next and Create
- **THEN** the workspace opens with a project for that custom vehicle, with one surface showing the template

#### Scenario: Created vehicle hidden by a filter
- **WHEN** the Vehicle step is filtered on ATS and the user creates an ETS2 custom vehicle
- **THEN** the step shows the new vehicle selected, and the game filter now allows ETS2
