# canvas-grid Specification

## Purpose

Defines the optional grid drawn over the artboard as a placement reference, its spacing and how it stays readable at every zoom level.

## Requirements

### Requirement: Grid overlay
View › Show Grid (⌘' on macOS, Ctrl+' elsewhere) SHALL toggle a grid drawn over the artboard and show a check mark when it is shown; the grid is hidden by default. Grid lines SHALL start at the artboard origin (0, 0) and repeat every grid spacing in texture pixels, every eighth line drawn slightly stronger. The grid SHALL be drawn above the artwork, below guides and the selection overlay, only over the artboard, and SHALL never be exported.

#### Scenario: Show the grid
- **WHEN** the grid spacing is 64 px and the user presses ⌘'
- **THEN** grid lines are drawn at x and y = 0, 64, 128, … across the artboard and View › Show Grid is checked

#### Scenario: Grid is not exported
- **WHEN** the texture is exported with the grid shown
- **THEN** the image contains no grid lines

### Requirement: Grid spacing
The grid spacing SHALL be set in Preferences (Grid spacing, texture pixels, 4 to 1024, default 64). Changing it SHALL update the grid and grid snapping immediately.

#### Scenario: Coarser grid
- **WHEN** the user sets Grid spacing to 256 in Preferences
- **THEN** grid lines are drawn every 256 texture pixels

### Requirement: Readable at any zoom
When grid lines would be closer than 8 screen points, the grid SHALL draw only every second, fourth, eighth… line (the smallest power of two that keeps lines at least 8 points apart), so it never fills the screen. Grid snapping SHALL keep using the configured spacing.

#### Scenario: Zoomed out
- **WHEN** the grid spacing is 16 px and a 4096 px artboard is shown at 10% zoom
- **THEN** visible grid lines are at least 8 screen points apart
