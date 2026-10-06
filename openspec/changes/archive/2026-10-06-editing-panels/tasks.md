## 1. Document model (tp-core)

- [x] 1.1 Add `name`, `visible`, `locked`, `children` to `Object` and `ShapeKind::Group`, with default names and `refresh_group_frame` (oriented bounds of visible children in the group rotation); verify unit tests for defaults and group frames, and that existing tests still pass
- [x] 1.2 Implement `document::tree` (find path, get, parent, tree-aware replace with ancestor refresh, remove, insert, move_to with cycle rejection, group, ungroup, effective flags, draw list with accumulated opacity, hit test returning top/inner skipping hidden/locked, top-level marquee); verify unit tests for each, including round-trips group→ungroup and the "Group drawing order" scenario
- [x] 1.3 Make transforms recursive over children and normalize selections (no object with its ancestor); verify unit tests that moving/rotating/resizing a group moves its children consistently and refreshes frames
- [x] 1.4 Make duplicate, delete, paste and bring forward/send backward work within each object's parent list; verify unit tests with nested groups
- [x] 1.5 Add `Project.palette` (ordered, no duplicates) and color conversions (`Hsva`, `Hsla`, hex parse/format for #RGB, #RRGGBB, #RRGGBBAA); verify unit tests for round-trips and invalid hex

## 2. Design-system widgets (tp-ui)

- [x] 2.1 Implement `NumericField` (scrubbable label with Shift/Alt steps, edit buffer, Enter/Tab/focus-loss commit, Escape revert, "Mixed" placeholder, suffix, clamp); verify kittests for commit, revert and scrub events
- [x] 2.2 Implement `ColorSwatch` (checkerboard, None and Mixed patterns), `FillStrokeSwatches` and `ToggleIconButton`; verify kittests for click responses and accessible labels
- [x] 2.3 Implement `SvSquare`, `HueSlider`, `AlphaSlider` as mesh gradients with handles and keyboard arrows; verify kittests that dragging reports changed values and drag-stopped
- [x] 2.4 Add all new widgets to the design gallery; verify by screenshot review

## 3. Editing infrastructure (tp-app)

- [x] 3.1 Add workspace UI state (current style, color target, picker HSV, color model, proportions lock, expanded groups, layers drag state) and `live_edit` / `commit_pending` / `cancel_pending`; commit pending edits before commands and canvas gestures; verify unit tests that a multi-step live edit is one history entry and cancel restores
- [x] 3.2 Extend `EditContext` and commands: Group, Ungroup, New Layer, Duplicate Layer, Delete Layer, X / Shift+X / D color commands, Eyedropper tool available; verify unit tests for enabled states and the shortcut-collision test
- [x] 3.3 Add `recent_colors` to `Prefs` with serde default and a 12-entry cap; verify a prefs round-trip test and that an existing version-1 file without the field still loads

## 4. Canvas integration

- [x] 4.1 Draw from `draw_list()` (hidden skipped, opacity accumulated) and hit-test with `Hit { top, inner }`: click selects top-level, Cmd/Ctrl+click innermost, hover shows the click target, marquee and Select All use visible unlocked top-level objects, hidden/locked never selected; verify kittests for "Click selects the group", "Hidden object is not hit" and "Locked object ignores canvas clicks"
- [x] 4.2 Create shapes with the current style in the active layer; verify kittests for "Current style applies to new shapes" and "Drawing into the active layer"
- [x] 4.3 Implement the Eyedropper tool (fill/stroke per target, artboard color on empty canvas, eyedropper cursor); verify the "Pick a fill" kittest

## 5. Panels

- [x] 5.1 Transform panel (X/Y center, W/H, rotation, scale, proportions lock, Mixed, empty state, one undo step per edit); verify kittests for "Single object values", "Typed width", "Escape reverts", "Scale field", "Locked proportions", "Mixed rotation"
- [x] 5.2 Properties panel (summary, opacity field + slider, corner radius when all rectangles, fill/stroke swatches revealing Colors); verify kittests for "Multiple selection summary", "Set opacity", "Rounded rectangle", "Clicking the stroke swatch"
- [x] 5.3 Colors panel (target swatches and X/Shift+X/D, picker, RGB/HSV/HSL tabs, hex with error state, None for stroke, recent colors, palette with add/apply/remove); verify kittests for "Switch target with X", "Editing with nothing selected", "RGB entry updates the picker", "Short hex", "Invalid hex", "Removing a stroke", "Add and apply a palette color", "Undo a color change"
- [x] 5.4 Stroke panel (enable toggle using current stroke, width field, Mixed, current style when empty); verify kittests for "Enable a stroke" and "Set width"
- [x] 5.5 Layers panel tree (rows, indentation, expand/collapse, kind icons, eye/lock toggles with dimming, empty state, auto-expand on canvas selection); verify kittests for "Group shown as a tree", "Hide an object" and "Hidden group hides children"
- [x] 5.6 Layers selection sync (click, Cmd/Ctrl toggle, Shift range) and inline rename with default-name fallback; verify kittests for "Row selects canvas object" and "Rename a layer"
- [x] 5.7 Layers drag and drop (pure target computation, indicator line, into-group highlight, multi-row drag, cycle rejection) plus footer buttons and context menu (Rename, Duplicate, Delete, Group, Ungroup, New Layer); verify unit tests for target computation and kittests for "Move an object into a group", "Reorder between rows", "Group then ungroup", "New layer"
- [x] 5.8 Persist recent colors through preferences; verify the "Recent colors persist" kittest using a temporary preferences store

## 6. Integration

- [x] 6.1 Screenshot review of each panel with a sample livery tree at 100% and 200% UI scale
- [x] 6.2 Run `openspec validate editing-panels --strict` and `mise run ci`; verify both pass locally and the CI matrix is green on the PR
