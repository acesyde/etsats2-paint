## 1. Workspace & CI foundation

- [x] 1.1 Create the Cargo workspace (`Cargo.toml` with `[workspace.dependencies]`, `crates/tp-core`, `crates/tp-ui`, `crates/tp-app`) with Rust 1.98.1 (rustfmt, clippy) pinned in `mise.toml`; verify `cargo build --workspace` succeeds
- [x] 1.2 Add a minimal `eframe` (wgpu backend) window in `tp-app` titled with the placeholder product name; verify `cargo run -p tp-app` opens a window on macOS
- [x] 1.3 Add `tracing` logging to stderr and a rolling file in the per-OS data dir; verify a log file is created on startup
- [x] 1.4 Show a clear error dialog/message and exit cleanly when no GPU adapter is available, and honor `WGPU_BACKEND`; verify by running with an invalid backend override
- [x] 1.5 Add mise tasks (`fmt`, `fmt:check`, `lint`, `test`, `build`, `run`, `screenshots`, `ci`) and `.github/workflows/ci.yml` with an ubuntu/windows/macos matrix that installs Rust via mise and runs those tasks (Linux system packages installed, rust-cache enabled, actions pinned by SHA); verify `mise run ci` passes locally and the workflow passes on all three runners
- [x] 1.6 Document build prerequisites per OS (Linux packages, `WGPU_BACKEND=gl` fallback) in `README.md`; verify instructions by following them on a clean checkout

## 2. Design system (tp-ui)

- [x] 2.1 Implement design tokens (surface elevations, text levels, accent, semantic colors, spacing, radii, strokes, type scale); verify with a unit test asserting WCAG contrast minimums (7:1 primary, 4.5:1 secondary, 3:1 disabled) for every declared pair
- [x] 2.2 Embed Inter and the Phosphor icon font, register font families and the type scale; verify UI renders with Inter and icons on a machine without extra fonts (screenshot attached to the PR)
- [x] 2.3 Implement `theme::install` mapping tokens to egui `Style`/`Visuals`/`Spacing` including focus ring and tooltip style; verify default egui widgets in a demo screen use token colors
- [x] 2.4 Implement UI scale (75–200%) and text size (85–150%) applied live on top of the OS scale factor; verify with a kittest that changing scale updates `pixels_per_point` and font sizes without restart
- [x] 2.5 Implement widgets `ToolButton` (24×24 min hit area, non-color active indicator), `PanelHeader`, `SegmentedControl`, `MenuRow` (label + right-aligned shortcut + disabled style), `EmptyState`, `IconButton` with mandatory tooltip; verify kittests for hover/active/disabled behavior and hit-area size
- [x] 2.6 Add a hidden developer "Design system" gallery window (debug builds) showing every widget in every state; verify all states are visually distinct in a grayscale screenshot

## 3. Command system

- [x] 3.1 Define `CommandId`, command metadata table (label, icon, default shortcut, context) and `is_enabled`; verify a unit test that every command has a label and that no two default shortcuts collide in the same context
- [x] 3.2 Implement the per-frame command queue and `App::dispatch`, with `commands.button` / `commands.menu_item` helpers; verify a kittest that triggering a command from a menu and from its shortcut produces the same state change
- [x] 3.3 Implement shortcut matching with the platform COMMAND modifier and platform-specific label formatting; verify unit tests for `⇧⌘Z` (macOS) vs `Ctrl+Shift+Z` (others)
- [x] 3.4 Suppress non-global shortcuts while a text input has focus; verify a kittest where typing `R` in a text field does not change the active tool
- [x] 3.5 Implement tool shortcuts (V, A, M, R, E, Y, P, `\`, T, Shift+I, I, Z, H) and Space-held temporary Hand tool; verify kittests for tool switching and Space press/release restoring the previous tool

## 4. Preferences

- [x] 4.1 Implement versioned `Prefs` (UI scale, text size, panel layout, 3D panel, view mode, recent projects) saved as RON in the per-OS config dir with atomic write and debounced saving; verify a unit test round-trips prefs through a temp dir
- [x] 4.2 Handle missing/invalid/unknown-version prefs by falling back to defaults and keeping a timestamped backup; verify a unit test with a corrupted file
- [x] 4.3 Restore window size/position/maximized state and clamp to a connected monitor; verify manually by moving the window off-screen in the saved state and relaunching
- [x] 4.4 Implement the Preferences dialog (Edit > Preferences, `⌘,`/`Ctrl+,`) with live UI scale/text size and "Reset to defaults"; verify a kittest that reset returns both values to 100%

## 5. Start screen

- [x] 5.1 Implement the Home screen (New Project, Open Project, Recent Projects) shown on launch and when the last project closes; verify a kittest for both cases
- [x] 5.2 Implement the Recent Projects list with empty state, unavailable entries (non-color indicator) and "Remove from list"; verify kittests with an empty list and with a missing file entry
- [x] 5.3 Disable Open Project and recent entries with an explanatory tooltip until persistence exists; verify a kittest that clicking does nothing
- [x] 5.4 Implement the step-based New Project wizard with step indicator and the "Name & resolution" step (2048/4096/8192, default 4096, empty name → "Untitled"), keyboard operable (Tab/Enter/Escape); verify kittests for create, cancel and empty name

## 6. Workspace layout

- [x] 6.1 Implement the screen state machine (Home ↔ Workspace) and window title with project name; verify File > Close returns to Home in a kittest
- [x] 6.2 Implement the menu bar (File, Edit, Object, Layer, View, Vehicle, Export, Help) built from commands, with shortcuts and disabled not-yet-available items; verify a kittest asserting menu order and a disabled item's behavior
- [x] 6.3 Implement the left tool bar with the 13 tools in spec order, single active tool and Selection as default; verify a kittest that clicking Rectangle makes it the only active tool
- [x] 6.4 Implement the canvas area placeholder: pasteboard, centered fitted artboard with shadow, fit zoom percentage and pointer position in texture pixels; verify a kittest that resizing keeps the artboard fully visible and coordinates map correctly at corners
- [x] 6.5 Implement the right panel stack (Properties, Layers, Colors, Stroke, Transform, Assets, Vehicle) with collapse, close, reopen via View menu, resizable clamped column, scrolling and empty states; verify kittests for collapse, close/reopen at same position, and width clamping
- [x] 6.6 Implement the 3D preview placeholder panel (show/hide/resize) and view modes 2D / 3D / Split with segmented control, View menu shortcuts and draggable split divider; verify kittests for each mode's visible regions
- [x] 6.7 Implement the status bar (zoom, pointer position, active surface, save state with icon + text "Saved"/"Unsaved changes"); verify a kittest that a new project shows "Unsaved changes"
- [x] 6.8 Implement context menus for canvas area, panel headers and tools; verify a kittest that right-clicking the Layers header offers Collapse/Expand and Close panel
- [x] 6.9 Implement View > Reset Workspace restoring default panel order, visibility and sizes; verify a kittest after closing and resizing panels

## 7. Integration & polish

- [x] 7.1 Keyboard navigation pass: focus rings visible, Tab order sensible in dialogs and Home screen; verify manually and with a kittest on the New Project dialog
- [x] 7.2 Visual QA pass on macOS, Windows and Linux (X11 and Wayland) at 100% and 200% scale; verify by attaching screenshots of Home, New Project and Workspace per platform to the PR — accepted without a Windows/Linux visual check (no machine available); macOS checked at 100% and 200%
- [x] 7.3 Run `openspec validate app-shell --strict` and the full CI matrix; verify both pass
