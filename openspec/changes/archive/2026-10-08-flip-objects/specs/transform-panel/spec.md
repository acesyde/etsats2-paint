## ADDED Requirements

### Requirement: Flip buttons
The Transform panel SHALL show two buttons: Flip Horizontal and Flip Vertical. They SHALL sit at the end of the row of distribute buttons, separated from them, so that the panel keeps its default width. They SHALL run the same commands as Object › Flip Horizontal and Object › Flip Vertical. They SHALL show the command name and shortcut in their tooltip, and be disabled with their command's reason when it cannot run.

#### Scenario: Flip from the panel
- **WHEN** an image is selected and the user clicks the Flip Horizontal button in the Transform panel
- **THEN** the image is mirrored, as with Object › Flip Horizontal

#### Scenario: Disabled without a selection
- **WHEN** nothing is selected
- **THEN** the Transform panel shows its empty state and no Flip button can be clicked
