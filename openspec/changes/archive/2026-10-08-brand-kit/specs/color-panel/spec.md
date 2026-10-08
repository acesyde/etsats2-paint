## MODIFIED Requirements

### Requirement: Project palette
The panel SHALL show the project's palette (see the document-model capability), each swatch with its name on hover.

- **Add to palette** saves the current target color, or the selected stop's color when the target is a gradient, as a new swatch. It adds no duplicates. The current target becomes linked to the new swatch.
- **Clicking a swatch** applies its color to the current target or to the selected stop, and links it to the swatch.
- **The linked swatch is marked:** when the current target, or the selected stop, is linked to a swatch, that swatch has a selection ring, and the panel shows the swatch's name. The mark doesn't rely on color alone.

Each swatch's context menu SHALL offer:
- **Edit Swatch…**: a popup with the swatch's name and a color picker with hex entry. Changing the color updates every color linked to the swatch live, on every texture. OK records the change as one undo step, and Cancel or Escape restores the swatch and the linked colors. A name can't be empty.
- **Delete Swatch**: removes the swatch. Every color keeps its value and stops being linked.

Palette changes SHALL be undoable and belong to the project.

#### Scenario: Add and apply a palette color
- **WHEN** the user adds the current color to the palette, selects another object and clicks that swatch
- **THEN** the other object gets that color, linked to the swatch

#### Scenario: Edit the company red
- **WHEN** objects on two textures are linked to the swatch "Company red", and the user opens Edit Swatch… on it, enters `#8B0000` and clicks OK
- **THEN** those objects are dark red on both textures, the swatch shows dark red, and one Undo restores the previous red everywhere

#### Scenario: Cancel an edit
- **WHEN** the user changes a swatch's color in Edit Swatch… and presses Escape
- **THEN** the swatch and every linked color are as before, and nothing is added to the undo history

#### Scenario: Linked swatch marked
- **WHEN** the selected rectangle's fill is linked to "Company red" and the Colors panel targets the fill
- **THEN** the "Company red" swatch has a selection ring and the panel shows "Company red"

#### Scenario: Picking another color
- **WHEN** the selected rectangle's fill is linked to "Company red" and the user picks a recent color
- **THEN** no swatch is marked, and the fill is no longer linked
