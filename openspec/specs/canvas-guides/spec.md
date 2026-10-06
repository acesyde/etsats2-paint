# canvas-guides Specification

## Purpose

Defines the canvas rulers and the guides users drag out of them to line up artwork, and how guides are shown, moved, deleted and cleared.

## Requirements

### Requirement: Rulers
The canvas SHALL show a horizontal ruler along its top edge and a vertical ruler along its left edge, graduated in texture pixels with labels at regular steps chosen from the zoom level (so labels never overlap), following zoom and pan. Each ruler SHALL mark the pointer position while the pointer is over the canvas. The rulers SHALL be shown whenever a project is open.

#### Scenario: Rulers follow navigation
- **WHEN** the user zooms in on the artboard's top-left corner
- **THEN** the rulers show finer graduations and the label "0" stays aligned with the artboard's left and top edges

#### Scenario: Pointer marker
- **WHEN** the pointer is over the artboard at texture pixel (1200, 800)
- **THEN** the top ruler marks 1200 and the left ruler marks 800

### Requirement: Create guides from the rulers
Dragging from the top ruler onto the canvas SHALL create a horizontal guide, and dragging from the left ruler SHALL create a vertical guide, following the pointer with its position shown while dragging. The guide SHALL be added to the active surface when released over the canvas; releasing over a ruler SHALL create nothing. Creating a guide SHALL show guides if they were hidden. When snapping is on, the guide being dragged SHALL snap to the grid, the artboard edges and center and other objects' edges and centers.

#### Scenario: Horizontal guide
- **WHEN** the user drags from the top ruler and releases at texture y = 1024
- **THEN** a horizontal guide at y = 1024 is added and shown across the whole canvas

#### Scenario: Released back on the ruler
- **WHEN** the user drags from the left ruler and releases over the ruler
- **THEN** no guide is created

### Requirement: Move and delete guides
With the Selection, Move or Direct Selection tool and guides shown, hovering a guide (within a few screen pixels) SHALL show a resize cursor, and dragging it SHALL move it along its axis, with its position shown. Releasing a guide over a ruler (or outside the canvas) SHALL delete it. Objects SHALL take priority over guides when both are under the pointer only if the pointer is inside an object's filled shape; a press on a guide over empty canvas or over an object's bounds outside its shape SHALL grab the guide. Guides SHALL NOT be selectable or movable while hidden.

#### Scenario: Move a guide
- **WHEN** the user drags a vertical guide from x = 500 to x = 2048
- **THEN** the guide is at x = 2048 and one Undo puts it back at x = 500

#### Scenario: Delete by dropping on a ruler
- **WHEN** the user drags a horizontal guide onto the top ruler and releases
- **THEN** the guide is removed

### Requirement: Show and clear guides
View › Show Guides (⌘; on macOS, Ctrl+; elsewhere) SHALL toggle the display of guides and show a check mark when they are shown; guides are shown by default. View › Clear Guides SHALL remove every guide of the active surface as one undo step, and SHALL be disabled when the surface has no guides. Guides SHALL be drawn as thin lines of a distinct guide color above the artwork and below the selection overlay, extending across the whole canvas, and SHALL never be exported.

#### Scenario: Hide guides
- **WHEN** guides exist and the user presses ⌘;
- **THEN** the guides are no longer drawn and View › Show Guides is unchecked

#### Scenario: Guides are not exported
- **WHEN** the texture is exported with guides shown
- **THEN** the image contains no guide lines
