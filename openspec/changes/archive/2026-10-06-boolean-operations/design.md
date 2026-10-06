## Context

- **Shapes:** every closed shape exposes its filled area in document space through `Object::fill_path()`: rectangles (rounded), ellipses and polygons as kurbo paths, paths as their closed subpaths. Paths are stored as `PathData` (cubic nodes, frame-local, see `vector-tools`). `PathData::from_bezpath` converts kurbo paths exactly.
- **Fill rule:** shapes are filled with the non-zero rule. Holes are opposite-wound subpaths, as in glyphs and outlined text.
- **`flo_curves` 0.8.1** offers `path_add`, `path_sub` and `path_intersect` on sets of Bézier paths. It keeps curves (graph-based, with arc intersection) and takes an `accuracy`. Its inputs are sets of closed paths read with the even-odd rule, which must not overlap themselves. `path_remove_overlapped_points` normalizes a self-overlapping set.

## Goals / Non-Goals

**Goals:**
- Exact, curve-preserving results.
- Never corrupt the document: failures leave it unchanged.
- One undo step.

**Non-Goals:**
- Live compound shapes.
- Divide, Trim and Merge.
- Stroke outlining.
- Supporting the even-odd vs non-zero difference for exotic self-overlapping same-direction subpaths beyond what normalization resolves.

## Decisions

### D1. Library: `flo_curves`

Alternatives considered:
- **`i_overlay`** (polygon booleans, very robust): rejected because it flattens curves into many segments, which go against editable results.
- **Writing our own:** too large for this change.

`flo_curves` is pure Rust and Apache-2.0. It's added to `tp-core` behind the `boolean` module, so the rest of the code never sees its types.

### D2. Operand preparation (`tp-core::document::boolean`)

For each operand:
1. Take the document-space filled area: `fill_path()`, or for a group, the fill paths of its visible shapes.
2. Convert each closed subpath to a `flo_curves` `SimpleBezierPath`: lines become cubics with control points at the thirds, quadratics are raised to cubics.
3. Shapes are first built the way Convert to Path builds them (an ellipse is 4 points), so results keep as few points as possible.
4. If no two subpaths cross (holes nested in outer contours, as in glyphs and outlines), the set goes in as it is: even-odd and non-zero agree. Only when subpaths cross is the set normalized, with `path_remove_interior_points`, which keeps everything they enclose. (`path_remove_overlapped_points` works like even-odd and would cut holes where same-direction subpaths overlap.)
5. A group is first united internally (successive `path_add`), so it acts as one shape.

### D3. Operations

The operands are sorted bottom to top (stacking order of `selected_objects`, resolved with tree positions):

| Operation | Computation |
|---|---|
| Unite | fold with `path_add` |
| Minus Front | bottom, then `path_sub` of the union of the others |
| Intersect | fold with `path_intersect` |
| Exclude | fold with `xor(a, b) = add(sub(a, b), sub(b, a))` |

The accuracy is 0.01 texture px.

### D4. Result back to a path

1. The `flo_curves` paths become kurbo paths, then `PathData::from_bezpath`.
2. Cubics whose control points sit on the chord (within 1e-6 relative) become straight segments, so straight edges stay straight.
3. Consecutive coincident nodes are merged.
4. `Object::from_path` fits an unrotated frame.

The library returns even-odd contours, but our renderer fills with the non-zero rule. Each result contour is therefore re-oriented by its nesting depth: outer contours one way, holes the other.

An empty result, or a result with no area, gives `BooleanError::Empty`. Panics inside the library are caught with `std::panic::catch_unwind`, which gives `BooleanError::Failed`: the library is complex and a malformed input must not take the app down.

### D5. Replacing the selection (`tp-app`)

`ws.combine(op, now)`:
1. Computes the result first; on an error it shows the hint and stops.
2. Otherwise, inside one `ws.edit(op label)`:
   - puts the result path in place of the topmost selected object (its parent and index), reusing its id;
   - removes the other selected objects;
   - selects the result.

**Style source:** the topmost object (bottom for Minus Front): fill, stroke, opacity, name, visibility and lock. The result reuses the topmost object's id, which keeps its place without a new insert-at-index operation. The originals come back with their ids on undo, through the snapshot.

### D6. Commands and availability

`CommandId::Combine(BooleanOp)`, with `BooleanOp { Unite, MinusFront, Intersect, Exclude }`. Unite is ⌘⇧U and Minus Front ⌘⇧-.

`EditContext.combine_block: Option<&'static str>` is computed from the selection: the first reason it can't combine, or none. The commands are enabled when it's none and at least two operands are selected. Disabled tooltips show the reason ("Create Outlines first", …), which needs a small extension: `Availability::When` gets an optional dynamic reason through a function `fn(&EditContext) -> &'static str`.

### D7. UI

- The Object › Combine submenu, after Align.
- A third row of four icon buttons in the Transform panel, through `CommandUi::icon_button`.
- Icons: `UNITE_SQUARE`, `SUBTRACT_SQUARE`, `INTERSECT_SQUARE`, `EXCLUDE_SQUARE`.

## Risks / Trade-offs

- [Robustness of curve booleans on tangent or coincident edges, a known weak spot of such libraries] → `catch_unwind`, plus validation of the result: finite coordinates, and a 96 × 96 sampling of the operands' bounds. At each sample, inside the result (non-zero) must match the expected boolean of the operands' own non-zero fills, with at most 1 % of samples disagreeing. An empty result is only accepted when the expected area is empty too (at most 0.2 %). On doubt the document is left unchanged with "These shapes couldn't be combined". The property-style tests include coincident edges (two squares sharing a side) and tangent circles.
- [Even-odd vs non-zero] → Normalization handles the common overlap cases. Exotic same-direction nested subpaths inside one path may be read as holes; this is documented and rare in practice.
- [Point counts on complex results] → The simplification step (D4) avoids splitting straight edges into curves. Curves keep the library's segmentation, which is close to the inputs'.

