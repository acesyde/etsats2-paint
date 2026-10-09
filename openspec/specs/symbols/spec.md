# symbols Specification

## Purpose

Lets a project define a drawing once, a logo, a lettering or stripes, as a symbol, and place it as instances on every texture of its fleet. Each instance has its own position, size and rotation, and every instance follows the symbol's edits.

## Requirements

### Requirement: Symbols and instances
A project SHALL hold an ordered list of **symbols**, starting empty. Each symbol has:
- a stable identity;
- a name, which is not empty and unique among the project's symbols;
- its content: any shapes, paths, texts, images and groups, drawn on the symbol's own square artboard.

An **instance** is an object of a texture that shows a symbol's content. The instance's own properties are:
- its placement: position, size (horizontal and vertical scale), rotation and flips;
- its opacity, visibility, lock and name.

An instance SHALL show the symbol's current content through its placement, and nothing else: its look can't be changed in place. Editing a symbol SHALL update every instance of it, on every texture, at once. A symbol SHALL NOT contain instances.

#### Scenario: One logo on two textures
- **WHEN** the symbol "Logo" has an instance on the Standard cab, at 100%, and one on the Chassis, at 50% and rotated by 90°
- **THEN** each texture shows the logo at its own position, size and rotation

#### Scenario: Editing the symbol updates every instance
- **WHEN** the player changes the color of a shape inside "Logo"
- **THEN** both instances show the new color

### Requirement: Convert to Symbol
**Object › Convert to Symbol** SHALL turn the selected objects into a new symbol:
- it is named "Symbol 1", "Symbol 2"…, the first unused number;
- its content is the selected objects, in stacking order, placed on the symbol's artboard so that the artboard just encloses them;
- the selected objects are replaced, at the same stacking position and inside the same group, by one instance of the new symbol. The instance shows them exactly where they were;
- the instance is selected;
- it is one undo step.

Instances inside the selection SHALL be detached first: their content goes into the new symbol as plain objects. The command SHALL be disabled with nothing selected, with a tooltip saying why.

#### Scenario: Convert a logo
- **WHEN** the player selects a circle and a text, and chooses Convert to Symbol
- **THEN** the symbol "Symbol 1" holds the circle and the text, the texture shows one instance in their place, selected, and the Symbols section of the Resources tab lists "Symbol 1" with 1 instance

#### Scenario: Undo a conversion
- **WHEN** the player presses Undo right after Convert to Symbol
- **THEN** the circle and the text are back as plain objects, and the project has no symbol

### Requirement: Symbols lists
The project's symbols SHALL be listed in two places: the **Symbols** section of the Workshop's **Resources** tab, and the **Symbols** section of the Brand space (see the brand-space capability). Both SHALL list the project's symbols, each with its name and its number of instances in the project. With no symbol, the Resources tab SHALL show a short explanation of what symbols are and how to make one, with a **Convert to Symbol** button.

Each symbol SHALL offer:
- **Place:** adds an instance at 100% in the middle of the visible part of the active texture, and selects it. Dragging the symbol's row of the Resources tab onto the canvas places it at the drop point;
- **Edit**, which opens the symbol's view (see Editing a symbol);
- **Rename**, inline;
- **Duplicate**: a new symbol "<name> copy" with the same content and no instance;
- **Delete**: when the symbol has instances, a confirmation names their number first. Each instance then becomes a group with the same look, at the same place. An unused symbol is deleted at once.

Each change SHALL be one undo step.

#### Scenario: Place on another texture
- **WHEN** the player activates the Chassis and clicks Place on "Logo" in the Resources tab
- **THEN** an instance of "Logo" is added to the Chassis at 100%, in the middle of the view, and selected

#### Scenario: Delete a used symbol
- **WHEN** the player deletes "Logo", which has three instances, and confirms
- **THEN** the three instances become groups that look the same, "Logo" is gone, and Undo restores the symbol and its instances

#### Scenario: Duplicate for a variant
- **WHEN** the player duplicates "Logo"
- **THEN** the symbol "Logo copy" appears with the same content and no instance, and editing it doesn't change "Logo"

#### Scenario: Instance count in the Resources tab
- **WHEN** "Logo" has six instances across the fleet
- **THEN** the Symbols section of the Resources tab lists "Logo" with "6 instances"

### Requirement: Editing a symbol
**Edit Symbol** SHALL show the symbol alone on the canvas of the Workshop, on its own artboard, with every tool, the rulers, the guides of the symbol and the Layers tab listing its content. It is reached from:
- a double-click on an instance;
- Object › Edit Symbol, with one instance selected;
- **Edit Symbol** in the inspector, with one instance selected;
- Edit on the symbol in the Resources tab or in the Brand space. From Brand, the Workshop is shown.

