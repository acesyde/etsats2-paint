## MODIFIED Requirements

### Requirement: Project palette
A project SHALL hold an ordered palette of **swatches**, starting empty. Each swatch has:
- a stable identity;
- a name, which is not empty;
- a color.

Adding a color that a swatch already has SHALL add nothing. A new swatch is named "Color N", where N is the first number not yet used by such a name.

**Links:**
- A solid fill, a solid stroke and each gradient stop MAY be **linked** to a swatch. The color of a linked paint SHALL always equal its swatch's color.
- Changing a swatch's color SHALL change every color linked to it, on every surface and in every shared style, at once.
- A link SHALL be dropped when the color it links is changed to anything other than its swatch's color, or when its swatch is deleted.
- Deleting a swatch SHALL leave every color unchanged.
- Copying an object keeps its links. A link to a swatch the project doesn't have, such as on an object pasted from another project, SHALL be dropped.

#### Scenario: No duplicate swatches
- **WHEN** the same color is added to the palette twice
- **THEN** the palette contains it once

#### Scenario: A swatch recolors the fleet
- **WHEN** rectangles on two textures fill with the swatch "Company red", a gradient stop of a third object is linked to it, and its color is changed to dark red
- **THEN** both rectangles and that stop are dark red, and the other stops are unchanged

#### Scenario: Picking another color unlinks
- **WHEN** a rectangle linked to "Company red" gets a blue fill from the hex field
- **THEN** its fill is no longer linked, and editing "Company red" doesn't change it

#### Scenario: Deleting a swatch
- **WHEN** the swatch "Company red" is deleted while three objects are linked to it
- **THEN** the three objects stay red and are no longer linked
