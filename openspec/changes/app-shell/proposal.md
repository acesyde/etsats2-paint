## Why

We are building a dedicated livery editor for Euro Truck Simulator 2 and American Truck Simulator, intended to feel like a professional graphic design tool (Illustrator / Photoshop / Affinity / Figma class) rather than a developer prototype. The repository is empty: before any canvas, vector or vehicle work can land, we need a running desktop application on Linux, Windows and macOS whose shell already looks and behaves like a finished product, plus the build/CI foundation that keeps it running on all three platforms at every subsequent step.

This is the first of a sequence of changes (app-shell → canvas-core → editing-panels → vehicle-library → persistence → export-and-preview → mod-export → distribution). Every later change plugs into the shell, theme, command system and panel layout defined here.

## What Changes

- Create a Cargo workspace (`crates/tp-core`, `crates/tp-ui`, `crates/tp-app`) with an `eframe` (wgpu backend) desktop application targeting Linux (X11 + Wayland), Windows and macOS. WebAssembly is explicitly out of scope for now, but core crates stay free of direct filesystem/thread usage so the door remains open.
- Introduce a custom design system on top of egui: dark theme with design tokens (colors, spacing, radii, typography), embedded UI font, icon font, and a kit of professional widgets (tool buttons, panel headers, numeric fields, menu items with shortcuts, tooltips) with distinct hover / active / disabled / focused states that never rely on color alone.
- Add a start screen (Adobe-style "Home"): New Project, Open Project, Recent Projects (with an empty state), and a New Project dialog that creates a blank document at a chosen texture resolution (2048 / 4096 / 8192). Vehicle selection steps are added later by `vehicle-library`.
- Add the editor workspace layout: menu bar (File, Edit, Object, Layer, View, Vehicle, Export, Help) showing platform-correct shortcuts, left tool bar with all planned tools and a clearly identifiable active tool, central canvas area, right-hand stack of collapsible / closable / resizable panels (Properties, Layers, Colors, Stroke, Transform, Assets, Vehicle), a hideable/resizable 3D preview panel placeholder, a 2D / 3D / Split view-mode switch, and a status bar (zoom, cursor position, active surface, save state).
- Introduce a central command registry: every user action (menu item, tool, shortcut, button) is a command with an id, label, icon, shortcut and enabled state; shortcuts use Cmd on macOS and Ctrl elsewhere.
- Add user preferences persisted to the platform config directory: UI scale, text size, panel visibility/sizes, window geometry, recent projects list.
- Add mise tasks for formatting, linting, testing and building, and GitHub Actions CI with a Linux / Windows / macOS matrix that installs the toolchain with mise and runs those same tasks.

Not in this change: canvas navigation, shapes, selection, undo, vehicles, templates, persistence of documents, export, 3D rendering. Menu entries and panels for these exist but are disabled or show an explicit empty state.

## Capabilities

### New Capabilities

- `ui-design-system`: theme tokens, typography, icons, widget states, accessibility rules (contrast, non-color state indication, minimum hit sizes, tooltips) and UI/text scaling.
- `command-system`: registry of user commands, keyboard shortcut handling and platform-aware shortcut display, enabled/disabled state, single dispatch path for menus, toolbar and keyboard.
- `start-screen`: home screen with New / Open / Recent, and the New Project dialog (resolution choice) that opens the editor workspace.
- `workspace-layout`: menu bar, tool bar and active tool, canvas area, right panel stack, 3D preview panel placeholder, view modes, status bar and save-state indicator.
- `app-preferences`: persisted user preferences (UI scale, text size, layout, window geometry, recent projects) and their defaults/recovery.

### Modified Capabilities

(none — no existing specs)

## Impact

- New code: entire `crates/` workspace, `Cargo.toml`, `mise.toml` Rust toolchain and build tasks (Rust 1.98.1), `.github/workflows/ci.yml`, embedded assets under `assets/` (UI font, icon font, app icon).
- New dependencies (indicative): `eframe`/`egui` (wgpu), `egui-phosphor` (icons), `serde`/`serde_json` or `ron`, `directories`, `rfd`, `tracing`/`tracing-subscriber`.
- Linux build requires system packages for winit/wgpu and file dialogs (documented in README and installed in CI).
- Open question carried forward, not blocking this change: the source of 3D vehicle models (provided glTF/OBJ, read from the game install, or generic proxy) is **to be determined**; this change only reserves the 3D panel and view modes.
