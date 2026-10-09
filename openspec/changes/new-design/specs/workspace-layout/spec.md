## ADDED Requirements

### Requirement: Workspace frame
The window of an open project SHALL be arranged, from top to bottom, as: the menu bar, the top bar (see the workspace-spaces capability), in the Workshop the tool options bar, then the space's content, then the status bar.

In the Workshop, the content SHALL be, from left to right: the tool rail, the left panel, the canvas area and the inspector. The Project and Brand spaces SHALL use the full width between the top bar and the status bar, without tool rail, tool options bar, left panel nor inspector.

View › Reset Workspace SHALL show the left panel and the inspector at their default widths, with the Textures tab.

#### Scenario: Workshop frame
- **WHEN** the Workshop is shown
- **THEN** the tool rail, the left panel, the canvas area and the inspector are shown from left to right, between the tool options bar and the status bar

#### Scenario: Project frame
- **WHEN** the Project space is shown
- **THEN** no tool rail, tool options bar, left panel nor inspector is shown, and the status bar shows only the save state

#### Scenario: Reset Workspace
- **WHEN** the panels are hidden, the Layers tab is shown and the inspector was widened, and the user chooses View › Reset Workspace
- **THEN** the left panel and the inspector are shown at their default widths and the Textures tab is shown

### Requirement: Tool options bar
In the Workshop, a tool options bar SHALL run under the top bar and show the settings of the active tool, labelled in plain words:
- **Polygon:** a Sides field (3 to 12), a Star toggle and, when Star is on, an Inner radius field (10% to 90%). They set the sides and star settings used for new polygons. When every selected object is a polygon, a change SHALL also apply to them as in the path-tools capability (Polygon settings), one undo step per change, and differing values SHALL show "Mixed". The inspector's Polygon section edits the same settings (see the properties-panel capability), and both places SHALL always show the same values.
- **Every other tool:** the tool's name only; no other setting is shown.

The tool options bar SHALL stay shown when the panels are hidden.

#### Scenario: Polygon settings for new polygons
- **WHEN** the Polygon tool is active, nothing is selected, and the user sets Sides to 5 and turns Star on in the tool options bar
- **THEN** the next polygon drawn is a five-point star, and no undo step was recorded by the settings

#### Scenario: Editing selected polygons
- **WHEN** two hexagons are selected, the Polygon tool is active and the user types 8 in Sides
- **THEN** both become octagons in their bounds, and one Undo restores both hexagons

#### Scenario: Tool without settings
- **WHEN** the Hand tool is active
- **THEN** the tool options bar shows "Hand" and no setting

### Requirement: Left panel
In the Workshop, a left panel between the tool rail and the canvas area SHALL show one of three tabs, in this order: **Textures** (see the vehicle-projects capability), **Layers** (see the layers-panel capability) and **Resources** (see the color-panel, symbols, shared-styles and assets-panel capabilities). The shown tab SHALL be identifiable without relying on color alone.

The Resources tab SHALL list, in this order, compact sections for the project's **Palette**, **Symbols**, **Styles** and **Images**, with the Import from Library… button (see the shared-library capability). The Palette section's swatches SHALL behave as the brand palette of the color popover (see the color-panel capability, Project palette): clicking a swatch applies its color to the current target, or to the selected stop when the target is a gradient, and links it to the swatch; the swatch linked to the current target is marked the same way; and each swatch's context menu offers Edit Swatch…, Delete Swatch and Add to Library or Update in Library.

Clicking a tab, View › Textures (1), View › Layers (2) and View › Resources (3) SHALL show that tab. Choosing a tab from the View menu or its key while another space is shown SHALL show the Workshop with that tab, and while the panels are hidden SHALL show them again. The single keys 1, 2 and 3 SHALL act only when no text field has keyboard focus, no text is being edited on the canvas and no modal dialog is open; otherwise they type as usual.

The panel's width SHALL be resizable by dragging its edge, clamped between a minimum and a maximum width. The shown tab and the width SHALL be remembered across sessions; the first launch shows the Textures tab.

#### Scenario: Tabs from the keyboard
- **WHEN** the Workshop is shown with the Textures tab and the user presses 2
- **THEN** the Layers tab is shown, listing the active texture's layers

#### Scenario: Keys in a text field
- **WHEN** the user types "2" in the Opacity field of the inspector
- **THEN** the field receives "2" and the shown tab does not change

#### Scenario: Resources tab sections
- **WHEN** the Resources tab is shown for a project with a palette, symbols, styles and images
- **THEN** it shows the Palette, Symbols, Styles and Images sections in that order

#### Scenario: Applying a palette swatch from the Resources tab
- **WHEN** a rectangle is selected with the fill as target, and the user clicks the swatch "Company red" in the Palette section of the Resources tab
- **THEN** the rectangle's fill takes that color, linked to "Company red", the Fill row reads "Company red" with a link icon, and one Undo restores the previous fill

#### Scenario: Swatch context menu in the Resources tab
- **WHEN** the user right-clicks the swatch "Company red" in the Resources tab
- **THEN** the context menu offers Edit Swatch…, Delete Swatch and Add to Library, as in the color popover

