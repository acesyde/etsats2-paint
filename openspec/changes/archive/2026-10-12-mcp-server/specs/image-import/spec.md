## ADDED Requirements

### Requirement: Import through the MCP server
The MCP server's **import_image** tool (see the mcp-server capability) SHALL import a PNG, JPEG or SVG image into a texture from exactly one of:
- `data`: the file's content in base64, with a `name` used as the asset's name and to tell the format;
- `path`: an absolute path to a local file with a `.png`, `.jpg`, `.jpeg` or `.svg` extension, read once.

It SHALL accept the same formats, give the same import errors and deduplicate assets as File › Import. A file larger than 50 MB, a path with another extension, a relative path, a missing file, or both or neither of `data` and `path` SHALL make the call fail with a message naming the problem, and nothing SHALL be imported. No other file SHALL be read.

The image SHALL be placed at the given `x, y, width, height` (and optional rotation). Without a box, it SHALL be centered on the texture at the size File › Import gives it, keeping its aspect ratio. An SVG SHALL stay vector, as with File › Import.

#### Scenario: Logo pasted in the chat
- **WHEN** the client calls import_image with a base64 PNG named "logo.png" and a box
- **THEN** the image is placed at that box, the asset "logo.png" is listed in Resources, and the result gives the new object's id

#### Scenario: Logo by path
- **WHEN** the client calls import_image with `path` "/Users/me/logos/acme.svg" and no box
- **THEN** the SVG is placed centered on the texture, sharp at any zoom

#### Scenario: Same image twice
- **WHEN** the client imports the same PNG twice, once by path and once as base64
- **THEN** two image objects are placed and Resources lists one asset

#### Scenario: Not an image
- **WHEN** the client calls import_image with `path` "/Users/me/.ssh/id_rsa"
- **THEN** the call fails saying only PNG, JPEG and SVG files can be imported, and the file is not read
