## 1. Geometry (tp-core)

- [x] 1.1 Add the `flo_curves` dependency to `tp-core` (check the build on all targets in CI). Add `document::boolean` with conversions between kurbo and `flo_curves` paths (lines, quads, cubics, closed subpaths) and the operand preparation (D2), including group union and normalization. Verify unit tests: a round trip of a rounded rectangle is exact within 0.01 px; a glyph-like shape with a hole keeps its hole.
- [x] 1.2 Implement `combine(objects, op)` for Unite, Minus Front, Intersect and Exclude (D3), the result simplification (D4) and the errors: empty result, and failure with `catch_unwind` plus the area check. Verify unit tests:
  - two 2×2 squares offset by (1, 1) united have area 7;
  - a notch cut with Minus Front;
  - Exclude of nested squares has a hole;
  - Intersect of separate circles is empty;
  - two circles united have at most 8 points, within 0.05 px;
  - coincident edges and tangent circles don't fail or corrupt;
  - straight edges stay straight segments.

## 2. Commands (tp-app)

- [x] 2.1 Add `BooleanOp`, `CommandId::Combine` with labels, icons and shortcuts (⌘⇧U, ⌘⇧-), `EditContext.combine_block` and the dynamic disabled reasons (D6). Add `ws.combine` (D5: compute first, hint on error, one edit, place and style of the result). Verify:
  - the shortcut conflict test;
  - unit tests "Style of Minus Front", the result's place in a parent group, and undo restoring the original ids;
  - "No overlap" leaves the document unchanged with the hint.
- [x] 2.2 Add the Object › Combine submenu and the Transform panel row. Verify kittests:
  - "Unite two overlapping squares" (menu);
  - "Cut a notch" (⌘⇧-);
  - "Exclude makes holes" (pixel: the hole shows the background in an export);
  - "Text in the selection" (disabled, reason);
  - "Groups count as one shape";
  - "Unite from the panel";
  - "Undo".

## 3. Integration

- [x] 3.1 Add the `render_combine` screenshots (the Transform panel with the combine row, and a notch result) at 100 % and 200 % UI scale. Review the images.
- [x] 3.2 Run `mise run checks`, the full test suite and the stress run (4 parallel × 5 rounds of the new combine, align, vector_tools and panels binaries). Verify everything passes.