While a symbol is edited:
- a bar above the canvas SHALL say "Editing symbol <name>" and offer **Done**;
- the breadcrumb SHALL show "Symbol › <name>";
- the Textures tab and the Project space SHALL highlight no texture;
- commands that need a texture SHALL be disabled with a tooltip saying why: the template, Copy From Cabin…, Next and Previous Texture, and Export Texture…;
- every recorded change SHALL update the symbol's instances at once.

**Done**, Escape (when nothing is selected, drawn or typed: a first Escape clears the selection, as on a texture) and choosing a texture in the Textures tab or the Project space SHALL return to a texture: the one the symbol was opened from, or the chosen one. Undo and Redo SHALL work across both views. Undoing a change made in a symbol SHALL show the symbol again.

#### Scenario: Edit from an instance
- **WHEN** the player double-clicks an instance of "Logo" on the Standard cab, recolors a shape, and clicks Done
- **THEN** the canvas shows the Standard cab again, and every instance of "Logo" on every texture shows the new color

#### Scenario: Undo across views
- **WHEN** the player edits "Logo", returns to the texture, and presses Undo
- **THEN** the symbol's last change is undone, the instances show the previous content, and the canvas shows "Logo" being edited

#### Scenario: Edit from the inspector
- **WHEN** one instance of "Logo" is selected and the player clicks Edit Symbol in the inspector
- **THEN** the canvas shows "Logo" being edited and the breadcrumb reads "Symbol › Logo"

### Requirement: Detach Instance
**Detach Instance** (Object menu, and the inspector's header with one or more instances selected) SHALL replace each selected instance with a group of plain objects that look the same, at the same place. The group keeps the instance's name, opacity, visibility and lock. The symbol and its other instances SHALL be unchanged. It is one undo step.

#### Scenario: A one-off variant
- **WHEN** the player detaches the instance of "Logo" on the trailer and recolors one of its shapes
- **THEN** only that trailer's logo changes, and the other instances still follow "Logo"

#### Scenario: Detach from the inspector
- **WHEN** one instance of "Logo" is selected and the player clicks Detach Instance in the inspector's header
- **THEN** the instance becomes a group of plain objects that look the same, and the inspector no longer offers Edit Symbol

### Requirement: Instances in the editor
An instance SHALL behave as one object:
- **Selection:** clicking any part of it selects the instance. Its content can't be selected on the texture;
- **Transforms:** it is moved, resized, rotated, flipped, nudged, aligned, distributed, duplicated, copied, pasted, grouped, reordered and copied from cabin like other objects. Its placement follows;
- **Layers tab:** it is one row with a symbol icon, its name and no expander, the icon and name drawn in the link color. Rename renames the instance;
- **Inspector:** its header shows the symbol's name in the link color, **Edit Symbol** and **Detach Instance**, and the inspector shows the instance's opacity;
- **Look:** fill and stroke colors, swatches, styles, stroke settings, Direct Selection, Convert to Path, Create Outlines and Combine SHALL NOT apply to it. With only instances selected, the inspector shows no Fill and Stroke rows and says the look is edited in the symbol. Commands that would change it are disabled, with a tooltip saying so. In a mixed selection, they apply to the other objects only;
- **Rendering:** it is drawn, exported, hit-tested and snapped to like a group of its content.

The link color SHALL NOT be the only sign of an instance: the symbol icon and the symbol's name are shown too.

Swatches and styles used inside a symbol SHALL follow their edits like any object, and the instances show them. An image inside a symbol SHALL count as one use of its asset in the images lists, whatever the number of instances. An instance pasted into a project without its symbol SHALL become a group.

#### Scenario: Resize an instance
- **WHEN** the player drags a corner handle of an instance to double its size
- **THEN** the instance is twice as large, and its content keeps its proportions and look

#### Scenario: Recolor a swatch used in a symbol
- **WHEN** a shape inside "Logo" is linked to the swatch "Company red", and the player edits "Company red" to dark red
- **THEN** every instance of "Logo" shows dark red

#### Scenario: Colors don't apply to an instance
- **WHEN** only an instance is selected
- **THEN** the inspector says the look is edited in the symbol, and Convert to Path is disabled with a tooltip saying to edit or detach the symbol

#### Scenario: Instance in the Layers tab
- **WHEN** an instance of "Logo" is on the active texture
- **THEN** its row in the Layers tab shows the symbol icon and its name in the link color
