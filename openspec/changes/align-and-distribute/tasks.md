## 1. Geometry (tp-core)

- [x] 1.1 Add `document::align` with `Edge`, `DistributeMode`, `align` and `distribute` (stable ordering, outermost fixed, one-axis moves through `translate`). Verify unit tests for:
  - "Align left edges";
  - rotated bounds ("Rotated objects align by what is seen");
  - groups moving as a whole;
  - "Equal gaps" (300 px, middle object at x = 400);
  - centers evenly spaced;
  - overlapping objects (negative gap);
  - a path moving exactly.

## 2. Commands and menu (tp-app)

- [x] 2.1 Add `CommandId::Align(Edge)` and `CommandId::Distribute(..)` with labels, icons, shortcuts and availability (`selection_count` in `EditContext`), plus `PanelState.align_to` and target resolution (selection, artboard, key object). Dispatch them as one undo step each. Verify that the command registry's conflict test passes, and unit tests of target resolution for every mode.
- [x] 2.2 Add the Object › Align submenu (six aligns, a separator, four distribute commands). Verify the kittests:
  - "Center a logo on the texture" (from the menu);
  - "Several objects to the artboard";
  - "Not enough objects";
  - "Nothing selected" (disabled items);
  - shortcuts Alt+A and Alt+Shift+H applied on the canvas, each one undo step.

## 3. Transform panel and key object

- [x] 3.1 Add the align row with the Align to combo box and the distribute row to the Transform panel, through the command registry (tooltip with shortcut, disabled state). Verify the kittests "Align from the panel" and the disabled distribute buttons with two objects.
- [x] 3.2 Make the key object mode work: the last selected object is the target and doesn't move, and the canvas draws it with a stronger outline. Verify the kittest "Align to a key object" (Shift+click order) and a screenshot with the highlight.

## 4. Integration

- [x] 4.1 Add the screenshots `render_align` (the Transform panel with the new rows, and a key-object highlight) at 100 % and 200 % UI scale. Review the images.
- [x] 4.2 Run `mise run checks`, the full test suite and the stress run (4 parallel × 5 rounds of the canvas, panels, precision and new align binaries). Verify everything passes.
