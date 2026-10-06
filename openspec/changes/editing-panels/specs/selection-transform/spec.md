## MODIFIED Requirements

### Requirement: Selecting objects
With the Selection or Move tool, clicking an object SHALL select it alone; when the clicked object is inside a group, the click SHALL select its top-level group, and Cmd/Ctrl+click SHALL select the innermost object instead. Shift+click SHALL add an unselected object to the selection or remove a selected one; clicking empty canvas SHALL clear the selection; dragging from empty canvas SHALL draw a marquee and select every top-level object that intersects it (Shift+marquee adds to the selection). Hidden and locked objects SHALL NOT be selectable on the canvas. Select All (Cmd/Ctrl+A) SHALL select every visible, unlocked top-level object of the active surface and Escape SHALL clear the selection. Hovering an unselected object SHALL show a thin outline of the object (or group) that a click would select.

#### Scenario: Shift+click toggles
- **WHEN** object A is selected and the user Shift+clicks object B, then Shift+clicks A
- **THEN** only B is selected

#### Scenario: Marquee selection
- **WHEN** the user drags a marquee from empty canvas that touches two of three objects
- **THEN** exactly those two objects are selected

#### Scenario: Click selects the group
- **WHEN** a rectangle inside the group "Graphics" is clicked
- **THEN** the group "Graphics" is selected, and Cmd/Ctrl+click on the same point selects the rectangle itself
