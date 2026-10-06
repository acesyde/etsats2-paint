## 1. Document model (tp-core)

- [x] 1.1 Add `kurbo` to the workspace and create `tp-core::document` with `Project`, `Surface`, `Object`, `ShapeKind`, `Frame`, `Rgba`, `StrokeStyle`, IDs and the default appearance; replace `ProjectStub` (migrate `state.rs`, `dialogs.rs`, `tests/ui.rs`); verify unit tests for new-project defaults ("Main texture", size, "Untitled" name) and that `mise run ci` passes
- [x] 1.2 Implement geometry: frame affine, local outlines (rectangle, rounded rectangle, ellipse), bounds and flattening; verify unit tests for rotated bounds and flattened point counts within tolerance
- [x] 1.3 Implement hit testing (topmost-first, local-space `contains`, screen-pixel tolerance) and marquee intersection; verify unit tests for ellipse corner miss, rounded corner miss, rotation, stacking order and tiny-object tolerance
- [x] 1.4 Implement object operations: add, delete, duplicate (+20 px), reorder forward/backward, minimum size clamp; verify unit tests for each, including stable IDs across moves and reorders
- [x] 1.5 Implement transforms: move, resize from anchor (Shift proportional, Alt from center, flip past anchor), rotate around pivot (Shift 15° snap), for single and multiple objects; verify unit tests from the selection-transform spec scenarios (200×100 → 400×200, center kept with Alt, snapped angles, multi-object move/rotate)
- [x] 1.6 Implement `History` with snapshots, labels, 200-step bound, redo clearing and nudge coalescing (1 s window); verify unit tests for each undo-history scenario

## 2. Viewport and navigation (tp-app)

- [x] 2.1 Implement `Viewport` (center, zoom, fitted flag) with screen↔texture mapping, zoom at point, clamp 2%–6400%, presets, fit with margin and refit-on-resize behavior; verify unit tests for pointer invariance, clamping, presets and the two Canvas area resize scenarios
- [x] 2.2 Disable egui `zoom_with_keyboard`, map raw wheel and pinch events (line → zoom, point → pan, Cmd/Ctrl → zoom, Shift+wheel → horizontal pan) and enable Zoom In/Out, Fit to Screen, Actual Size commands; verify kittests injecting line and point wheel events and that Cmd/Ctrl+= changes the canvas zoom but not `ctx.zoom_factor()`
- [x] 2.3 Implement panning via Hand tool, Space+drag and middle-drag with grab/grabbing cursors; verify a kittest that Space+drag with the Rectangle tool pans without creating a shape and restores the tool
- [x] 2.4 Implement the Zoom tool (click in, Alt+click out, drag-to-area) with zoom cursors; verify a kittest for drag-to-area fitting the dragged rectangle
- [x] 2.5 Show the live zoom in the status bar and keep pointer coordinates correct at any zoom/pan; verify a kittest after zooming and panning

## 3. Canvas rendering

- [x] 3.1 Rewrite the canvas widget around the viewport: pasteboard, artboard, objects as convex polygons with fill, stroke and opacity, clipped to the canvas; verify a screenshot of a scene with rotated, rounded, stroked and semi-transparent shapes
- [x] 3.2 Add the flattened-outline cache (Arc pointer + id + frame + zoom bucket, pruned per frame); verify a unit test that panning reuses cached geometry and the ignored release-mode perf check with 1000 shapes
- [x] 3.3 Add overlay tokens to `tp-ui` and draw hover outline, selection bounds (oriented for single, axis-aligned for multi), handles, center mark and per-object outlines with halo, at constant screen size; verify screenshots at 25% and 400% zoom

## 4. Tools and gestures

- [x] 4.1 Implement the gesture state machine with drag threshold, hit-test priority (rotation zone > handles > objects > empty) and Escape cancel restoring the start state; verify unit tests of press classification
- [x] 4.2 Implement the Rectangle and Ellipse tools (live preview, Shift square/circle, Alt from center, no shape on click, stay active, select result, crosshair cursor); verify kittests for the shape-tools scenarios
- [x] 4.3 Implement selection with the Selection and Move tools: click, Shift+click toggle, click empty to clear, marquee (Shift additive), Select All, Escape, hover outline; verify kittests for the Shift+click and marquee scenarios
- [x] 4.4 Implement move, resize and rotate gestures with live modifiers, direction-aware resize cursors, drawn rotate cursor and the value label near the pointer; verify kittests for drag-move of a multi-selection, proportional resize and snapped rotation
- [x] 4.5 Show the "not available yet" canvas hint for Polygon, Pen, Line, Text, Image, Eyedropper and Direct Selection without modifying the document; verify a kittest for the Pen tool scenario

## 5. Commands and history wiring

- [x] 5.1 Replace static availability with `EditContext`-based `is_enabled` and dynamic Undo/Redo labels; verify unit tests for enabled states and a kittest that the Edit menu shows "Undo Resize" after a resize
- [x] 5.2 Wire Undo/Redo (incl. Cmd/Ctrl+Y), commit one history entry per gesture, restore selection, mark the project unsaved; verify kittests for "undo a creation", "zoom is not undone" and "drag is a single step"
- [x] 5.3 Wire Delete (+ Backspace), Duplicate, Cut/Copy/Paste with the application clipboard and repeated-paste offset, Bring Forward/Send Backward, and arrow-key nudges (1 px / Shift 10 px, coalesced); verify kittests for duplicate offset, paste disabled when empty, and nudge with Shift
- [x] 5.4 Update the canvas context menu with the now-active commands; verify a kittest that right-clicking a selected object offers Cut, Copy, Duplicate, Delete, Bring Forward, Send Backward

## 6. Integration

- [x] 6.1 Update the screenshot test with a canvas scene and review images at 100% and 200% UI scale
- [ ] 6.2 Run `openspec validate canvas-core --strict` and `mise run ci`; verify both pass locally and the CI matrix is green on the PR
