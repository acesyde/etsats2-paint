## MODIFIED Requirements

### Requirement: Combine commands
The application SHALL provide four commands in an Object › Combine submenu:

| Command | Result | Shortcut |
|---|---|---|
| Unite | the area covered by any selected shape | ⌘⇧U / Ctrl+Shift+U |
| Minus Front | the bottom shape minus the area of every shape above it | ⌘⇧- / Ctrl+Shift+- |
| Intersect | the area covered by every selected shape | |
| Exclude | the area covered by an odd number of the selected shapes | |

"Bottom" and "above" refer to the stacking order in the Layers tab.

#### Scenario: Unite two overlapping squares
- **WHEN** two overlapping squares are selected and the user chooses Unite
- **THEN** they are replaced by one closed path outlining both, with no internal edges

#### Scenario: Cut a notch
- **WHEN** a circle overlaps the right edge of a stripe drawn below it and the user chooses Minus Front
- **THEN** the stripe becomes one path with a round notch, and the circle is gone

#### Scenario: Exclude makes holes
- **WHEN** a small square fully inside a large one is selected and the user chooses Exclude
- **THEN** the result is the large square with a hole where the small one was
