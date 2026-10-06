# selection-transform Specification

## Purpose

Defines how objects are selected and transformed on the canvas — the selection overlay, moving, resizing and rotating one or many objects — and the object commands that act on the selection.

## Requirements

### Requirement: Selecting objects
With the Selection or Move tool, clicking an object SHALL select it alone; Shift+click SHALL add an unselected object to the selection or remove a selected one; clicking empty canvas SHALL clear the selection; dragging from empty canvas SHALL draw a marquee and select every object that intersects it (Shift+marquee adds to the selection). Select All (Cmd/Ctrl+A) SHALL select every object of the active surface and Escape SHALL clear the selection. Hovering an unselected object SHALL show a thin outline of its shape.

#### Scenario: Shift+click toggles
- **WHEN** object A is selected and the user Shift+clicks object B, then Shift+clicks A
- **THEN** only B is selected

#### Scenario: Marquee selection
- **WHEN** the user drags a marquee from empty canvas that touches two of three objects
- **THEN** exactly those two objects are selected

### Requirement: Selection overlay
When objects are selected, the canvas SHALL draw, in screen space at a constant size regardless of zoom: the selection bounds, eight resize handles (corners and edge midpoints), and a center mark. For a single object the bounds SHALL follow its rotation; for multiple objects the bounds SHALL be the axis-aligned box enclosing them. Each selected object SHALL also show its own outline. The overlay SHALL use a high-contrast color with a contrasting halo so it stays visible on any artwork.

#### Scenario: Rotated single selection
- **WHEN** a rectangle rotated by 30° is selected
- **THEN** its bounds and handles are drawn rotated by 30°

#### Scenario: Overlay size is constant
- **WHEN** the user zooms from 25% to 400% with an object selected
- **THEN** the handles keep the same size on screen

### Requirement: Moving
Dragging a selected object (or an unselected object, which selects it first) SHALL move the whole selection with the pointer. Holding Shift while dragging SHALL constrain the movement to horizontal, vertical or 45°. Arrow keys SHALL nudge the selection by 1 texture pixel, and by 10 texture pixels with Shift.

#### Scenario: Drag moves the selection
- **WHEN** two objects are selected and the user drags one of them by (100, 0) texture pixels
- **THEN** both objects move by (100, 0)

#### Scenario: Nudge with Shift
- **WHEN** an object is selected and the user presses Shift+Right Arrow
- **THEN** it moves 10 texture pixels to the right

### Requirement: Resizing
Dragging a resize handle SHALL resize the selection from the opposite handle; Shift SHALL keep the proportions (corner handles) and Alt/Option SHALL resize from the center. Dragging a handle past the opposite side SHALL flip the selection rather than produce a negative size. For multiple objects, positions and sizes SHALL scale relative to the selection bounds.

#### Scenario: Proportional resize
- **WHEN** a 200×100 rectangle is resized from its bottom-right handle with Shift held until its width is 400
- **THEN** its height is 200

#### Scenario: Resize from the center
- **WHEN** a rectangle centered at (500, 500) is resized from a corner with Alt/Option held
- **THEN** its center stays at (500, 500)

### Requirement: Rotating
Hovering just outside a corner handle SHALL show a rotation cursor; dragging there SHALL rotate the selection around the center of its bounds. Holding Shift SHALL snap the rotation to multiples of 15°. For multiple objects, each object's position and rotation SHALL rotate around the common center.

#### Scenario: Snapped rotation
- **WHEN** the user rotates a selected object while holding Shift
- **THEN** its rotation is a multiple of 15°

### Requirement: Transform feedback
While moving, resizing or rotating, a small label near the pointer SHALL show the current value (offset, width × height, or angle) and the cursor SHALL reflect the operation (move, resize along the handle direction, rotate).

#### Scenario: Size label while resizing
- **WHEN** the user drags a resize handle
- **THEN** a label shows the current width and height in texture pixels

### Requirement: Object commands
The following commands SHALL act on the selection: Delete (Delete or Backspace) removes the selected objects; Duplicate (Cmd/Ctrl+D) creates copies offset by 20 texture pixels and selects them; Copy (Cmd/Ctrl+C) and Cut (Cmd/Ctrl+X) place the selected objects on an application clipboard; Paste (Cmd/Ctrl+V) inserts the clipboard objects at the same position on the active surface, offset by 20 texture pixels for each repeated paste, and selects them; Bring Forward (Cmd/Ctrl+]) and Send Backward (Cmd/Ctrl+[) move the selection one step in the stacking order. These commands SHALL be disabled when they cannot apply (e.g. empty selection, empty clipboard).

#### Scenario: Duplicate
- **WHEN** a rectangle at (100, 100) is selected and the user presses Cmd/Ctrl+D
- **THEN** a copy at (120, 120) is created above it and becomes the selection

#### Scenario: Paste with empty clipboard
- **WHEN** nothing has been copied yet
- **THEN** Paste is disabled in the Edit menu
