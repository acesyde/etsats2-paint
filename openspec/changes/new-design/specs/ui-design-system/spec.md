## MODIFIED Requirements

### Requirement: Custom dark theme
The application SHALL render all screens with a custom dark theme defined by a single set of design tokens (surface colors per elevation level, text colors per emphasis level, accent colors, semantic colors for success / warning / error, spacing scale, corner radii, stroke widths). No screen SHALL use the toolkit's default visual style.

The theme SHALL be low in saturation so that livery colors read true. Its surfaces and ink SHALL be:

| Token | Value | Use |
|---|---|---|
| Canvas surface | `#1C1C1F` | pasteboard and canvas area |
| Panel surface | `#222225` | bars, panels, dialogs (outlined `#3E3E43`, radius 14) |
| Raised surface | `#2E2E33` | hovered rows and cards |
| Field surface (sunken) | `#1C1C1F` with a `#343439` outline, radius 6 | text and numeric fields |
| Control surface | `#28282C` | buttons and dropdowns, tracks of segmented controls and sliders, menus, popovers, the breadcrumb pill |
| Chip surface | `#34343A` | the game badge, the active space of the switcher, the open menu |
| Ink | `#EDEDED` | primary text |
| Muted text | `#9A9AA2` | inactive tabs and options, shortcuts, the status bar's text, section headings, the labels inside inset fields; only where it keeps 4.5:1 |

The theme SHALL have three accents, each with one meaning and used for nothing else:

| Accent | Value | Meaning |
|---|---|---|
| Primary (white) | `#ECECEC` | the primary action of a screen or dialog, the active option of a segmented control, the active tool |
| Signal (red) | `#F56B6B` | alerts, "update available", the selection on the canvas |
| Link (blue) | `#50B9DF` (oklch 0.74 0.11 225) | everything linked to the brand: a fill or stroke linked to a palette swatch, an object following a shared style, a symbol instance |

Success, warning and error SHALL keep their own semantic tokens, distinct from the three accents.

#### Scenario: Theme applied on launch
- **WHEN** the application starts
- **THEN** every visible surface (window background, panels, menus, tooltips, dialogs) uses colors from the design tokens

#### Scenario: Visual hierarchy between surfaces
- **WHEN** a panel, the canvas area and a popup menu are visible at the same time
- **THEN** the canvas area is drawn with the canvas surface `#1C1C1F`, the panel with the panel surface `#222225` and the popup menu with the control surface `#28282C`

#### Scenario: Sunken fields
- **WHEN** a rectangle is selected and the inspector shows its position fields
- **THEN** the fields are drawn on the field surface `#1C1C1F` with a `#343439` outline, below the panel surface around them

#### Scenario: Primary action in white
- **WHEN** the Export Mod dialog is open
- **THEN** its Export button is drawn with the primary accent and its other buttons are not

#### Scenario: Linked fill in the link color
- **WHEN** the selected rectangle's fill is linked to a palette swatch
- **THEN** its Fill row in the inspector shows the link in the link color, and a rectangle with an unlinked fill shows no link color

#### Scenario: Update available in the signal color
- **WHEN** a newer version of a vehicle's package is installed
- **THEN** the "Update available" mark is drawn in the signal color

### Requirement: Text contrast
Primary text SHALL have a contrast ratio of at least 7:1 against its background, and secondary text and icons used to convey information SHALL have a contrast ratio of at least 4.5:1. Disabled content SHALL remain at least 3:1.

These ratios SHALL hold on each surface (canvas, panel, raised, field, control, chip) for every pair the theme uses, including:
- ink on each surface (at least 7:1);
- muted text on each surface it is used on (at least 4.5:1);
- the signal and link accents, used as text or informative icons, on each surface (at least 4.5:1);
- the dark text of the active option on the white pill of a segmented control and on a primary button (at least 7:1).

#### Scenario: Contrast verification
- **WHEN** the design token set is checked by the automated contrast test
- **THEN** every text/background token pair used by the theme meets its minimum ratio

#### Scenario: Accents on the raised surface
- **WHEN** the automated contrast test checks the signal and link accents against the raised surface `#2E2E33`
- **THEN** both reach at least 4.5:1

#### Scenario: Text on the white pill
- **WHEN** the automated contrast test checks the active segment's text against the white pill
- **THEN** the ratio is at least 7:1

### Requirement: Embedded typography
The application SHALL ship its own UI typefaces and icon set embedded in the binary, so that the interface looks identical on Linux, Windows and macOS regardless of installed system fonts. The interface typeface SHALL be Geist (Regular, Medium, SemiBold) and the monospace typeface JetBrains Mono (Regular, Medium), both shipped with their SIL Open Font License. JetBrains Mono SHALL be used for values: numeric fields, pixel sizes and positions, hexadecimal colors and file paths; Geist for every other interface text. Inter SHALL stay available as a font for text objects in documents but SHALL NOT be the interface typeface.

The application SHALL use this typographic scale consistently across screens:

| Style | Size (points) | Weight |
|---|---|---|
| Title | 22 | 600 |
| Heading | 15 | 600 |
| Body and label | 13 | 400 |
| Mono | 12 | 400 |
| Caption | 11 | 400 |

#### Scenario: Same rendering on every platform
- **WHEN** the application runs on a machine without any additional fonts installed
- **THEN** UI text renders in Geist, values in JetBrains Mono, and icons with the embedded icon set

#### Scenario: Values in monospace
- **WHEN** a rectangle is selected and the inspector shows its position and a hexadecimal fill color
- **THEN** the X, Y and hexadecimal values are drawn in JetBrains Mono 12 and their labels in Geist 13

#### Scenario: Inter still available for documents
- **WHEN** the user picks a font for a text object
- **THEN** Inter is still offered as a font family

#### Scenario: Font licenses shipped
- **WHEN** the application's bundled font files are listed
- **THEN** the OFL license of Geist and of JetBrains Mono is present next to them

## ADDED Requirements

### Requirement: Segmented controls
Choices between a few mutually exclusive options (for example Solid / Linear / Radial, or the Textures / Layers / Resources tabs) SHALL be shown as a segmented control: the options side by side in one track on the control surface, the active option drawn as a white pill (primary accent) with dark text, the other options as muted text on the track. Clicking an option SHALL make it active. The active option SHALL be marked by the pill's fill, not only by its color, so that it stays visible in a grayscale rendering.

The space switcher of the top bar (Project / Workshop / Brand) SHALL be the same choice drawn as tabs: no track, the other options as muted text, the active option on the chip surface. Its active option SHALL also be marked by a fill.

#### Scenario: Choosing an option
- **WHEN** a segmented control shows Solid active and the user clicks Linear
- **THEN** Linear is drawn in the white pill with dark text and Solid as plain text on the track

#### Scenario: Active option in grayscale
- **WHEN** a screen with a segmented control is rendered in grayscale
- **THEN** the active option is still distinguishable from the others by its pill
