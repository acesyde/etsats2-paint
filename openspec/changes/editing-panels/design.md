## Context

After `canvas-core`, `tp-core` has `Object { id, kind: ShapeKind (Copy), frame, fill, stroke, opacity }`, surfaces hold a flat `Vec<Arc<Object>>`, transforms (`translate`, `resize`, `rotate`) map `&[Object]` to new objects, and `History<Snapshot>` shares unchanged objects through `Arc`. In `tp-app`, `Workspace` owns project, selection, viewport, history, gesture and a geometry cache keyed by `ObjectId`; the canvas draws `surface.objects` in order and hit-tests the top level; panels (`ui/workspace/panels.rs`) only render empty states; `tp-ui` has no input widgets beyond buttons. See proposal.md for scope and specs/ for behavior.

## Goals / Non-Goals

**Goals:**
- A tree-shaped document whose edits stay copy-on-write, so snapshots remain cheap.
- One "live edit, single commit" mechanism shared by every panel control (fields, sliders, picker drags).
- Reusable professional input widgets in `tp-ui` (numeric scrub field, swatches, picker) that later panels (text, export) reuse.

**Non-Goals:**
- True group opacity compositing (offscreen layers) — approximated, see D4.
- Gradients, blend modes, stroke alignment/joins/dashes, layer thumbnails.

## Decisions

### D1. Groups as objects with children
`ShapeKind` stays `Copy` and gains `Group`; `Object` gains `name: String`, `visible: bool`, `locked: bool`, `children: Vec<Arc<Object>>` (empty for shapes). A group's `frame` is derived: its rotation is stored (so a rotated group keeps a rotated box and the rotation field stays meaningful) and its center/size are recomputed as the oriented bounds of its visible children in that rotation (`Object::refresh_group_frame`, bottom-up after every edit). Group opacity multiplies into children.
*Alternatives:* `ShapeKind::Group { children }` (makes `ShapeKind` non-`Copy` and every match heavier); a separate `Node` enum (forces two code paths everywhere objects are handled).

### D2. Tree operations in `tp-core::document::tree`
Objects are addressed by id; internally by an index path. Mutations walk the path with `Arc::make_mut`, so only the edited branch is copied and snapshots keep sharing the rest. Operations: `find_path`, `get`, `parent_of`, `replace` (tree-aware, refreshing ancestor group frames), `remove`, `insert(parent, index, objects)`, `move_to(ids, parent, index)` (rejects moving a group into itself or a descendant), `group(ids) -> id`, `ungroup(ids) -> children ids`, `effective_flags(id)` (hidden/locked through ancestors), `draw_list()` (visible leaves in paint order with accumulated opacity), `hit_test(point, tol) -> Option<Hit { top: ObjectId, inner: ObjectId }>` skipping hidden/locked, `top_level_in_rect`. `Surface.objects` stays the top-level list, so existing code keeps working for flat documents.

### D3. Selection normalization and group transforms
The selection may contain ids at any depth but never an object together with one of its ancestors (`normalize_selection` keeps the ancestor). Transform functions become recursive: `translate`, `rotate` and `resize` apply the same global mapping to every descendant (for `resize`, the affine computed from bounds/handle/pointer is independent of the objects, so applying it to descendants keeps them consistent), then group frames are refreshed. `selection_frame` uses each selected node's frame (groups included). Bring Forward/Send Backward, Duplicate, Delete, Paste operate within each object's own parent list.

### D4. Drawing order and group opacity
The canvas iterates `draw_list()` (visible leaves, paint order, opacity multiplied through ancestors) instead of `surface.objects`; the geometry cache is unchanged (keyed by leaf id). Group opacity is therefore applied per child: overlapping children inside a semi-transparent group show their overlap. True isolation needs offscreen compositing and is deferred to the export renderer work.

### D5. Current style, color target and color models
`Workspace` gains UI state outside the history: `style: Style { fill: Rgba, stroke: StrokeStyle, stroke_enabled: bool }` used for new shapes, `color_target: Fill | Stroke`, the picker's HSV (kept separately so hue survives at zero saturation or value), the color model tab, expanded layer groups and the layers drag state. `tp-core::document::color` adds `Hsva`/`Hsla` conversions and hex parse/format (`#RGB`, `#RRGGBB`, `#RRGGBBAA`). Applying a color sets it on every selected shape (recursively for groups) or on `style` when nothing is selected, and pushes it to recent colors. Default colors (D): fill `DEFAULT_FILL`, stroke black 4 px disabled.

### D6. Live edit, single commit
`Workspace::live_edit(label, |project, selection| …)` captures a `before` snapshot on the first call of an interaction (stored as `pending: Option<(label, Snapshot)>`) and applies the change immediately; `Workspace::commit_pending(now)` records one history entry (and does nothing if the document did not change). Panels call `commit_pending` when a drag stops, a field commits (Enter/Tab/focus lost) or a click-type control is used; `AppState` also commits any pending edit before dispatching a command and at the start of a canvas gesture, so Undo never splits an interaction. Escape in a field restores `before` via `cancel_pending`.

