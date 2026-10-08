## MODIFIED Requirements

### Requirement: Adding and removing vehicles and textures
The user SHALL be able to change which vehicles and textures a project covers. Each change is one undo step.

- **Add a vehicle:** Vehicle › Add Vehicle… opens a dialog listing the installed vehicles of the project's game that aren't in the project yet. It is searchable by name and brand and filterable by kind. Picking a vehicle shows its main textures and accessories as checkboxes, as in New Project, and at least one main texture must be checked. Confirming adds the vehicle at its newest installed version and adds one surface per chosen texture, after the existing surfaces. The first new surface becomes active.
  - The dialog SHALL also offer **Custom vehicle…**, which opens the Custom Vehicle dialog with the project's game, locked (see the custom-vehicles capability). A vehicle created there SHALL be listed and selected when Add Vehicle… is shown again, with its textures checked as for any picked vehicle; it is not added to the project until the user confirms Add.
- **Change the textures of a vehicle:** the vehicle's **Textures…** action in the sidebar (⋯ menu) shows checkboxes for the main textures and accessories of the project's recorded version. A single main texture is shown checked and can't be unchecked.
  - Checking a texture adds its surface, in package order among the vehicle's surfaces.
  - Unchecking one removes its surface. If any removed surface holds artwork, a confirmation names those textures first.
  - At least one main texture stays checked.
  - Adding a texture needs the recorded version to be installed. When it isn't, the action explains that the version is missing and offers Update Template… instead.
- **Remove a vehicle:** the vehicle's **Remove from Project** action (⋯ menu) removes the vehicle and its surfaces. If any of them holds artwork, a confirmation names the vehicle first. The last vehicle of a project can't be removed.

Template images no longer used by any surface SHALL be dropped from the project's assets. Undo SHALL restore them with the surfaces.

#### Scenario: A truck and a trailer
- **WHEN** a project made for the sample truck's Standard cab gets the sample trailer through Add Vehicle…, with every accessory checked
- **THEN** the project lists both vehicles, the trailer with Base and its three accessories, and the trailer's Base is active

#### Scenario: Removing a texture with artwork
- **WHEN** the user unchecks the High roof main texture, whose surface holds a rectangle, in Textures… and confirms
- **THEN** its surface is removed, and Undo brings it back with the rectangle

#### Scenario: Adding an accessory later
- **WHEN** a project was created without the sample truck's "Cab accessories", and the user checks it in Textures…
- **THEN** a Cab accessories surface is added between the Chassis and Side skirts surfaces

#### Scenario: The last main texture stays
- **WHEN** a truck in a project paints a single one of its main textures
- **THEN** that texture's checkbox in Textures… can't be unchecked

#### Scenario: The last vehicle stays
- **WHEN** a project has a single vehicle
- **THEN** its Remove from Project action is disabled

#### Scenario: Custom trailer added to a fleet
- **WHEN** in an ETS2 project, the user opens Add Vehicle…, clicks Custom vehicle…, creates a trailer from the template `base.png`, then clicks Add
- **THEN** the trailer is added to the project with its "base" texture active, and the Custom Vehicle dialog showed ETS2 without a way to change it
