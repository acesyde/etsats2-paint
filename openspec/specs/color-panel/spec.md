# color-panel Specification

## Purpose

Provides a complete color editor for fills and strokes — picker, color models, hex, recent colors, project palette and eyedropper — so livery colors can be chosen and reused precisely.

## Requirements

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

### Requirement: Hex entry
A hex field SHALL accept `#RRGGBB`, `RRGGBB`, `#RGB` and `#RRGGBBAA` (case-insensitive) and apply on Enter; invalid input SHALL be rejected with a visible error state and SHALL NOT change the color.

#### Scenario: Short hex
- **WHEN** the user types "#f80" and presses Enter
- **THEN** the color becomes #FF8800

#### Scenario: Invalid hex
- **WHEN** the user types "zz12" and presses Enter
- **THEN** the field shows an error outline and the color is unchanged

### Requirement: Stroke none
When the stroke is the target, a "None" swatch SHALL remove the stroke from the selection; choosing any color SHALL add a stroke (with the current stroke width, 4 px by default) if there was none.

#### Scenario: Removing a stroke
- **WHEN** a stroked rectangle is selected, the stroke is the target, and the user clicks None
- **THEN** the rectangle has no stroke

### Requirement: Recent colors
The color popover SHALL list, after the brand palette, the last 12 distinct colors applied (most recent first), persisted across sessions; clicking one applies it to the current target, or to the selected stop when the target is a gradient. Colors applied to gradient stops SHALL be added to the list like other colors.

#### Scenario: Recent colors persist
- **WHEN** the user applies three colors, quits and restarts the application
- **THEN** the three colors are listed in Recent colors in reverse order of use

#### Scenario: Recent color on a stop
- **WHEN** a gradient's second stop is selected and the user clicks a recent color
- **THEN** only the second stop takes that color

### Requirement: Project palette
The color popover SHALL show the project's palette (see the document-model capability) as the **brand palette**, before the recent colors, each swatch with its name and its usage (see the brand-impact capability) on hover. The Brand space SHALL show the same palette with the same swatch actions (see the brand-space capability).

- **Add to palette** saves the current target color, or the selected stop's color when the target is a gradient, as a new swatch. It adds no duplicates. The current target becomes linked to the new swatch.
- **Clicking a swatch** applies its color to the current target or to the selected stop, and links it to the swatch.
- **The linked swatch is marked:** when the current target, or the selected stop, is linked to a swatch, that swatch has a selection ring in the link color (see the ui-design-system capability), and the popover and the target's row show the swatch's name with a link icon. The mark doesn't rely on color alone.

Each swatch's context menu, in the popover and in the Brand space, SHALL offer:
- **Edit Swatch…**: opens the swatch's before/after editor (see the brand-impact capability), with its name, color picker and hex entry. Nothing changes before Apply to Fleet, which records the new color and name as one undo step; Cancel or Escape leaves the project untouched. From the color popover, the editor opens as a dialog over the Workshop.
- **Delete Swatch**: removes the swatch. Every color keeps its value and stops being linked.
- **Add to Library** or **Update in Library** (see the shared-library capability).

Palette changes SHALL be undoable and belong to the project.

#### Scenario: Add and apply a palette color
- **WHEN** the user adds the current color to the palette, selects another object and clicks that swatch in the color popover
- **THEN** the other object gets that color, linked to the swatch

#### Scenario: Edit the company red
- **WHEN** objects on two textures are linked to the swatch "Company red", and the user opens Edit Swatch… on it from the color popover, enters `#8B0000` and clicks Apply to Fleet
- **THEN** those objects are dark red on both textures, the swatch shows dark red, and one Undo restores the previous red everywhere

#### Scenario: Cancel an edit
- **WHEN** the user changes a swatch's color in Edit Swatch… and presses Escape
- **THEN** the swatch and every linked color are as before, the canvas never showed the new color, and nothing is added to the undo history

#### Scenario: Linked swatch marked
- **WHEN** the selected rectangle's fill is linked to "Company red" and the fill is the target
- **THEN** the Fill row reads "Company red" with a link icon in the link color, and in the color popover the "Company red" swatch has a selection ring in the link color

#### Scenario: Picking another color
- **WHEN** the selected rectangle's fill is linked to "Company red" and the user picks a recent color
- **THEN** no swatch is marked, the Fill row shows the hex code without the link color, and the fill is no longer linked

#### Scenario: Edit Swatch from the Brand space
- **WHEN** the user opens the context menu of "Company red" in the Brand space and chooses Edit Swatch…
- **THEN** the same before/after editor opens, as a panel on the right of the Brand space, and works as from the color popover

#### Scenario: Usage on hover
- **WHEN** "Company red" is used by 4 objects on 2 textures and the user hovers it in the color popover
- **THEN** the tooltip reads "Company red" and "2 textures · 4 objects"

### Requirement: Eyedropper tool
With the Eyedropper tool (I), clicking an object on the canvas SHALL take its fill paint (or its stroke paint when the stroke is the target), whether a solid color or a whole gradient. It SHALL apply that paint to the selection, or to the current style when nothing is selected. Clicking an image SHALL take the color of the image pixel under the pointer. Clicking empty canvas SHALL take the artboard background color. Hidden objects SHALL be ignored. The cursor SHALL show an eyedropper.

#### Scenario: Pick a fill
- **WHEN** rectangle A is selected, the Eyedropper is active, and the user clicks a red ellipse
- **THEN** rectangle A's fill becomes red and the selection is unchanged

#### Scenario: Pick a color from a logo
- **WHEN** a text is selected, the Eyedropper is active, and the user clicks the blue part of an imported PNG logo
- **THEN** the text's fill becomes that blue

#### Scenario: Pick a gradient
- **WHEN** rectangle A is selected, the Eyedropper is active, and the user clicks an ellipse with a radial fill gradient
- **THEN** rectangle A gets the same radial gradient, at the same place relative to its own frame

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
