## Purpose

Lets a livery's lettering, logos and stripes cast one drop shadow, so that they stay readable on any background without a hand-made copy kept in sync on every texture.

## ADDED Requirements

### Requirement: Shadow property
A shape, a path, a text or an image SHALL be able to have at most one **shadow**, made of:
- a **color**, which MAY be linked to a palette swatch, as a fill is;
- an **opacity**, from 0 to 100%;
- an **offset** x and y, in texture pixels, from −1000 to 1000;
- a **blur** radius, in texture pixels, from 0 (a hard edge) to 200.

Groups and symbol instances SHALL have no shadow of their own; the objects inside a group, or inside a symbol's content, MAY have one. Objects have no shadow by default, and new objects take none from the look used for new objects.

A shadow color linked to a swatch SHALL follow Edit Swatch… and stay linked while it has the swatch's value, as a fill does. Picking another color SHALL unlink it. Deleting the swatch SHALL keep the shadow's color and unlink it.

#### Scenario: A shadow on a text
- **WHEN** a text has a shadow of black at 50%, offset 8 / 8 px, blur 8 px
- **THEN** the project stores those four values for the text, and the text has no other shadow

#### Scenario: No shadow on a group
- **WHEN** only a group is selected
- **THEN** the inspector offers no shadow, and the texts inside the group keep their own shadows

#### Scenario: Shadow color follows its swatch
- **WHEN** a text's shadow is linked to the swatch "Night", and the user edits "Night" to dark blue and applies it to the fleet
- **THEN** the text's shadow is dark blue and still linked to "Night"

### Requirement: Shadow drawing
The shadow SHALL be the object's silhouette (its fill and stroke as drawn, or for an image its opaque pixels) in the shadow color at the shadow's opacity, moved by the offset, blurred by the blur radius, and drawn just under the object, above everything below the object. The object's opacity, and the opacities of the groups holding it, SHALL apply to the shadow too. A blur of 0 SHALL draw a hard-edged shadow.

The shadow SHALL be drawn on the canvas, in Export Texture and in Export Mod's textures with the same result. A shadow that goes past the texture's edges SHALL be clipped like any object. A hidden object's shadow SHALL NOT be drawn.

The shadow SHALL NOT be part of the object for selection: clicking on the shadow alone SHALL NOT select the object, and the object's bounds, the selection box, snapping and alignment SHALL ignore it.

#### Scenario: Hard shadow
- **WHEN** a white 100 × 100 px square at (500, 500) has a black shadow at 100%, offset 10 / 10, blur 0, on a red background
- **THEN** the pixels from (600, 510) to (610, 610) are black, and the square itself is white

#### Scenario: Opacity applies to the shadow
- **WHEN** the square of the previous scenario has an opacity of 50%
- **THEN** its shadow is drawn at half of its own opacity

#### Scenario: Clicking the shadow
- **WHEN** the user clicks on a point covered only by a text's shadow
- **THEN** the text is not selected

#### Scenario: Same in the export
- **WHEN** a texture with a blurred shadow is shown on the canvas at 100% and exported as PNG at full size
- **THEN** the exported shadow matches the canvas, within the canvas's anti-aliasing

### Requirement: Shadow row in the inspector
The inspector's Appearance section SHALL show, after the Fill and Stroke rows, either **+ Add a shadow** when no selected object has a shadow, or a **Shadow** row. The row SHALL span the section's width like the Fill and Stroke rows and show a swatch of the shadow color, a short summary of its offset and blur that reads the same in every language ("8 / 8 · 8"), or "Mixed" when the selected objects differ, with the full values on hover ("Offset 8 / 8, blur 8"), and a button to remove the shadow at the right end of the row.

- **+ Add a shadow** SHALL give every selected shape, path, text and image the default shadow: black, 50%, offset 8 / 8 px, blur 8 px.
- Clicking the Shadow row SHALL open a popover beside the inspector with the color (the color picker, the palette and hex entry, as for Fill), the opacity, the offset x and y and the blur. Escape SHALL close it.
- Removing the shadow SHALL remove it from every selected object.

The row SHALL be left out when the selection holds only groups, instances, or both. Each change SHALL be one undo step; dragging a value SHALL be one undo step. With nothing selected, the inspector SHALL NOT show a shadow for new objects.

#### Scenario: Add a shadow
- **WHEN** a single text without shadow is selected and the user clicks + Add a shadow
- **THEN** the text gets a black shadow at 50%, offset 8 / 8 px, blur 8 px, the row reads "8 / 8 · 8", and one Undo removes it

#### Scenario: Edit the blur by dragging
- **WHEN** the user drags the blur value in the popover from 8 to 20
- **THEN** the shadow on the canvas follows the drag, and one Undo brings back blur 8

#### Scenario: Mixed values
- **WHEN** two texts are selected, one with blur 8 and one with blur 20
- **THEN** the Shadow row reads "Mixed", and setting the blur to 12 in the popover gives both texts blur 12

#### Scenario: Remove the shadow
- **WHEN** the user clicks the remove button of the Shadow row
- **THEN** the selected objects have no shadow, and the row reads + Add a shadow
