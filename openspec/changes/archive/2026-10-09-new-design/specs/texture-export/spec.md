## MODIFIED Requirements

### Requirement: Export Texture dialog
Export › Export Texture… (Shift+Cmd/Ctrl+E), available when a project is open, SHALL open a dialog offering:
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
