## MODIFIED Requirements

### Requirement: Preferences dialog
The application SHALL provide a Preferences dialog (Edit > Preferences, shortcut `⌘,` on macOS and `Ctrl+,` elsewhere) to change UI scale and text size with a live preview, the grid spacing (texture pixels, 4 to 1024) and the interface language, and a "Reset to defaults" action that restores UI scale, text size and grid spacing. The Language setting SHALL list "System default" followed by English, Français, Español and Deutsch, each named in its own language. "System default" follows the operating system's language, and choosing a language applies it immediately. "Reset to defaults" SHALL NOT change the language. On Windows and Linux, the dialog SHALL also offer **Use the system title bar** (see the window-title-bar capability), applied at once; "Reset to defaults" SHALL NOT change it.

Under **Template packs**, with a line saying what the packs are for, the dialog SHALL show, for each game that has official vehicles, the game's name above a read-only field holding the remembered pack folder (cut in the middle when too long, in full in its tooltip) or "Not set", followed by two icon buttons: **Choose Folder…**, which checks the chosen folder as the template pack dialog does and keeps the previous folder when it is refused, and **Clear**, which forgets it and is disabled when nothing is set. Clicking the field of an existing folder SHALL show it in the system's file manager; the field SHALL look like a link on hover, with a tooltip naming the file manager ("Show in Finder", "Show in Explorer", "Show in File Manager"), and its context menu SHALL offer **Copy Path**. A remembered folder that no longer exists SHALL be shown in the error color with "Folder not found" and SHALL NOT open. Changing or clearing a pack folder SHALL NOT change installed vehicles or projects. "Reset to defaults" SHALL NOT change the pack folders.

View > Reset Workspace SHALL restore the workspace layout to its defaults: the left panel and the inspector shown at their default widths, with the Textures tab active. It SHALL NOT change the active space, the document, the selection or the zoom.

#### Scenario: Reset to defaults
- **WHEN** the user clicks "Reset to defaults" in Preferences
- **THEN** UI scale and text size return to 100% and the grid spacing to 64 px immediately

#### Scenario: Reset workspace layout
- **WHEN** the panels are hidden, the inspector has been widened, the Resources tab is active, and the user chooses View > Reset Workspace
- **THEN** the left panel and the inspector are shown at their default widths and the left panel shows the Textures tab

#### Scenario: Back to the system language
- **WHEN** the language was set to "Español" on a French system and the user picks "System default"
- **THEN** the interface switches to French and no language choice is saved

#### Scenario: Change the pack folder
- **WHEN** the user clicks Choose Folder… under Template packs for ETS2 and chooses a folder holding the ETS2 pack for a newer game version
- **THEN** that folder is shown and remembered, and the vehicles already installed are unchanged

#### Scenario: Refused pack folder
- **WHEN** the user clicks Choose Folder… for ETS2 and chooses a folder holding no vehicle folder of the pack
- **THEN** a message says the folder doesn't hold the ETS2 template pack, and the previous folder is kept

#### Scenario: Showing the pack folder
- **WHEN** the ETS2 pack folder is set and exists, and the user clicks its field in Preferences
- **THEN** the folder is shown in the system's file manager

#### Scenario: Pack folder gone
- **WHEN** the remembered ETS2 pack folder was deleted and the user opens Preferences
- **THEN** its field shows the path in the error color with "Folder not found", and clicking it opens nothing
