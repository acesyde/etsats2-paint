# window-title-bar Specification

## Purpose
Gives the window one 40 px bar at the top instead of a title bar, a menu row and a top bar: on macOS the top bar shares the native title strip and the menus are in the system menu bar; on Windows and Linux the window draws its own title bar holding the menus, the space switcher, Export… and the window controls, unless the player chooses the system title bar.

## Requirements

### Requirement: Title bar on macOS
On macOS the window SHALL keep its native frame: the close, minimize and zoom buttons at the top left, rounded corners, the shadow, resizing from the edges and the system's tiling and full screen. The native title text SHALL NOT be shown. In its place, the top of the window SHALL be one bar, 40 px high, holding after the window buttons:
- with a project open: the top bar's content (see the workspace-spaces capability);
- on the home screen: nothing but the window buttons.

No menu row SHALL be drawn in the window: the menus are in the system menu bar (see the workspace-layout capability).

The empty parts of the bar SHALL move the window when dragged, and a double-click on them SHALL do what the system setting "Double-click a window's title bar" says (zoom by default). The bar's buttons and the space switcher SHALL keep their own clicks.

#### Scenario: One bar on macOS
- **WHEN** a project is open in the Workshop on macOS
- **THEN** the top of the window shows the window buttons, the project name with its badge, the space switcher and Export… on one 40 px row, and no menu row is drawn in the window

#### Scenario: Moving the window from the bar
- **WHEN** the user drags the empty part of the bar between the space switcher and Export…
- **THEN** the window moves, and no space is switched

### Requirement: Title bar on Windows and Linux
On Windows and Linux, unless the system title bar is chosen (see System title bar option), the window SHALL be drawn without the system's title bar and frame, and SHALL show at its top one bar, 40 px high, holding from left to right:
- the TruckPaint mark;
- the menus (see the workspace-layout capability), each opening on click, with the menus' usual keyboard navigation;
- with a project open: the space switcher, centered in the free space, and the **Export…** button (see the workspace-spaces capability);
- the window controls: **Minimize**, **Maximize** or **Restore** (depending on the window's state) and **Close**, each 44 px wide, with a tooltip naming it, and Close turning red on hover.

The project name is not shown in this bar; it is the window's title, shown by the taskbar and the window switcher, as "<project name> — TruckPaint".

The bar SHALL behave like a system title bar:
- dragging any part of the bar that is not a menu, a button or the switcher moves the window, including to the screen edges for the system's snapping;
- a double-click on such a part maximizes the window, or restores it when it is maximized;
- Close asks to save unsaved changes as closing the window always does.

The window SHALL be resizable from its four edges and four corners by a grip at least 5 px wide, which is not shown while the window is maximized. The window SHALL keep its shadow on Windows. The window's size, position and maximized state SHALL be restored at the next launch as today.

#### Scenario: Bar on Windows
- **WHEN** a project is open in the Workshop on Windows, with the language French
- **THEN** the top of the window shows the mark, "Fichier", "Édition", "Objet", "Calque", "Affichage", "Véhicule", "Aide", the switcher Projet / Atelier / Marque, "Exporter…" and the minimize, maximize and close controls on one 40 px row

#### Scenario: Maximize by double-click
- **WHEN** the window is not maximized and the user double-clicks the empty part of the bar
- **THEN** the window is maximized and the middle control reads Restore; a second double-click restores it

#### Scenario: Close with unsaved changes
- **WHEN** a project has unsaved changes and the user clicks Close in the bar
- **THEN** the save prompt is shown and choosing Cancel keeps the window open with the project

#### Scenario: Resize from a corner
- **WHEN** the user drags the bottom right corner of the window
- **THEN** the window is resized, and not below its minimum size

#### Scenario: Home screen bar
- **WHEN** no project is open on Windows
- **THEN** the bar shows the mark, the menus and the window controls, without switcher nor Export…

### Requirement: System title bar option
On Windows and Linux, Preferences SHALL offer **Use the system title bar**, off by default. When it is on, the window SHALL use the system's title bar and frame, and show the menus as an in-window row, 28 px high, above the top bar, with the top bar's content as on macOS (project name and badge, space switcher, Export…). Turning it on or off SHALL apply at once, without restarting, and SHALL keep the window's size and position, the project and its unsaved changes.

The drawn title bar SHALL be the default on Windows, on Linux under X11 and on Linux under Wayland.

When the environment variable `TRUCKPAINT_SYSTEM_TITLE_BAR` is set to `1`, the system title bar SHALL be used whatever the preference, so the window stays usable on a desktop where the drawn bar can't be moved or resized.

Under the system title bar, everything else (menus, shortcuts, the top bar, the window's title) SHALL work as with the drawn title bar. The option is not offered on macOS.

#### Scenario: Turning the system title bar on
- **WHEN** a project with unsaved changes is open on Linux and the user turns on Use the system title bar in Preferences
- **THEN** the window shows the system's title bar at once, the menus become a row inside the window with the top bar under them, and the project still has its unsaved changes

#### Scenario: Back to the drawn bar
- **WHEN** the option is on and the user turns it off
- **THEN** the window shows the drawn title bar at once, with the menus, the switcher, Export… and the window controls

#### Scenario: Wayland default
- **WHEN** TruckPaint starts for the first time in a Linux Wayland session
- **THEN** the window shows the drawn title bar

#### Scenario: Forced by the environment
- **WHEN** TruckPaint starts on Windows with `TRUCKPAINT_SYSTEM_TITLE_BAR=1` and the option off
- **THEN** the window has the system's title bar and the menus are a row inside the window

#### Scenario: Not on macOS
- **WHEN** the user opens Preferences on macOS
- **THEN** there is no Use the system title bar option
