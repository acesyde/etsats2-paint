## MODIFIED Requirements

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
