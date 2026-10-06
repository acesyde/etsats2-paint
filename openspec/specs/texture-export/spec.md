# texture-export Specification

## Purpose

Turns the vector livery into a texture file — PNG, or DDS for Euro Truck Simulator 2 and American Truck Simulator — rendered at full resolution, with a preview of exactly what will be written.

## Requirements

### Requirement: Export Texture dialog
Export › Export Texture… (Cmd/Ctrl+E), available when a project is open, SHALL open a dialog offering: the format (PNG or DDS), the output size (the project's texture resolution by default, or 1/2 or 1/4 of it), the background (an opaque color, white by default, or transparent) and, for DDS, the encoding (BC3/DXT5, default, or uncompressed RGBA). Confirming SHALL ask for a destination with a native save dialog proposing "<project name>.png" or ".dds", and then export. Cancelling SHALL close the dialog without writing anything. The dialog SHALL remember the last choices for the session.

#### Scenario: Default export settings
- **WHEN** the user opens Export Texture… on a 4096×4096 project for the first time
- **THEN** the dialog shows PNG, 4096 × 4096, a white background, and a preview of the livery

#### Scenario: Export to PNG
- **WHEN** the user confirms the dialog and chooses "ace.png"
- **THEN** a 4096×4096 PNG of the active surface is written to "ace.png"

### Requirement: Live preview
The dialog SHALL show a preview of the image that will be written, updated when the format, size, background or encoding changes, together with the output pixel size, the format and an estimated file size. The preview SHALL be rendered by the same renderer as the export (at a reduced size) and SHALL NOT freeze the dialog.

#### Scenario: Transparent background preview
- **WHEN** the user selects a transparent background
- **THEN** the preview shows a checkerboard where nothing is drawn

### Requirement: Rendering fidelity
The exported image SHALL be rendered from the vector document at the chosen size, covering exactly the surface (texture pixel (0,0) to (size,size)), with anti-aliased edges. It SHALL include every visible object in stacking order, with fill, stroke, opacity (including group opacity), rotation, rounded corners and text drawn from the same glyph outlines as the canvas. Raster images SHALL be resampled smoothly; SVG images SHALL be rendered from their vector source at the output resolution. Hidden objects (and children of hidden groups) SHALL NOT be rendered; locked objects SHALL be rendered. Parts of objects outside the surface SHALL be clipped.

#### Scenario: Hidden layer is not exported
- **WHEN** a group is hidden in the Layers panel and the texture is exported
- **THEN** none of its objects appear in the image

#### Scenario: Sharp SVG at 8K
- **WHEN** an SVG logo placed at 512 × 256 texture pixels on an 8192 project is exported at full size
- **THEN** its edges are as sharp as its other vector shapes, not an upscaled bitmap

#### Scenario: Stacking and opacity
- **WHEN** a red rectangle at 50% opacity lies above a blue ellipse
- **THEN** the overlapping pixels are the 50% blend of red over blue

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
