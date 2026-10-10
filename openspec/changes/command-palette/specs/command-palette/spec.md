## Purpose

Lets the player reach every command and every texture of the project from the keyboard, by typing part of its name, without knowing which menu, space or panel it lives in.

## ADDED Requirements

### Requirement: Opening and closing the palette
The application SHALL offer a command palette: a search field with a list of results, drawn over the window. It SHALL open with `Cmd+K` on macOS and `Ctrl+K` on Windows and Linux, with View › Command Palette…, and from the Search textures field of the Textures tab (see "Searching textures only"). It SHALL open on the home screen and with a project open, in every space, including while a text field has keyboard focus, while a text is edited on the canvas and while a pen path is drawn. It SHALL NOT open while a modal dialog is shown, nor during a canvas drag.

The palette SHALL open with an empty query and the keyboard focus in its field. It SHALL close when the user presses Escape, presses `Cmd/Ctrl+K` again, presses outside it, runs a command or opens a texture from it.

#### Scenario: Opening from the Workshop
- **WHEN** a project is open in the Workshop and the user presses `Cmd/Ctrl+K`
- **THEN** the palette is shown with an empty field that has the keyboard focus

#### Scenario: Opening while typing in a field
- **WHEN** the X field of the inspector has focus and the user presses `Cmd/Ctrl+K`
- **THEN** the palette opens, and the X field's value is unchanged

#### Scenario: Opening on the home screen
- **WHEN** no project is open and the user presses `Cmd/Ctrl+K`
- **THEN** the palette opens and lists commands only

#### Scenario: Not over a dialog
- **WHEN** the Export Mod dialog is open and the user presses `Cmd/Ctrl+K`
- **THEN** the palette does not open and the dialog stays as it was

#### Scenario: Toggling with the shortcut
- **WHEN** the palette is open and the user presses `Cmd/Ctrl+K`
- **THEN** the palette closes

#### Scenario: Closing by a press outside
- **WHEN** the palette is open and the user presses on the canvas outside it
- **THEN** the palette closes, and nothing is selected or drawn by that press

### Requirement: Commands in the palette
The palette SHALL list every command of the command registry, except the commands that only make sense as a held or repeated key (Nudge, in every direction and with Shift) and the Command Palette command itself. A command that the menus only show in some builds (the design system gallery) SHALL be listed only in those builds.

Each command row SHALL show:
- its icon when it has one;
- its label in the interface language, as its menu item reads it (Undo and Redo name the operation: "Undo Move");
- its context, before the label and dimmed: its menu path ("Object › Align"), or for a command outside the menus, its group: **Tools** for the tools, **Colors** for Switch Fill/Stroke Target, Swap Fill and Stroke and Default Colors, **Symbol** for Done Editing Symbol;
- its shortcut in the platform's notation, at its right end, in the monospaced font;
- for a toggle (Show Template, Show Grid, Show Guides, Snapping, Hide Panels, the spaces and the left tabs), a check mark when it is on, as in the View menu.

A command that is disabled in the current state SHALL be shown in the disabled style and SHALL NOT run when chosen. The reason it is disabled (the explanation its menu item gives) SHALL be shown when its row is highlighted and when it is hovered, and SHALL be part of the row's name for assistive technologies.

#### Scenario: A command with its shortcut and path
- **WHEN** an object is selected in the Workshop and the user types "flip hor" in the palette on macOS in English
- **THEN** a row reads "Object › Flip Horizontal" with the shortcut "⇧H"

#### Scenario: A disabled command
- **WHEN** nothing is selected and the user types "group" in the palette and highlights the Group row
- **THEN** the row is shown disabled, the palette shows "Select one or more objects first.", and pressing Enter leaves the palette open and runs nothing

#### Scenario: Gesture commands left out
- **WHEN** the user types "nudge" in the palette
- **THEN** no Nudge command is listed

#### Scenario: A toggle shows its state
- **WHEN** the grid is shown and the user types "grid" in the palette
- **THEN** the View › Show Grid row shows a check mark

#### Scenario: A tool outside the menus
- **WHEN** the user types "ellipse" in the palette
- **THEN** a row reads "Tools › Ellipse" with the shortcut "E"

### Requirement: Textures in the palette
With a project open, the palette SHALL list every texture of the project, in project order. Each texture row SHALL show its context "<vehicle> › Main textures|Accessories ›", dimmed, before the texture's name (as the breadcrumb names it), its size in the monospaced font, and at its right end the marker of its state as the Textures tab draws it (see the texture-status capability): hollow ring for Empty, filled dot for Modified, warning icon in the signal color for To check. The row's name for assistive technologies SHALL end with the state. The active texture's row SHALL be marked as active without relying on color alone.

