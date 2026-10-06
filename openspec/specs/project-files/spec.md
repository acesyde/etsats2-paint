# project-files Specification

## Purpose

Lets users keep their liveries: save a project to a single `.truckpaint` file, reopen it later on any computer, and never lose work silently when closing.

## Requirements

### Requirement: Self-contained project file
A project SHALL be saved as one `.truckpaint` file containing everything needed to reopen it identically. The file SHALL contain:

- the project's name, texture resolution and surfaces;
- the full object tree, with every object's identity, name, kind, geometry (including polygon settings, the points and handles of every path subpath with its open or closed state, and the path's line width), fill, stroke, opacity, visibility and lock flags;
- each text's content and character style;
- image references;
- the guides of each surface;
- the project palette;
- the bytes of every imported asset.

Font families SHALL be stored by name only; font files SHALL NOT be embedded. Editor state that is not part of the document (selection, point selection, zoom, panel layout, grid and snapping settings, whether guides are shown, undo history) SHALL NOT be stored.

#### Scenario: Round trip
- **WHEN** the user saves a project with grouped shapes, an outlined text, a placed SVG logo, a star, a curved path with a hole and two guides, closes it and opens the file again
- **THEN** every object, group, text, image, path point and handle, guide, palette color and asset is restored with the same properties and stacking order

#### Scenario: Opening on another computer
- **WHEN** a project using a system font that is not installed is opened on another computer
- **THEN** it opens, the text is drawn with Inter and the font control shows the "Font not found" warning with the original family name

### Requirement: Versioned format
Every project file SHALL record the version of the file format it was written with. The application SHALL open files of every format version it has ever written, converting them to the current format when opening; such a file SHALL open marked as having unsaved changes and SHALL be written in the current format at the next save (the original file is only replaced when the user saves). Files written with a newer format version than the application supports SHALL be refused (see Invalid files).

#### Scenario: Opening an older project
- **WHEN** a project saved by an earlier version of TruckPaint, using an older file format, is opened
- **THEN** it opens with all its content, the status bar shows "Unsaved changes", and saving writes it in the current format

#### Scenario: Version recorded
- **WHEN** a project is saved
- **THEN** the file records the current format version

### Requirement: Save and Save As
File › Save (Cmd/Ctrl+S) SHALL write the project to its file, or behave as Save As when the project has never been saved. File › Save As… (Cmd/Ctrl+Shift+S) SHALL ask for a location with a native dialog, proposing "<project name>.truckpaint", and from then on Save SHALL write to that new file. Save SHALL be available while typing in a field. Saving SHALL NOT freeze the editor; while a save is in progress the status bar SHALL show "Saving…".

#### Scenario: First save
- **WHEN** the user presses Cmd/Ctrl+S on a new project named "ACE Logistics"
- **THEN** a save dialog proposes "ACE Logistics.truckpaint", and after confirming the status bar shows "Saved"

#### Scenario: Save after changes
- **WHEN** a saved project is modified and the user presses Cmd/Ctrl+S
- **THEN** the same file is updated without a dialog and the status bar shows "Saved"

### Requirement: Safe writes
Saving SHALL never leave a damaged project file: the new content SHALL be written completely before it replaces the existing file, so that a failed or interrupted save keeps the previous version intact. When saving fails, the user SHALL see a message naming the file and the reason, and the project SHALL remain marked as having unsaved changes.

#### Scenario: Save to a read-only location
- **WHEN** saving fails because the destination is not writable
- **THEN** a message explains that the file could not be saved and why, the previous file is unchanged and the status bar still shows "Unsaved changes"

### Requirement: Open a project
File › Open… (Cmd/Ctrl+O) SHALL show a native dialog filtered to `.truckpaint` files and open the chosen project in the editor, with nothing selected, the view fitted to the surface and an empty undo history, marked as saved. Opening the file of the project already open SHALL keep it as it is.

#### Scenario: Open from the menu
- **WHEN** the user chooses File › Open… and selects "ace.truckpaint"
- **THEN** the editor shows that project, the window title shows its name and the status bar shows "Saved"

### Requirement: Invalid files
Files that are not TruckPaint projects, are damaged, or were written by a newer version of TruckPaint SHALL NOT be opened; the user SHALL see a message naming the file and the reason (for example "ace.truckpaint was created with a newer version of TruckPaint"). The currently open project, if any, SHALL stay open and unchanged.

#### Scenario: Damaged file
- **WHEN** the user opens a truncated `.truckpaint` file
- **THEN** a message says the file is damaged and cannot be opened, and nothing else changes

#### Scenario: Newer format
- **WHEN** the user opens a file whose format version is newer than the application supports
- **THEN** a message says it was created with a newer version of TruckPaint

### Requirement: Unsaved changes prompt
When the project has unsaved changes, closing it, quitting the application, closing the window, creating a new project or opening another project SHALL first ask "Save changes to “<name>” before closing?" with the choices Save, Don't Save and Cancel. Save SHALL save (asking for a location if needed) and then continue only if saving succeeded; Don't Save SHALL continue without saving; Cancel (or Escape) SHALL return to the editor with nothing changed. Without unsaved changes, these actions SHALL proceed without asking.

#### Scenario: Quit with unsaved changes
- **WHEN** the user modifies a project and closes the window
- **THEN** the save prompt appears and the window stays open until a choice is made

#### Scenario: Cancel keeps working
- **WHEN** the prompt is shown and the user presses Escape
- **THEN** the prompt closes and the project stays open with its unsaved changes

#### Scenario: No prompt when saved
- **WHEN** the user closes a project right after saving it
- **THEN** the home screen is shown without a prompt

### Requirement: Recent projects follow files
Opening or saving a project SHALL add its file to the top of the recent projects list (without duplicates, keeping its name and the current date).

#### Scenario: Saved project appears in recent list
- **WHEN** the user saves a new project as "ace.truckpaint" and closes it
- **THEN** the home screen lists "ace.truckpaint" first among recent projects
