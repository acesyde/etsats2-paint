## ADDED Requirements

### Requirement: Game data recorded with each vehicle
Each vehicle of a project SHALL record the game data of its package version, which the mod export needs:
- its game path, supported game versions, whether it uses the alternate UV set and the colour picker, and the mods it requires;
- for each of its textures in the project, the texture's game ids: the cabins of a main texture, or the accessory ids of an accessory.

This data SHALL be recorded when the vehicle enters the project, through New Project or Add Vehicle…, and when Textures… adds textures. Update Template SHALL replace it with the data of the new version, and undoing the update SHALL restore the previous data. A texture marked "Not in this version" keeps the game ids it had.

#### Scenario: Recorded on creation
- **WHEN** the user creates a project from the sample truck 1.1.0 painting Standard cab and Side skirts
- **THEN** the vehicle records the game path "truckpaint.sample", Standard cab records the game id "standard" and Side skirts records "sideskirt.sample"

#### Scenario: Replaced by Update Template
- **WHEN** a package's version 1.3.0 changes an accessory's game ids and the user updates a project's vehicle from 1.2.0 to 1.3.0
- **THEN** that accessory records the game ids of 1.3.0, and Undo restores those of 1.2.0