### D7. `tp-ui` widgets
- `NumericField`: label (drag to scrub: Shift ×10, Alt ×0.1) + text box with its own edit buffer in egui memory; returns `FieldEvent::{None, Live(f64), Commit(f64), Revert}`; shows "Mixed" as a placeholder; optional suffix (`px`, `°`, `%`) and range clamp.
- `ColorSwatch` (solid over a checkerboard; `None` diagonal pattern; `Mixed` pattern), `FillStrokeSwatches` (two overlapping swatches, active one in front with outline).
- `SvSquare`, `HueSlider`, `AlphaSlider` drawn as `egui::Mesh` gradients with a ring/handle and keyboard arrows; each returns changed/drag-stopped responses.
- `ToggleIconButton` (eye / lock with distinct open/closed glyphs).
*Alternative:* egui's built-in `color_picker` and `DragValue` — functional but their look, mixed-state and commit semantics don't match the design system.

### D8. Panels
- Transform: fields from `selection_frame`; X/Y/W/H/rotation/scale through `live_edit` with labels "Move", "Resize", "Rotate", "Scale"; the proportions lock lives in workspace UI state.
- Properties: summary, opacity (field + slider, "Change Opacity"), corner radius ("Change Corner Radius"), fill/stroke swatches that set `color_target` and open/expand the Colors panel in `prefs.layout`.
- Colors: swatches, picker, model tabs (RGB/HSV/HSL), hex field, None (stroke target), recent colors, palette ("Add to Palette"/"Remove from Palette" are history entries because the palette belongs to the project). Labels "Change Fill" / "Change Stroke".
- Stroke: toggle ("Add Stroke"/"Remove Stroke"), width ("Change Stroke Width").
- Layers: rows rendered from the tree (top-first), 26 pt high, 14 pt indent per level; click/Cmd/Shift selection over the visible-row order; double-click inline rename ("Rename"); eye/lock ("Hide"/"Show"/"Lock"/"Unlock"); manual drag and drop — a row drag after the threshold collects the dragged ids (the selection if the row is selected), the hovered row's upper/lower third gives before/after and the middle third of a group row gives "into", drawn as an accent line or a highlighted row; drop calls `move_to` ("Move to Group"/"Reorder"); footer buttons New Layer, Group, Delete; context menu.

### D9. Canvas changes
Hit testing uses `Hit { top, inner }`: click selects `top` (or `inner` with Cmd/Ctrl), hover outlines what a click would select, marquee selects top-level objects, Select All selects visible unlocked top-level objects. New shapes go to the active layer: the selected group, or the common parent of the selection, else the top level. The Eyedropper tool picks `inner`'s fill/stroke; on empty canvas it picks the artboard color. Hiding or locking removes affected ids from the selection.

### D10. Commands and preferences
New commands: `Group`, `Ungroup`, `NewLayer`, `DuplicateLayer`, `DeleteLayer` (now enabled via `When` predicates on `EditContext`, which gains `selection_has_group` and `selection_count`), `SwapColorTarget` (X), `SwapFillStroke` (Shift+X), `DefaultColors` (D); the Eyedropper tool leaves the "not available" list. `Prefs` gains `recent_colors: Vec<[u8; 4]>` with a serde default, so existing `prefs.ron` files keep loading at version 1.

### D11. Testing
- `tp-core`: tree operations (insert/move/group/ungroup round-trips, cycle rejection, effective flags, draw order, hit-test with hidden/locked/groups, recursive transforms and group frame refresh), color conversions (round-trips, hex parsing edge cases).
- `tp-ui`: NumericField commit/revert/scrub and swatch rendering in kittest.
- `tp-app`: kittest scenarios for each panel spec scenario (typed width + undo, scale field, mixed rotation, opacity undo, radius clamp, swatch reveals Colors, X switch, hex entry/error, None stroke, eyedropper, palette, recent colors persistence via prefs round-trip, layers tree/rename/hide/lock/drag-drop/group/ungroup/new layer, canvas group click and Cmd+click, drawing into the active layer); screenshots of each panel at 100% and 200%.

## Risks / Trade-offs

- [Per-child group opacity differs from true group compositing] → documented (D4); fix with the CPU renderer in the export change.
- [Tree mutations by index path can go stale across frames] → paths are never stored; every operation resolves ids at call time.
- [Panels and canvas both editing the same objects in one frame] → panels run before the canvas only through queued commands or `live_edit`; any canvas gesture start commits pending panel edits first (D6).
- [Manual drag-and-drop in the tree has many edge cases] → target computation is a pure function (rows, pointer y, dragged ids) unit-tested separately from rendering.
- [Recent colors stored in preferences grow the file] → capped at 12 entries.

## Migration Plan

No persisted documents exist yet. `Prefs` stays version 1 (additive field with default). `Object::new` keeps its signature; new fields get defaults (`name` from the kind, visible, unlocked, no children).

## Open Questions

- Whether hidden objects should still appear in a future "outline" view mode — irrelevant until such a mode exists.
