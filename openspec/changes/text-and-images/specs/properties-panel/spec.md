## ADDED Requirements

### Requirement: Character settings section
When the selection contains texts (only texts, or texts within groups), the Properties panel SHALL show a Character section with the font family picker, weight, italic toggle, size, alignment (segmented: left, center, right), letter spacing and line height, applying to every selected text; differing values SHALL show "Mixed". With nothing selected, the section edits the character style used for new texts.

#### Scenario: Change size of two texts
- **WHEN** two texts are selected and the user sets the size to 300
- **THEN** both texts use 300 px and one undo step restores both

### Requirement: Image information
When a single image is selected, the Properties panel SHALL show its asset name, its kind and source size (pixels or "Vector"), and a "Reset Size" action that restores its placement size and aspect ratio while keeping its center and rotation.

#### Scenario: Reset a distorted image
- **WHEN** an 800×400 image was stretched to 800×800 and the user clicks Reset Size
- **THEN** it is 800×400 again, centered at the same point
