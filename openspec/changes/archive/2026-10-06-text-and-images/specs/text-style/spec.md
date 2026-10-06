## Purpose

Defines how text looks — font, weight, slant, size, alignment and spacing — and where fonts come from, so lettering is precise and looks the same on every computer.

## ADDED Requirements

### Requirement: Character settings
Each text object SHALL have one character style: font family, weight (100–900), italic, size in texture pixels, alignment (left, center, right), letter spacing (in thousandths of an em, −200 to 1000) and line height (percent of the size, 50–300%). Text SHALL be laid out from these settings and drawn from the fonts' vector glyph outlines, with the object's fill, stroke and opacity. Resizing a text with handles SHALL scale it; the Transform panel shows the resulting size.

#### Scenario: Centered multi-line text
- **WHEN** a two-line text is set to center alignment
- **THEN** both lines are centered on the text's anchor

#### Scenario: Outlined lettering
- **WHEN** a white text gets a black 6 px stroke
- **THEN** every glyph is outlined in black on the canvas

### Requirement: Font sources
The font list SHALL contain the bundled fonts (Inter, Barlow Condensed, Oswald, Bebas Neue, Montserrat, available on every computer) followed by the fonts installed on the computer, deduplicated by family name. The default style for new texts SHALL be Inter Bold, 200 px, left-aligned, 0 letter spacing, 120% line height.

#### Scenario: Bundled font always available
- **WHEN** the application runs on a computer without Oswald installed
- **THEN** Oswald is listed and texts using it render correctly

### Requirement: Font picker
The font family control SHALL open a searchable list where each family name is shown in its own font, filtering as the user types and choosable with the keyboard. Weights not available for the chosen family SHALL fall back to the nearest available weight.

#### Scenario: Search a font
- **WHEN** the user opens the font picker and types "bebas"
- **THEN** only families containing "bebas" (case-insensitive) are listed

### Requirement: Missing fonts
When a text uses a family that is not available, it SHALL be drawn with Inter and its family SHALL be shown with a warning icon and the tooltip "Font not found: <name>"; the original family name SHALL be kept so the text renders correctly again where the font exists.

#### Scenario: Font not installed
- **WHEN** a text's family is "Some Missing Font"
- **THEN** it is drawn with Inter and the font control shows "Some Missing Font" with a warning
