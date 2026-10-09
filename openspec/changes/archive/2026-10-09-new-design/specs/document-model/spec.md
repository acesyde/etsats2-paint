## MODIFIED Requirements

### Requirement: Path objects
A path object SHALL be made of one or more subpaths. Each subpath is an ordered list of anchor points and is either open or closed.

- **Points**: each anchor point has a position and optional incoming and outgoing Bézier handles. A point is smooth when its handles are aligned (opposite directions), otherwise a corner.
- **Segments**: between two consecutive points, a segment is straight when neither of the facing handles is set, otherwise a cubic curve. A closed subpath also has a segment from its last point back to its first.
- **Filling**: closed subpaths SHALL be filled with the fill color using the non-zero winding rule, so that a subpath drawn in the opposite direction inside another makes a hole, and the stroke SHALL follow their outline.
- **Lines**: open subpaths SHALL be drawn as lines of the path's line width (default 8 texture pixels) in the fill color, with rounded ends; the area between the points of an open subpath is never filled. The stroke, when set, SHALL outline each line (centered on its edges, like a shape's outline), so a white line with a black stroke shows a black border around a white line.
- **Geometry**: path geometry SHALL stay vector. Moving, resizing, rotating and flipping a path SHALL transform its points and handles exactly. Flipping SHALL mirror the path rather than turn it.
- **Frame**: a path's frame (bounds, center and rotation, used by the selection overlay and the inspector's Layout section) SHALL always be the bounds of its geometry in its own rotation, at least 1 texture pixel on each side.

#### Scenario: Flipping an arrow
- **WHEN** a path shaped like an arrow pointing right is flipped by dragging its right handle past its left side
- **THEN** the arrow points left

#### Scenario: Hole in a closed path
- **WHEN** a path has an outer square subpath and a smaller inner square subpath drawn in the opposite direction
- **THEN** the inner square area is not filled and objects below show through it

#### Scenario: Open path is a line
- **WHEN** an open path with three points forming a "V" has a red fill, a line width of 20 px and no stroke
- **THEN** a red "V" line 20 px wide is drawn and the area between its arms is empty

#### Scenario: Outlined line
- **WHEN** that "V" also has a black 4 px stroke
- **THEN** the red line has a black border along both edges and around its rounded ends
