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
The workspace SHALL show a vertical tool bar on the left with, in order: Selection, Direct Selection, Move, Rectangle, Ellipse, Polygon, Pen, Line, Text, Image, Eyedropper, Gradient, Zoom, Hand. Exactly one tool SHALL be active at a time and the active tool SHALL be identifiable without relying on color alone. Clicking a tool SHALL make it active.

#### Scenario: Default tool
- **WHEN** a project is opened
- **THEN** the Selection tool is active

#### Scenario: Switching tools
- **WHEN** the user clicks the Rectangle tool
- **THEN** the Rectangle tool shows the active state and the previously active tool no longer does

### Requirement: Canvas area
The canvas area SHALL occupy all space not used by bars and panels, SHALL show rulers along its top and left edges (see `canvas-guides`), and SHALL display the project's active texture surface as an artboard framed against a neutral pasteboard background. When a project is opened, the view SHALL be fitted so the whole artboard is centered and visible; afterwards the view is controlled by the user (see `canvas-navigation`). Resizing the canvas area SHALL keep the document point at the center of the canvas area fixed and SHALL NOT change the zoom level, unless the user has not navigated since the last fit, in which case the view is refitted.

#### Scenario: Artboard visible
- **WHEN** a new 4096×4096 project is opened
- **THEN** a square artboard is displayed centered in the canvas area and entirely visible

#### Scenario: Window resize
- **WHEN** the window is resized and the user has not zoomed or panned since the project was opened or last fitted
- **THEN** the view is refitted so the artboard remains centered and fully visible

#### Scenario: Window resize after navigating
- **WHEN** the user has zoomed to 200% and then resizes the window
- **THEN** the zoom stays at 200% and the document point that was at the center of the canvas area stays at its center

### Requirement: Right panel stack
The workspace SHALL show a right-hand panel column containing the panels Properties, Layers, Colors, Styles, Stroke, Transform and Assets. Each panel SHALL have a header with its title, can be collapsed/expanded by clicking its header, and can be closed. Closed panels SHALL be reopenable from the View menu. The column width SHALL be resizable by dragging its edge within sensible bounds. Panels whose feature is not yet available SHALL show an explicit empty state.

A panel layout saved before a panel existed SHALL get that panel at its default place, open and collapsed.

#### Scenario: Collapsing a panel
- **WHEN** the user clicks the header of the Layers panel
- **THEN** the panel body collapses, the header remains visible with a collapsed indicator, and other panels reflow

#### Scenario: Closing and reopening a panel
- **WHEN** the user closes the Colors panel and then chooses View > Colors
- **THEN** the Colors panel is shown again at its previous position in the stack

#### Scenario: Resizing the panel column
- **WHEN** the user drags the left edge of the panel column
- **THEN** the column width changes live and is clamped between a minimum and a maximum width

#### Scenario: Layout from before the Styles panel
- **WHEN** the application starts with a panel layout saved before the Styles panel existed
- **THEN** the Styles panel appears after Colors, collapsed, and the other panels keep their places and states

### Requirement: 3D preview panel placeholder
The workspace SHALL include a 3D preview panel that can be shown, hidden and resized. Until 3D rendering exists, the panel SHALL display an explicit placeholder state.

#### Scenario: Hiding the 3D panel
- **WHEN** the user toggles the 3D preview from the View menu
- **THEN** the 3D panel is hidden and the canvas area expands to use the freed space

### Requirement: View modes
The workspace SHALL offer three view modes — 2D, 3D and Split — selectable from a segmented control and from the View menu with shortcuts. 2D shows only the canvas area, 3D shows only the 3D preview area, Split shows the canvas area and the 3D preview side by side with a draggable divider.

Opening or creating a project SHALL show its canvas: when the remembered view mode is 3D, the project opens in 2D. 2D and Split are kept.

#### Scenario: Switching to split view
- **WHEN** the user selects Split
- **THEN** the canvas area is shown on the left and the 3D preview on the right, separated by a draggable divider

#### Scenario: Opening a project after using the 3D view
- **WHEN** the user last used the 3D view and opens a project
- **THEN** the project opens in 2D with its canvas visible

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

### Requirement: Vehicles sidebar
The workspace SHALL show a sidebar on the left, between the tool bar and the canvas area, open by default. It holds two collapsible sections:
- **Project:** the project's Name, Version and Game versions, read only for now (Version and Game versions are empty until a later feature sets them). The game is not listed: it can't change once chosen and titles the Vehicles section;
- **Vehicles:** the fleet's navigation (see the vehicle-projects capability).

The Project header has a Hide Sidebar button that reduces the sidebar to a narrow strip, whose Show Sidebar button opens it again. View › Sidebar (F5) SHALL toggle it, and Vehicle › Vehicle Information SHALL open it. Its width SHALL be resizable by dragging its edge within sensible bounds. Whether it is open, and its width, SHALL be remembered across sessions; Reset Workspace SHALL open it at its default width.

#### Scenario: Hiding the sidebar
- **WHEN** the user clicks Hide Sidebar
- **THEN** the sidebar becomes a narrow strip with a Show Sidebar button, the canvas area widens, and the sidebar stays hidden after a restart

#### Scenario: Toggling with the keyboard
- **WHEN** the sidebar is hidden and the user presses F5
- **THEN** the sidebar is shown again

#### Scenario: Project properties
- **WHEN** a project named "ACE Logistics" is open
- **THEN** the Project section shows the name "ACE Logistics" and empty Version and Game versions fields, none of them editable
