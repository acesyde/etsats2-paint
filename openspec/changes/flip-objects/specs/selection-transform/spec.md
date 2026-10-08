## MODIFIED Requirements

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

## ADDED Requirements

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
