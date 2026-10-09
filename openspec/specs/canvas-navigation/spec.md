# canvas-navigation Specification

## Purpose

Defines how the user moves around the canvas — zooming, panning, fitting — with behavior close to Illustrator, Photoshop and Figma and smooth on both mouse and trackpad.

## Requirements

### Requirement: Zoom toward the pointer
Zooming with the mouse wheel, a pinch gesture or Cmd/Ctrl+wheel SHALL keep the document point under the pointer fixed on screen. Zoom SHALL be limited to the range 2% to 6400%, where 100% means one texture pixel per physical screen pixel.

#### Scenario: Wheel zoom keeps the point under the pointer
- **WHEN** the pointer is over a corner of a rectangle and the user zooms in with the mouse wheel
- **THEN** the zoom increases and that corner stays under the pointer

#### Scenario: Zoom limits
- **WHEN** the user keeps zooming in past 6400%
- **THEN** the zoom stops at 6400%

### Requirement: Hybrid wheel and trackpad mapping
On the canvas, input SHALL be mapped as follows: a mouse wheel (line-based scrolling) zooms; trackpad two-finger scrolling (pixel-based scrolling) pans in both directions; a pinch gesture zooms; Cmd/Ctrl + any scroll zooms; Shift + mouse wheel pans horizontally.

#### Scenario: Mouse wheel zooms
- **WHEN** the user scrolls the mouse wheel over the canvas
- **THEN** the canvas zooms toward the pointer

#### Scenario: Trackpad scroll pans
- **WHEN** the user scrolls with two fingers on a trackpad
- **THEN** the canvas pans by the scrolled distance without changing zoom

#### Scenario: Shift + wheel pans horizontally
- **WHEN** the user holds Shift and scrolls the mouse wheel
- **THEN** the canvas pans horizontally

### Requirement: Panning
The user SHALL be able to pan by dragging with the Hand tool, by holding Space and dragging with any tool active, and by dragging with the middle mouse button. While panning, the cursor SHALL show a grabbing hand.

#### Scenario: Space + drag
- **WHEN** the Rectangle tool is active and the user holds Space and drags on the canvas
- **THEN** the canvas pans, no rectangle is created, and the Rectangle tool is active again after Space is released

### Requirement: View commands
The application SHALL provide Zoom In (Cmd/Ctrl+=), Zoom Out (Cmd/Ctrl+-), Fit to Screen (Shift+Cmd/Ctrl+0) and Actual Size 100% (Cmd/Ctrl+0). Zoom In/Out SHALL step through preset levels (for example 25%, 33%, 50%, 67%, 100%, 150%, 200%, …) centered on the canvas area. Fit to Screen SHALL show the whole artboard centered with a margin. These shortcuts SHALL zoom the canvas, not the application interface. Cmd/Ctrl+1 SHALL NOT change the zoom: it shows the Project space.

#### Scenario: Fit to screen
- **WHEN** the user is zoomed in on a corner and presses Shift+Cmd/Ctrl+0
- **THEN** the whole artboard is visible and centered

#### Scenario: Actual size
- **WHEN** the user presses Cmd/Ctrl+0
- **THEN** the zoom becomes 100% centered on the current view center

#### Scenario: Cmd/Ctrl+1 no longer zooms
- **WHEN** the Workshop is shown at 50% and the user presses Cmd/Ctrl+1
- **THEN** the Project space is shown, and the zoom is still 50% when the user returns to the Workshop

#### Scenario: Interface does not zoom
- **WHEN** the user presses Cmd/Ctrl+=
- **THEN** the canvas zooms in and menus, panels and text keep their size

### Requirement: Zoom tool
With the Zoom tool active, clicking SHALL zoom in one step centered on the clicked point, Alt/Option+click SHALL zoom out one step, and dragging a rectangle SHALL zoom to fit that rectangle. The cursor SHALL show a magnifier with "+" or "−".

#### Scenario: Zoom to area
- **WHEN** the Zoom tool is active and the user drags a rectangle around an object
- **THEN** the view zooms so that rectangle fills the canvas area

### Requirement: Smooth and responsive navigation
Zooming and panning SHALL update on the same frame as the input and SHALL NOT recompute document geometry when only the view changes.

#### Scenario: Panning a heavy document
- **WHEN** a surface contains 1000 shapes and the user pans continuously
- **THEN** the canvas follows the pointer without visible lag
