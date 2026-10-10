## MODIFIED Requirements

### Requirement: Usage of shared elements
Each swatch, graphic style and text style SHALL have a **usage**: the number of objects it changes and the number of textures holding them, over every texture of the project.

- A **swatch** changes an object when the object's fill, stroke, shadow or one of its gradient stops is linked to it. It also changes each instance of a symbol whose content has such an object: the instance counts as one object.
- A **style** changes the objects that follow it, including the texts that follow a text style.
- Objects inside groups count one by one; a group itself is not counted. An instance counts as one object, whatever its content holds.
- A texture counts once, however many of its objects are counted.

Each **symbol** SHALL have its number of instances on every texture and the number of textures holding them.

The usage SHALL read "11 textures · 38 objects", with singular forms for one ("1 texture · 1 object"), or **Unused** when nothing is counted. A symbol's usage SHALL read "14 instances · 9 textures", or Unused. The usage SHALL follow the project: after any edit, undo or redo, it shows the new counts.

#### Scenario: A swatch used on two textures
- **WHEN** "Company red" is the fill of three rectangles of the Cab and the stroke of one circle of the Chassis
- **THEN** its usage reads "2 textures · 4 objects"

#### Scenario: Through a symbol
- **WHEN** the only object linked to "Company red" is inside the symbol "Logo", which has five instances on three textures
- **THEN** the usage of "Company red" reads "3 textures · 5 objects"

#### Scenario: An object linked twice
- **WHEN** one rectangle has its fill and its stroke linked to "Company red"
- **THEN** the rectangle counts once

#### Scenario: Unused swatch
- **WHEN** no color of the project is linked to "Cream"
- **THEN** the usage of "Cream" reads "Unused"

#### Scenario: Usage follows undo
- **WHEN** the user links one more rectangle to "Company red" and then undoes it
- **THEN** the usage of "Company red" goes up by one object, then back

#### Scenario: Symbol usage
- **WHEN** "Logo" has four instances on the Cab and two on the Chassis
- **THEN** its usage reads "6 instances · 2 textures"

#### Scenario: Through a shadow
- **WHEN** the only use of "Night" is the shadow of one text on the Cab
- **THEN** the usage of "Night" reads "1 texture · 1 object"
