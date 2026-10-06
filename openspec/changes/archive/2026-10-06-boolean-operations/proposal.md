## Why

Livery artwork is built from combined shapes: a stripe with a notch cut out, a logo merged from several pieces, lettering cut by a swoosh, a shield made by intersecting two curves. Today these combinations can only be faked by stacking shapes in the background color, which breaks as soon as the background changes or the texture is exported with transparency. Real boolean operations produce one clean path. With Create Outlines, they also work on lettering.

## What Changes

- **Four commands, Illustrator Pathfinder style**, in a new **Object › Combine** submenu and as buttons in the Transform panel:
  - **Unite**: merge all selected shapes into one (⌘⇧U / Ctrl+Shift+U);
  - **Minus Front**: cut every upper shape out of the bottom one (⌘⇧- / Ctrl+Shift+-);
  - **Intersect**: keep only the area where all shapes overlap;
  - **Exclude**: keep the areas covered by an odd number of shapes (overlaps become holes).
- **The result is one plain path**:
  - it replaces the selected objects at the topmost one's place in the layers;
  - it takes the style (fill, stroke, opacity, name) of the topmost object, or of the bottom object for Minus Front, as in Illustrator;
  - curves stay curves: the result has few, editable points;
  - holes come out as holes.
- **Operands:** rectangles, ellipses, polygons, closed paths, and groups of them (a group counts as one shape). Commands are enabled with at least two such objects. Texts (outline them first), images and open lines can't be combined, and the commands explain why.
- **Robustness:** if the result is empty (e.g. Intersect on shapes that don't overlap) or can't be computed, nothing changes and a hint explains why.
- **Undo:** each operation is one undo step, which restores the original objects.

Not in this change: live, re-editable compound shapes; Divide, Trim and Merge (cutting into separate pieces); booleans on open lines or on strokes (Outline Stroke).

## Capabilities

### New Capabilities

- `path-booleans`: the Unite, Minus Front, Intersect and Exclude commands. Covers valid operands, the resulting path (geometry, style, place), empty or failed results, availability and undo.

### Modified Capabilities

- `transform-panel`: a row of combine buttons is added under the align rows.

## Impact

- **New dependency:** `flo_curves` 0.8.1 (Apache-2.0, pure Rust). It provides boolean arithmetic on Bézier paths, so curves are kept.
- **`tp-core`:** a `boolean` module with `combine(objects, op) -> Result<PathData, BooleanError>` (document space):
  - conversion between kurbo paths and `flo_curves` paths;
  - normalizing operands (overlapping subpaths resolved);
  - simplifying the result (straight segments made straight, duplicate points removed).
- **`tp-app`:**
  - `CommandId::Combine(BooleanOp)` with shortcuts and availability (`EditContext` counts combinable objects and reports the first reason it's disabled);
  - the Object › Combine submenu and the Transform panel buttons;
  - the replacement in the document, done as one edit.
- **`tp-ui`:** icons for the four operations (Phosphor `UNITE`, `SUBTRACT`, `INTERSECT`, `EXCLUDE`).
