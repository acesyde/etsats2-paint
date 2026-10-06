# path-editing Specification

## Purpose

Defines how the anchor points and handles of paths are selected and edited with the Direct Selection tool, and how shapes are converted to editable paths.

## Requirements

### Requirement: Direct Selection shows points
While the Direct Selection tool is active, the canvas SHALL show the anchor points of every selected path: selected points filled, unselected points hollow, and the handles of selected points (and of the segments next to them) as lines ending in small dots. Clicking an object with Direct Selection SHALL select that object itself, even inside a group (like Cmd/Ctrl+click with the Selection tool). Clicking an object that is not a path SHALL select it and show a hint that Convert to Path makes its points editable. Hidden and locked objects SHALL NOT be selectable or editable.

#### Scenario: Points of a path inside a group
- **WHEN** a path inside a group is clicked with the Direct Selection tool
- **THEN** the path itself is selected and its anchor points are shown

#### Scenario: Rectangle hint
- **WHEN** the user clicks a rectangle with the Direct Selection tool
- **THEN** the rectangle is selected and a hint mentions Convert to Path

### Requirement: Point selection
With the Direct Selection tool, clicking an anchor point of a selected path SHALL select that point alone; Shift+click SHALL add or remove a point. Clicking a path's segment or fill SHALL select the path without selecting points. Dragging from empty canvas SHALL draw a marquee selecting every anchor point inside it among the visible, unlocked paths (selecting those paths), and Shift+marquee SHALL add to the point selection. Clicking empty canvas or pressing Escape SHALL clear the point selection; a second Escape SHALL clear the object selection. The point selection SHALL NOT create undo steps.

#### Scenario: Marquee selects points
- **WHEN** a marquee is dragged around two of the four points of a path
- **THEN** exactly those two points are selected

#### Scenario: Shift+click toggles a point
- **WHEN** point 1 is selected and the user Shift+clicks point 3, then Shift+clicks point 1
- **THEN** only point 3 is selected

### Requirement: Moving points and handles
Dragging a selected anchor point SHALL move every selected point with the pointer, their handles moving with them; dragging an unselected point SHALL select it alone first. Holding Shift SHALL constrain the movement to horizontal, vertical or 45°. Arrow keys SHALL nudge the selected points by 1 texture pixel, and by 10 with Shift. Dragging a handle SHALL move that handle; for a smooth point the opposite handle SHALL stay aligned (opposite direction, keeping its own length), and holding Alt/Option while dragging a handle SHALL move it alone and make the point a corner. The object's bounds, selection overlay and identity SHALL follow the edit; the object's rotation SHALL be unchanged.

#### Scenario: Move one point
- **WHEN** the corner point at (500, 100) of a path is dragged to (520, 140)
- **THEN** only that point moves, the segments attached to it follow, and the selection bounds update

#### Scenario: Smooth handle stays aligned
- **WHEN** the outgoing handle of a smooth point at (500, 500) is dragged from (600, 500) to (500, 400)
- **THEN** the incoming handle points straight down from the point, keeping its previous length

#### Scenario: Alt breaks the handle
- **WHEN** a handle of a smooth point is dragged with Alt/Option held
- **THEN** only that handle moves and the point becomes a corner point

### Requirement: Changing, inserting and deleting points
With the Direct Selection tool:

- Double-clicking an anchor point SHALL toggle it between corner and smooth. A corner point that becomes smooth SHALL get aligned handles along the direction of its neighbors. A smooth point that becomes a corner SHALL lose its handles.
- Double-clicking a segment SHALL insert an anchor point at that location without changing the shape of the path.
- Pressing Delete or Backspace with points selected SHALL remove those points, and the segments SHALL reconnect across the removed points:
  - an open subpath left with fewer than 2 points, or a closed subpath left with fewer than 3 points, SHALL be removed;
  - a path left with no subpaths SHALL be deleted.
- Pressing Delete with objects selected and no points selected SHALL delete the objects, as with the Selection tool.

#### Scenario: Insert keeps the shape
- **WHEN** the user double-clicks the middle of a curved segment
- **THEN** a new point appears there and the path is drawn exactly as before

#### Scenario: Delete a point of a square
- **WHEN** one corner of a closed four-point path is selected and the user presses Delete
- **THEN** the path becomes a closed triangle

#### Scenario: Corner to smooth
- **WHEN** the user double-clicks a corner point
- **THEN** it becomes a smooth point with two aligned handles

### Requirement: Path edits are undoable
Each point or handle drag, each run of nudges with the arrow keys, each corner/smooth toggle, each insertion and each deletion SHALL be one undo step. Undo SHALL restore the path's points and handles.

#### Scenario: Undo a point drag
- **WHEN** the user drags a point and presses Cmd/Ctrl+Z
- **THEN** the point returns to its previous position

### Requirement: Convert to Path
The Object menu SHALL provide "Convert to Path". It SHALL be enabled when the selection contains at least one rectangle, ellipse or polygon, directly or inside selected groups.

- It SHALL turn each of these shapes into a path that looks identical, including rounded corners. The new path SHALL keep the shape's identity, name, fill, stroke, opacity, visibility, lock, rotation and position in the stacking order.
- Texts, images and existing paths SHALL be left unchanged.
- The conversion SHALL be one undo step.

#### Scenario: Convert a rounded rectangle
- **WHEN** a rectangle with a 40 px corner radius is selected and the user chooses Convert to Path
- **THEN** it becomes a closed path with the same outline, fill and stroke, and the Direct Selection tool shows its anchor points

#### Scenario: Convert inside a group
- **WHEN** a group containing an ellipse and a text is selected and the user chooses Convert to Path
- **THEN** the ellipse becomes a path and the text is unchanged

#### Scenario: Disabled without convertible shapes
- **WHEN** only a text is selected
- **THEN** Convert to Path is disabled
