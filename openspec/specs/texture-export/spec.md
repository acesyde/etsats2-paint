# texture-export Specification

## Purpose

Turns the vector livery into a texture file — PNG, or DDS for Euro Truck Simulator 2 and American Truck Simulator — rendered at full resolution, with a preview of exactly what will be written.

## Requirements

### Requirement: Export Texture dialog
File › Export Texture… (Shift+Cmd/Ctrl+E), available when a project is open, SHALL open a dialog offering:
- the format: PNG or DDS;
- the output size: the active surface's size by default, or 1/2 or 1/4 of it;
- the background: an opaque color (white by default) or transparent;
- for DDS, the encoding: BC3/DXT5 (default) or uncompressed RGBA.

Confirming SHALL ask for a destination with a native save dialog, then export. The proposed file name is "<project name> - <vehicle name> - <texture name>" with the format's extension. Cancelling SHALL close the dialog without writing anything. The dialog SHALL remember the last choices for the session. The export SHALL contain the active surface's artwork only, never its template. Cmd/Ctrl+E SHALL NOT open this dialog: it opens Export Mod….

#### Scenario: Default export settings
- **WHEN** the user opens Export Texture… on a project whose active texture is 4096 px, for the first time
- **THEN** the dialog shows PNG, 4096 × 4096, a white background, and a preview of the livery

#### Scenario: Export to PNG
- **WHEN** the user confirms the dialog and chooses "ace.png"
- **THEN** a 4096×4096 PNG of the active surface is written to "ace.png"

#### Scenario: Exporting a smaller texture
- **WHEN** the active surface of a project "ACE" is the sample truck's 1024 px "Cab accessories" texture, and the user opens Export Texture…
- **THEN** the dialog offers 1024 × 1024 by default and proposes "ACE - TruckPaint Sample Truck - Cab accessories.png"

#### Scenario: Opening by key
- **WHEN** a project is open and the user presses Shift+Cmd/Ctrl+E
- **THEN** the Export Texture dialog opens, and pressing Cmd/Ctrl+E instead opens the Export Mod dialog

### Requirement: Live preview
The dialog SHALL show a preview of the image that will be written, updated when the format, size, background or encoding changes, together with the output pixel size, the format and an estimated file size. The preview SHALL be rendered by the same renderer as the export (at a reduced size) and SHALL NOT freeze the dialog.

#### Scenario: Transparent background preview
- **WHEN** the user selects a transparent background
- **THEN** the preview shows a checkerboard where nothing is drawn

### Requirement: Rendering fidelity
The exported image SHALL be rendered from the vector document at the chosen size, covering exactly the surface (texture pixel (0,0) to (size,size)), with anti-aliased edges.

It SHALL include every visible object in stacking order, drawn with:

- fill, stroke, opacity (including group opacity), rotation and rounded corners;
- drop shadows, drawn as on the canvas (see the drop-shadow capability);
- solid colors and linear and radial gradients, for fills and strokes alike, matching the canvas;
- polygons, stars and paths with their exact curves, the same as on the canvas: closed subpaths filled with the non-zero rule (holes included), open subpaths drawn as lines of their width in the fill color with rounded ends, outlined by the stroke when set;
- text drawn from the same glyph outlines as the canvas.

Raster images SHALL be resampled smoothly; SVG images SHALL be rendered from their vector source at the output resolution. Hidden objects (and children of hidden groups) SHALL NOT be rendered; locked objects SHALL be rendered. Parts of objects outside the surface SHALL be clipped.

#### Scenario: Hidden layer is not exported
- **WHEN** a group is hidden in the Layers panel and the texture is exported
- **THEN** none of its objects appear in the image

#### Scenario: Sharp SVG at 8K
- **WHEN** an SVG logo placed at 512 × 256 texture pixels on an 8192 project is exported at full size
- **THEN** its edges are as sharp as its other vector shapes, not an upscaled bitmap

#### Scenario: Stacking and opacity
- **WHEN** a red rectangle at 50% opacity lies above a blue ellipse
- **THEN** the overlapping pixels are the 50% blend of red over blue

#### Scenario: Open path in the export
- **WHEN** a red open "V" path with a line width of 20 px and no stroke is exported
- **THEN** the image shows a red "V" line 20 px wide, with no red area between its arms

#### Scenario: Gradient in the export
- **WHEN** a 1024 × 1024 project has a full-surface rectangle with a linear fill gradient from opaque black at the left edge to opaque white at the right edge, and it is exported at 1024 px
- **THEN** each pixel column's gray level increases from 0 to 255 from left to right, and the middle column is within one step of 128

#### Scenario: Shadow in the export
- **WHEN** a white square with a hard black shadow offset 10 / 10 lies on a red background and the texture is exported as PNG at full size
- **THEN** the image shows the black shadow 10 px right and below the square, under it, as on the canvas

### Requirement: DDS for the games
DDS export SHALL write a standard DDS file of the chosen size with a full mipmap chain down to 1×1, encoded either as BC3 (DXT5, alpha kept) or as uncompressed 32-bit RGBA, loadable by Euro Truck Simulator 2 and American Truck Simulator and common DDS tools.

#### Scenario: DXT5 with mipmaps
- **WHEN** a 2048×2048 project is exported as DDS with the default encoding
- **THEN** the file declares DXT5 compression, a 2048×2048 size and 12 mipmap levels

### Requirement: Background export with progress
Rendering, encoding and writing SHALL run without freezing the editor; while they run, the dialog SHALL show progress and a Cancel button. Cancelling SHALL stop the export and leave no partial file at the destination. When the export finishes, the dialog SHALL close and a confirmation naming the file SHALL be shown; when it fails, a message SHALL name the file and the reason. Exporting SHALL NOT modify the project or its save state.

#### Scenario: Export does not change the project
- **WHEN** a saved project is exported
- **THEN** the status bar still shows "Saved" and the undo history is unchanged

#### Scenario: Unwritable destination
- **WHEN** the chosen destination cannot be written
- **THEN** a message names the file and the reason, and no file is created
