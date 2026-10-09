# command-system Specification

## Purpose

Provides one consistent way to define, trigger, enable/disable and display every user action, so that menus, the tool bar and keyboard shortcuts always agree with each other.

## Requirements

### Requirement: Commands as the single action path
Every user action exposed in a menu, the tool bar, a context menu or a keyboard shortcut SHALL be a registered command with a stable identifier, a display label, an optional icon, an optional default shortcut and an enabled state. Triggering a command from any entry point SHALL produce the same effect.

#### Scenario: Same command from menu and shortcut
- **WHEN** the user triggers View › Layers from the View menu and later presses `2`, in the Workshop with no text field focused
- **THEN** both show the Layers tab of the left panel in the same way

### Requirement: Platform-aware shortcuts
Shortcuts SHALL use the Command key on macOS and the Control key on Linux and Windows for the primary modifier. Shortcut labels shown in menus and tooltips SHALL use the platform's notation (for example `⇧⌘Z` on macOS, `Ctrl+Shift+Z` elsewhere).

#### Scenario: Undo label on macOS
- **WHEN** the Edit menu is opened on macOS
- **THEN** the Undo item shows `⌘Z`

#### Scenario: Undo label on Windows
- **WHEN** the Edit menu is opened on Windows or Linux
- **THEN** the Undo item shows `Ctrl+Z`

### Requirement: Disabled commands
A disabled command SHALL NOT execute when triggered by any entry point, and its menu item and tool bar button SHALL be shown in the disabled style.

#### Scenario: Shortcut of a disabled command
- **WHEN** the user presses the shortcut of a command that is currently disabled
- **THEN** nothing happens and no error is shown

### Requirement: Shortcuts do not fire while typing
Single-key and modifier shortcuts SHALL NOT be triggered while a text input has keyboard focus, except for shortcuts explicitly marked as global (such as Save).

#### Scenario: Typing a tool letter in a text field
- **WHEN** a text field is focused and the user types `R`
- **THEN** the character is inserted in the field and the Rectangle tool is not selected

