## MODIFIED Requirements

### Requirement: Adding and removing vehicles and textures
The user SHALL be able to change which vehicles and textures a project covers. Each change is one undo step.

- **Add a vehicle:** Vehicle › Add Vehicle…, also reached from the Add Vehicle button of the Textures tab and of the Project space, opens a dialog listing the vehicles of the project's game that aren't in the project yet: first the official vehicles of the catalog, installed or not, marked "Official", then the other installed vehicles (see the official-vehicles capability). It is searchable by name and brand and filterable by kind. Picking an official vehicle installs it first when needed, asking for the template pack when its folder isn't known. Picking a vehicle shows its main textures and accessories as checkboxes, as in New Project, and at least one main texture must be checked. Confirming adds the vehicle at its newest installed version and adds one surface per chosen texture, after the existing surfaces. The first new surface becomes active.
  - The dialog SHALL also offer **Custom vehicle…**, which opens the Custom Vehicle dialog with the project's game, locked (see the custom-vehicles capability). A vehicle created there SHALL be listed and selected when Add Vehicle… is shown again, with its textures checked as for any picked vehicle; it is not added to the project until the user confirms Add.
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
- **WHEN** in an ETS2 project, the user opens Add Vehicle…, clicks Custom vehicle…, creates a trailer from the template `base.png`, then clicks Add
- **THEN** the trailer is added to the project with its "base" texture active, and the Custom Vehicle dialog showed ETS2 without a way to change it

#### Scenario: Official trailer added to a fleet
- **WHEN** in an ETS2 project holding the sample truck, with the ETS2 pack folder known, the user opens Add Vehicle…, picks the Krone Cool Liner, waits for it to install, and clicks Add
- **THEN** the trailer is added with its Body and four chassis accessories, and Body is active

### Requirement: Game data recorded with each vehicle
Each vehicle of a project SHALL record the game data of its package version, which the mod export needs:
- its game path, supported game versions, whether it uses the alternate UV set and the colour picker, and the mods it requires;
- for each of its textures in the project, the texture's game ids: the cabins of a main texture, or the accessory ids of an accessory;
- for a main texture, the accessory ids it is also applied to, when its package lists some.

This data SHALL be recorded when the vehicle enters the project, through New Project or Add Vehicle…, and when Textures… adds textures. Update Template SHALL replace it with the data of the new version, and undoing the update SHALL restore the previous data. A texture marked "Not in this version" keeps the game ids it had.

#### Scenario: Recorded on creation
- **WHEN** the user creates a project from the sample truck 1.1.0 painting Standard cab and Side skirts
- **THEN** the vehicle records the game path "truckpaint.sample", Standard cab records the game id "standard" and Side skirts records "sideskirt.sample"

#### Scenario: Replaced by Update Template
- **WHEN** a package's version 1.3.0 changes an accessory's game ids and the user updates a project's vehicle from 1.2.0 to 1.3.0
- **THEN** that accessory records the game ids of 1.3.0, and Undo restores those of 1.2.0

#### Scenario: Accessories painted by a main texture
- **WHEN** the user creates a project from the Krone Cool Liner
- **THEN** its Body records the accessory ids `body.reefer` and `body.reef_13ct` it is also applied to, and they survive saving and reopening the project
