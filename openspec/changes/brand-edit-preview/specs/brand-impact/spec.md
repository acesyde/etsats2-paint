## Purpose

Tells the player how far an edit to a shared element reaches before it is made: where each swatch, style and symbol is used, and a before/after editor that shows the impact on the fleet and changes nothing until Apply to Fleet.

## ADDED Requirements

### Requirement: Usage of shared elements
Each swatch, graphic style and text style SHALL have a **usage**: the number of objects it changes and the number of textures holding them, over every texture of the project.

- A **swatch** changes an object when the object's fill, stroke or one of its gradient stops is linked to it. It also changes each instance of a symbol whose content has such an object: the instance counts as one object.
- A **style** changes the objects that follow it, including the texts that follow a text style.
- Objects inside groups count one by one; a group itself is not counted. An instance counts as one object, whatever its content holds.
- A texture counts once, however many of its objects are counted.

Each **symbol** SHALL have its number of instances on every texture and the number of textures holding them.

The usage SHALL read "11 textures · 38 objects", with singular forms for one ("1 texture · 1 object"), or **Unused** when nothing is counted. A symbol's usage SHALL read "14 instances · 9 textures", or Unused. The usage SHALL follow the project: after any edit, undo or redo, it shows the new counts.

#### Scenario: A swatch used on two textures
- **WHEN** "Company red" is the fill of three rectangles of the Cab and the stroke of one circle of the Chassis
- **THEN** its usage reads "2 textures · 4 objects"

#### Scenario: Through a symbol
- **WHEN** the only object linked to "Company red" is inside the symbol "Logo", which has five instances on three textures
- **THEN** the usage of "Company red" reads "3 textures · 5 objects"

#### Scenario: An object linked twice
- **WHEN** one rectangle has its fill and its stroke linked to "Company red"
- **THEN** the rectangle counts once

#### Scenario: Unused swatch
- **WHEN** no color of the project is linked to "Cream"
- **THEN** the usage of "Cream" reads "Unused"

#### Scenario: Usage follows undo
- **WHEN** the user links one more rectangle to "Company red" and then undoes it
- **THEN** the usage of "Company red" goes up by one object, then back

#### Scenario: Symbol usage
- **WHEN** "Logo" has four instances on the Cab and two on the Chassis
- **THEN** its usage reads "6 instances · 2 textures"

### Requirement: Before/after editor
Editing a swatch or a graphic style SHALL go through a **before/after editor**. It SHALL show:
- what is edited ("Edit color" or "Edit style") and the element's name, which can be changed;
- the controls of the value: for a swatch, the color picker with hex entry; for a graphic style, its fill, stroke and opacity, as in the inspector's Appearance rows;
- a **Before** preview of the current value and an **After** preview of the new one;
- the **impact**: "Impact: 11 textures, 38 objects", the usage of the element, or a line saying nothing uses it yet;
- a **thumbnail of each affected texture** with the new value, and a line saying that thumbnails update live and nothing is saved before Apply to Fleet;
- **Cancel** and **Apply to Fleet**.

Nothing in the project SHALL change while the editor is open: the canvas, the textures, the other panels and the undo history show the current value. The thumbnails SHALL follow the controls while they change; while the picker is dragged they MAY lag behind, and SHALL show the last value when the drag ends.

**Apply to Fleet** SHALL record the new value and name as one undo step: every object linked to the swatch, or following the style, takes the new value on every texture, and the links hold. With nothing changed, Apply to Fleet SHALL close the editor and record nothing. A name can't be empty: Apply to Fleet is then disabled, with the reason shown.

**Cancel** or Escape SHALL close the editor and leave the project and the undo history untouched.

#### Scenario: Edit the company red
- **WHEN** objects on two textures are linked to "Company red", and the user opens its editor, enters `#8B0000` and clicks Apply to Fleet
- **THEN** those objects are dark red on both textures, the swatch shows dark red, and one Undo restores the previous red everywhere

#### Scenario: Nothing changes before Apply
- **WHEN** the user enters `#8B0000` in the editor of "Company red" and has not applied it
- **THEN** the canvas and the textures still show the previous red, the After preview and the thumbnails of the two textures show dark red, and the Before preview shows the previous red

#### Scenario: Impact before applying
- **WHEN** "Company red" is used by 38 objects on 11 textures and the user opens its editor
- **THEN** the editor reads "Impact: 11 textures, 38 objects" and shows 11 thumbnails

#### Scenario: Cancel
- **WHEN** the user changes the color in the editor of a swatch and presses Escape
- **THEN** the editor closes, the swatch and every linked color are as before, and nothing is added to the undo history

#### Scenario: Rename only
- **WHEN** the user types "Ardent red" as the name of "Company red" and clicks Apply to Fleet
- **THEN** the palette lists "Ardent red" with the same color, and one Undo restores "Company red"

#### Scenario: Restyle the stripes
- **WHEN** rectangles on three textures follow the graphic style "Stripe", and the user opens Edit Style… on it, sets its stroke width to 8 px and clicks Apply to Fleet
- **THEN** the rectangles on the three textures have an 8 px stroke and still follow "Stripe", and one Undo restores the previous stroke everywhere

#### Scenario: Empty name
- **WHEN** the user clears the name in the editor
- **THEN** Apply to Fleet is disabled, and the editor says a name is needed
