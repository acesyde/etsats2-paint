## Purpose

Makes every document change reversible with predictable undo and redo, so users can experiment freely.

## ADDED Requirements

### Requirement: Undo and redo every document change
Every change to the document — creating, deleting, moving, resizing, rotating, pasting, duplicating and reordering objects — SHALL be undoable with Undo (Cmd/Ctrl+Z) and redoable with Redo (Cmd/Ctrl+Shift+Z or Cmd/Ctrl+Y). Undo and redo SHALL also restore the selection that existed at that point. View changes (zoom, pan) and selection-only changes SHALL NOT create history entries.

#### Scenario: Undo a creation
- **WHEN** the user draws a rectangle and presses Cmd/Ctrl+Z
- **THEN** the rectangle disappears; pressing Cmd/Ctrl+Shift+Z brings it back, selected

#### Scenario: Zoom is not undone
- **WHEN** the user moves an object, zooms in, and presses Cmd/Ctrl+Z
- **THEN** the object returns to its previous position and the zoom is unchanged

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
Any document change, undo or redo SHALL mark the project as having unsaved changes.

#### Scenario: Edit marks unsaved
- **WHEN** the user moves an object
- **THEN** the status bar shows "Unsaved changes"
