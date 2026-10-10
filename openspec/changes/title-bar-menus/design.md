## Context

**Today:**
- `main.rs` builds one decorated window (1440×900, min 960×600, `persist_window`, multisampling 4).
- Every screen draws a 28 px menu row (`ui/menu_bar.rs`, egui `MenuBar`) from `menus::menus()`. `ui/home.rs` and `ui/workspace/mod.rs` call `menu_bar::show`, then the workspace draws the 44 px top bar (`ui/workspace/top_bar.rs`: name and badge, switcher, breadcrumb, Export…).
- `menus.rs` already holds the menus as data (`Menu`, `Entry`), shared with the command palette (`catalog()` reads menu paths from it).
- **Commands:**
  - a click goes through `CommandUi::menu_item`/`menu_toggle` (`ui/command_ui.rs`), with enabled state from `CommandUi::enabled`;
  - shortcuts are matched on egui input from `CommandId::shortcut()`;
  - Quit goes through `guard(PendingAction::Quit)`, and closing the window is guarded in `project_io.rs` (`close_requested` → `CancelClose` + save prompt);
  - `state.rs::sync_title` already sets the window title to "<name> — TruckPaint".
- **Mockup** (TruckPaint 5):
  - macOS (artboards 03/04): a single 40 px bar with traffic lights, name and badge, switcher, Export…;
  - Windows/Linux (artboard 05): a single 40 px bar with the TP mark (22 px), the menus (13 px, padding 5×8), the switcher centered, Export…, and three 44 px window controls;
  - no breadcrumb in either bar.

**Spike** (scratchpad `spike-titlebar/`, macOS; eframe/egui 0.36.2, winit 0.30.13, muda 0.21.2):
- **Verified, muda menu:**
  - `Menu::init_for_nsapp()` in the eframe creator closure replaces winit's default menu;
  - `MenuEvent::set_event_handler` runs on the main thread, and must call `request_repaint()` (a menu click produces no winit event);
  - `set_enabled`/`set_checked` cost about 25 ns in release; `set_text` is the expensive call, and a full rebuild takes 2–9 ms.
- **Verified, shortcuts:**
  - a key bound to an item is taken by the menu: egui sees only the key release, and nothing runs twice;
  - a disabled item still takes its key, and nothing runs;
  - keys with no menu item reach egui;
  - predefined Copy/Undo items swallow Cmd+C/Cmd+Z, so egui text fields lose copy and undo;
  - predefined Quit sends `terminate:` and skips `close_requested`, so the unsaved-changes guard is bypassed.
- **Verified, window commands:**
  - `ViewportCommand::Maximized`/`Minimized` work;
  - `BeginResize` is unsupported on macOS;
  - `with_fullsize_content_view(true)` with the title and title bar hidden keeps the native traffic lights, corners, tiling and shadow.
- **Inferred from winit source:**
  - Windows undecorated: Aero snap works through `drag_window`, egui-winit adds the undecorated shadow, there is no `WM_NCHITTEST`, so edge resizing needs `BeginResize`, and there is no Snap Layouts flyout;
  - Wayland: `drag_window`/`drag_resize_window` map to xdg move/resize, with no shadow and no resize margin;
  - X11: `_NET_WM_MOVERESIZE`.

## Goals / Non-Goals

**Goals:**
- One 40 px bar at the top on every platform, as in the mockup. The menu row is gone, except with the system title bar option.
- One source for menus (`menus.rs`), drawn by egui on Windows/Linux and by muda on macOS. Every shortcut runs its command once, and text editing keeps its shortcuts.
- The system frame as an option the player can turn on (Preferences, applied live), plus an environment variable as a rescue.

**Non-Goals:**
- Native menus on Windows/Linux.
- Snap Layouts flyout.
- Client-side shadows on Wayland.
- Moving the traffic lights to center them in the 40 px bar: the native position is kept. It is a later polish if needed.

## Decisions

### 1. Title bar mode: platform, preference and environment

```rust
pub enum TitleBarMode { MacNative, Drawn, System }
```

- `AppState::title_bar_mode()` is computed each frame from:
  - the OS: `MacNative` on macOS, always;
  - the preference `prefs.system_title_bar: bool` (Windows/Linux, default `false`, persisted, untouched by Reset to defaults);
  - the environment variable `TRUCKPAINT_SYSTEM_TITLE_BAR=1`, read once at startup into `AppState.force_system_title_bar`.

  The result is `System` when the preference or the variable says so, and `Drawn` otherwise.
- There is no Wayland special case: the drawn bar is the default there too. GNOME draws no server-side decorations, and our winit build lacks `wayland-csd-adwaita`, so today's GNOME frame is winit's bare fallback anyway. `drag_window`/`drag_resize_window` map to xdg move/resize.
- `main.rs` sets the initial viewport from the mode at startup:
  - `MacNative`: `with_fullsize_content_view(true).with_title_shown(false).with_titlebar_shown(false)`;
  - `Drawn`: `with_decorations(false)`;
  - `System`: as today.
