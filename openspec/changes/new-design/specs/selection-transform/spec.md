## MODIFIED Requirements

### Requirement: Object commands
The following commands SHALL act on the selection: Delete (Delete or Backspace) removes the selected objects; Duplicate (Cmd/Ctrl+D) creates copies offset by 20 texture pixels and selects them; Copy (Cmd/Ctrl+C) and Cut (Cmd/Ctrl+X) place the selected objects on an application clipboard; Paste (Cmd/Ctrl+V) inserts the clipboard objects at the same position on the active surface, offset by 20 texture pixels for each repeated paste, and selects them; Bring Forward (Alt+Cmd/Ctrl+]) and Send Backward (Alt+Cmd/Ctrl+[) move the selection one step in the stacking order. These commands SHALL be disabled when they cannot apply (e.g. empty selection, empty clipboard).

#### Scenario: Duplicate
- **WHEN** a rectangle at (100, 100) is selected and the user presses Cmd/Ctrl+D
- **THEN** a copy at (120, 120) is created above it and becomes the selection

#### Scenario: Paste with empty clipboard
- **WHEN** nothing has been copied yet
- **THEN** Paste is disabled in the Edit menu

#### Scenario: Bring Forward by key
- **WHEN** a rectangle lies below an ellipse, the rectangle is selected, and the user presses Alt+Cmd/Ctrl+]
- **THEN** the rectangle is above the ellipse and the active texture is unchanged
