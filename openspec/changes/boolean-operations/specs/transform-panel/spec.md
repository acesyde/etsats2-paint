## ADDED Requirements

### Requirement: Combine buttons
Below the align and distribute rows, the Transform panel SHALL show a row of four buttons: Unite, Minus Front, Intersect and Exclude. They SHALL run the same commands as the Object › Combine submenu, show the command name and shortcut in their tooltip, and be disabled with their command's reason when it cannot run.

#### Scenario: Unite from the panel
- **WHEN** two overlapping rectangles are selected and the user clicks the Unite button in the Transform panel
- **THEN** they become one path, as with Object › Combine › Unite