- **Live switch:** when the mode changes between `Drawn` and `System` (the preference toggled in Preferences), `state.rs` sends `ViewportCommand::Decorations(mode == System)` once, and the layout follows the mode on the same frame. The window's size and position are kept by the window system. This is checked manually on Windows, X11 and Wayland (task 4.3).
- Tests set the preference (or the forced flag), so kittest can render `Drawn` and `System` on any OS.
- A pure function `title_bar_mode(mac, pref, forced)` is unit-tested.

### 2. Menus data: Export moves to File, platform variants

- `menus::menus()`:
  - drops `menu-export`;
  - File gains `Separator, Item(ExportTexture), Item(ExportMod)` before `Separator, Item(Quit)`;
  - `menu_bar::MENUS` loses `menu-export`.
- `menus::menus_for(mac: bool)`. On macOS it removes Quit from File, Preferences (and its separator) from Edit, and About (and its separator) from Help, and returns them in a leading `app_menu()` titled `APP_NAME`.
- `catalog()` uses the same `menus_for(cfg!(target_os = "macos"))`, so the palette path of Preferences on macOS reads "TruckPaint".
- The menu-row code (`menu_bar::show`) is unchanged apart from the list. It is drawn inside the title bar (`Drawn`) or as its row (`System`).

### 3. macOS: `native_menu` module with muda, shortcuts re-injected as input

`crates/tp-app/src/native_menu.rs` (`#[cfg(target_os = "macos")]`; dependency `muda = "0.21"` under `[target.'cfg(target_os = "macos")'.dependencies]`).

**Build:**
- From `menus_for(true)`, add:
  - the app menu: About (custom item, opens our About dialog), separator, Settings… (`Preferences`, Cmd+,), separator, Services (predefined), separator, Hide / Hide Others / Show All (predefined), separator, Quit TruckPaint (custom item, `CommandId::Quit`, Cmd+Q);
  - the Window menu: predefined Minimize, Zoom, Fullscreen, Bring All to Front, then `set_as_windows_menu_for_nsapp`;
  - Help is marked with `set_as_help_menu_for_nsapp`.
- Each `Item`/`Toggle` becomes a `MenuItem`/`CheckMenuItem` with id `CommandId` (a stable string key) and the accelerator converted from `CommandId::shortcut()` by `accelerator(&KeyboardShortcut) -> Option<Accelerator>`, a pure function, unit-tested.
- Submenus map to `Submenu`.
- No predefined Undo/Redo/Cut/Copy/Paste/Select All/Quit items.
- Built and installed in `TruckPaintApp::new` (`init_for_nsapp`); the `Menu` is kept in `AppState`.

**Events:**
- `MenuEvent::set_event_handler(move |e| { tx.send(e); ctx.request_repaint(); })`.
- Each frame, `NativeMenu::take_events()` drains the channel.
- For a command **with a shortcut**: queue the input egui-winit would have produced for that key, a `Key { pressed: true }` + `Key { pressed: false }` with the shortcut's modifiers. For Cmd+C/X/V, it is `Event::Copy`/`Event::Cut`/`Event::Paste(clipboard text)` instead (clipboard read with `arboard`, already in the lockfile through egui-winit, added as a macOS dependency).
- These events are added in `eframe::App::raw_input_hook` at the start of the next frame.
- So the existing shortcut dispatch, the text fields and the text session see exactly what they saw before the menu existed. Whether the key or the menu item was used, the command runs once, through the code that runs today.
- For a command **without a shortcut**: queue it like `CommandUi::menu_item` does.

*Alternative considered:* dispatch the command directly and skip egui's shortcut on macOS. Rejected: it duplicates the routing rules (text session, palette open, dialogs), and copy/undo in text fields would need special cases anyway.

**State sync**, once per frame after the UI:
- `enabled` = `CommandUi::enabled(id)`. For Undo, Redo, Cut, Copy, Paste and SelectAll it is also enabled when `ctx.wants_keyboard_input()` or a text session is active, so their keys still reach the text field.
- `checked` = `menus::is_checked`.
- Only changed values are pushed (cached `HashMap<CommandId, (bool, bool)>`).
- On a language change (the existing language-changed path), the menu is rebuilt and installed again.

**Keys without Command/Control** (Backspace for Delete, Shift+H for Flip, …):
- AppKit sends key equivalents to the menu mainly for Command/Control combinations, but this must be confirmed for plain and Shift keys, especially while typing in a text field (task 3.2).
- If such keys reach the menu, they are not registered as accelerators. The shortcut is shown in the title instead, right-aligned with a tab (`"Flip Horizontal\t⇧H"`), and egui keeps dispatching them.
- Either way, typing "H" in a text field must stay text.
- **Checked (task 3.2):** with posted NSEvents and enabled items bound to them, Shift+H, Backspace, G, Escape, Tab, Enter, Alt+A and 1 all reach the window and none fires its item, with or without a text field focused; Cmd+D is taken by the menu. So these keys are registered as accelerators (shown natively), egui keeps dispatching them, and a click on their item queues the command directly. Only shortcuts with Command or Control are re-injected.

### 4. macOS bar: top bar in the title strip

