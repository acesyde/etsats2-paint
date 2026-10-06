## 1. Guides in the document

- [x] 1.1 Add `Guide`, `Axis` and `Surface.guides` with the `Project` operations (add, move, remove, clear), and capture the guides in `Snapshot` (`same_document` compares them). Verify unit tests: a new project has no guides; snapshot and restore round-trips guides; `same_document` detects a moved guide.
- [x] 1.2 Add the optional `guides` field to v2 `FileSurface` (empty from `from_v1`) and regenerate the v2 fixture with two guides. Verify the format round trip with guides, that a v2 document without the field still opens, and the fixture test.

## 2. Preferences and commands

- [x] 2.1 Add `ViewAids` to `Prefs` (grid, guides, snapping, grid_spacing; `serde(default)`) and expose it to the workspace each frame. Verify that a preferences file without the field loads with the defaults and that the values round-trip through `PrefsStore`.
- [x] 2.2 Enable Show Grid, Show Guides and Snapping as toggles with check marks in the View menu, and add Clear Guides (disabled without guides, one undo step). Verify kittests: ⌘' toggles the grid and its check mark; "Hide guides"; Clear Guides is disabled on an empty surface and then undoable.
- [x] 2.3 Add the Grid spacing field (4–1024) to the Preferences dialog and include it in Reset to defaults. Verify the kittest "Coarser grid", and that Reset to defaults restores 64.

## 3. Rulers, grid and guides on the canvas

- [x] 3.1 Split the canvas area into the rulers, the corner and the inner canvas (fit and viewport use the inner rect), and paint graduations, labels and the pointer marker. Verify unit tests for the step choice (labels ≥ 60 pt apart at several zooms), the kittest "Pointer marker" through the ruler geometry, and that the existing canvas tests still pass.
- [x] 3.2 Paint the grid (clipped to the artboard, power-of-two thinning, major lines every 8) and the guides in the paint order of D8. Verify a unit test for the thinning step ("Zoomed out": ≥ 8 pt), and that exports contain no grid or guide lines (pixel check of an export with both shown).
- [x] 3.3 Add guide gestures: drag from a ruler to create, grab within 4 pt (objects' filled shapes win), move, drop on a ruler or outside to delete, resize cursors, the position label, and hidden guides not grabbable. Verify kittests "Horizontal guide", "Released back on the ruler", "Move a guide" (one Undo restores it), "Delete by dropping on a ruler", and that a press inside a rectangle on a guide moves the rectangle.

## 4. Snapping

- [x] 4.1 Add the `snap` module: build targets once per gesture (guides, artboard, grid on demand, other objects' edges and centers, anchor points and bounds corners as point targets; edited and hidden objects excluded), with `snap_axis` and `snap_point`, nearest within 6 pt, and hits for painting. Verify unit tests: the nearest target wins; out-of-tolerance values don't snap; the grid uses the configured spacing; edited objects are excluded; point targets win over axis targets; behavior with 5000 objects stays under 1 ms per query (release).
- [x] 4.2 Snap moving (bounds candidates, free-axis rule with Shift), resizing (pointer, not when proportional), drawing (press point and pointer, not the pointer when constrained), the Pen, and Direct Selection points and handles. Honor the Snapping toggle and Cmd/Ctrl held mid-drag. Verify kittests "Snap to a guide", "Snap to the artboard center", "Hidden grid does not snap", "Drawing from a grid intersection", "Pen point on another path's point", "Turn snapping off" and "Temporary bypass".
- [x] 4.3 Make guides snap while created or moved (along their axis, excluding themselves). Verify a kittest: a dragged guide snaps to the artboard center.
- [x] 4.4 Paint the alignment lines (extent between the element and the target, full canvas for the grid, guides and artboard) and the crosses for snapped points; clear them when a gesture ends. Verify the kittest "Smart guide between two objects" (the hit is recorded while dragging and cleared after) and a screenshot.

## 5. Integration

- [x] 5.1 Add the `render_precision_aids` screenshots: rulers, grid, guides, a snapped move with its alignment lines, at 100 % and 200 % UI scale. Review the images.
- [x] 5.2 Re-run the panning benchmarks with the grid and guides shown (1000 shapes; 50 stars and 50 paths) and record the timings. Verify they stay under 16 ms per frame in release.
- [x] 5.3 Run `mise run checks`, the full test suite and the stress run (4 parallel × 5 rounds of the canvas, panels, persistence, vector_tools and new precision binaries). Verify everything passes.
