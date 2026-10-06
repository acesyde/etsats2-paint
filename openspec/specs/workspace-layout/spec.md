# workspace-layout Specification

## Purpose

Defines the editor workspace layout (menus, tools, canvas area, panels, 3D preview area, view modes and status bar) that all editing features plug into.

## Requirements

### Requirement: Menu bar
The workspace SHALL show a menu bar with the menus File, Edit, Object, Layer, View, Vehicle, Export and Help. Each menu item SHALL display its shortcut (when it has one) right-aligned in the platform's notation. Items whose feature is not yet available SHALL be shown disabled.

#### Scenario: Menus present
- **WHEN** a project is open
- **THEN** the menu bar shows exactly File, Edit, Object, Layer, View, Vehicle, Export, Help in that order

#### Scenario: Closing a project from the File menu
- **WHEN** the user chooses File > Close
- **THEN** the project closes and the home screen is shown

### Requirement: Tool bar
The workspace SHALL show a vertical tool bar on the left with, in order: Selection, Direct Selection, Move, Rectangle, Ellipse, Polygon, Pen, Line, Text, Image, Eyedropper, Zoom, Hand. Exactly one tool SHALL be active at a time and the active tool SHALL be identifiable without relying on color alone. Clicking a tool SHALL make it active.

#### Scenario: Default tool
- **WHEN** a project is opened
- **THEN** the Selection tool is active

#### Scenario: Switching tools
- **WHEN** the user clicks the Rectangle tool
- **THEN** the Rectangle tool shows the active state and the previously active tool no longer does

### Requirement: Canvas area
The canvas area SHALL occupy all space not used by bars and panels, and SHALL display the project's texture surface as a centered artboard fitted to the available space, framed against a neutral pasteboard background.

#### Scenario: Artboard visible
- **WHEN** a new 4096×4096 project is opened
- **THEN** a square artboard is displayed centered in the canvas area and entirely visible

#### Scenario: Window resize
- **WHEN** the window is resized
- **THEN** the canvas area grows or shrinks to fill the remaining space and the artboard remains fully visible

### Requirement: Right panel stack
The workspace SHALL show a right-hand panel column containing the panels Properties, Layers, Colors, Stroke, Transform, Assets and Vehicle. Each panel SHALL have a header with its title, can be collapsed/expanded by clicking its header, and can be closed. Closed panels SHALL be reopenable from the View menu. The column width SHALL be resizable by dragging its edge within sensible bounds. Panels whose feature is not yet available SHALL show an explicit empty state.

#### Scenario: Collapsing a panel
- **WHEN** the user clicks the header of the Layers panel
- **THEN** the panel body collapses, the header remains visible with a collapsed indicator, and other panels reflow

#### Scenario: Closing and reopening a panel
- **WHEN** the user closes the Colors panel and then chooses View > Colors
- **THEN** the Colors panel is shown again at its previous position in the stack

#### Scenario: Resizing the panel column
- **WHEN** the user drags the left edge of the panel column
- **THEN** the column width changes live and is clamped between a minimum and a maximum width

### Requirement: 3D preview panel placeholder
The workspace SHALL include a 3D preview panel that can be shown, hidden and resized. Until 3D rendering exists, the panel SHALL display an explicit placeholder state.

#### Scenario: Hiding the 3D panel
- **WHEN** the user toggles the 3D preview from the View menu
- **THEN** the 3D panel is hidden and the canvas area expands to use the freed space

### Requirement: View modes
The workspace SHALL offer three view modes — 2D, 3D and Split — selectable from a segmented control and from the View menu with shortcuts. 2D shows only the canvas area, 3D shows only the 3D preview area, Split shows the canvas area and the 3D preview side by side with a draggable divider.

#### Scenario: Switching to split view
- **WHEN** the user selects Split
- **THEN** the canvas area is shown on the left and the 3D preview on the right, separated by a draggable divider

### Requirement: Status bar
The workspace SHALL show a status bar displaying at least: current zoom level, pointer position in texture pixels when over the artboard, the active surface name, and the document save state. The save state SHALL be shown as text ("Saved" / "Unsaved changes") together with an icon, never by color alone.

#### Scenario: Pointer coordinates
- **WHEN** the pointer moves over the artboard
- **THEN** the status bar shows the pointer position in texture pixel coordinates

#### Scenario: Save state indicator for a new project
- **WHEN** a new project has just been created and never saved
- **THEN** the status bar shows "Unsaved changes" with its icon

### Requirement: Context menus
Right-clicking in the canvas area, on a panel header and on a tool SHALL open a context menu showing the commands relevant to that location, with their shortcuts.

#### Scenario: Context menu on a panel header
- **WHEN** the user right-clicks the Layers panel header
- **THEN** a context menu offers at least Collapse/Expand and Close panel
