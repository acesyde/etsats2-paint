## MODIFIED Requirements

### Requirement: Commands as the single action path
Every user action exposed in a menu, the tool bar, a context menu, the command palette (see the command-palette capability) or a keyboard shortcut SHALL be a registered command with a stable identifier, a display label, an optional icon, an optional default shortcut and an enabled state. Triggering a command from any entry point SHALL produce the same effect.

#### Scenario: Same command from menu and shortcut
- **WHEN** the user triggers View › Layers from the View menu and later presses `2`, in the Workshop with no text field focused
- **THEN** both show the Layers tab of the left panel in the same way

#### Scenario: Same command from the palette
- **WHEN** a rectangle is selected and the user chooses Flip Horizontal in the command palette, then undoes it and chooses Object › Flip Horizontal from the menu
- **THEN** both mirror the rectangle in the same way, as one undo step named "Flip Horizontal"

### Requirement: Disabled commands
A disabled command SHALL NOT execute when triggered by any entry point, and its menu item, tool bar button and command palette row SHALL be shown in the disabled style.

#### Scenario: Shortcut of a disabled command
- **WHEN** the user presses the shortcut of a command that is currently disabled
- **THEN** nothing happens and no error is shown

#### Scenario: Disabled command in the palette
- **WHEN** nothing is selected and the user highlights Delete in the command palette and presses Enter
- **THEN** nothing is deleted and the palette stays open

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
| Command Palette | `Cmd/Ctrl+K` |

Command Palette's shortcut SHALL be global: it acts on every screen, with or without a project, and while a text field has keyboard focus.

The keys F5, F6, F7 and F8 SHALL NOT be bound: the commands View › Sidebar, the panel toggles (Colors, Layers, Properties and the other panels of the former panel stack) and Vehicle › Vehicle Information SHALL NOT exist. The Keyboard Shortcuts window SHALL list the shortcuts of this table.

#### Scenario: Next texture by key
- **WHEN** a project holds two textures, the first is active, and the user presses `Cmd/Ctrl+]`
- **THEN** the second texture becomes active and no object moves in the stacking order

#### Scenario: Export by key
- **WHEN** a project is open and the user presses `Cmd/Ctrl+E`
- **THEN** the Export Mod dialog opens

#### Scenario: Command palette by key
- **WHEN** the Opacity field of the inspector has focus and the user presses `Cmd/Ctrl+K`
- **THEN** the command palette opens

#### Scenario: Command palette in the shortcuts window
- **WHEN** the user opens Help › Keyboard Shortcuts
- **THEN** it lists Command Palette with `⌘K` on macOS and `Ctrl+K` on Windows and Linux

#### Scenario: Former panel keys do nothing
- **WHEN** a project is open in the Workshop and the user presses F5, F6, F7 or F8
- **THEN** nothing changes in the layout and no command runs

#### Scenario: No duplicate binding
- **WHEN** the automated test suite builds the default command registry with these shortcuts
- **THEN** no two commands share the same shortcut in the same context
