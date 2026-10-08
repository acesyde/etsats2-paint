## ADDED Requirements

### Requirement: Copy from cabin
**Vehicle › Copy From Cabin…** SHALL copy the artwork of another main texture of the same truck onto the active texture. The cabin layouts of a truck share most of their texture, so the same coordinates fit.

It SHALL be enabled only when:
- the active texture is a main texture of a vehicle;
- the project paints at least one other main texture of that vehicle.

Otherwise, a tooltip says why. Trailers and accessories have no other main texture to copy from.

It SHALL open a dialog listing the vehicle's other main textures in the project, each with the number of objects it holds. The first texture that holds objects is chosen by default. Confirming SHALL:
- copy every object of the chosen texture, in stacking order, including hidden and locked ones, onto the active texture, above its objects;
- keep each copy's position in texture pixels. When the two textures differ in size, the copies are scaled by the ratio of the sizes, as Update Template scales artwork;
- keep each copy's name, look, links to swatches and styles, visibility and lock;
- select the copies;
- record one undo step.

The chosen texture SHALL be unchanged. Guides are not copied. When the chosen texture holds no object, the dialog SHALL say so and Copy is disabled.

#### Scenario: Copy the standard cab onto the high roof
- **WHEN** the sample truck's Standard cab holds a logo and lettering, the High roof is active and empty, and the user chooses Copy From Cabin… with Standard cab
- **THEN** the High roof holds copies of the logo and the lettering at the same positions, selected, and one Undo removes them

#### Scenario: Different sizes
- **WHEN** a 2048 px main texture holds a rectangle at (100, 100), 200 × 50, and it is copied onto a 4096 px main texture of the same truck
- **THEN** the copy is at (200, 200), 400 × 100

#### Scenario: Not offered for accessories
- **WHEN** the active texture is the sample truck's Chassis
- **THEN** Copy From Cabin… is disabled, with a tooltip saying it copies between main textures

#### Scenario: Links kept
- **WHEN** a text following the text style "Lettering" is copied from cabin
- **THEN** the copy follows "Lettering" too
