## ADDED Requirements

### Requirement: Color popover
The Colors panel SHALL be replaced by the **Fill** and **Stroke** rows of the inspector (see the properties-panel capability) and a **color popover**. There SHALL be no color picker that stays open on its own.

- **Fill row:** a swatch of the fill paint, followed by its value: the hex code of a solid color, the linked swatch's name when the fill is linked to the palette, "Linear" or "Radial" for a gradient, or "Mixed".
- **Opening:** clicking a row's swatch SHALL make that row the target and open the color popover anchored to the row: beside the inspector, over the canvas, level with the row; under or above the row when there is no room beside the inspector.
- **Closing:** Escape, a click outside the popover, or opening another popover SHALL close it. Only one popover is open at a time. Closing SHALL NOT undo the edits made in it.
- **Content, in this order:** the Solid / Linear / Radial control and, for a gradient, the gradient bar (see the gradients capability); the picker with its color models and the hex field; the **brand palette** (the project's palette); then the **recent colors**. With the stroke as target, the popover SHALL also offer the "None" swatch.
- **Live edits:** edits in the popover SHALL apply to the selection live and be recorded as they are today: one undo step per committed edit or drag.

#### Scenario: Open and close the picker
- **WHEN** a rectangle is selected and the user clicks the Fill row's swatch, then presses Escape
- **THEN** the color popover opens beside the inspector, level with the Fill row, and Escape closes it, keeping the color chosen in it

#### Scenario: Palette before recents
- **WHEN** the color popover is open on a project with a palette and recent colors
- **THEN** the brand palette is listed above the recent colors

#### Scenario: Click outside
- **WHEN** the color popover is open and the user clicks on the canvas pasteboard
- **THEN** the popover closes

### Requirement: Instances have no color rows
When only symbol instances are selected, the inspector SHALL show neither the Fill nor the Stroke row, and SHALL say that the look is edited in the symbol (see the symbols capability). In a mixed selection, the rows SHALL edit the other objects only.

#### Scenario: Instance selected
- **WHEN** only an instance is selected
- **THEN** the inspector has no Fill or Stroke row and says that the look is edited in the symbol

## MODIFIED Requirements

### Requirement: Fill and stroke target
The color editor SHALL edit either the fill or the stroke ("target"). The target SHALL be shown in the inspector by marking the target's row (Fill or Stroke) with an outline, and the color popover edits the target. Each row's swatch SHALL preview its paint: a solid color, or the gradient itself. X SHALL switch the target; when the color popover is open, it SHALL then edit the new target, anchored to its row. Shift+X SHALL swap the fill and stroke paints of the selection, gradients included. D SHALL reset the selection (or current style) to the default solid fill and no stroke. With a selection, edits apply to every selected object (recursively for groups); with no selection, edits apply to the current style used for new shapes, shown in the inspector as the look of new objects. When the target paint is a gradient, the picker, color models, hex field, recent colors and palette SHALL edit the gradient's selected stop (see the gradients capability).

#### Scenario: Switch target with X
- **WHEN** the fill is the target and the user presses X
- **THEN** the stroke becomes the target and the Stroke row is marked as the target

#### Scenario: Switch target with the popover open
- **WHEN** the color popover is open on the Fill row and the user presses X
- **THEN** the popover edits the stroke, anchored to the Stroke row

#### Scenario: Editing with nothing selected
- **WHEN** nothing is selected, the user sets the fill of new objects to red and then draws a rectangle
- **THEN** the new rectangle is red

#### Scenario: Swap a gradient fill with a solid stroke
- **WHEN** a shape with a linear fill gradient and a solid black stroke is selected and the user presses Shift+X
- **THEN** its fill is solid black and its stroke has the linear gradient

#### Scenario: Gradient swatch
- **WHEN** a shape with a red-to-blue fill gradient is selected
- **THEN** the Fill row's swatch shows the red-to-blue gradient

### Requirement: Picker and color models
The color popover SHALL include a saturation/value square with a hue slider and an alpha slider, and SHALL show the color as RGB, HSV or HSL sliders with numeric fields (0–255 for RGB, degrees and percent for HSV/HSL), selectable with a segmented control. All representations SHALL stay in sync. Dragging in the square or a slider SHALL update the selection live and SHALL be one undo step per drag.

#### Scenario: RGB entry updates the picker
- **WHEN** the user types 255, 0, 0 in the RGB fields
- **THEN** the hue slider shows 0°, the square shows full saturation and value, and the object turns red

### Requirement: Recent colors
The color popover SHALL list, after the brand palette, the last 12 distinct colors applied (most recent first), persisted across sessions; clicking one applies it to the current target, or to the selected stop when the target is a gradient. Colors applied to gradient stops SHALL be added to the list like other colors.

#### Scenario: Recent colors persist
- **WHEN** the user applies three colors, quits and restarts the application
- **THEN** the three colors are listed in Recent colors in reverse order of use

#### Scenario: Recent color on a stop
- **WHEN** a gradient's second stop is selected and the user clicks a recent color
- **THEN** only the second stop takes that color

### Requirement: Project palette
The color popover SHALL show the project's palette (see the document-model capability) as the **brand palette**, before the recent colors, each swatch with its name on hover. The Brand space SHALL show the same palette with the same swatch actions (see the brand-space capability).

- **Add to palette** saves the current target color, or the selected stop's color when the target is a gradient, as a new swatch. It adds no duplicates. The current target becomes linked to the new swatch.
- **Clicking a swatch** applies its color to the current target or to the selected stop, and links it to the swatch.
- **The linked swatch is marked:** when the current target, or the selected stop, is linked to a swatch, that swatch has a selection ring in the link color (see the ui-design-system capability), and the popover and the target's row show the swatch's name with a link icon. The mark doesn't rely on color alone.

Each swatch's context menu, in the popover and in the Brand space, SHALL offer:
- **Edit Swatch…**: a popup with the swatch's name and a color picker with hex entry. Changing the color updates every color linked to the swatch live, on every texture. OK records the change as one undo step, and Cancel or Escape restores the swatch and the linked colors. A name can't be empty.
- **Delete Swatch**: removes the swatch. Every color keeps its value and stops being linked.
- **Add to Library** or **Update in Library** (see the shared-library capability).

Palette changes SHALL be undoable and belong to the project.

#### Scenario: Add and apply a palette color
- **WHEN** the user adds the current color to the palette, selects another object and clicks that swatch in the color popover
- **THEN** the other object gets that color, linked to the swatch

#### Scenario: Edit the company red
- **WHEN** objects on two textures are linked to the swatch "Company red", and the user opens Edit Swatch… on it, enters `#8B0000` and clicks OK
- **THEN** those objects are dark red on both textures, the swatch shows dark red, and one Undo restores the previous red everywhere

#### Scenario: Cancel an edit
- **WHEN** the user changes a swatch's color in Edit Swatch… and presses Escape
- **THEN** the swatch and every linked color are as before, and nothing is added to the undo history

#### Scenario: Linked swatch marked
- **WHEN** the selected rectangle's fill is linked to "Company red" and the fill is the target
- **THEN** the Fill row reads "Company red" with a link icon in the link color, and in the color popover the "Company red" swatch has a selection ring in the link color

#### Scenario: Picking another color
- **WHEN** the selected rectangle's fill is linked to "Company red" and the user picks a recent color
- **THEN** no swatch is marked, the Fill row shows the hex code without the link color, and the fill is no longer linked

#### Scenario: Edit Swatch from the Brand space
- **WHEN** the user opens the context menu of "Company red" in the Brand space and chooses Edit Swatch…
- **THEN** the same Edit Swatch… popup opens and works as from the color popover
