## ADDED Requirements

### Requirement: Align and distribute buttons
Below its fields, the Transform panel SHALL show:
- a row of six align buttons (left, horizontal centers, right, top, vertical centers, bottom) with an "Align to" selector (Selection, Artboard, Key object);
- a row of four distribute buttons (horizontal centers, vertical centers, horizontal spacing, vertical spacing).

The buttons SHALL run the same commands as the Object › Align submenu, show the command name and shortcut in their tooltip, and be disabled when their command is.

#### Scenario: Align from the panel
- **WHEN** two objects are selected and the user clicks the Align Top button in the Transform panel
- **THEN** both objects' tops are aligned, as with Object › Align › Align Top