- `top_bar::show` gets the mode:
  - in `MacNative` it starts after a left inset reserving the traffic lights (`eframe::WindowChromeMetrics` if 0.36 exposes it, else a 78 px constant), and is 40 px high;
  - on the home screen, `ui/home.rs` draws an empty 40 px bar (no menu row).
- Empty parts of the bar (`ui.interact` on the remaining rect, `Sense::click_and_drag`):
  - `drag_started_by(Primary)` → `ViewportCommand::StartDrag`;
  - `double_clicked` → the system action read once from `NSUserDefaults` `AppleActionOnDoubleClick` (`Maximize` → `Maximized(!maximized)`, `Minimize` → `Minimized(true)`, `None` → nothing; default Maximize).

### 5. Windows/Linux bar: `ui/title_bar.rs`

- In `Drawn` mode, a 40 px `Panel::top("title_bar")` replaces both the menu row and the top bar, laid out with the mockup's metrics:
  - mark (22 px, `icons::VEHICLE` on a dark square);
  - `menu_bar::menu_buttons(ui, cmds, …)` (the existing loop, extracted);
  - flexible drag area with the switcher centered in it (`top_bar::switcher`, extracted);
  - Export… (`top_bar::export_button`, extracted);
  - window controls 44×40: Minimize, Maximize/Restore (icon and tooltip from `ctx.input(|i| i.viewport().maximized)`), Close (hover fill `SIGNAL`), with tooltips `window-minimize`, `window-maximize`, `window-restore`, `window-close`.
- **Fitting:** the switcher is centered in the whole bar when there is room, otherwise placed just after the menus. Menus and Export… never shrink. The 960 px German case is tested.
- Drag and double-click on the free area as in Decision 4, but a double-click always toggles maximize.
- **Close** sends `ViewportCommand::Close`. The existing `close_requested` guard handles unsaved changes.
- **Resize grips** (`ui/title_bar.rs::resize_grips`):
  - an `Area` on the `Foreground` order, 5 px wide along each edge and 10×10 at the corners, not shown when maximized or fullscreen;
  - hover sets the matching resize cursor, and `drag_started` sends `ViewportCommand::BeginResize(dir)`;
  - drawn last each frame so they win over panels at the edges.
- **Window title:** unchanged (`sync_title`).
- **System mode** (preference or environment variable): today's layout, with the menu row (28 px) then the top bar, at 40 px with name and badge and without breadcrumb.

### 6. Breadcrumb and top bar

- `top_bar.rs` drops the breadcrumb (the canvas inlay stays) and goes from 44 to 40 px (`size::TOP_BAR_HEIGHT`).
- The name and badge are shown in `MacNative` and `System`, not in `Drawn`.
- `Crumbs` stays for the inlay and the command palette.

### 7. Strings

New keys in en/fr/de/es:
- `window-minimize`, `window-maximize`, `window-restore`, `window-close`;
- `prefs-system-title-bar` ("Use the system title bar") and its hint;
- `app-menu-about` ("About TruckPaint"), `app-menu-settings` ("Settings…"), `app-menu-hide`, `app-menu-hide-others`, `app-menu-show-all`, `app-menu-quit` ("Quit TruckPaint"), `app-menu-services`;
- `menu-window` ("Window").

`menu-export` is removed.

## Risks / Trade-offs

- **Re-injected keys are only as good as the copy of egui-winit's translation.** Copy, Cut and Paste are events, other keys are `Key` events, and egui-winit also sends `Text` for printable keys without Command. Mitigation: only shortcuts with Command or Control are injected (Decision 3), a unit test compares the injected events with egui-winit's for each shortcut, and the macOS manual pass covers copy, paste and undo in text fields.
- **A disabled item swallows its key.** If our enabled state is wrong, a shortcut silently does nothing on macOS. Mitigation: enabled state comes from `CommandUi::enabled`, the same rule the palette and the egui menus use, plus the text-editing override; and a test asserts that every command in `menus_for(true)` maps to a muda item whose enabled state follows `CommandUi::enabled`.
- **The traffic lights sit at their native height** (about 28 px strip) in a 40 px bar, so they are slightly high. Accepted for now. Adjusting them means moving `standardWindowButton` frames through objc2 on every resize.
- **Drawn title bar on Windows:**
  - no Snap Layouts flyout (Win+Z and dragging to edges still work);
  - the 1 px top border hack of egui-winit;
  - resize grips steal 5 px of clicks at the window edges (the scrollbars and panel resize handles at the edges move inward by that much).
- **Linux:**
  - X11 under unusual window managers may refuse `_NET_WM_MOVERESIZE`;
  - Wayland compositors give an undecorated window no shadow and no resize margin (our grips provide the margin), and are untested so far.

  Mitigation: the Preferences option, and the environment variable when the window can't be used to reach Preferences.
- **`ViewportCommand::Decorations` at runtime** may not restore exactly the same window size on every platform (title bar height added or removed). Accepted: the content area changes by the bar's height, and the position is kept.
- **Not testable in CI:** macOS native menu behavior and real window dragging and resizing. Unit tests cover the model (menus per platform, accelerator conversion, injected events, enabled sync), kittest covers the drawn bar's layout and the viewport commands it sends, and a manual checklist (tasks.md) covers each OS.
