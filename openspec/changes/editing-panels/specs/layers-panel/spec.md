## Purpose

Shows the object tree of the active surface and lets users organize it like in Photoshop or Illustrator: select, rename, hide, lock, reorder, group and ungroup.

## ADDED Requirements

### Requirement: Tree view
The Layers panel SHALL list the active surface's objects as a tree, topmost first, with groups expandable and collapsible and their children indented. Each row SHALL show a kind icon, the name, a visibility toggle and a lock toggle; hidden and locked states SHALL be shown by icon shape and by dimming the name, not only by color. An empty surface SHALL show an empty state.

#### Scenario: Group shown as a tree
- **WHEN** the surface contains a group "Branding" with "Logo" and "Company name", above a rectangle "Background"
- **THEN** the panel shows "Branding" first with its two children indented below it, then "Background"

### Requirement: Selection sync
Clicking a row SHALL select that object (including objects inside groups); Cmd/Ctrl+click SHALL toggle it in the selection; Shift+click SHALL select the range of visible rows from the last clicked row. Selected rows SHALL be highlighted with a fill and an accent bar. Selecting on the canvas SHALL highlight the corresponding rows and expand collapsed ancestors so they are visible.

#### Scenario: Row selects canvas object
- **WHEN** the user clicks the "Logo" row inside the "Branding" group
- **THEN** only the Logo object is selected on the canvas with its handles

### Requirement: Rename
Double-clicking a name or choosing Rename from the context menu SHALL edit the name inline; Enter commits, Escape cancels; empty names revert to the default name for the kind ("Rectangle", "Ellipse", "Group").

#### Scenario: Rename a layer
- **WHEN** the user double-clicks "Rectangle", types "Burgundy base" and presses Enter
- **THEN** the row and the Properties panel show "Burgundy base"

### Requirement: Visibility and lock
The eye toggle SHALL hide or show the object (and all its children for a group); hidden objects are not drawn, not hit on the canvas and not exported later, but stay in the tree. The lock toggle SHALL prevent the object (and its children) from being selected or modified on the canvas; locked objects remain selectable from the panel only for unlocking. Hiding or locking a selected object SHALL remove it from the selection.

#### Scenario: Hide an object
- **WHEN** the user clicks the eye of "White stripe"
- **THEN** the stripe disappears from the canvas, its row is dimmed with a closed-eye icon, and clicking where it was does not select it

#### Scenario: Locked object ignores canvas clicks
- **WHEN** "Background" is locked and the user clicks on it on the canvas
- **THEN** it is not selected

### Requirement: Drag and drop
Dragging rows SHALL reorder objects: a drop indicator line shows the target position between rows, and dropping onto the middle of a group row (highlighted) SHALL move the dragged objects into that group at its top. Dragging several selected rows SHALL move them together, keeping their relative order. A group SHALL NOT be dropped into itself or its descendants.

#### Scenario: Move an object into a group
- **WHEN** the user drags "Truck number" onto the middle of the "Branding" group row
- **THEN** "Truck number" becomes the topmost child of "Branding"

#### Scenario: Reorder between rows
- **WHEN** the user drags "Grey stripe" above "White stripe" inside "Graphics"
- **THEN** "Grey stripe" is drawn above "White stripe"

### Requirement: Group and ungroup
Group (Cmd/Ctrl+G) SHALL wrap the selected objects in a new group named "Group" placed at the position of the topmost selected object, keeping their stacking order, and select the group. Ungroup (Cmd/Ctrl+Shift+G) SHALL replace each selected group by its children at the same position and select those children. New Layer (Cmd/Ctrl+Shift+N) SHALL add an empty top-level group named "Layer N" at the top and select it; Duplicate Layer and Delete Layer act like Duplicate and Delete on the selected rows. These commands SHALL also be available from the panel's context menu and footer buttons.

#### Scenario: Group then ungroup
- **WHEN** two rectangles are selected and the user presses Cmd/Ctrl+G and then Cmd/Ctrl+Shift+G
- **THEN** after grouping, one group containing both is selected; after ungrouping, the two rectangles are back at the same stacking position and selected

#### Scenario: New layer
- **WHEN** the user presses Cmd/Ctrl+Shift+N in a project without groups
- **THEN** an empty group "Layer 1" appears at the top of the tree and is selected
