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
