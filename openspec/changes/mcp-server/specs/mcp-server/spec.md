## Purpose

Lets an external LLM client draw on the open project through a local MCP server: read the project, see a texture's template and artwork, and create or change objects on a texture, with every change undoable and no file written.

## ADDED Requirements

### Requirement: Local server, off by default
The application SHALL provide an MCP server, using the Streamable HTTP transport at `http://127.0.0.1:<port>/mcp`, that runs only while the application is open and the **AI assistant (MCP)** setting is on (see the app-preferences capability). The setting SHALL be off by default. The server SHALL listen on the loopback address only and SHALL refuse any request that carries an `Origin` header other than `http://127.0.0.1:<port>` or `http://localhost:<port>`.

When the port is already in use, the server SHALL stay off and Preferences SHALL say that the port is in use.

The server SHALL answer `initialize`, `ping`, `tools/list` and `tools/call`. Its `initialize` result SHALL include instructions describing the workflow: read the project, look at the texture's template, draw, render the texture, compare with the reference and correct.

#### Scenario: Off on first launch
- **WHEN** the application starts for the first time and an MCP client tries to connect to `http://127.0.0.1:7373/mcp`
- **THEN** the connection is refused

#### Scenario: Client lists the tools
- **WHEN** the setting is on and an MCP client connects and asks for the tool list
- **THEN** it receives the read, see and write tools described below, each with a description and an input schema

#### Scenario: Web page blocked
- **WHEN** the setting is on and a request arrives with `Origin: https://example.com`
- **THEN** it is refused and the document is unchanged

#### Scenario: Port in use
- **WHEN** another program listens on the configured port and the user turns the setting on
- **THEN** the server stays off and Preferences shows that the port is in use

#### Scenario: Turned off
- **WHEN** a client is connected and the user turns the setting off
- **THEN** the server stops, later calls fail to connect, and the document keeps every change already made

### Requirement: Read tools
The server SHALL provide tools that read the open project without changing it:
- **get_project** returns the project name, the game, the vehicles, and for each vehicle its textures with an id, a name, whether it is a main texture or an accessory, and its width and height in texture pixels; the palette swatches (id, name, color); the graphic and text styles (id, name); and the symbols (id, name);
- **list_objects** returns the object tree of one texture, top to bottom, with for each object its id, kind, name, box in texture pixels, rotation, visibility and lock;
- **get_object** returns one object's properties: kind, name, box, rotation, fill, stroke, opacity, shadow, the swatch and style it follows, its text for a text, its path data (SVG syntax) for a path.

A texture id SHALL stay the same across sessions for the same vehicle texture. With no project open, read tools SHALL fail with a message saying no project is open.

#### Scenario: Reading a fleet
- **WHEN** a project with a DAF truck (one main texture and two accessories) and a Krone trailer is open and the client calls get_project
- **THEN** the result lists both vehicles, the three DAF textures and the Krone texture, each with its id and size

#### Scenario: No project open
- **WHEN** the Start screen is shown and the client calls get_project
- **THEN** the call fails with a message saying no project is open

#### Scenario: Reading a path
- **WHEN** a texture holds a path and the client calls get_object with its id
- **THEN** the result holds the path's data in SVG path syntax, in texture pixels

### Requirement: See tools
The server SHALL provide tools that return images:
- **get_template** returns the texture's template as a PNG image;
- **render_texture** returns the texture's artwork as a PNG image, drawn as Export Texture draws it, with the template underneath unless `with_template` is false.

Both SHALL take an optional `max_size` (longest side in pixels), default 1024, never more than 2048, never upscaling, and SHALL state in their text result the image size and how many texture pixels one image pixel covers. A texture without a template SHALL make get_template fail with a message saying so.

#### Scenario: Seeing the result
- **WHEN** the client adds a red rectangle on a 4096 × 4096 texture and calls render_texture without `max_size`
- **THEN** it receives a 1024 × 1024 PNG showing the rectangle over the template, and the text says 1 px = 4 texture px

#### Scenario: Size capped
- **WHEN** the client calls render_texture with `max_size` 8000 on a 4096 × 4096 texture
- **THEN** the image is 2048 × 2048

