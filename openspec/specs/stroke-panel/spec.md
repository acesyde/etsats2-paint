# stroke-panel Specification

## Purpose

Controls whether the selected objects have an outline and how thick it is.

## Requirements

### Requirement: Stroke enable and width
The Stroke panel SHALL show a toggle to enable or disable the stroke of the selection and a width field in texture pixels (0.5 to 500, scrubbable). Enabling a stroke on objects without one SHALL use the current stroke color and width. With nothing selected, the panel SHALL edit the current style. Differing widths SHALL show "Mixed". Each change SHALL be one undo step.

#### Scenario: Enable a stroke
- **WHEN** a rectangle without stroke is selected and the user enables the stroke
- **THEN** the rectangle gets a stroke with the current stroke color and width

#### Scenario: Set width
- **WHEN** a stroked rectangle is selected and the user types 12 in the width field
- **THEN** its stroke is 12 texture pixels wide

### Requirement: Stroke alignment control
The Stroke panel SHALL show a segmented control with Center, Inside and Outside, setting the alignment of every selected stroke. With nothing selected it SHALL set the alignment of the current style. Differing alignments SHALL show no segment selected ("Mixed"). Each change SHALL be one undo step.

#### Scenario: Outside outline in one click
- **WHEN** a stroked text is selected and the user clicks Outside
- **THEN** the outline is drawn outside the letters, and Undo puts it back to Center

### Requirement: Dash controls
For shapes and texts (lines use their own settings in the Properties panel), the Stroke panel SHALL show a Dashed toggle with Dash and Gap fields (texture pixels, dash 0 to 2000, gap 0.5 to 2000) and a preset menu (Dashed 20/10, Dotted 0/12 with round caps, Long dash 60/20). Turning Dashed off SHALL make the stroke solid again and keep the lengths for the next time. Changes apply to every selected stroke, or to the current style with nothing selected, as one undo step each.

#### Scenario: Dotted preset
- **WHEN** a stroked ellipse is selected and the user picks the Dotted preset
- **THEN** its outline becomes round dots with a 12 px gap

### Requirement: Cap and join controls
For shapes and texts (lines use their own settings in the Properties panel), the Stroke panel SHALL show segmented controls for the cap (Butt, Round, Square) and the join (Miter, Round, Bevel), and a Miter limit field (1 to 20) when the join is Miter. Changes apply to every selected stroke, or to the current style with nothing selected, as one undo step each; differing values SHALL show "Mixed".

#### Scenario: Round corners
- **WHEN** a stroked rectangle is selected and the user picks the Round join
- **THEN** the outline's corners are rounded
