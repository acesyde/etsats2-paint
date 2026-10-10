## MODIFIED Requirements

### Requirement: Symbols lists
The project's symbols SHALL be listed in two places: the **Symbols** section of the Workshop's **Resources** tab, and the **Symbols** section of the Brand space (see the brand-space capability). Both SHALL list the project's symbols, each with its name and its number of instances in the project; the Brand space SHALL also show the number of textures holding them, as "14 instances · 9 textures", or "Unused" (see the brand-impact capability). With no symbol, the Resources tab SHALL show a short explanation of what symbols are and how to make one, with a **Convert to Symbol** button.

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

#### Scenario: Instances and textures in Brand
- **WHEN** "Logo" has four instances on the Cab and two on the Chassis
- **THEN** the Symbols section of Brand lists "Logo" with "6 instances · 2 textures"
