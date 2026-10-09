## ADDED Requirements

### Requirement: Stroke row
The Stroke panel SHALL be replaced by a **Stroke** row in the inspector's Appearance section, under the Fill row (see the properties-panel capability), and a **stroke popover**:

- **The row** SHALL show the stroke color swatch (a "None" pattern without a stroke, a "Mixed" pattern for differing paints) and a summary of the width and alignment, such as "6 px · Outside", or "None" without a stroke and "Mixed" for differing values.
- **The swatch** SHALL open the color popover with the stroke as target (see the color-panel capability).
- **The summary** SHALL open the stroke popover, anchored to the row, holding the stroke settings: the stroke toggle and width, the alignment, the dash controls, and the cap, join and miter limit controls.
- **Closing:** Escape, a click outside the popover, or opening another popover SHALL close it. Only one popover is open at a time.

With nothing selected, the Stroke row SHALL show and edit the current style, as the look of new objects.

#### Scenario: Stroke summary
- **WHEN** a rectangle with a 6 px Outside stroke is selected
- **THEN** the Stroke row reads "6 px · Outside" next to its color swatch

#### Scenario: Open the stroke options
- **WHEN** a stroked rectangle is selected and the user clicks the Stroke row's summary
- **THEN** the stroke popover opens with the width, alignment, dash, cap and join controls, and Escape closes it

## MODIFIED Requirements

### Requirement: Stroke enable and width
The stroke popover SHALL show a toggle to enable or disable the stroke of the selection and a width field in texture pixels (0.5 to 500, scrubbable). Enabling a stroke on objects without one SHALL use the current stroke color and width. With nothing selected, the popover SHALL edit the current style. Differing widths SHALL show "Mixed". Each change SHALL be one undo step, and the Stroke row's summary SHALL follow it.

#### Scenario: Enable a stroke
- **WHEN** a rectangle without stroke is selected and the user enables the stroke in the stroke popover
- **THEN** the rectangle gets a stroke with the current stroke color and width

#### Scenario: Set width
- **WHEN** a stroked rectangle is selected and the user types 12 in the width field
- **THEN** its stroke is 12 texture pixels wide and the Stroke row's summary shows "12 px"

### Requirement: Stroke alignment control
The stroke popover SHALL show a segmented control with Center, Inside and Outside, setting the alignment of every selected stroke. With nothing selected it SHALL set the alignment of the current style. Differing alignments SHALL show no segment selected ("Mixed"). Each change SHALL be one undo step.

#### Scenario: Outside outline in one click
- **WHEN** a stroked text is selected and the user clicks Outside in the stroke popover
- **THEN** the outline is drawn outside the letters, the Stroke row's summary shows "Outside", and Undo puts it back to Center

### Requirement: Dash controls
For shapes and texts (lines use their own line settings in the inspector, see the properties-panel capability), the stroke popover SHALL show a Dashed toggle with Dash and Gap fields (texture pixels, dash 0 to 2000, gap 0.5 to 2000) and a preset menu (Dashed 20/10, Dotted 0/12 with round caps, Long dash 60/20). Turning Dashed off SHALL make the stroke solid again and keep the lengths for the next time. Changes apply to every selected stroke, or to the current style with nothing selected, as one undo step each.

#### Scenario: Dotted preset
- **WHEN** a stroked ellipse is selected and the user picks the Dotted preset in the stroke popover
- **THEN** its outline becomes round dots with a 12 px gap

### Requirement: Cap and join controls
For shapes and texts (lines use their own line settings in the inspector, see the properties-panel capability), the stroke popover SHALL show segmented controls for the cap (Butt, Round, Square) and the join (Miter, Round, Bevel), and a Miter limit field (1 to 20) when the join is Miter. Changes apply to every selected stroke, or to the current style with nothing selected, as one undo step each; differing values SHALL show "Mixed".

#### Scenario: Round corners
- **WHEN** a stroked rectangle is selected and the user picks the Round join in the stroke popover
- **THEN** the outline's corners are rounded
