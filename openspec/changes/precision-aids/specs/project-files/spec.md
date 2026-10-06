## MODIFIED Requirements

### Requirement: Self-contained project file
A project SHALL be saved as one `.truckpaint` file containing everything needed to reopen it identically. The file SHALL contain:

- the project's name, texture resolution and surfaces;
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
