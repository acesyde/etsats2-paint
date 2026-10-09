# path-booleans Specification

## Purpose

Lets users combine shapes into one clean path — merge, cut out, intersect or exclude — to build stripes, notches, shields and logos from simple pieces.

## Requirements

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

### Requirement: Operands
Rectangles (including rounded corners), ellipses, polygons and stars, closed paths, and groups made only of these SHALL be valid operands. A group SHALL count as one shape: the union of its visible shapes. Only the filled area SHALL be combined: strokes are not part of it. Hidden objects inside groups SHALL be ignored.

The commands SHALL be enabled when at least two valid operands are selected and nothing else is selected. Otherwise, they SHALL be disabled, with a reason:
- for texts: "Create Outlines first";
- for images: images can't be combined;
- for open paths: lines have no area;
- with fewer than two operands: select at least two shapes.

#### Scenario: Text in the selection
- **WHEN** a rectangle and a text are selected
- **THEN** the combine commands are disabled and their tooltip suggests Create Outlines

#### Scenario: Groups count as one shape
- **WHEN** a group of two circles and a rectangle are selected and the user chooses Intersect
- **THEN** the result is the area where the rectangle overlaps either circle

### Requirement: The result
The result SHALL be one path, placed in the layers where the topmost selected object was (same parent group), and selected. The selected objects SHALL be removed. The result SHALL take the fill, stroke, opacity and name of the topmost selected object, or of the bottom one for Minus Front.

The geometry SHALL be exact within 0.05 texture pixels:
- curved edges stay curves with few points;
- straight edges are straight segments;
- holes are separate subpaths that are not filled.

Each command SHALL be one undo step named after it, and Undo SHALL restore the original objects with their identities.

#### Scenario: Curves stay editable
- **WHEN** two overlapping circles are united
- **THEN** the result has at most 8 anchor points, and its outline is within 0.05 px of the two circles' outer arcs

#### Scenario: Style of Minus Front
- **WHEN** a blue stripe below a red circle is combined with Minus Front
- **THEN** the result is blue and named like the stripe

#### Scenario: Undo
- **WHEN** the user presses Cmd/Ctrl+Z after Unite
- **THEN** the original shapes are back with their identities and the result is gone

### Requirement: Empty or failed results
When the result has no area (for example Intersect on shapes that do not overlap, or Minus Front that removes everything), or cannot be computed, the document SHALL be left unchanged and a short hint SHALL explain it ("The shapes don't overlap", "Nothing would remain", "These shapes couldn't be combined").

#### Scenario: No overlap
- **WHEN** two separate circles are selected and the user chooses Intersect
- **THEN** both circles are unchanged and a hint says the shapes don't overlap
