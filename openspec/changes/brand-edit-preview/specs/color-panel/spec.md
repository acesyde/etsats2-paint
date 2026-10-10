## MODIFIED Requirements

### Requirement: Project palette
The color popover SHALL show the project's palette (see the document-model capability) as the **brand palette**, before the recent colors, each swatch with its name and its usage (see the brand-impact capability) on hover. The Brand space SHALL show the same palette with the same swatch actions (see the brand-space capability).

- **Add to palette** saves the current target color, or the selected stop's color when the target is a gradient, as a new swatch. It adds no duplicates. The current target becomes linked to the new swatch.
- **Clicking a swatch** applies its color to the current target or to the selected stop, and links it to the swatch.
- **The linked swatch is marked:** when the current target, or the selected stop, is linked to a swatch, that swatch has a selection ring in the link color (see the ui-design-system capability), and the popover and the target's row show the swatch's name with a link icon. The mark doesn't rely on color alone.

Each swatch's context menu, in the popover and in the Brand space, SHALL offer:
- **Edit Swatch…**: opens the swatch's before/after editor (see the brand-impact capability), with its name, color picker and hex entry. Nothing changes before Apply to Fleet, which records the new color and name as one undo step; Cancel or Escape leaves the project untouched. From the color popover, the editor opens as a dialog over the Workshop.
- **Delete Swatch**: removes the swatch. Every color keeps its value and stops being linked.
- **Add to Library** or **Update in Library** (see the shared-library capability).

Palette changes SHALL be undoable and belong to the project.

#### Scenario: Add and apply a palette color
- **WHEN** the user adds the current color to the palette, selects another object and clicks that swatch in the color popover
- **THEN** the other object gets that color, linked to the swatch

#### Scenario: Edit the company red
- **WHEN** objects on two textures are linked to the swatch "Company red", and the user opens Edit Swatch… on it from the color popover, enters `#8B0000` and clicks Apply to Fleet
- **THEN** those objects are dark red on both textures, the swatch shows dark red, and one Undo restores the previous red everywhere

#### Scenario: Cancel an edit
- **WHEN** the user changes a swatch's color in Edit Swatch… and presses Escape
- **THEN** the swatch and every linked color are as before, the canvas never showed the new color, and nothing is added to the undo history

#### Scenario: Linked swatch marked
- **WHEN** the selected rectangle's fill is linked to "Company red" and the fill is the target
- **THEN** the Fill row reads "Company red" with a link icon in the link color, and in the color popover the "Company red" swatch has a selection ring in the link color

#### Scenario: Picking another color
- **WHEN** the selected rectangle's fill is linked to "Company red" and the user picks a recent color
- **THEN** no swatch is marked, the Fill row shows the hex code without the link color, and the fill is no longer linked

#### Scenario: Edit Swatch from the Brand space
- **WHEN** the user opens the context menu of "Company red" in the Brand space and chooses Edit Swatch…
- **THEN** the same before/after editor opens, as a panel on the right of the Brand space, and works as from the color popover

#### Scenario: Usage on hover
- **WHEN** "Company red" is used by 4 objects on 2 textures and the user hovers it in the color popover
- **THEN** the tooltip reads "Company red" and "2 textures · 4 objects"
