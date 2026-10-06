## MODIFIED Requirements

### Requirement: Save state follows the history
Any document change, undo or redo SHALL mark the project as having unsaved changes, except when it returns the document to the state that was last saved (or opened), which SHALL show the project as saved. Saving SHALL mark the current state as the saved state. A new project that has never been saved SHALL always show unsaved changes.

#### Scenario: Edit marks unsaved
- **WHEN** the user moves an object
- **THEN** the status bar shows "Unsaved changes"

#### Scenario: Undo back to the saved state
- **WHEN** the user saves, moves an object, then presses Cmd/Ctrl+Z
- **THEN** the status bar shows "Saved"
