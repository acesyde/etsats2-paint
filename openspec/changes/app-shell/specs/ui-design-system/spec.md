## Purpose

Defines the visual language and accessibility baseline of the editor so that every screen looks like a finished professional design tool rather than a default toolkit application.

## ADDED Requirements

### Requirement: Custom dark theme
The application SHALL render all screens with a custom dark theme defined by a single set of design tokens (surface colors per elevation level, text colors per emphasis level, accent color, semantic colors for success / warning / error, spacing scale, corner radii, stroke widths). No screen SHALL use the toolkit's default visual style.

#### Scenario: Theme applied on launch
- **WHEN** the application starts
- **THEN** every visible surface (window background, panels, menus, tooltips, dialogs) uses colors from the design tokens

#### Scenario: Visual hierarchy between surfaces
- **WHEN** a panel, the canvas area and a popup menu are visible at the same time
- **THEN** they are rendered with distinct elevation levels so that each is visually separable from its neighbors

### Requirement: Text contrast
Primary text SHALL have a contrast ratio of at least 7:1 against its background, and secondary text and icons used to convey information SHALL have a contrast ratio of at least 4.5:1. Disabled content SHALL remain at least 3:1.

#### Scenario: Contrast verification
- **WHEN** the design token set is checked by the automated contrast test
- **THEN** every text/background token pair used by the theme meets its minimum ratio

### Requirement: Embedded typography
The application SHALL ship its own UI typeface and icon set embedded in the binary, so that the interface looks identical on Linux, Windows and macOS regardless of installed system fonts. It SHALL define a typographic scale (at least: caption, body, label, heading) used consistently across screens.

#### Scenario: Same rendering on every platform
- **WHEN** the application runs on a machine without any additional fonts installed
- **THEN** UI text and icons render with the embedded typeface and icon set

### Requirement: Interactive widget states
Every interactive widget SHALL have visually distinct idle, hover, pressed/active, focused and disabled states. Active (selected / toggled-on) state SHALL be indicated by at least one non-color cue (such as an indicator bar, fill change, outline or icon change) in addition to color.

#### Scenario: Hovering a button
- **WHEN** the pointer hovers an enabled button
- **THEN** the button changes appearance immediately (same frame) to its hover state

#### Scenario: Disabled widget
- **WHEN** a widget is disabled
- **THEN** it is rendered in the disabled style, ignores clicks, and still shows its tooltip explaining why it is unavailable when one is provided

#### Scenario: Active state not color-only
- **WHEN** a toggle or tool is active
- **THEN** its active state is perceivable in a grayscale rendering of the screen

### Requirement: Keyboard focus visibility
Widgets reachable by keyboard navigation SHALL display a visible focus indicator when focused via the keyboard.

#### Scenario: Tabbing through a dialog
- **WHEN** the user presses Tab in a dialog
- **THEN** focus moves to the next control and that control shows a focus ring

### Requirement: Minimum hit target size
Clickable controls SHALL have an interactive area of at least 24×24 logical points at the default UI scale, even when their visible icon is smaller.

#### Scenario: Small icon button
- **WHEN** an icon button displays a 16-point icon
- **THEN** clicks anywhere within its 24×24 point hit area activate it

### Requirement: Tooltips on icons
Every icon-only control SHALL have a tooltip that shows its name and, when one exists, its keyboard shortcut.

#### Scenario: Hovering a tool icon
- **WHEN** the pointer rests on an icon-only control for the tooltip delay
- **THEN** a tooltip shows the control's name and its shortcut in the platform's notation

### Requirement: UI and text scaling
The user SHALL be able to change the overall UI scale (from 75% to 200%) and, independently, the text size (from 85% to 150%). Changes SHALL apply immediately without restarting. The application SHALL respect the operating system's display scale factor (HiDPI) as the 100% baseline.

#### Scenario: Increasing UI scale
- **WHEN** the user sets the UI scale to 150%
- **THEN** all widgets, icons, spacing and text are rendered 1.5× larger immediately and remain crisp

#### Scenario: HiDPI display
- **WHEN** the application runs on a display with an OS scale factor of 2
- **THEN** the interface renders at native resolution with the same logical layout as on a scale factor of 1
