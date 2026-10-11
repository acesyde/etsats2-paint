## ADDED Requirements

### Requirement: Checkboxes
Every checkbox of the app SHALL have the same look: a square box with small rounded corners, empty when unchecked and filled with a check mark when checked, so that it is never mistaken for a radio button (a circle). A checked state SHALL not be shown by a check mark alone, without its box.

#### Scenario: Export Texture's Transparent option
- **WHEN** the Export Texture dialog is open with the PNG format
- **THEN** the Transparent option is a square box, while the Size options above it are circles

#### Scenario: Import from Library
- **WHEN** Import from Library is open with one item already in the project and one not checked
- **THEN** both items show a square box, the first one checked

### Requirement: Text that doesn't fit
A single-line text with less room than it needs (names in lists, rows and headers) SHALL be shortened with "…" at its end and SHALL show in full in a tooltip; it SHALL never be cut in the middle of a letter, overlap another control or push its container wider than its window.

#### Scenario: Long vehicle name in the left panel
- **WHEN** the Workshop's left panel lists the vehicle "TruckPaint Sample Truck" in a 40% longer text mode
- **THEN** the name stays on one line, ending with "…" if it doesn't fit, and resting the pointer on it shows the whole name

#### Scenario: Custom Vehicle dialog stays in the window
- **WHEN** the Custom Vehicle dialog lists a main texture and an accessory texture in a 1440 × 1000 window
- **THEN** the whole dialog, its title and its Create button are inside the window
