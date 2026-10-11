## MODIFIED Requirements

### Requirement: Selection summary
The inspector header SHALL show what is selected: the object's name and kind (name and kind icon) for a single object or group, or "N objects" for several. With nothing selected, the inspector SHALL show the active texture's properties instead:
- its name, its kind (main texture or accessory texture) and its size ("4096²", or "8192 × 1024" when its sides differ);
- when a package update flagged it, that it is **To check** (see the texture-status capability) and why: its layout changed in the vehicle's version, with a **Mark as Checked** action, or it is not in this version and is left out of the mod, with no action;
- a hint to select an object to set its layout, fill and stroke.

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
