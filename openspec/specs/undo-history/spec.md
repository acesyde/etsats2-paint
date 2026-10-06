# undo-history Specification

## Purpose

Makes every document change reversible with predictable undo and redo, so users can experiment freely.

## Requirements

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

### Requirement: One gesture, one step
A continuous gesture (dragging to move, resize, rotate or draw) SHALL produce exactly one history entry when it ends. Consecutive nudges with arrow keys SHALL be merged into one entry as long as less than one second separates them. A gesture cancelled with Escape SHALL produce no entry.

#### Scenario: Drag is a single step
- **WHEN** the user drags an object across the canvas in one gesture and presses Cmd/Ctrl+Z once
- **THEN** the object returns to where the drag started

### Requirement: Redo is cleared by new changes
Making a new document change after undoing SHALL discard the steps that could have been redone.

#### Scenario: New change after undo
- **WHEN** the user undoes a move and then creates an ellipse
- **THEN** Redo is disabled

### Requirement: Bounded history
The history SHALL keep at least the last 200 steps per project; older steps are discarded. Undo and Redo SHALL be disabled when there is nothing to undo or redo, and their menu labels SHALL name the operation (for example "Undo Move", "Redo Delete").

#### Scenario: Descriptive menu label
- **WHEN** the last change was resizing an object
- **THEN** the Edit menu shows "Undo Resize"

### Requirement: Save state follows the history
Any document change, undo or redo SHALL mark the project as having unsaved changes, except when it returns the document to the state that was last saved (or opened), which SHALL show the project as saved. Saving SHALL mark the current state as the saved state. A new project that has never been saved SHALL always show unsaved changes.

#### Scenario: Edit marks unsaved
- **WHEN** the user moves an object
- **THEN** the status bar shows "Unsaved changes"

#### Scenario: Undo back to the saved state
- **WHEN** the user saves, moves an object, then presses Cmd/Ctrl+Z
- **THEN** the status bar shows "Saved"
