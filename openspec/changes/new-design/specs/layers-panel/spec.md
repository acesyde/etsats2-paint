## MODIFIED Requirements

### Requirement: Tree view
The Layers panel SHALL be the **Layers** tab of the Workshop's left panel (key 2, see the workspace-layout capability). The tab SHALL have a header reading "<texture> · N layers", where N is the number of top-level objects of the active surface, with an **add** button that runs New Layer. Below it, the tab SHALL list the active surface's objects as a tree, topmost first, with groups expandable and collapsible and their children indented. Each row SHALL show a kind icon, the name, a visibility toggle and a lock toggle; hidden and locked states SHALL be shown by icon shape and by dimming the name, not only by color. An empty surface SHALL show an empty state in the list, under the header.

#### Scenario: Group shown as a tree
- **WHEN** the surface contains a group "Branding" with "Logo" and "Company name", above a rectangle "Background"
- **THEN** the tab shows "Branding" first with its two children indented below it, then "Background"

#### Scenario: Tab header
- **WHEN** the active texture "Side skirts" has five top-level objects and the user opens the Layers tab
- **THEN** the header reads "Side skirts · 5 layers", and clicking its add button adds an empty group "Layer N" at the top of the tree

### Requirement: Rename
Double-clicking a name or choosing Rename from the context menu SHALL edit the name inline; Enter commits, Escape cancels; empty names revert to the default name for the kind ("Rectangle", "Ellipse", "Group").

#### Scenario: Rename a layer
- **WHEN** the user double-clicks "Rectangle", types "Burgundy base" and presses Enter
- **THEN** the row and the inspector header show "Burgundy base"

### Requirement: Group and ungroup
Group (Cmd/Ctrl+G) SHALL wrap the selected objects in a new group named "Group" placed at the position of the topmost selected object, keeping their stacking order, and select the group. Ungroup (Cmd/Ctrl+Shift+G) SHALL replace each selected group by its children at the same position and select those children. New Layer (Cmd/Ctrl+Shift+N) SHALL add an empty top-level group named "Layer N" at the top and select it; Duplicate Layer and Delete Layer act like Duplicate and Delete on the selected rows. These commands SHALL also be available from the Layers tab's context menu and footer buttons, and New Layer from the add button of its header.

#### Scenario: Group then ungroup
- **WHEN** two rectangles are selected and the user presses Cmd/Ctrl+G and then Cmd/Ctrl+Shift+G
- **THEN** after grouping, one group containing both is selected; after ungrouping, the two rectangles are back at the same stacking position and selected

#### Scenario: New layer
- **WHEN** the user presses Cmd/Ctrl+Shift+N in a project without groups
- **THEN** an empty group "Layer 1" appears at the top of the tree and is selected
