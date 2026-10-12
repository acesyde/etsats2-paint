# image-import Specification

## Purpose

Brings logos and artwork (PNG, JPG, SVG) into a livery as movable, resizable objects, with SVG kept sharp at any size.

## Requirements

### Requirement: Import entry points
The user SHALL be able to import PNG, JPG/JPEG and SVG files via File › Place… (Cmd/Ctrl+Shift+P, file dialog allowing several files), via the Image tool (Shift+I: clicking the canvas opens the same dialog and places the result at the click), and by dragging files from the operating system onto the window (placed under the drop point). Each imported file SHALL become one image object, selected after import, inserted in the active layer.

#### Scenario: Place a PNG
- **WHEN** the user chooses File › Place… and selects "logo.png" (800×400 pixels)
- **THEN** an image object named "logo" appears centered in the view with an 800×400 aspect ratio and is selected

#### Scenario: Drop two files
- **WHEN** the user drops "logo.svg" and "badge.jpg" onto the canvas
- **THEN** two image objects are created around the drop point and both are selected

### Requirement: Placement size
A placed image SHALL keep its natural aspect ratio. Its size SHALL be its natural pixel size (for SVG, its declared size at 1 texture pixel per SVG user unit), scaled down to fit within half of the surface if larger.

#### Scenario: Large image is scaled down
- **WHEN** a 6000×3000 pixel image is placed on a 4096×4096 surface
- **THEN** it is placed at 2048×1024 texture pixels

### Requirement: Image objects
Image objects SHALL support move, resize (Shift keeps proportions, as for every object), rotate, opacity, hide, lock, group, duplicate, copy/paste and delete like other objects. Fill, stroke and corner radius do not apply to images and their controls SHALL be hidden for an image-only selection.

#### Scenario: Proportional resize with Shift
- **WHEN** a 800×400 image is resized from a corner handle with Shift held
- **THEN** it keeps a 2:1 aspect ratio

### Requirement: SVG stays vector
SVG images SHALL be rendered from their vector source at a resolution matching the current zoom (re-rendered when zooming in further), so they stay sharp at any zoom; the document SHALL keep the SVG source, never a rasterized copy.

#### Scenario: Zooming into an SVG logo
- **WHEN** an SVG logo is placed and the user zooms to 800%
- **THEN** its edges remain sharp after a short re-render, without pixelation

### Requirement: Import errors
Files that are not valid PNG, JPG or SVG, or that cannot be read, SHALL NOT create objects; the user SHALL see a non-blocking message naming the file and the reason (for example "badge.gif: unsupported format"). Other files of the same import SHALL still be imported.

#### Scenario: Unsupported file
- **WHEN** the user drops "anim.gif" and "logo.png"
- **THEN** the logo is placed and a message says "anim.gif: unsupported format"

### Requirement: Assets are deduplicated
Importing a file whose content is identical to an existing asset SHALL reuse that asset instead of storing a second copy.

#### Scenario: Same logo twice
- **WHEN** the user places "logo.png" twice
- **THEN** two image objects exist and the images list of the Resources tab lists one asset used 2 times

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
