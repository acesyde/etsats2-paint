## MODIFIED Requirements

### Requirement: Surfaces in texture-pixel coordinates
A project SHALL contain one or more surfaces. Each surface represents one texture of the vehicle and has a name and a square size: the project resolution for a blank-texture project, or the texture's size for a vehicle project. Sizes are expressed in texture pixels with the origin at the top-left corner, x to the right and y downward. A new blank-texture project SHALL contain exactly one surface named "Main texture". Exactly one surface is active at a time. A surface MAY have a template: a reference image of the vehicle's texture layout, with an opacity, a visibility, a layout version and a status. The template is not part of the surface's artwork: it is not an object, it is never selected or exported, and changing its opacity or visibility does not change the document's undo history.

#### Scenario: New project surface
- **WHEN** a 4096×4096 project is created
- **THEN** it contains one active surface named "Main texture" spanning (0, 0) to (4096, 4096)

#### Scenario: Surfaces of different sizes
- **WHEN** a vehicle project has a 4096 px "Cabin" texture and a 1024 px "Accessories" texture
- **THEN** its Cabin surface spans (0, 0) to (4096, 4096) and its Accessories surface spans (0, 0) to (1024, 1024)
