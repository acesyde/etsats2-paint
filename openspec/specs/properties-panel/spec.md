# properties-panel Specification

## Purpose

Gives a contextual summary of the selection and quick access to its appearance properties.

## Requirements

### Requirement: Selection summary
The Properties panel SHALL show what is selected: the object's name and kind icon for a single object or group, or "N objects" for several. With nothing selected, it SHALL show the active surface name and size, a hint to select an object, and the active surface's **Template** settings (see the vehicle-projects capability): what an update flagged, Show Template and the template opacity.

#### Scenario: Multiple selection summary
- **WHEN** three objects are selected
- **THEN** the panel header area reads "3 objects"

#### Scenario: Nothing selected
- **WHEN** nothing is selected and the active texture is the 4096 px Cabin
- **THEN** the panel shows "Cabin", "4096 × 4096 px", the hint, and the Template settings with Show Template and the opacity slider

### Requirement: Opacity
The panel SHALL provide an opacity field and slider from 0% to 100% applying to every selected object. A slider drag SHALL be one undo step. Differing values SHALL show "Mixed".

#### Scenario: Set opacity
- **WHEN** an object is selected and the user sets opacity to 40%
- **THEN** the object is drawn at 40% opacity and Undo restores 100%

### Requirement: Corner radius
When every selected object is a rectangle, the panel SHALL show a corner radius field (texture pixels, clamped to half of the smaller side). It SHALL be hidden otherwise.

#### Scenario: Rounded rectangle
- **WHEN** a 400×200 rectangle is selected and the user sets the corner radius to 300
- **THEN** the radius is clamped to 100

### Requirement: Fill and stroke swatches
The panel SHALL show fill and stroke swatches of the selection (a "None" pattern for no stroke and a "Mixed" pattern for differing colors). Clicking a swatch SHALL make it the Colors panel target and reveal the Colors panel if it is collapsed or closed.

#### Scenario: Clicking the stroke swatch
- **WHEN** the Colors panel is collapsed and the user clicks the stroke swatch
- **THEN** the Colors panel expands with the stroke as its target

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