Without a project, no texture is listed.

#### Scenario: Texture rows of a fleet
- **WHEN** a project holds the sample truck and the sample trailer and the user types "mudfl" in the palette
- **THEN** a row reads "TruckPaint Sample Trailer › Accessories › Mudflaps" with its size and the hollow ring of an Empty texture

#### Scenario: Flagged texture
- **WHEN** Update Template flagged the Standard cab texture "Layout changed" and the user types "standard" in the palette
- **THEN** the Standard cab row shows the warning icon in the signal color, and its accessible name ends with "To check"

### Requirement: Searching
The palette SHALL filter its entries as the user types, matching the query against each entry's label and context in the interface language. The search SHALL ignore case and accents (é matches e, ß matches ss, œ matches oe). The query SHALL be split into words at spaces; an entry matches when every word is found in its label or context with its letters in order, not necessarily next to each other ("flp" matches "Flip"), the words in any order.

Matches SHALL be ordered best first: a word matching the start of a word in the label ranks above one matching inside a word; letters matched next to each other rank above scattered ones; a match in the label ranks above a match in the context only; a shorter label ranks above a longer one. Entries with the same score keep their list order (textures in project order, commands in menu order). Enabled commands and textures SHALL come before disabled commands. The letters of the label that match SHALL be highlighted.

When nothing matches, the palette SHALL say that no command or texture matches the query.

#### Scenario: Letters in order
- **WHEN** the language is English and the user types "flp hor"
- **THEN** Flip Horizontal is the first result

#### Scenario: Case and accents ignored
- **WHEN** the language is French and the user types "tout selectionner"
- **THEN** "Édition › Tout sélectionner" is listed

#### Scenario: Words in any order
- **WHEN** the language is English and the user types "horizontal flip"
- **THEN** Flip Horizontal is listed

#### Scenario: Match in the context
- **WHEN** a project holds the sample truck and the user types "accessories chassis"
- **THEN** the Chassis texture is listed

#### Scenario: Interface language
- **WHEN** the language is German and the user types "spiegeln"
- **THEN** "Objekt › Horizontal spiegeln" and "Objekt › Vertikal spiegeln" are the first results

#### Scenario: Nothing matches
- **WHEN** the user types "zzzz"
- **THEN** the list is empty and the palette says that nothing matches "zzzz"

### Requirement: Empty query and recent commands
With an empty query, the palette SHALL list, in this order: the commands recently run from the palette, most recent first, at most five, under the heading **Recent**; then the project's textures under **Textures**; then every other command in menu order under **Commands**. A recent command SHALL appear once and SHALL NOT appear again under Commands. Recent commands SHALL be remembered for the session only, across projects, and SHALL be shown with their current enabled state. Running a command from a menu, a button or its shortcut SHALL NOT add it to the recent commands.

#### Scenario: Recent first
- **WHEN** the user runs Flip Horizontal then Show Grid from the palette, then opens it again
- **THEN** the list starts with a Recent heading over Show Grid then Flip Horizontal, followed by the Textures heading

#### Scenario: First use
- **WHEN** the application has just started, a project is open, and the user opens the palette
- **THEN** it lists the Textures heading first, then the Commands heading

#### Scenario: Not kept across sessions
- **WHEN** the user runs Show Grid from the palette, quits, restarts and opens the palette
- **THEN** no Recent heading is shown

### Requirement: Searching textures only
The Textures tab's Search textures field (see the vehicle-projects capability) SHALL open the palette limited to the project's textures: its placeholder reads "Search textures", and only textures are listed and searched. Pressing Backspace in its empty field SHALL lift the limit, so that commands are listed and searched too, with the usual placeholder. Opening the palette with its shortcut or the View menu SHALL never limit it.

#### Scenario: Opening from the Textures tab
- **WHEN** the user clicks the Search textures field of the Textures tab and types "ch"
- **THEN** the palette lists the textures matching "ch", such as Chassis, and no command

#### Scenario: Widening to commands
- **WHEN** the palette was opened from the Search textures field with an empty query and the user presses Backspace, then types "grid"
- **THEN** the Show Grid command is listed

### Requirement: Running a command or opening a texture
Choosing an enabled command, with Enter on its highlighted row or a click on it, SHALL close the palette and then run the command exactly as its menu item does, including the undo step and its label. Choosing a texture SHALL close the palette, make that texture active as clicking it in the Textures tab does (the selection is cleared, each texture keeps its own view, a symbol being edited is left), and show the Workshop when another space is shown.

