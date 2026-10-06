# properties-panel Specification

## Purpose

Gives a contextual summary of the selection and quick access to its appearance properties.

## Requirements

### Requirement: Selection summary
The Properties panel SHALL show what is selected: the object's name and kind icon for a single object or group, or "N objects" for several. With nothing selected, it SHALL show the active surface name and size and a hint to select an object.

#### Scenario: Multiple selection summary
- **WHEN** three objects are selected
- **THEN** the panel header area reads "3 objects"

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
