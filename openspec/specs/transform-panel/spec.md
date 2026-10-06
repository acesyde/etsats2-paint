# transform-panel Specification

## Purpose

Lets users read and type the exact position, size, rotation and scale of the selection, which precise livery placement depends on.

## Requirements

### Requirement: Transform fields
The Transform panel SHALL show, for the current selection, the fields X and Y (center of the selection bounds, in texture pixels), W and H (size of the selection bounds: the object's own frame for a single object, the axis-aligned bounds for several), rotation in degrees (single object or single group only) and scale in percent. With nothing selected, the panel SHALL show an empty state.

#### Scenario: Single object values
- **WHEN** a 400×200 rectangle centered at (300, 200) and rotated by 15° is selected
- **THEN** the panel shows X 300, Y 200, W 400, H 200, rotation 15° and scale 100%

#### Scenario: Nothing selected
- **WHEN** the selection is empty
- **THEN** the Transform panel shows an empty state instead of fields

### Requirement: Editing values
Typing a value and pressing Enter or Tab, or moving focus away, SHALL apply it; Escape SHALL restore the previous value without applying. Dragging horizontally on a field's label SHALL scrub the value live (Shift for steps ×10, Alt/Option for steps ×0.1). Changing X or Y moves the selection; changing W or H resizes it around its center; changing rotation rotates it around its center; entering a scale applies a uniform scale around the center and the field returns to 100%. Values outside valid ranges SHALL be clamped (size at least 1 px). Each committed edit or scrub gesture SHALL be one undo step.

#### Scenario: Typed width
- **WHEN** a 400×200 rectangle is selected and the user types 600 in W and presses Enter
- **THEN** the rectangle is 600×200, keeps its center, and Undo restores 400×200

#### Scenario: Escape reverts
- **WHEN** the user types 999 in X and presses Escape
- **THEN** X shows the previous value and the object has not moved

#### Scenario: Scale field
- **WHEN** a 400×200 rectangle is selected and the user enters 50 in scale
- **THEN** the rectangle becomes 200×100 and the scale field shows 100% again

### Requirement: Proportional lock
A lock toggle between W and H SHALL keep the aspect ratio when either value is changed. The toggle state SHALL be shown by icon shape, not only by color.

#### Scenario: Locked proportions
- **WHEN** the proportion lock is on and the user sets W of a 400×200 rectangle to 800
- **THEN** H becomes 400

### Requirement: Mixed values
When several objects are selected, X, Y, W and H SHALL refer to the selection bounds; rotation SHALL show "Mixed" when the objects' rotations differ and editing it SHALL set every selected object to the entered rotation around its own center.

#### Scenario: Mixed rotation
- **WHEN** two objects rotated by 0° and 30° are selected
- **THEN** the rotation field shows "Mixed"

### Requirement: Align and distribute buttons
Below its fields, the Transform panel SHALL show:
- a row of six align buttons (left, horizontal centers, right, top, vertical centers, bottom) with an "Align to" selector (Selection, Artboard, Key object);
- a row of four distribute buttons (horizontal centers, vertical centers, horizontal spacing, vertical spacing).

The buttons SHALL run the same commands as the Object › Align submenu, show the command name and shortcut in their tooltip, and be disabled when their command is.

#### Scenario: Align from the panel
- **WHEN** two objects are selected and the user clicks the Align Top button in the Transform panel
- **THEN** both objects' tops are aligned, as with Object › Align › Align Top
