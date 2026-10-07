## MODIFIED Requirements

### Requirement: Self-contained project file
A project SHALL be saved as one `.truckpaint` file containing everything needed to reopen it identically. The file SHALL contain:

- the project's name and surfaces, with each surface's name and size;
- the project's vehicles, each with its package id, version, name, brand, kind and game;
- each surface's template: the vehicle and texture it belongs to, whether that texture is a main texture or an accessory, its image, opacity, visibility, layout version and status;
- the full object tree, with every object's identity, name, kind, geometry (including polygon settings, the points and handles of every path subpath with its open or closed state, and the path's line width), fill, stroke, opacity, visibility and lock flags;
- each text's content and character style;
- image references;
- the guides of each surface;
- the project palette;
- the bytes of every imported asset.

Font families SHALL be stored by name only; font files SHALL NOT be embedded. Editor state that is not part of the document (selection, point selection, zoom, panel layout, grid and snapping settings, whether guides are shown, undo history) SHALL NOT be stored.

#### Scenario: Round trip
- **WHEN** the user saves a project with grouped shapes, an outlined text, a placed SVG logo, a star, a curved path with a hole and two guides, closes it and opens the file again
- **THEN** every object, group, text, image, path point and handle, guide, palette color and asset is restored with the same properties and stacking order

#### Scenario: Opening on another computer
- **WHEN** a project using a system font that is not installed is opened on another computer
- **THEN** it opens, the text is drawn with Inter and the font control shows the "Font not found" warning with the original family name

#### Scenario: Vehicle project round trip
- **WHEN** the user saves a project with a truck painting two main textures and two accessories and a trailer painting its Base texture, with one template hidden and one flagged "Layout changed", closes it and opens the file again
- **THEN** the vehicles and their recorded versions, the surfaces with their main texture or accessory role, their templates, opacities, visibilities and flags are restored

### Requirement: Invalid files
Files that are not TruckPaint projects, are damaged, were written by a newer version of TruckPaint, or use a format from unreleased development builds SHALL NOT be opened. The user SHALL see a message naming the file and the reason (for example "ace.truckpaint was created with a newer version of TruckPaint"). The currently open project, if any, SHALL stay open and unchanged.

Development builds wrote project files that this version refuses as a development format:
- files without a vehicle, written before every project needed one;
- files whose vehicles record variants, written before projects followed the game's paint job structure.

Files mixing games SHALL be refused as damaged.

#### Scenario: Damaged file
- **WHEN** the user opens a truncated `.truckpaint` file
- **THEN** a message says the file is damaged and cannot be opened, and nothing else changes

#### Scenario: Newer format
- **WHEN** the user opens a file whose format version is newer than the application supports
- **THEN** a message says it was created with a newer version of TruckPaint

#### Scenario: Development build file
- **WHEN** the user opens a file saved by an unreleased development build with format 3
- **THEN** a message says the file uses a development format that this version cannot open, and nothing else changes

#### Scenario: Blank-texture file from a development build
- **WHEN** the user opens a file saved by a development build for a blank texture, with no vehicle
- **THEN** a message says the file uses a development format that this version cannot open, and nothing else changes

#### Scenario: Variant file from a development build
- **WHEN** the user opens a file saved by a development build whose vehicle records the variants "Standard cab" and "High roof"
- **THEN** a message says the file uses a development format that this version cannot open, and nothing else changes
