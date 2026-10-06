## MODIFIED Requirements

### Requirement: Self-contained project file
A project SHALL be saved as one `.truckpaint` file containing everything needed to reopen it identically. The file SHALL contain:

- the project's name, texture resolution and surfaces;
- the full object tree, with every object's identity, name, kind, geometry (including polygon settings and the points and handles of every path subpath, with its open or closed state), fill, stroke, opacity, visibility and lock flags;
- each text's content and character style;
- image references;
- the project palette;
- the bytes of every imported asset.

Font families SHALL be stored by name only; font files SHALL NOT be embedded. Editor state that is not part of the document (selection, point selection, zoom, panel layout, undo history) SHALL NOT be stored.

#### Scenario: Round trip
- **WHEN** the user saves a project with grouped shapes, an outlined text, a placed SVG logo, a star and a curved path with a hole, closes it and opens the file again
- **THEN** every object, group, text, image, path point and handle, palette color and asset is restored with the same properties and stacking order

#### Scenario: Opening on another computer
- **WHEN** a project using a system font that is not installed is opened on another computer
- **THEN** it opens, the text is drawn with Inter and the font control shows the "Font not found" warning with the original family name
