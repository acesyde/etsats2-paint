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
