## MODIFIED Requirements

### Requirement: Character settings
Each text object SHALL have one character style: font family, weight (100–900), italic, size in texture pixels, alignment (left, center, right), letter spacing (in thousandths of an em, −200 to 1000) and line height (percent of the size, 50–300%). Text SHALL be laid out from these settings and drawn from the fonts' vector glyph outlines, with the object's fill, stroke and opacity. Resizing a text with handles SHALL scale it; the inspector's Layout section shows the resulting size.

#### Scenario: Centered multi-line text
- **WHEN** a two-line text is set to center alignment
- **THEN** both lines are centered on the text's anchor

#### Scenario: Outlined lettering
- **WHEN** a white text gets a black 6 px stroke
- **THEN** every glyph is outlined in black on the canvas
