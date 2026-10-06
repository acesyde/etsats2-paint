# command-system Specification

## Purpose

Provides one consistent way to define, trigger, enable/disable and display every user action, so that menus, the tool bar and keyboard shortcuts always agree with each other.

## Requirements

### Requirement: Commands as the single action path
Every user action exposed in a menu, the tool bar, a context menu or a keyboard shortcut SHALL be a registered command with a stable identifier, a display label, an optional icon, an optional default shortcut and an enabled state. Triggering a command from any entry point SHALL produce the same effect.

#### Scenario: Same command from menu and shortcut
- **WHEN** the user triggers "Toggle Layers panel" from the Window/View menu and later via its keyboard shortcut
- **THEN** both toggle the same panel in the same way

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
The application SHALL provide default single-key shortcuts for tools: Selection `V`, Direct Selection `A`, Move `M`, Rectangle `R`, Ellipse `E`, Polygon `Y`, Pen `P`, Line `\`, Text `T`, Image `Shift+I`, Eyedropper `I`, Zoom `Z`, Hand `H`. Holding Space SHALL temporarily activate the Hand tool and releasing it SHALL restore the previous tool.

#### Scenario: Selecting a tool by key
- **WHEN** the canvas area has focus and the user presses `E`
- **THEN** the Ellipse tool becomes the active tool

#### Scenario: Temporary hand tool
- **WHEN** the Rectangle tool is active and the user holds Space then releases it
- **THEN** the Hand tool is active while Space is held and the Rectangle tool is active again after release

### Requirement: Shortcut conflicts detected
The command registry SHALL reject (at startup, in debug builds and tests) two enabled-by-default commands bound to the same shortcut in the same context.

#### Scenario: Duplicate binding in tests
- **WHEN** the automated test suite builds the default command registry
- **THEN** it fails if any two commands share the same shortcut in the same context
