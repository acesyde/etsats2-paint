# app-preferences Specification

## Purpose

Remembers the user's interface preferences and workspace layout between sessions, stored in the platform's standard configuration location on Linux, Windows and macOS.

## Requirements

### Requirement: Persisted preferences
The application SHALL persist, in the operating system's standard per-user configuration directory, at least: UI scale, text size, panel visibility / collapsed state / order, panel column width, 3D panel visibility and size, view mode, window size/position/maximized state, the recent projects list, whether the grid and guides are shown, whether snapping is on, and the grid spacing.

#### Scenario: Preferences survive restart
- **WHEN** the user sets the UI scale to 125%, closes the Assets panel, quits and restarts the application
- **THEN** the UI scale is 125% and the Assets panel is closed

#### Scenario: Grid and snapping settings survive restart
- **WHEN** the user shows the grid, sets the grid spacing to 128, turns snapping off, quits and restarts
- **THEN** the grid is shown with a 128 px spacing and snapping is off

### Requirement: Preferences dialog
The application SHALL provide a Preferences dialog (Edit > Preferences, shortcut `⌘,` on macOS and `Ctrl+,` elsewhere) to change UI scale and text size with a live preview and the grid spacing (texture pixels, 4 to 1024), and a "Reset to defaults" action that restores UI scale, text size and grid spacing.

#### Scenario: Reset to defaults
- **WHEN** the user clicks "Reset to defaults" in Preferences
- **THEN** UI scale and text size return to 100% and the grid spacing to 64 px immediately

#### Scenario: Reset workspace layout
- **WHEN** the user chooses View > Reset Workspace
- **THEN** all panels are reopened, expanded and restored to their default order and sizes

### Requirement: Robust preferences loading
If the preferences file is missing, unreadable, from an unknown version or invalid, the application SHALL start with default preferences, SHALL NOT crash, and SHALL keep a backup copy of the invalid file. Saving preferences SHALL be atomic so that a crash during write never leaves a truncated file.

#### Scenario: Corrupted preferences file
- **WHEN** the preferences file contains invalid content at startup
- **THEN** the application starts with defaults and the invalid file is preserved next to it with a backup suffix

### Requirement: Window geometry restoration
The application SHALL restore the previous window size, position and maximized state, and SHALL ensure the window is visible on a currently connected display.

#### Scenario: Monitor disconnected
- **WHEN** the window was last placed on a monitor that is no longer connected
- **THEN** the window opens on a connected display, fully visible
