## MODIFIED Requirements

### Requirement: Surfaces in texture-pixel coordinates
A project SHALL contain one or more surfaces. Each surface represents one texture of one of the project's vehicles: a main texture or an accessory. It has the texture's name and the texture's width and height, in texture pixels, each a power of two from 64 to 8192 (equal for a square texture, different otherwise), with the origin at the top-left corner, x to the right and y downward. Exactly one surface is active at a time.

Wherever a texture's size is shown, it SHALL read "4096²" when its sides are equal and "8192 × 1024" (width × height) otherwise.

Every surface SHALL have a template: a reference image of the texture's layout, with an opacity, a visibility, a layout version and a status, and the vehicle and texture it belongs to, with whether that texture is a main texture or an accessory. The template isn't part of the surface's artwork:
- it is not an object;
- it is never selected or exported;
- changing its opacity or visibility doesn't change the document's undo history.

A surface whose texture is no longer in its package version keeps its template record, marked "Not in this version", and is drawn without a template.

#### Scenario: New project surface
- **WHEN** a project is created from the sample truck's "Standard cab"
- **THEN** its active surface is the 4096 px "Standard cab" main texture, spanning (0, 0) to (4096, 4096)

#### Scenario: Surfaces of different sizes
- **WHEN** a vehicle project has a 4096 px "Standard cab" texture and a 1024 px "Cab accessories" texture
- **THEN** its Standard cab surface spans (0, 0) to (4096, 4096) and its Cab accessories surface spans (0, 0) to (1024, 1024)

#### Scenario: Rectangular texture
- **WHEN** a project paints a trailer whose main texture is 8192 × 1024
- **THEN** its surface spans (0, 0) to (8192, 1024), and its size reads "8192 × 1024"
