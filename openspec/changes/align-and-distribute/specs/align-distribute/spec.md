## Purpose

Lets users line up and evenly space the selected objects, or center them on the texture, in one action instead of typing coordinates object by object.

## ADDED Requirements

### Requirement: Align commands
The application SHALL provide six align commands:

| Command | Effect | Shortcut |
|---|---|---|
| Align Left | moves each object so its left edge is on the target's left edge | Alt+A |
| Align Horizontal Centers | aligns centers horizontally | Alt+H |
| Align Right | aligns right edges | Alt+D |
| Align Top | aligns top edges | Alt+W |
| Align Vertical Centers | aligns centers vertically | Alt+V |
| Align Bottom | aligns bottom edges | Alt+S |

Each command SHALL move the selected objects along one axis only, measuring each object, or each group as a whole, by its visible axis-aligned bounds. Objects SHALL keep their size, rotation and stacking order. Each command SHALL be one undo step labelled with its name. The commands SHALL be listed in an Object › Align submenu.

#### Scenario: Align left edges
- **WHEN** rectangles with left edges at x = 100, 300 and 500 are selected and the user chooses Align Left
- **THEN** all three left edges are at x = 100, their y positions are unchanged, and one Undo restores them

#### Scenario: Rotated objects align by what is seen
- **WHEN** a square rotated by 45° and an unrotated square are aligned to the top
- **THEN** the top corner of the rotated square and the top edge of the other are at the same y

### Requirement: Align to
The align commands SHALL align to a target chosen with "Align to":

- **Selection** (default): the common bounds of the selected objects; with a single selected object, the artboard.
- **Artboard**: the artboard (the whole texture).
- **Key object**: the bounds of the most recently selected object, which does not move. While Key object is chosen and at least two objects are selected, the key object SHALL be outlined more strongly on the canvas.

The Align to choice SHALL be kept until the application quits.

#### Scenario: Center a logo on the texture
- **WHEN** one logo is selected (Align to: Selection) on a 4096 px texture and the user chooses Align Horizontal Centers
- **THEN** the logo's center is at x = 2048

#### Scenario: Align to a key object
- **WHEN** Align to is Key object, a stripe is selected and then a logo is added to the selection with Shift+click, and the user chooses Align Top
- **THEN** the stripe's top moves to the logo's top and the logo does not move

#### Scenario: Several objects to the artboard
- **WHEN** Align to is Artboard and two objects are selected and the user chooses Align Bottom
- **THEN** both bottoms are at the artboard's bottom edge

### Requirement: Distribute and equal spacing
The application SHALL provide four commands, enabled when at least three objects are selected:

| Command | Effect | Shortcut |
|---|---|---|
| Distribute Horizontal Centers | spaces the objects' horizontal centers evenly | |
| Distribute Vertical Centers | spaces the vertical centers evenly | |
| Distribute Horizontal Spacing | makes the horizontal gaps between neighboring objects equal | Alt+Shift+H |
| Distribute Vertical Spacing | makes the vertical gaps equal | Alt+Shift+V |

Objects SHALL be ordered by position along the axis. The first and last SHALL not move; the others move along that axis only. The commands SHALL use the selection's own extent whatever the Align to choice. Each command SHALL be one undo step and SHALL be listed in the Object › Align submenu.

#### Scenario: Equal gaps
- **WHEN** three objects 100, 200 and 100 px wide are selected with left edges at x = 0, 150 and 900, and the user chooses Distribute Horizontal Spacing
- **THEN** the gaps between them are both 300 px (the middle object's left edge is at x = 400), and the first and last objects have not moved

#### Scenario: Not enough objects
- **WHEN** only two objects are selected
- **THEN** the distribute commands are disabled

### Requirement: Availability
Align commands SHALL be enabled when at least one editable object is selected, or two when Align to is Key object, and no canvas gesture or text editing is in progress. Distribute commands SHALL require at least three. Disabled commands SHALL explain why in their tooltip.

#### Scenario: Nothing selected
- **WHEN** nothing is selected
- **THEN** every align and distribute command is disabled