### Requirement: Write tools
The server SHALL provide tools that change one texture, named by its id in every call, using texture pixels (origin top-left, y down) and colors as hex (`#rrggbb` or `#rrggbbaa`) or as a palette swatch, which links the color to that swatch:
- **add_shape**: a rectangle (with corner radius), an ellipse, a polygon (3 to 12 sides) or a star, at `x, y, width, height` with an optional rotation, fill, stroke, opacity and name;
- **add_path**: a path from SVG path data (`d`, absolute or relative commands, arcs included), with a fill rule, fill, stroke, opacity and name; several subpaths make one path with holes;
- **add_text**: a text with its content, font family, size, alignment and color, at a position;
- **import_image**: see the image-import capability;
- **update_object**: changes any of an object's box, rotation, fill (color, gradient or swatch), stroke, opacity, shadow, name, visibility and lock; a changed look detaches the object from its style, as in the inspector;
- **arrange_objects**: brings forward, sends backward, to front or to back; groups or ungroups; aligns to each other or to the texture; combines with a boolean operation (unite, subtract, intersect, exclude);
- **delete_objects**: deletes objects;
- **add_swatch**: adds a named color to the palette;
- **apply_style**: applies a graphic or text style to objects;
- **place_symbol**: places an instance of a symbol at a box with an optional rotation.

Every created object SHALL be added at the top of the texture's layers unless a parent group is given. The result of a write call SHALL list the ids of the objects it created or changed. After a write call, those objects SHALL be the selection when that texture is the one shown.

A write call SHALL apply completely or not at all. Invalid input (unknown texture or object id, non-finite number, a size of zero or less, malformed path data or color, an operation that doesn't apply to the given objects) SHALL fail with a message that names the problem and SHALL leave the document unchanged.

#### Scenario: Drawing a stripe
- **WHEN** the client calls add_path on a DAF main texture with `d = "M 0 1800 C 1200 1500 2800 2100 4096 1800 L 4096 2000 C 2800 2300 1200 1700 0 2000 Z"` and fill `#d4202c`
- **THEN** a red path with that outline is added at the top of that texture's layers and its id is returned

#### Scenario: Ring with a hole
- **WHEN** the client calls add_path with two circles as two subpaths and the even-odd fill rule
- **THEN** one path is added, drawn as a ring

#### Scenario: Linking to the palette
- **WHEN** the palette holds a swatch "Company red" and the client adds a shape with fill `{ "swatch": "Company red" }`, then the user edits the swatch to dark red
- **THEN** the shape is dark red

#### Scenario: Malformed path data
- **WHEN** the client calls add_path with `d = "M 0 0 L foo"`
- **THEN** the call fails with a message about the path data and nothing is added

#### Scenario: Partial failure
- **WHEN** the client calls delete_objects with two existing ids and one unknown id
- **THEN** the call fails naming the unknown id and no object is deleted

#### Scenario: Drawing on another texture
- **WHEN** the user is on the DAF main texture and the client adds a shape on the DAF sideskirt texture
- **THEN** the shape is on the sideskirt texture, the DAF main texture is still shown, and its selection is unchanged

### Requirement: No file writes
No tool SHALL write, create or delete a file, save the project, export a texture or a mod, open, create or close a project, or change preferences. The only file the server reads is an image given by path to import_image (see the image-import capability). Changes made through the server SHALL mark the project as unsaved, like any other change.

#### Scenario: Saving stays with the user
- **WHEN** the client adds a shape to a saved project
- **THEN** the status bar shows "Unsaved changes" and the file on disk is unchanged until the user saves

### Requirement: The user's work comes first
A tool call that arrives while the user is in the middle of a gesture (dragging or transforming objects, drawing a path with the Pen, editing a text, a gradient drag) or while a modal dialog is open SHALL wait until it ends, then run. A call that has waited 30 seconds SHALL fail with a message saying TruckPaint is busy, and SHALL NOT run later.

While a symbol is being edited, write tools SHALL fail with a message asking to finish editing the symbol; read and see tools SHALL still work.

#### Scenario: Call during a drag
- **WHEN** the user is dragging a rectangle and the client adds an ellipse
- **THEN** the ellipse appears after the user releases the mouse, and the drag and the addition are two undo steps

#### Scenario: Dialog left open
- **WHEN** the Export Mod dialog is open and the client calls add_shape, and the dialog stays open for 30 seconds
- **THEN** the call fails saying TruckPaint is busy, and closing the dialog later doesn't add the shape

#### Scenario: Symbol being edited
- **WHEN** the user is editing a symbol and the client calls add_shape on a texture
- **THEN** the call fails asking to finish editing the symbol
