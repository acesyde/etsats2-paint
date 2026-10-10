## Why

The menu bar takes a full row of the window, above the top bar of `new-design`, for menus most players open now and then. The menus still earn their place: they are where a beginner discovers the commands and learns their shortcuts (⌘⇧K, Shift+H) without opening any documentation, so removing them would cost more than it saves. What should go is the row. The mockup puts the menus where each platform expects them, which gives the canvas about 28 px of height.

## What Changes

Follows the mockup (TruckPaint 5, artboards 03–05): one 40 px bar at the top of the window, on every platform.

- **macOS: system menu bar, top bar in the title strip.** TruckPaint's menus move to the macOS menu bar. The window keeps its native frame (traffic lights, rounded corners, tiling, full screen) and hides the title text. The top bar (project name and game badge, space switcher, Export…) is drawn in the title strip next to the traffic lights. No menu row in the window.
  - The application menu (TruckPaint) holds About, Settings… (Preferences) and Quit, as macOS apps do.
  - Items show their shortcuts, follow the commands' enabled state, toggles show their check mark, and labels follow a live language switch.
- **Windows and Linux: menus in a drawn title bar.** The window draws its own title bar, like recent code editors:
  - from left to right: the TP mark, the menus, the space switcher (centered), Export…, and the window controls (minimize, maximize/restore, close);
  - the project name becomes the window's title (taskbar), as the mockup shows no name in this bar;
  - dragging the empty part of the bar moves the window, double-clicking it maximizes or restores, and the edges and corners resize it.
- **System title bar as an option:** on Windows and Linux, Preferences offers **Use the system title bar** (off by default, applied at once): the system title bar with an in-window menu row, as today. The drawn bar is the default everywhere, Wayland included. `TRUCKPAINT_SYSTEM_TITLE_BAR=1` forces the system bar, as a rescue when the window can't be used.
- **The breadcrumb leaves the top bar.** It stays inlaid in the canvas, as in the mockup.
- **Menus:** File, Edit, Object, Layer, View, Vehicle, Help. The **Export menu goes away**: Export Texture… and Export Mod… move to the end of the File menu, before Quit. Shortcuts don't change.
- **Same commands everywhere:** each menu item stays a registered command, whatever draws it, so menus, shortcuts and the command palette keep agreeing.

**Spike done** (scratch project, macOS, eframe 0.36.2 / winit 0.30.13 / muda 0.21.2; details in design.md):
- the native menu works with `muda`;
- a shortcut bound to a menu item is taken by the menu and never reaches egui, so on macOS the menu dispatches those shortcuts;
- the predefined Copy/Undo/Quit items must not be used (they break text-field editing and skip the unsaved-changes prompt);
- decorations-off windows support drag, maximize and minimize through viewport commands. On Windows, edge resizing must be drawn by the app. On macOS the native frame is kept, with a full-size content view.

Non-goals:
- a native menu on Windows or Linux;
- changing which commands the menus hold, beyond moving the Export items and the macOS application menu;
- hiding menus behind a hamburger button;
- the Windows 11 Snap Layouts flyout on the maximize button (Win+Z and edge snapping still work).

## Capabilities

### New Capabilities
- `window-title-bar`: the one-bar top of the window: on macOS the native frame with the top bar in the title strip; on Windows and Linux the drawn title bar (menus, switcher, Export…, window controls, drag, double-click, resize edges); the system title bar option (Preferences, applied live; environment variable as a rescue).

### Modified Capabilities
- `workspace-spaces`: the top bar is 40 px and merged with the title bar, its content per platform, no breadcrumb in it; Export… follows File › Export Mod….
- `localization`, `mod-export`, `texture-export`: the Export items are under File.
- `app-preferences`: the Use the system title bar option (Windows/Linux), persisted and kept by Reset to defaults.
- `workspace-layout`: "Workspace frame" starts with the title bar; "Menu bar" lists File, Edit, Object, Layer, View, Vehicle, Help (no Export), says where the menus live on each platform (system menu bar on macOS, title bar on Windows and Linux, in-window row with the system title bar option), the macOS application menu, how shortcuts and text editing behave with the system menu,, and moves Export Texture… and Export Mod… to File.

## Impact

- **Depends on `new-design`,** which keeps the in-window menu bar and defines the top bar this change merges into the title bar.
- **tp-app:**
  - `main.rs`: viewport options per platform (decorations off on Windows and Linux; full-size content view on macOS);
  - `menus.rs` (already the menus as data, shared with the command palette): the Export menu removed, its items in File, the macOS application menu; `ui/menu_bar.rs` draws them in the title bar or the fallback row; a new title bar with its window controls and resize grips;
  - a `native_menu` module (macOS only): building, refreshing and dispatching the system menu to the command queue;
  - tests: menu contents and the Export items under File; title bar hit areas.
- **tp-i18n:** labels for the window controls and the application menu, in en/fr/es/de.
- **Dependencies:** `muda` 0.21 on macOS only (confirmed by the spike; it shares objc2 with eframe). No change to eframe/egui versions.
- **Risk:** the highest of the redesign's follow-ups, because it depends on the windowing layer of each platform. Release builds (`distribution`) must be tested on Windows, macOS, and Linux under X11 and Wayland.

Decided: both Export items stay in File, so every command is in a menu.