#### Scenario: Flip from the palette
- **WHEN** a rectangle is selected, the user opens the palette, types "flip hor" and presses Enter
- **THEN** the palette closes, the rectangle is mirrored, and the Edit menu offers "Undo Flip Horizontal"

#### Scenario: Opening a texture from the Project space
- **WHEN** the Project space is shown, the user opens the palette, types "mudflaps" and presses Enter
- **THEN** the Workshop is shown with the Mudflaps texture active and highlighted in the Textures tab

#### Scenario: Command that opens a dialog
- **WHEN** a project is open and the user chooses Export… (Export Mod) in the palette
- **THEN** the palette closes and the Export Mod dialog opens

#### Scenario: Space command
- **WHEN** the Workshop is shown and the user chooses Brand in the palette
- **THEN** the Brand space is shown, with the selection and the active texture unchanged

### Requirement: Keyboard and mouse use
While the palette is open:
- the first result SHALL be highlighted after every change of the query;
- Down and Up SHALL move the highlight to the next and previous result, skipping headings and stopping at the ends; Page Down and Page Up SHALL move it by one visible page; Home and End SHALL NOT move it (they move the caret in the field);
- the list SHALL scroll to keep the highlighted row visible;
- Enter SHALL choose the highlighted result; Escape SHALL close the palette;
- Tab SHALL keep the keyboard focus in the field;
- hovering a row SHALL highlight it, and clicking it SHALL choose it;
- the keys SHALL NOT reach the canvas or the panels: arrows do not nudge, Enter does not edit a text, Escape does not deselect, and single-key shortcuts type into the field.

Global shortcuts other than `Cmd/Ctrl+K` (such as Save) SHALL close the palette and run as usual.

#### Scenario: Moving with the arrows
- **WHEN** the palette lists several results and the user presses Down twice then Enter
- **THEN** the third result is chosen

#### Scenario: Arrows don't nudge
- **WHEN** a rectangle is selected and the user opens the palette and presses Down
- **THEN** the highlight moves and the rectangle does not move

#### Scenario: Letters type in the field
- **WHEN** the palette is open and the user types "r"
- **THEN** "r" is in the field and the active tool is unchanged

#### Scenario: Save while the palette is open
- **WHEN** a project with unsaved changes is open, the palette is open and the user presses `Cmd/Ctrl+S`
- **THEN** the palette closes and the project is saved

### Requirement: The palette leaves the work unchanged
Opening the palette, typing in it, moving its highlight and closing it without choosing SHALL NOT change the document, the undo history, the selection, the active tool, the active texture, the space shown or the view. A text being edited on the canvas or a pen path being drawn SHALL still be edited after the palette is closed without choosing; choosing a command then treats them as its menu item does.

#### Scenario: Escape keeps the selection
- **WHEN** a rectangle is selected with the Rectangle tool active and the user opens the palette, types "chassis" and presses Escape
- **THEN** the rectangle is still selected, the Rectangle tool is still active, and the active texture is unchanged

#### Scenario: Text editing kept
- **WHEN** a text is being edited on the canvas and the user opens the palette and presses Escape
- **THEN** the text is still being edited, with its caret where it was

### Requirement: Palette appearance
The palette SHALL be drawn in the design system (see the ui-design-system capability): a panel of the control surface with a strong border, rounded corners and a shadow, centered horizontally near the top of the window, at most 560 points wide and narrower on a narrow window; its field at the top with a search icon and the placeholder "Search commands and textures"; the results below, at most ten rows tall before scrolling, each row at least the minimum hit target high; headings in small capitals; the highlighted row filled with the selection color; shortcuts and sizes in the monospaced font and the muted text color. A footer SHALL show the keys (↑↓ to move, Enter to run, Esc to close), or the reason when the highlighted command is disabled.

The palette's field, rows, headings and footer SHALL be translated in English, French, Spanish and German, and SHALL show whole in German and with texts 40% longer (see the localization capability). The field SHALL be named for assistive technologies, and each row SHALL be named by its context, label and, for a disabled command, its reason, or for a texture, its state.

#### Scenario: German palette
- **WHEN** the language is German and the user opens the palette in the Workshop of a fleet
- **THEN** the placeholder, the headings and the footer are in German, and no label is clipped

#### Scenario: Narrow window
- **WHEN** the window is 480 points wide and the user opens the palette
- **THEN** the palette fits in the window with a margin on each side