#### Scenario: Resizing the left panel
- **WHEN** the user drags the right edge of the left panel
- **THEN** its width changes live, clamped between a minimum and a maximum, and the canvas area takes the rest

#### Scenario: Tab remembered
- **WHEN** the user shows the Resources tab, quits and reopens a project
- **THEN** the Workshop shows the Resources tab

### Requirement: Inspector
In the Workshop, an inspector on the right of the canvas area SHALL show the settings of the selection, or of the active texture when nothing is selected; its content is specified by the properties-panel capability. It SHALL never be shown as an empty panel. Its width SHALL be resizable by dragging its left edge, clamped between a minimum and a maximum width, and remembered across sessions.

#### Scenario: Resizing the inspector
- **WHEN** the user drags the left edge of the inspector
- **THEN** its width changes live, clamped between a minimum and a maximum width, and it keeps that width after a restart

#### Scenario: Nothing selected
- **WHEN** the Workshop is shown and nothing is selected
- **THEN** the inspector shows the active texture's properties

### Requirement: Focus mode
View › Hide Panels (Tab) SHALL hide the left panel and the inspector of the Workshop so the canvas area takes their place; choosing it again SHALL show them at their previous widths. The menu bar, the top bar, the tool options bar, the tool rail and the status bar SHALL stay. The canvas view follows the resizing rule of the Canvas area requirement. Whether the panels are hidden SHALL be remembered across sessions. Hide Panels SHALL be enabled only in the Workshop.

Tab SHALL toggle the panels only when no text field has keyboard focus, no text is being edited on the canvas and no modal dialog is open; otherwise Tab moves the keyboard focus as usual, including inside dialogs.

#### Scenario: Hiding the panels
- **WHEN** the Workshop is shown and the user presses Tab
- **THEN** the left panel and the inspector are hidden, the tool rail, the bars and the breadcrumb on the canvas stay, and pressing Tab again shows the panels

#### Scenario: Tab in a field
- **WHEN** the keyboard focus is in the X field of the inspector and the user presses Tab
- **THEN** the focus moves to the next field and the panels stay shown

#### Scenario: Tab in a dialog
- **WHEN** the Export Mod dialog is open and the user presses Tab
- **THEN** the focus moves to the next control of the dialog and the panels behind it do not change

### Requirement: Canvas and tool context menus
Right-clicking in the canvas area and on a tool of the tool rail SHALL open a context menu showing the commands relevant to that location, with their shortcuts.

#### Scenario: Context menu on a tool
- **WHEN** the user right-clicks the Pen tool
- **THEN** a context menu offers the Pen tool with its shortcut and Keyboard Shortcuts

## MODIFIED Requirements

### Requirement: Menu bar
The workspace SHALL show a menu bar with the menus File, Edit, Object, Layer, View, Vehicle, Export and Help, in every space. Each menu item SHALL display its shortcut (when it has one) right-aligned in the platform's notation. Items whose feature is not yet available SHALL be shown disabled.

The View menu SHALL list, in groups: the spaces Project, Workshop and Brand; the left tabs Textures, Layers and Resources; Hide Panels; then Show Template, Show Grid, Show Guides, Clear Guides and Snapping; then Zoom In, Zoom Out, Fit to Screen and Actual Size; then Reset Workspace. Toggles SHALL show their state with a check mark. The Vehicle menu SHALL NOT have a Vehicle Information item, and the View menu SHALL NOT have Sidebar nor panel items: the spaces and the left tabs replace them.

#### Scenario: Menus present
- **WHEN** a project is open
- **THEN** the menu bar shows exactly File, Edit, Object, Layer, View, Vehicle, Export, Help in that order

#### Scenario: Closing a project from the File menu
- **WHEN** the user chooses File > Close
- **THEN** the project closes and the home screen is shown

#### Scenario: View menu
- **WHEN** the user opens the View menu in the Workshop with the Layers tab shown
- **THEN** it lists Project, Workshop (checked) and Brand, then Textures, Layers (checked) and Resources, then Hide Panels, then the template, grid, guides and snapping items, then the zoom commands, then Reset Workspace

#### Scenario: No sidebar item
- **WHEN** the user opens the Vehicle menu
- **THEN** it has no Vehicle Information item, and pressing F5 does nothing

### Requirement: Tool rail
The Workshop SHALL show a vertical tool rail, 48 px wide, on the left, with, in order: Selection, Direct Selection, Move, Rectangle, Ellipse, Polygon, Pen, Line, Text, Image, Eyedropper, Gradient, Zoom, Hand. Each tool SHALL be shown as an icon without text; hovering it SHALL show a tooltip with its translated name and its shortcut in the platform's notation (the Gradient tool's shortcut is Shift+G), and each tool SHALL announce its name to assistive technologies. Exactly one tool SHALL be active at a time and the active tool SHALL be identifiable without relying on color alone. Clicking a tool SHALL make it active. The tool rail SHALL stay shown when the panels are hidden.

#### Scenario: Default tool
- **WHEN** a project is opened
- **THEN** the Selection tool is active

#### Scenario: Switching tools
- **WHEN** the user clicks the Rectangle tool
- **THEN** the Rectangle tool shows the active state and the previously active tool no longer does