### Requirement: Default tool shortcuts
The application SHALL provide default single-key shortcuts for tools: Selection `V`, Direct Selection `A`, Move `M`, Rectangle `R`, Ellipse `E`, Polygon `Y`, Pen `P`, Line `\`, Text `T`, Image `Shift+I`, Eyedropper `I`, Gradient `Shift+G`, Zoom `Z`, Hand `H`. Holding Space SHALL temporarily activate the Hand tool and releasing it SHALL restore the previous tool.

#### Scenario: Selecting a tool by key
- **WHEN** the canvas area has focus and the user presses `E`
- **THEN** the Ellipse tool becomes the active tool

#### Scenario: Temporary hand tool
- **WHEN** the Rectangle tool is active and the user holds Space then releases it
- **THEN** the Hand tool is active while Space is held and the Rectangle tool is active again after release

#### Scenario: Gradient tool by key
- **WHEN** the canvas area has focus and the user presses `Shift+G`
- **THEN** the Gradient tool becomes the active tool

#### Scenario: G no longer selects the Gradient tool
- **WHEN** the Selection tool is active, the canvas area has focus and the user presses `G`
- **THEN** the Selection tool stays active and the template's visibility is toggled

### Requirement: Shortcut conflicts detected
The command registry SHALL reject (at startup, in debug builds and tests) two enabled-by-default commands bound to the same shortcut in the same context.

#### Scenario: Duplicate binding in tests
- **WHEN** the automated test suite builds the default command registry
- **THEN** it fails if any two commands share the same shortcut in the same context

### Requirement: Space, tab and panel commands
The command registry SHALL provide these commands, each listed in the View menu in this order, before the Template, Grid, Guides, Snapping and zoom commands:
- Project, Workshop and Brand, which show that space (`Cmd/Ctrl+1`, `Cmd/Ctrl+2`, `Cmd/Ctrl+3`);
- Textures, Layers and Resources, which make that tab the active tab of the Workshop's left panel (`1`, `2`, `3`);
- Hide Panels (`Tab`), which hides the left panel and the inspector of the Workshop when they are shown and shows them again when they are hidden. The tool rail, the top bar, the tool options bar and the status bar stay.

These commands SHALL be available when a project is open. The View menu SHALL mark the active space, the active tab and whether the panels are hidden.

#### Scenario: Switching space by key
- **WHEN** the Workshop is shown and the user presses `Cmd/Ctrl+3`
- **THEN** the Brand space is shown, and the selection, the zoom and the active texture are unchanged when the user presses `Cmd/Ctrl+2`

#### Scenario: Left tab by key
- **WHEN** the Workshop shows the Textures tab, no text field has focus, and the user presses `3`
- **THEN** the left panel shows the Resources tab

#### Scenario: Hide Panels
- **WHEN** the Workshop is shown with both panels and the user presses `Tab` with no text field focused
- **THEN** the left panel and the inspector are hidden while the tool rail and the status bar stay, and pressing `Tab` again shows both panels

### Requirement: Single-key workspace shortcuts respect focus
The single-key shortcuts `Tab`, `1`, `2`, `3` and `G` SHALL act only when no text field has keyboard focus, no text is being edited on the canvas, and no modal dialog is open. Otherwise the key SHALL keep its normal effect: the typed character is inserted in the field or in the edited text, and `Tab` moves keyboard focus between the controls of the dialog or panel.

#### Scenario: Typing a digit in a field
- **WHEN** the X field of the inspector has focus and the user types `2`
- **THEN** "2" is inserted in the field and the left panel's tab does not change

#### Scenario: Tab in a dialog
- **WHEN** the Export Mod dialog is open and the user presses `Tab`
- **THEN** keyboard focus moves to the next control of the dialog and the Workshop's panels are not hidden

#### Scenario: Editing text on the canvas
- **WHEN** a text object is being edited on the canvas and the user types `G`
- **THEN** "G" is inserted in the text and the template's visibility is unchanged

### Requirement: Default workspace shortcuts
The default command registry SHALL bind these shortcuts:

| Command | Shortcut |
|---|---|
| Project / Workshop / Brand | `Cmd/Ctrl+1` / `Cmd/Ctrl+2` / `Cmd/Ctrl+3` |
| Textures / Layers / Resources tab | `1` / `2` / `3` |
| Hide Panels | `Tab` |
| Actual Size | `Cmd/Ctrl+0` |
| Fit to Screen | `Shift+Cmd/Ctrl+0` |
| Bring Forward / Send Backward | `Alt+Cmd/Ctrl+]` / `Alt+Cmd/Ctrl+[` |
| Next / Previous Texture | `Cmd/Ctrl+]` / `Cmd/Ctrl+[`, with `Cmd/Ctrl+Page Down` / `Cmd/Ctrl+Page Up` as alternates |
| Show Template | `G` |
| Gradient tool | `Shift+G` |
| Export… (Export Mod) | `Cmd/Ctrl+E` |
| Export Texture… | `Shift+Cmd/Ctrl+E` |

The keys F5, F6, F7 and F8 SHALL NOT be bound: the commands View › Sidebar, the panel toggles (Colors, Layers, Properties and the other panels of the former panel stack) and Vehicle › Vehicle Information SHALL NOT exist. The Keyboard Shortcuts window SHALL list the shortcuts of this table.

#### Scenario: Next texture by key
- **WHEN** a project holds two textures, the first is active, and the user presses `Cmd/Ctrl+]`
- **THEN** the second texture becomes active and no object moves in the stacking order

#### Scenario: Export by key
- **WHEN** a project is open and the user presses `Cmd/Ctrl+E`
- **THEN** the Export Mod dialog opens

#### Scenario: Former panel keys do nothing
- **WHEN** a project is open in the Workshop and the user presses F5, F6, F7 or F8
- **THEN** nothing changes in the layout and no command runs

#### Scenario: No duplicate binding
- **WHEN** the automated test suite builds the default command registry with these shortcuts
- **THEN** no two commands share the same shortcut in the same context
