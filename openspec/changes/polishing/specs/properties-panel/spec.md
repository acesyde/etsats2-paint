## MODIFIED Requirements

### Requirement: Selection summary
The inspector header SHALL show what is selected: the object's name and kind (name and kind icon) for a single object or group, or "N objects" for several. The name SHALL follow the Layers panel's rule (see the layers-panel capability): a name that is the kind's default name in any supported language is the kind's, said once, and a text named after its kind shows its first line. With nothing selected, the inspector SHALL show the active texture's properties instead:
- its name, its kind (main texture or accessory texture) and its size ("4096²", or "8192 × 1024" when its sides differ);
- when a package update flagged it, that it is **To check** (see the texture-status capability) and why: its layout changed in the vehicle's version, with a **Mark as Checked** action, or it is not in this version and is left out of the mod, with no action;
- a hint to select an object to set its layout, fill and stroke.

While a symbol is edited (see the symbols capability) and nothing is selected, the header SHALL name the symbol, say that it is a symbol and give its size, not the texture holding it.

The template's Show Template and opacity settings SHALL NOT be in the inspector: they are view settings in the status bar (see the workspace-layout capability).

#### Scenario: Multiple selection summary
- **WHEN** three objects are selected
- **THEN** the inspector header reads "3 objects"

#### Scenario: Nothing selected
- **WHEN** nothing is selected and the active texture is the 4096 px main texture "Cabin"
- **THEN** the inspector shows "Cabin", that it is a main texture, "4096²" and the hint, and no template show or opacity setting

#### Scenario: Layout changed notice
- **WHEN** nothing is selected and an update to version 1.3.0 flagged the active texture's layout as changed
- **THEN** the inspector says the texture is To check because its layout changed in 1.3.0, with a Mark as Checked action, and clicking Mark as Checked removes the notice

#### Scenario: Not in this version notice
- **WHEN** nothing is selected and the active texture is marked "Not in this version"
- **THEN** the inspector says the texture is To check because it is not in this version and is left out of the mod, with no Mark as Checked action

#### Scenario: Rectangular texture summary
- **WHEN** nothing is selected and the active texture is a trailer's 8192 × 1024 main texture
- **THEN** the inspector shows its size as "8192 × 1024"

#### Scenario: Text from another language
- **WHEN** the language is French and the selected text, named "Text" when it was created in English, reads "ACE"
- **THEN** the inspector header reads "ACE" with the kind "Texte"

#### Scenario: Editing a symbol with nothing selected
- **WHEN** the symbol "ACE badge" of 440 × 440 px is edited and nothing is selected
- **THEN** the inspector header reads "ACE badge", that it is a symbol, and "440²"

### Requirement: Fill and stroke swatches
The Appearance section SHALL show a **Fill** row and, under it, a **Stroke** row for the selection's shapes and texts (see the color-panel and stroke-panel capabilities). Each row's swatch SHALL show the paint (a "None" pattern for no stroke and a "Mixed" pattern for differing paints). Clicking a swatch SHALL make it the color target and open the color popover for it. The row that is the color target SHALL be marked, except while another popover of the inspector (stroke options, line settings or shadow) is open: no Fill or Stroke row is then marked. With nothing selected, the inspector SHALL show the Fill and Stroke rows of the look used for new objects, under a heading saying so.

#### Scenario: Clicking the stroke swatch
- **WHEN** a stroked rectangle is selected and the user clicks the Stroke row's swatch
- **THEN** the stroke becomes the color target and the color popover opens beside the inspector, level with the Stroke row

#### Scenario: Look of new objects
- **WHEN** nothing is selected and the user sets the Fill row of new objects to red, then draws a rectangle
- **THEN** the new rectangle is red

#### Scenario: Shadow popover open
- **WHEN** a rectangle is selected, its fill is the color target, and the user opens the Shadow row's popover
- **THEN** neither the Fill nor the Stroke row is marked while the popover is open, and the Fill row is marked again once it closes