#### Scenario: Tooltip
- **WHEN** the user hovers the Gradient tool with the interface in French
- **THEN** a tooltip shows its French name and the shortcut Shift+G

### Requirement: Canvas area
The canvas area SHALL occupy, in the Workshop, all space not used by the bars, the tool rail, the left panel and the inspector, SHALL show rulers along its top and left edges (see `canvas-guides`), and SHALL display the project's active texture surface as an artboard framed against a neutral pasteboard background, with the breadcrumb inlaid (see the workspace-spaces capability). When a project is opened, the view SHALL be fitted so the whole artboard is centered and visible; afterwards the view is controlled by the user (see `canvas-navigation`). Resizing the canvas area, including by resizing, hiding or showing the left panel or the inspector, SHALL keep the document point at the center of the canvas area fixed and SHALL NOT change the zoom level, unless the user has not navigated since the last fit, in which case the view is refitted.

#### Scenario: Artboard visible
- **WHEN** a new 4096×4096 project is opened and the Workshop is shown
- **THEN** a square artboard is displayed centered in the canvas area and entirely visible

#### Scenario: Window resize
- **WHEN** the window is resized and the user has not zoomed or panned since the project was opened or last fitted
- **THEN** the view is refitted so the artboard remains centered and fully visible

#### Scenario: Window resize after navigating
- **WHEN** the user has zoomed to 200% and then resizes the window
- **THEN** the zoom stays at 200% and the document point that was at the center of the canvas area stays at its center

#### Scenario: Hiding the panels after navigating
- **WHEN** the user has zoomed to 200% and then presses Tab to hide the panels
- **THEN** the canvas area widens, the zoom stays at 200% and the document point that was at its center stays at its center

### Requirement: Status bar
The workspace SHALL show a status bar, 28 px high, in every space.

In the Workshop it SHALL display:
- the current zoom level;
- the pointer position in texture pixels when over the artboard;
- the template settings of the active texture: a Template toggle, its opacity, and the key G (see the vehicle-projects capability);
- toggles for Snapping, Grid and Guides, which do the same as View › Snapping, View › Show Grid and View › Show Guides and show their state without relying on color alone;
- the document save state.

In the Project and Brand spaces it SHALL display only the save state.

The save state SHALL be shown as text ("Saved" / "Unsaved changes") together with an icon, never by color alone.

#### Scenario: Pointer coordinates
- **WHEN** the pointer moves over the artboard
- **THEN** the status bar shows the pointer position in texture pixel coordinates

#### Scenario: Save state indicator for a new project
- **WHEN** a new project has just been created and never saved
- **THEN** the status bar shows "Unsaved changes" with its icon

#### Scenario: Grid toggle
- **WHEN** the grid is hidden and the user clicks Grid in the status bar
- **THEN** the grid is shown on the canvas, the Grid toggle shows its on state, and View › Show Grid is checked

#### Scenario: Status bar in the Brand space
- **WHEN** the Brand space is shown
- **THEN** the status bar shows only the save state

## RENAMED Requirements

- FROM: `### Requirement: Tool bar`
- TO: `### Requirement: Tool rail`

## REMOVED Requirements

### Requirement: Right panel stack
**Reason**: The stack of eight collapsible, closable panels is replaced by containers that show each function where it is used, with no empty panel.
**Migration**: Properties becomes the inspector (Inspector requirement and the properties-panel capability); Layers becomes the Layers tab (Left panel); Colors and Stroke become the Fill and Stroke rows of the inspector with their popovers (color-panel, gradients, stroke-panel); Transform becomes the inspector's Layout section (transform-panel); Styles, Symbols and Assets go to the Resources tab and the Brand space (shared-styles, symbols, assets-panel, brand-space); the Polygon settings go to the Tool options bar and the inspector's Polygon section (properties-panel). The resizable column becomes the resizable inspector, and panel collapse/close is replaced by the left tabs and Focus mode (Hide Panels). Panel layouts saved by earlier builds are ignored and the defaults apply (app-preferences).

### Requirement: Vehicles sidebar
**Reason**: The sidebar mixed the project's settings and the fleet's navigation; they move to the Project space and the Textures tab.
**Migration**: The Project section (Name, Version, editable Game versions with the versions supported by the fleet) moves to the Project space's mod information column (project-screen). The Vehicles section becomes the Textures tab (vehicle-projects) and the vehicle cards of the Project space (project-screen). Hide Sidebar, View › Sidebar (F5) and Vehicle › Vehicle Information are replaced by the spaces (Cmd/Ctrl+1/2/3, workspace-spaces) and Focus mode (Tab). The resizable, remembered width becomes the Left panel's; Reset Workspace shows the panels (Workspace frame).

### Requirement: Context menus
**Reason**: Panel headers no longer exist, so their context menu (Collapse/Expand, Close panel) goes away with them.
**Migration**: The canvas and tool context menus are kept by the Canvas and tool context menus requirement. Rows of the Layers tab, the Resources tab and the Brand space keep their own context menus (layers-panel, color-panel, shared-styles, symbols, shared-library).
