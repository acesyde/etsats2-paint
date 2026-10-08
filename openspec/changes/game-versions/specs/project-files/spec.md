## MODIFIED Requirements

### Requirement: Self-contained project file
A project SHALL be saved as one `.truckpaint` file containing everything needed to reopen it identically. The file SHALL contain:

- the project's name, game versions and surfaces, with each surface's name and size;
- the project's vehicles, each with its package id, version, name, brand, kind and game, and its game data: game path, supported game versions, whether it uses the alternate UV set and the colour picker, and the mods it requires;
- each surface's template: the vehicle and texture it belongs to, whether that texture is a main texture or an accessory, its game ids, its image, opacity, visibility, layout version and status;
- the full object tree, with every object's identity, name, kind, geometry (including polygon settings, the points and handles of every path subpath with its open or closed state, and the path's line width), fill, stroke, opacity, visibility and lock flags;
- each text's content and character style;
- image references;
- whether each text and image is mirrored;
- the guides of each surface;
- the project palette: each swatch's identity, name and color;
- the shared styles: each graphic style's identity, name, fill, stroke and opacity, and each text style's identity, name, character style, fill, stroke and opacity;
- the links: for every solid fill, solid stroke and gradient stop, the swatch it is linked to; for every object, the graphic style and the text style it follows;
- the symbols: each symbol's identity, name, artboard size, guides and content (an object tree, as for a surface);
- each instance's symbol, placement, opacity, visibility, lock and name. An instance's content is not stored: it is rebuilt from its symbol when the file opens;
- the mod settings: Name, Version, Author, Description, Price, Unlock level, internal name and whether it was edited, and the chosen shop icon and Mod Manager image, if any;
- the bytes of every imported asset, chosen mod images included.

Font families SHALL be stored by name only; font files SHALL NOT be embedded. Editor state that is not part of the document (selection, point selection, zoom, panel layout, grid and snapping settings, whether guides are shown, undo history) SHALL NOT be stored.

A file written before swatches had names SHALL open with each palette color as a swatch named "Color 1", "Color 2"… in palette order, and nothing linked. A file written before texts and images could be mirrored SHALL open with none of them mirrored. A file written before mod settings existed SHALL open with the default mod settings. A file written before game versions existed SHALL open with none.

#### Scenario: Round trip
- **WHEN** the user saves a project with grouped shapes, an outlined text, a placed SVG logo, a star, a curved path with a hole and two guides, closes it and opens the file again
- **THEN** every object, group, text, image, path point and handle, guide, palette color and asset is restored with the same properties and stacking order

#### Scenario: Opening on another computer
- **WHEN** a project using a system font that is not installed is opened on another computer
- **THEN** it opens, the text is drawn with Inter and the font control shows the "Font not found" warning with the original family name

#### Scenario: Vehicle project round trip
- **WHEN** the user saves a project with a truck painting two main textures and two accessories and a trailer painting its Base texture, with one template hidden and one flagged "Layout changed", closes it and opens the file again
- **THEN** the vehicles and their recorded versions and game data, the surfaces with their main texture or accessory role and game ids, their templates, opacities, visibilities and flags are restored

#### Scenario: Brand kit round trip
- **WHEN** the user saves a project with the swatches "Company red" and "Company grey", a graphic style whose gradient has a stop linked to "Company red", a text style, a rectangle following the graphic style and a text following the text style with its fill linked to "Company grey", then closes and reopens it
- **THEN** the swatches, the styles and every link are restored, and editing "Company red" still recolors the style and the rectangle

#### Scenario: Symbols round trip
- **WHEN** the user saves a project with the symbol "Logo", holding a circle and a text, with an instance on two textures, one rotated and flipped, closes it and opens the file again
- **THEN** "Logo" and both instances are restored with their placements, and editing "Logo" updates both

#### Scenario: Palette of an earlier file
- **WHEN** a file whose palette holds three colors without names is opened
- **THEN** the palette shows the swatches "Color 1", "Color 2" and "Color 3" with those colors, and no object is linked

#### Scenario: Mirrored objects round trip
- **WHEN** the user saves a project with a mirrored image, a text flipped vertically and an image that is not mirrored, closes it and opens the file again
- **THEN** the first image is still mirrored, the text is still drawn upside down and mirrored, and the other image is not mirrored

#### Scenario: Mod settings round trip
- **WHEN** the user saves a project whose mod settings have the Version "2.0", the Author "Jane", the Price 7500, the edited internal name "acelog" and a chosen Mod Manager image, closes it and opens the file again
- **THEN** Export Mod… shows the same settings and image, and changing the Name leaves the internal name "acelog"

#### Scenario: File without mod settings
- **WHEN** a project named "ACE" saved before mod settings existed is opened
- **THEN** Export Mod… shows the Name "ACE", the Version "1.0", the Price 5000 and generated images

#### Scenario: Game versions round trip
- **WHEN** the user saves a project whose Game versions are `1.56.*, 1.57.*`, closes it and opens the file again
- **THEN** the Game versions field shows "1.56.*, 1.57.*"
