## MODIFIED Requirements

### Requirement: Eyedropper tool
With the Eyedropper tool (I), clicking an object on the canvas SHALL take its fill color (or its stroke color when the stroke is the target) and apply it to the selection, or to the current style when nothing is selected. Clicking an image SHALL take the color of the image pixel under the pointer. Clicking empty canvas SHALL take the artboard background color. Hidden objects SHALL be ignored. The cursor SHALL show an eyedropper.

#### Scenario: Pick a fill
- **WHEN** rectangle A is selected, the Eyedropper is active, and the user clicks a red ellipse
- **THEN** rectangle A's fill becomes red and the selection is unchanged

#### Scenario: Pick a color from a logo
- **WHEN** a text is selected, the Eyedropper is active, and the user clicks the blue part of an imported PNG logo
- **THEN** the text's fill becomes that blue
