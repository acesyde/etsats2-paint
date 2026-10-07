## ADDED Requirements

### Requirement: Line dashes, caps and joins
When every selected object is a path with open subpaths, the Properties panel SHALL show, next to Width:
- a Dashed toggle with Dash and Gap fields and the presets of the Stroke panel;
- a Cap control (Butt, Round, Square);
- a Join control (Miter, Round, Bevel), with a Miter limit field when the join is Miter.

They SHALL set the line's own dash pattern, cap and join, apply to every selected line, show "Mixed" for differing values, and be one undo step per change. New lines SHALL use the values last set, initially solid with round caps and miter joins.

#### Scenario: Dashed line from the Properties panel
- **WHEN** a line is selected and the user turns Dashed on with Dash 30 and Gap 15
- **THEN** the line is drawn as 30 px dashes separated by 15 px gaps, without needing a stroke

#### Scenario: Butt ends
- **WHEN** a line with round ends is set to the Butt cap
- **THEN** its ends stop exactly at its end points
