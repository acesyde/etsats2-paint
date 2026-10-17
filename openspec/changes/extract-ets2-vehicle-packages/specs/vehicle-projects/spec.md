## MODIFIED Requirements

### Requirement: Adding and removing vehicles and textures
The user SHALL be able to change which vehicles and textures a project covers. Each change is one undo step.

- **Add vehicles:** Vehicle › Add Vehicle…, also reached from the Add Vehicle button of the Textures tab and of the Project space, opens a dialog listing only installed vehicles of the project's game that are not already in the project. It SHALL NOT list uninstalled catalog entries or trigger template pack installation. It is searchable by name and brand and filterable by kind. Each row has a checkbox: clicking a row checks or unchecks the vehicle, and several vehicles can be checked. Each checked vehicle shows its main textures and accessories as checkboxes under its row, as in New Project, and keeps at least one main texture checked. **Add** is enabled when at least one vehicle is checked. Confirming adds every checked vehicle at its newest installed version, in selection order, with one surface per chosen texture after the existing surfaces, in one undo step. The first new surface becomes active.
  - The dialog SHALL also offer **Custom Vehicle…**, which opens the Custom Vehicle dialog with the project's game, locked (see the custom-vehicles capability). A vehicle created there SHALL be listed and checked when Add Vehicle… is shown again, with its textures checked as for any checked vehicle, and the vehicles checked before stay checked; it is not added to the project until the user confirms Add.
- **Change the textures of a vehicle:** the vehicle's **Textures…** action (its ⋯ menu in the Textures tab, or its vehicle card in the Project space) shows checkboxes for the main textures and accessories of the project's recorded version. A single main texture is shown checked and can't be unchecked.
  - Checking a texture adds its surface, in package order among the vehicle's surfaces.
  - Unchecking one removes its surface. If any removed surface holds artwork, a confirmation names those textures first.
  - At least one main texture stays checked.
  - Adding a texture needs the recorded version to be installed. When it isn't, the action explains that the version is missing and offers Update Template… instead.
- **Remove a vehicle:** the vehicle's **Remove from Project** action (its ⋯ menu in the Textures tab, or its vehicle card in the Project space) removes the vehicle and its surfaces. If any of them holds artwork, a confirmation names the vehicle first. The last vehicle of a project can't be removed.

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
- **THEN** its Remove from Project action is disabled, in the Textures tab and in the Project space

#### Scenario: Custom trailer added to a fleet
- **WHEN** in an ETS2 project, the user opens Add Vehicle…, clicks Custom Vehicle…, creates a trailer from the template `base.png`, then clicks Add
- **THEN** the trailer is added to the project with its "base" texture active, and the Custom Vehicle dialog showed ETS2 without a way to change it

#### Scenario: Extracted truck added to a fleet
- **WHEN** in an ETS2 project holding the sample truck, the user opens Add Vehicle… with an extracted DAF NGD installed, checks that truck and clicks Add
- **THEN** the DAF is added with its chosen cabin and accessory textures in one undo step, without a template pack prompt

#### Scenario: Several vehicles in one step
- **WHEN** in an ETS2 project holding the sample truck, the user opens Add Vehicle…, checks the installed sample trailer and an already installed Krone Cool Liner package, and clicks Add
- **THEN** both trailers are added, the sample trailer's textures then the Krone's, the sample trailer's Base is active, and one Undo removes both


#### Scenario: Official trailer added to a fleet
- **WHEN** an ETS2 project holds the sample truck and a valid official trailer package is already manually installed
- **THEN** Add Vehicle… can add that trailer with its chosen textures without invoking a catalog or template pack installation
