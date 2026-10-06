## Purpose

Defines how positions snap to the grid, guides, the artboard and other objects while moving, resizing and drawing, and how snapping is shown, toggled and bypassed.

## ADDED Requirements

### Requirement: Snapping toggle
View › Snapping (⌘⇧; on macOS, Ctrl+Shift+; elsewhere) SHALL turn snapping on or off and show a check mark when it is on; snapping is on by default. Holding Cmd/Ctrl during a drag that has already started SHALL suspend snapping for that drag while held.

#### Scenario: Turn snapping off
- **WHEN** the user turns snapping off and drags a rectangle next to a guide
- **THEN** the rectangle follows the pointer exactly

#### Scenario: Temporary bypass
- **WHEN** snapping is on and the user holds Cmd/Ctrl while dragging a shape near the artboard center
- **THEN** the shape does not snap to the center

### Requirement: Snap targets
When snapping is on, positions SHALL snap, independently on each axis, to the nearest of these targets within 6 screen points:

- grid lines, when the grid is shown;
- guides, when guides are shown;
- the artboard's edges and center lines;
- the left, right, top and bottom edges and the center lines of the bounds of other visible objects (not the ones being edited; objects inside groups count individually);
- for point snapping (see below), also the anchor points of other visible paths and the corners and center of other objects' bounds, snapping both axes at once.

When several targets are within reach on an axis, the closest SHALL win.

#### Scenario: Snap to a guide
- **WHEN** a vertical guide is at x = 1000 and the user moves a 200 px wide rectangle so its left edge comes 3 screen points from the guide
- **THEN** the rectangle's left edge is exactly at x = 1000

#### Scenario: Snap to the artboard center
- **WHEN** the user moves a logo so that its center comes close to the artboard's vertical center line
- **THEN** the logo's center is exactly on the center line

#### Scenario: Hidden grid does not snap
- **WHEN** the grid is hidden
- **THEN** nothing snaps to grid lines

### Requirement: What snaps
With snapping on:

- **Moving** objects SHALL snap the selection bounds: its left, center and right on x, and its top, center and bottom on y (the closest of the three per axis).
- **Resizing** SHALL snap the dragged handle's position.
- **Drawing** a rectangle, ellipse, polygon or line SHALL snap the press point and the pointer.
- **Pen** points, **Direct Selection** point and handle drags SHALL use point snapping.
- **Guides** being created or moved SHALL snap along their axis.

Rotation, text editing and panel edits SHALL NOT snap. Snapping SHALL apply after the modifier constraints (Shift for 45° or proportions) only on axes the constraint leaves free.

#### Scenario: Drawing from a grid intersection
- **WHEN** the grid (64 px) is shown and the user draws a rectangle starting 2 screen points from (128, 256)
- **THEN** the rectangle's top-left corner is exactly (128, 256)

#### Scenario: Pen point on another path's point
- **WHEN** the user clicks with the Pen tool 3 screen points away from an anchor point of another path
- **THEN** the new point is placed exactly on that anchor point

### Requirement: Alignment lines
While a drag is snapped, the canvas SHALL show a thin alignment line along each snapped axis, extending between the snapped element and the target it snapped to (the full canvas for grid lines, guides and the artboard), in a dedicated snapping color; a snapped point SHALL be marked with a small cross. The lines SHALL disappear when the drag ends or stops snapping.

#### Scenario: Smart guide between two objects
- **WHEN** the user moves a rectangle so that its top edge snaps to the top edge of another rectangle
- **THEN** a horizontal alignment line is drawn along that edge between the two rectangles
