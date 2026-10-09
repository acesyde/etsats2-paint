# selection-transform Specification

## Purpose

Defines how objects are selected and transformed on the canvas — the selection overlay, moving, resizing and rotating one or many objects — and the object commands that act on the selection.

## Requirements

### Requirement: Selecting objects
With the Selection or Move tool, clicking an object SHALL select it alone; when the clicked object is inside a group, the click SHALL select its top-level group, and Cmd/Ctrl+click SHALL select the innermost object instead. Shift+click SHALL add an unselected object to the selection or remove a selected one; clicking empty canvas SHALL clear the selection; dragging from empty canvas SHALL draw a marquee and select every top-level object that intersects it (Shift+marquee adds to the selection). Hidden and locked objects SHALL NOT be selectable on the canvas. Select All (Cmd/Ctrl+A) SHALL select every visible, unlocked top-level object of the active surface and Escape SHALL clear the selection. Hovering an unselected object SHALL show a thin outline of the object (or group) that a click would select.

#### Scenario: Shift+click toggles
- **WHEN** object A is selected and the user Shift+clicks object B, then Shift+clicks A
- **THEN** only B is selected

#### Scenario: Marquee selection
- **WHEN** the user drags a marquee from empty canvas that touches two of three objects
- **THEN** exactly those two objects are selected

#### Scenario: Click selects the group
- **WHEN** a rectangle inside the group "Graphics" is clicked
- **THEN** the group "Graphics" is selected, and Cmd/Ctrl+click on the same point selects the rectangle itself

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
Dragging a resize handle SHALL resize the selection from the opposite handle; Shift SHALL keep the proportions (corner handles) and Alt/Option SHALL resize from the center. Dragging a handle past the opposite side SHALL flip the selection rather than produce a negative size; the flipped objects SHALL be mirrored as with the Flip commands, images and texts included. For multiple objects, positions and sizes SHALL scale relative to the selection bounds.

#### Scenario: Proportional resize
- **WHEN** a 200×100 rectangle is resized from its bottom-right handle with Shift held until its width is 400
- **THEN** its height is 200

#### Scenario: Resize from the center
- **WHEN** a rectangle centered at (500, 500) is resized from a corner with Alt/Option held
- **THEN** its center stays at (500, 500)

#### Scenario: Dragging past the opposite side mirrors an image
- **WHEN** an image of a horse facing right is selected and its right handle is dragged past its left side
- **THEN** the image is drawn mirrored, the horse facing left

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

### Requirement: Flipping
The commands Flip Horizontal (Shift+H) and Flip Vertical (Shift+V) SHALL mirror the selection in one step:

- **Axis:** Flip Horizontal SHALL mirror across the vertical line through the center of the selection bounds, and Flip Vertical across the horizontal line through it. The lines follow the texture's axes, whatever the selection's rotation.
- **Size and position:** the selection SHALL keep its bounds. One object stays in place with its size. With several objects, the selection is mirrored as one block: the objects swap sides and each is mirrored.
- **Rotation:** a rotated object's rotation SHALL be reversed by Flip Horizontal (30° becomes −30°). Flip Vertical SHALL do the same and add a half-turn when the object is not symmetric about its horizontal axis.
- **Content:** shapes, paths, polygons, gradients, groups and their children, texts, images and symbol instances SHALL all be mirrored. Flipping twice on the same axis SHALL restore the selection.
- **Links:** flipping SHALL NOT break a link to a swatch or a shared style.

Each flip SHALL be one undo step, named after its command. The commands SHALL be in the Object menu and in the canvas context menu. Like the other object commands, they SHALL be disabled with a reason when no editable object is selected or while a text is being edited.

#### Scenario: Flip a logo horizontally
- **WHEN** an image of a horse facing right, centered at (500, 300) and 200×100, is selected and the user presses Shift+H
- **THEN** the horse faces left, the image is still centered at (500, 300) and 200×100, and Undo restores it facing right

#### Scenario: Flipping twice restores
- **WHEN** a path shaped like an arrow pointing right is flipped horizontally twice
- **THEN** it points right again, at the same place and size

#### Scenario: Two objects swap sides
- **WHEN** a circle at the left end and a star at the right end of the selection are selected together and the user chooses Object › Flip Horizontal
- **THEN** the star is at the left end, the circle at the right end, and the selection bounds are unchanged

#### Scenario: Rotated rectangle
- **WHEN** a rectangle centered at (400, 400) and rotated by 30° is flipped horizontally
- **THEN** it is centered at (400, 400) and rotated by −30°

#### Scenario: Flip a text vertically
- **WHEN** the text "TRANS" is selected and the user presses Shift+V
- **THEN** the text is drawn upside down and mirrored, as in a reflection on water, with the same bounds

#### Scenario: Flip an instance
- **WHEN** an instance of a symbol holding a logo image is flipped horizontally
- **THEN** the instance shows the logo mirrored, its symbol is unchanged, and the other instances are not mirrored

#### Scenario: Nothing selected
- **WHEN** the selection is empty
- **THEN** Flip Horizontal and Flip Vertical are disabled in the Object menu and their tooltip says to select an object

### Requirement: Paste into another project
Objects copied or cut in one project and pasted into another SHALL keep their look: their images, the swatches and styles they are linked to, and the symbols of their instances SHALL be imported into the target project with the library's import rules (see shared-library), as part of the Paste undo step. Pasting into the project the objects were copied from SHALL behave as before. The clipboard SHALL keep the objects' dependencies as they were when copied, so they can be pasted after the source project is closed.

#### Scenario: Pasting a logo into another project
- **WHEN** the user copies an image and a text linked to the swatch "Vert Ardent" in project A, opens project B and pastes
- **THEN** project B shows the same image and text, has a swatch "Vert Ardent" with the same color, and the text is linked to it

#### Scenario: Pasting an instance into another project
- **WHEN** the user copies an instance of the symbol "Logo Ardent" in project A, closes A, opens project B and pastes
- **THEN** project B has the symbol "Logo Ardent" and the pasted object is an instance of it

#### Scenario: Undo a paste into another project
- **WHEN** the user presses Undo right after such a paste
- **THEN** the pasted objects and the elements imported with them are removed
