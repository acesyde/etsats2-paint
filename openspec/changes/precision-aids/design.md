## Context

- **Canvas**: `canvas::show` allocates the whole canvas area as one `click_and_drag` response. Gestures start, update and finish in `start_gesture`, `update_gesture` and `finish_gesture`, from the press origin and the current pointer, all in document coordinates through `ScreenMap`.
- **Pointer-driven edits**, where snapping has to plug in:
  - moving objects (`translate` from a delta);
  - resizing (`resize` from the pointer);
  - drawing (`Gesture::Drawing` with a start point and the pointer);
  - Pen points (`pen_add` and `pen_drag_handle`);
  - Direct Selection (`drag_points` and `drag_handle`).
- **Document state**: `Surface` holds the object tree, and `Snapshot` captures it for undo.
- **File format**: v2 (`tp-file/src/v2.rs`) is merged but no version has been released or tagged.
- **Preferences**: `Prefs` is serialized with `#[serde(default)]`; the Preferences dialog lives in `ui/dialogs.rs`.
- **Commands**: Show Grid, Show Guides and Snapping already exist as `NotYet` commands with their shortcuts, and `CommandUi::menu_toggle` renders checkable menu rows.

## Goals / Non-Goals

**Goals:**
- One snapping engine shared by every gesture, cheap enough to run every frame with thousands of objects.
- Guides are document data (saved, undoable); grid, guide visibility and snapping are editor preferences.
- No regression in canvas performance or in existing gestures when snapping is off.

**Non-Goals:**
- Equal-spacing hints (distribute) and snapping to object rotation angles.
- Snapping text carets.
- Snapping objects to a path's curves (only its anchor points).

## Decisions

### D1. Guides in the document

- `tp-core` gains `Guide { axis: Axis, position: f64 }` (`Axis::Horizontal | Vertical`) and `Surface.guides: Vec<Guide>`.
- `Project` gets `add_guide`, `move_guide(index, position)`, `remove_guide(index)` and `clear_guides()`.
- `Snapshot` stores the guides of each surface next to the objects, so undo restores them; `same_document` compares them.
- Guides are referenced by index during a gesture. The gesture starts from a snapshot and restores it on cancel, so indexes are stable for its duration.

Alternative considered: guides in editor state, like zoom. Rejected: the user chose project-level guides, which have to round-trip and be undone like the rest of the document.

### D2. File format: additive field in v2

`FileSurface` gets `#[serde(default)] guides: Vec<FileGuide { vertical: bool, position: f64 }>`, and `from_v1` sets it empty.

There's no version bump. Format 2 has never been released, and the field is optional: a file without guides reads as an empty list, and serde ignores unknown fields, so builds of the merged v2 still open files with guides.

The v2 fixture is regenerated with two guides. Once a version is released, a change like this would need a v3.

### D3. Rulers carve the canvas area

`canvas::show` splits the allocated area into:
- a top ruler strip and a left ruler strip, `RULER` = 20 pt;
- a corner square;
- the remaining canvas rect.

The viewport, the fitting and `ws.canvas_rect` use the inner rect, so the artboard fit and the existing navigation behave as before, inside a slightly smaller area.

Each strip is its own response with `Sense::drag`:
- dragging from it starts `Gesture::Guide { axis, index: None, before }`;
- hovering the canvas while that gesture runs updates the guide preview;
- releasing over the inner canvas adds the guide; releasing elsewhere drops it.

**Graduations:** the step is the smallest value in {1, 2, 5}·10ᵏ texture pixels that keeps labels at least 60 pt apart, with minor ticks at a tenth (or a fifth) of the step. The labels are texture coordinates. The pointer marker is a short line in the selection color.

### D4. Grabbing existing guides

In `start_gesture` for the Selection, Move and Direct Selection tools, when guides are shown:
1. Find the guide nearest the press, within `GUIDE_HIT` = 4 pt.
2. It wins unless the press is inside the filled shape of the object under the pointer (`hit_test` with zero tolerance). A press on a handle of the selection overlay wins over guides.
3. The gesture is `Gesture::Guide { axis, index: Some(i), before }`.
4. On release over a ruler or outside the canvas, the guide is removed; otherwise it's recorded as "Move Guide".

Hovering a grabbable guide shows `ResizeVertical` (horizontal guides) or `ResizeHorizontal` (vertical guides).

**Undo labels:** "Add Guide", "Move Guide", "Delete Guide", "Clear Guides".

### D5. The snapping engine (`tp-app/src/snap.rs`)

