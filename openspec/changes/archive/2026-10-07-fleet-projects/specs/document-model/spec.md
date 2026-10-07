## MODIFIED Requirements

### Requirement: Surfaces in texture-pixel coordinates
A project SHALL contain one or more surfaces. Each surface represents one texture of one variant of one of the project's vehicles. It has the texture's name and the texture's square size, in texture pixels, with the origin at the top-left corner, x to the right and y downward. Exactly one surface is active at a time.

Every surface SHALL have a template: a reference image of the texture's layout, with an opacity, a visibility, a layout version and a status, and the vehicle, variant and texture it belongs to. The template isn't part of the surface's artwork:
- it is not an object;
- it is never selected or exported;
- changing its opacity or visibility doesn't change the document's undo history.

A surface whose texture is no longer in its package version keeps its template record, marked "Not in this version", and is drawn without a template.

#### Scenario: New project surface
- **WHEN** a project is created from the sample truck's "Standard cab"
- **THEN** its active surface is the 4096 px "Cabin" texture, spanning (0, 0) to (4096, 4096)

#### Scenario: Surfaces of different sizes
- **WHEN** a vehicle project has a 4096 px "Cabin" texture and a 1024 px "Accessories" texture
- **THEN** its Cabin surface spans (0, 0) to (4096, 4096) and its Accessories surface spans (0, 0) to (1024, 1024)
