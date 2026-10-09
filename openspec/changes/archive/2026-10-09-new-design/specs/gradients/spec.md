## MODIFIED Requirements

### Requirement: Choosing a paint kind
The color popover (see the color-panel capability) SHALL show, at its top, a Solid / Linear / Radial segmented control for the current target (fill or stroke). It SHALL show "Mixed" (no segment selected) when the selection's paints differ in kind. Choosing a kind SHALL apply it to every selected object, as one undo step:

- **Solid → gradient:** the new gradient goes from the current color to the same color at 0% alpha. A linear gradient spans the object from the middle of its left edge to the middle of its right edge. A radial gradient is centered with a radius reaching the right edge, so its ellipse fits the object's frame.
- **Gradient → gradient:** switching between linear and radial keeps the stops.
- **Gradient → solid:** the paint becomes the color of the first stop.

With nothing selected, the choice applies to the current style, and new shapes get that paint.

#### Scenario: Make a fill linear
- **WHEN** a red rectangle is selected, the user opens the color popover from the Fill row and chooses Linear
- **THEN** the rectangle fades from opaque red at its left edge to transparent at its right edge, the Fill row reads "Linear", and one Undo makes it solid red again

#### Scenario: New shapes use the current gradient
- **WHEN** nothing is selected, the user chooses Radial for the fill of new objects and draws an ellipse
- **THEN** the new ellipse has a radial fill gradient

### Requirement: Gradient bar and stops
In gradient mode, the color popover SHALL show, under the Solid / Linear / Radial control and above the picker, a gradient bar previewing the gradient over a checkerboard, with one marker per stop. Exactly one stop is the selected stop. The popover SHALL support these operations:

- **Select a stop:** clicking a marker selects it. The color picker, sliders, hex field, recent colors and palette then edit and apply to the selected stop.
- **Move a stop:** dragging a marker moves its stop along the bar. A Location field (0–100%) sets its location exactly. Stops may pass each other.
- **Add a stop:** clicking the bar away from markers adds a stop at that location, with the color the gradient has there, and selects it. Nothing is added when the gradient already has sixteen stops.
- **Delete a stop:** Delete or Backspace on a focused marker, or dragging a marker well away from the bar, removes its stop. A gradient SHALL keep at least two stops.
- **Reverse:** a Reverse button mirrors every stop's location.
- **Angle (both kinds):** an Angle field shows the direction from start to end (linear) or center to radius point (radial), in degrees on the document. Setting it rotates the gradient around its midpoint (linear) or center (radial), keeping its length.
- **Aspect (radial only):** an Aspect field sets the aspect ratio.

Each drag SHALL be one undo step, and each field change SHALL be one undo step. Markers SHALL be keyboard-focusable, and Left and Right arrows move the focused stop by 1%. Delete, Backspace and the arrows on a focused marker SHALL act on the stop only, not on the selected objects. When the selected objects have different gradients, the bar SHALL show the first selected object's gradient, and an edit SHALL apply that whole gradient to every selected object.

#### Scenario: Add a middle stop
- **WHEN** a red-to-blue gradient is edited in the color popover and the user clicks the middle of the bar
- **THEN** a third stop is added at 50% with color (128, 0, 128) and is selected

#### Scenario: Recolor a stop with the hex field
- **WHEN** the second stop is selected and the user types "#00ff00" and presses Enter
- **THEN** only that stop becomes green

#### Scenario: Two stops minimum
- **WHEN** a gradient has two stops and the user presses Delete on one of them
- **THEN** both stops remain, and the selected object is not deleted

#### Scenario: Hard-edged band
- **WHEN** the user sets two adjacent stops, white and red, to the same location 50%
- **THEN** the gradient switches from white to red sharply at the middle
