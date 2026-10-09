## MODIFIED Requirements

### Requirement: Transform fields
The Transform panel SHALL become the **Layout** section of the inspector (see the properties-panel capability), shown only when something is selected. It SHALL show, for the current selection, the fields X and Y (center of the selection bounds, in texture pixels), W and H (size of the selection bounds: the object's own frame for a single object, the axis-aligned bounds for several), rotation in degrees (single object or single group only) and scale in percent. With nothing selected, the inspector SHALL show no Layout section and no empty state in its place.

#### Scenario: Single object values
- **WHEN** a 400×200 rectangle centered at (300, 200) and rotated by 15° is selected
- **THEN** the Layout section shows X 300, Y 200, W 400, H 200, rotation 15° and scale 100%

#### Scenario: Nothing selected
- **WHEN** the selection is empty
- **THEN** the inspector shows no Layout section and no "Nothing to transform" empty state

### Requirement: Align and distribute buttons
Below its fields, the Layout section SHALL show:
- a row of six align buttons (left, horizontal centers, right, top, vertical centers, bottom) with an "Align to" selector (Selection, Artboard, Key object);
- a row of four distribute buttons (horizontal centers, vertical centers, horizontal spacing, vertical spacing).

The buttons SHALL run the same commands as the Object › Align submenu, show the command name and shortcut in their tooltip, and be disabled when their command is.

#### Scenario: Align from the panel
- **WHEN** two objects are selected and the user clicks the Align Top button in the Layout section
- **THEN** both objects' tops are aligned, as with Object › Align › Align Top

### Requirement: Combine buttons
Below the align and distribute rows, the Layout section SHALL show a row of four buttons: Unite, Minus Front, Intersect and Exclude. They SHALL run the same commands as the Object › Combine submenu, show the command name and shortcut in their tooltip, and be disabled with their command's reason when it cannot run.

#### Scenario: Unite from the panel
- **WHEN** two overlapping rectangles are selected and the user clicks the Unite button in the Layout section
- **THEN** they become one path, as with Object › Combine › Unite

### Requirement: Flip buttons
The Layout section SHALL show two buttons: Flip Horizontal and Flip Vertical, grouped apart from the align, distribute and combine buttons, so that the inspector keeps its default width. They SHALL run the same commands as Object › Flip Horizontal and Object › Flip Vertical. They SHALL show the command name and shortcut in their tooltip, and be disabled with their command's reason when it cannot run.

#### Scenario: Flip from the panel
- **WHEN** an image is selected and the user clicks the Flip Horizontal button in the Layout section
- **THEN** the image is mirrored, as with Object › Flip Horizontal

#### Scenario: Disabled without a selection
- **WHEN** nothing is selected
- **THEN** the inspector shows no Layout section, so no Flip button is shown, and Object › Flip Horizontal is disabled with its reason
