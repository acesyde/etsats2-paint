## MODIFIED Requirements

### Requirement: Fill and stroke target
The Colors panel SHALL edit either the fill or the stroke ("target"), shown as two overlapping swatches where the active one is in front and outlined. Each swatch SHALL preview its paint: a solid color, or the gradient itself. X SHALL switch the target. Shift+X SHALL swap the fill and stroke paints of the selection, gradients included. D SHALL reset the selection (or current style) to the default solid fill and no stroke. With a selection, edits apply to every selected object (recursively for groups); with no selection, edits apply to the current style used for new shapes. When the target paint is a gradient, the picker, color models, hex field, recent colors and palette SHALL edit the gradient's selected stop (see the gradients capability).

#### Scenario: Switch target with X
- **WHEN** the fill is the target and the user presses X
- **THEN** the stroke becomes the target and its swatch is drawn in front

#### Scenario: Editing with nothing selected
- **WHEN** nothing is selected, the user sets the fill to red and then draws a rectangle
- **THEN** the new rectangle is red

#### Scenario: Swap a gradient fill with a solid stroke
- **WHEN** a shape with a linear fill gradient and a solid black stroke is selected and the user presses Shift+X
- **THEN** its fill is solid black and its stroke has the linear gradient

#### Scenario: Gradient swatch
- **WHEN** a shape with a red-to-blue fill gradient is selected
- **THEN** the fill swatch shows the red-to-blue gradient

### Requirement: Recent colors
The panel SHALL list the last 12 distinct colors applied (most recent first), persisted across sessions; clicking one applies it to the current target, or to the selected stop when the target is a gradient. Colors applied to gradient stops SHALL be added to the list like other colors.

#### Scenario: Recent colors persist
- **WHEN** the user applies three colors, quits and restarts the application
- **THEN** the three colors are listed in Recent colors in reverse order of use

#### Scenario: Recent color on a stop
- **WHEN** a gradient's second stop is selected and the user clicks a recent color
- **THEN** only the second stop takes that color

### Requirement: Project palette
The panel SHALL show the project's palette: "Add to palette" saves the current target color (or the selected stop's color when the target is a gradient) as a swatch (no duplicates), clicking a swatch applies it to the current target or the selected stop, and a context menu removes it. Palette changes SHALL be undoable and belong to the project.

#### Scenario: Add and apply a palette color
- **WHEN** the user adds the current color to the palette, selects another object and clicks that swatch
- **THEN** the other object gets that color

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