`Snapper::new(ws, map, exclude)` is built once at the start of a gesture and holds:
- sorted `xs` and `ys` target lists, each value tagged with its source (guide, artboard, or an object's edge or center, with that object's bounds for drawing the alignment line);
- point targets: other paths' anchor points and the corners and centers of other objects' bounds;
- the grid spacing, when the grid is shown (grid lines are computed on demand: `round(v / s) · s`, never enumerated);
- the tolerance, `map.doc_len(6.0)`.

Its API:
- `snap_axis(values, axis)`: for a few candidate values (e.g. left, center, right), returns the smallest correction within tolerance and the target, using a binary search in the sorted list plus the nearest grid line.
- `snap_point(p)`: tries the point targets first (both axes at once, within tolerance), then falls back to `snap_axis` on each axis.
- Results carry `SnapHit`s; the gesture stores them in `ws.snap_hits` for painting. They're cleared when a gesture finishes or is cancelled.

**What each gesture snaps:**
- **Moving**: the candidates are the original selection bounds plus the delta (left, center and right on x; top, center and bottom on y). The correction is added to the delta.
- **Resizing**: the pointer, with point snapping.
- **Drawing** (rectangle, ellipse, polygon, line): the press point when the gesture starts, and the pointer.
- **Pen points** and **Direct Selection** handles: point snapping. For Direct Selection point drags, the grabbed point is snapped and the same correction is applied to every selected point.
- **Guides**: the axis value, with the dragged guide excluded from the targets.

**Constraints:**
- With Shift on a move constrained to horizontal or vertical, only the free axis snaps. A diagonal constraint disables snapping.
- Proportional or square resizing and drawing, the 45° constraints of lines and Pen points, and the Shift on the polygon tool: the pointer isn't snapped (the axes are coupled). The press point of a drawing still snaps.
- Holding Cmd/Ctrl, or Snapping turned off, means no snapping. It's checked every frame, so pressing and releasing Cmd/Ctrl mid-drag takes effect immediately.

**Exclusions:** objects being edited (the selection and its descendants, or the path whose points are being edited) and hidden objects aren't targets. Locked objects are targets, since they're often references.

Alternative considered: computing targets every frame. Rejected because of the cost with large documents; targets don't change during a gesture except the edited objects, which are excluded anyway.

### D6. Grid painting

- **Lines:** grid lines are painted between the artwork and the guides, clipped to the artboard. The step is `spacing · 2ᵏ` with the smallest k that keeps lines 8 pt apart. Every eighth base line is drawn with the major color.
- **Cost:** about (artboard side / step) line segments per axis, at most around 250 per axis given the 8 pt minimum. That's negligible next to the artwork.

### D7. Preferences and commands

- **Preferences:** `Prefs` gains `view_aids: ViewAids { grid: bool, guides: bool, snapping: bool, grid_spacing: f64 }`, with defaults `false`, `true`, `true` and 64, and `#[serde(default)]` so existing preference files load unchanged. The workspace reads a copy each frame (`ws.aids`); `AppState` sets it before the UI runs.
- **Commands:** Show Grid, Show Guides and Snapping become `NeedsProject` and toggle `prefs.view_aids`. The View menu shows them with `menu_toggle`. The new `ClearGuides` command is enabled when the active surface has guides.
- **Preferences dialog:** a Grid spacing numeric field (4–1024 px), included in "Reset to defaults".
- **Tokens** in `tp_ui::tokens::canvas`: `RULER`, ruler background, tick and label colors, `GUIDE` (cyan), `GRID_MINOR` and `GRID_MAJOR` (translucent), `SNAP` (magenta) for the alignment lines, `GUIDE_HIT`.

### D8. Paint order

1. Pasteboard and artboard.
2. Objects.
3. Grid.
4. Guides, plus the guide being dragged.
5. The selection overlay, Direct Selection points and the pen session.
6. Snap alignment lines and crosses.
7. Gesture feedback (labels).
8. Rulers, painted in their own strips.

## Risks / Trade-offs

- [Guides stealing presses meant for objects near them] → Objects win inside their filled shape, guides win elsewhere within 4 pt. When guides get in the way, hiding them (⌘;) disables grabbing.
- [Snapping fighting fine adjustments] → The tolerance is 6 pt on screen, so it shrinks in document units when zooming in. Cmd/Ctrl bypasses snapping during a drag, and View › Snapping turns it off.
- [The canvas area shrinks by 20 pt for the rulers] → Existing UI tests that compute screen positions through `screen_map` are unaffected. Screenshots change (expected).
- [The additive v2 field] → It's safe only because v2 is unreleased. D2 states that a later change of this kind needs v3.

## Migration Plan

Existing preference files get the defaults for the new fields. Existing v2 project files have no guides. Nothing to migrate.
