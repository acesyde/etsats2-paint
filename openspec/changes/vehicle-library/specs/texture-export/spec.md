## MODIFIED Requirements

### Requirement: Export Texture dialog
Export › Export Texture… (Cmd/Ctrl+E), available when a project is open, SHALL open a dialog offering: the format (PNG or DDS), the output size (the active surface's size by default, or 1/2 or 1/4 of it), the background (an opaque color, white by default, or transparent) and, for DDS, the encoding (BC3/DXT5, default, or uncompressed RGBA). Confirming SHALL ask for a destination with a native save dialog proposing "<project name>.png" or ".dds", and then export. Cancelling SHALL close the dialog without writing anything. The dialog SHALL remember the last choices for the session. The export SHALL contain the active surface's artwork only, never its template. For a vehicle project, the proposed file name is "<project name> - <texture name>" with the format's extension.

#### Scenario: Default export settings
- **WHEN** the user opens Export Texture… on a 4096×4096 project for the first time
- **THEN** the dialog shows PNG, 4096 × 4096, a white background, and a preview of the livery

#### Scenario: Export to PNG
- **WHEN** the user confirms the dialog and chooses "ace.png"
- **THEN** a 4096×4096 PNG of the active surface is written to "ace.png"

#### Scenario: Exporting a smaller texture
- **WHEN** the active surface of a vehicle project is its 1024 px "Accessories" texture and the user opens Export Texture…
- **THEN** the dialog offers 1024 × 1024 by default and proposes "<project name> - Accessories.png"
