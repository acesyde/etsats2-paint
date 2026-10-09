## Why

The menu bar takes a full row of the window, above the top bar of `new-design`, for menus most players open now and then. The menus still earn their place: they are where a beginner discovers the commands and learns their shortcuts (⌘⇧K, Shift+H) without opening any documentation, so removing them would cost more than it saves. What should go is the row. The mockup puts the menus where each platform expects them, which gives the canvas about 28 px of height.

## What Changes

- **macOS: the system menu bar.** TruckPaint's menus move to the macOS menu bar at the top of the screen, and the window shows no menu row. The application menu (TruckPaint) holds About, Preferences… and Quit, as macOS apps do. Items show their shortcuts, follow the commands' enabled state, toggles show their check mark, and labels follow a live language switch.
- **Windows and Linux: menus in a custom title bar.** The window draws its own title bar, like recent code editors: the app mark, the menus, the content of `new-design`'s top bar (space switcher, Export…), and the window controls (minimize, maximize/restore, close). Dragging the empty part of the bar moves the window, double-clicking it maximizes or restores, and the window's edges and corners resize it. The menu row is gone.
- **Menus:** File, Edit, Object, Layer, View, Vehicle, Help, as in the mockup. The **Export menu goes away**: Export Texture… and Export Mod… move to the end of the File menu (File › Export Texture…, File › Export Mod…), next to the Export… button `new-design` puts in the top bar. Shortcuts don't change.
- **Same commands everywhere:** each menu item stays a registered command, whatever draws it (system menu or title bar), so menus, shortcuts and the command palette keep agreeing.
- **Starts with a spike.** Neither half is free with eframe 0.36:
  - egui has no native macOS menu bar. A crate such as `muda` would build an NSMenu; to be checked: installing it alongside winit's own default menu, key equivalents that could swallow or double the shortcuts egui already handles, refreshing enabled states and labels without rebuilding the menu every frame, and the menu's events reaching the egui frame.
  - A custom title bar means `with_decorations(false)` and driving the window through viewport commands (`StartDrag`, `BeginResize`, `Maximized`, `Minimized`). To be checked: Windows snap layouts and the window shadow and border without native decorations; Wayland (client-side decorations, compositors that refuse moves or resizes) versus X11; resize edges on both; window geometry restore.

  The spike's result is recorded in design.md. **It may conclude to keep the in-window menu bar on some platform** (for example Linux under Wayland, or everywhere if the native macOS menu proves fragile); the change then ships only where it works and the fallback keeps today's behavior.

Non-goals:
- a native menu on Windows or Linux;
- changing which commands the menus hold, beyond moving the Export items;
- hiding menus behind a hamburger button.

## Capabilities

### New Capabilities
- `window-title-bar`: the custom title bar on Windows and Linux: what it shows, the window controls, drag to move, double-click to maximize, resize edges and corners, and the fallback to system decorations where it isn't supported.

### Modified Capabilities
- `workspace-layout`: "Menu bar" lists File, Edit, Object, Layer, View, Vehicle, Help (no Export), says where the menus live on each platform (system menu bar on macOS, title bar on Windows and Linux, in-window row where the spike keeps it), and moves Export Texture… and Export Mod… to File.

## Impact

- **Depends on `new-design`,** which keeps the in-window menu bar and defines the top bar this change merges into the title bar.
- **tp-app:**
  - `main.rs`: viewport options per platform (decorations off on Windows and Linux; on macOS, the native menu installed once the app is created);
  - `ui/menu_bar.rs`: the menu structure becomes data that both the title bar and the native menu build from (`MENUS` loses `menu-export`); the title bar and its window controls;
  - a `native_menu` module (macOS only): building, refreshing and dispatching the system menu to the command queue;
  - tests: menu contents and the Export items under File; title bar hit areas.
- **tp-i18n:** labels for the window controls and the application menu, in en/fr/es/de.
- **Dependencies:** probably `muda` on macOS only; to be confirmed by the spike. No change to eframe/egui versions.
- **Risk:** the highest of the redesign's follow-ups, because it depends on the windowing layer of each platform. Release builds (`distribution`) must be tested on Windows, macOS, and Linux under X11 and Wayland.

Open question: whether Export… in the top bar is enough and the File menu keeps only Export Mod…; this proposal keeps both Export items in File so every command stays in a menu.
