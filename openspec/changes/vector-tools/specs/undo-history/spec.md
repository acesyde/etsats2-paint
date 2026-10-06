## MODIFIED Requirements

### Requirement: Undo and redo every document change
Every change to the document — creating, deleting, moving, resizing, rotating, pasting, duplicating and reordering objects; changing fill, stroke, opacity, corner radius, polygon settings, line width or transform values from panels; editing path points and handles and converting shapes to paths; renaming, hiding, showing, locking, unlocking, grouping and ungrouping; adding or removing palette colors; text editing sessions and character style changes; and importing, placing, renaming or removing assets — SHALL be undoable with Undo (Cmd/Ctrl+Z) and redoable with Redo (Cmd/Ctrl+Shift+Z or Cmd/Ctrl+Y). Undo and redo SHALL also restore the selection (including the selected path points) that existed at that point. View changes (zoom, pan), selection-only changes (objects or path points), panel layout changes, the current style for new shapes and texts and recent colors SHALL NOT create history entries.

#### Scenario: Undo a creation
- **WHEN** the user draws a rectangle and presses Cmd/Ctrl+Z
- **THEN** the rectangle disappears; pressing Cmd/Ctrl+Shift+Z brings it back, selected

#### Scenario: Zoom is not undone
- **WHEN** the user moves an object, zooms in, and presses Cmd/Ctrl+Z
- **THEN** the object returns to its previous position and the zoom is unchanged

#### Scenario: Undo a color change
- **WHEN** the user changes a rectangle's fill from blue to red with the picker in one drag and presses Cmd/Ctrl+Z
- **THEN** the rectangle is blue again and the Edit menu offered "Undo Change Fill"

#### Scenario: Undo an import
- **WHEN** the user places a logo and presses Cmd/Ctrl+Z
- **THEN** the logo object is removed and the asset is no longer listed


#### Scenario: Undo a point edit
- **WHEN** the user moves a path point with the Direct Selection tool and presses Cmd/Ctrl+Z
- **THEN** the point returns to its previous position and the point selection is restored
