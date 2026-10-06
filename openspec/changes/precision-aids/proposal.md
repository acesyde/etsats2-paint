## Why

Liveries need precise placement: stripes that line up across the cab, logos centered on a door, mirrored elements at the same height. Today everything is placed by eye or by typing numbers into the Transform panel. Show Grid, Show Guides and Snapping are already in the View menu but disabled. Now that paths and lines can be drawn point by point, the lack of snapping is felt on every shape.

## What Changes

- **Rulers**: horizontal and vertical rulers along the top and left edges of the canvas.
  - Graduated in texture pixels, they follow zoom and pan.
  - They show the pointer position.
- **Guides**:
  - Drag from a ruler onto the canvas to create a guide: horizontal from the top ruler, vertical from the left one.
  - Drag a guide to move it. Drop it back on a ruler to delete it.
  - The position shows while dragging.
  - Guides belong to the surface: they're saved in the project and every change is undoable.
  - View › Show Guides (⌘; / Ctrl+;) toggles them, and View › Clear Guides removes all guides of the surface.
- **Grid**:
  - View › Show Grid (⌘' / Ctrl+') draws a grid over the artboard.
  - The spacing (64 texture px by default) is set in Preferences.
  - Lines too dense at the current zoom are thinned so the grid stays readable.
- **Snapping**: View › Snapping (⌘⇧; / Ctrl+Shift+;) turns it on or off.
  - When on, moving, resizing and drawing snap to:
    - the grid (when shown);
    - guides (when shown);
    - the artboard edges and center;
    - the edges and centers of other objects' bounds;
    - path anchor points, when placing or editing points.
  - Snapping applies within a few screen pixels.
  - Alignment lines (smart guides) show what was snapped to.
  - Holding Cmd/Ctrl during a drag suspends snapping.
- **Preferences**: whether the grid, guides and snapping are on, and the grid spacing, are remembered between sessions.

Not in this change: snapping rotations to other objects; a guides panel or numeric guide dialog; locking guides; angled guides; snapping while editing text; per-project grid spacing.

## Capabilities

### New Capabilities

- `canvas-guides`: rulers and guides. Covers creating, moving and deleting guides, showing and clearing them, and their appearance.
- `canvas-grid`: the grid overlay, its spacing and how it's thinned at low zoom.
- `snapping`: what snaps (moving, resizing, drawing shapes and lines, pen points, path points and handles) and to what (grid, guides, artboard, other objects, anchor points). Covers the snap distance, the alignment lines, the temporary bypass and the Snapping toggle.

### Modified Capabilities

- `document-model`: a surface also holds its guides (horizontal and vertical positions in texture pixels).
- `project-files`: guides are saved with each surface.
- `undo-history`: adding, moving, deleting and clearing guides are undoable.
- `app-preferences`: the grid, guides and snapping toggles and the grid spacing are persisted; Preferences gets a Grid spacing field.
- `workspace-layout`: the canvas area shows rulers along its top and left edges.

## Impact

- **`tp-core`**: `Surface.guides` (`Guide { axis, position }`) and guide edits in `Project`, captured in `Snapshot`. There's no new dependency.
- **`tp-file`**: optional `guides` on v2 `FileSurface` with `#[serde(default)]`. Format 2 is unreleased and the field is additive, so there's no version bump. The v2 fixture is regenerated with guides.
- **`tp-app`**:
  - a new `snap` module: targets, nearest candidate per axis, and the alignment lines;
  - snapping is applied in the move, resize, draw, Pen and Direct Selection gestures;
  - rulers and guide gestures on the canvas;
  - grid painting;
  - the Show Grid, Show Guides and Snapping commands become available, as checkable View menu items;
  - a new Clear Guides command;
  - preferences fields;
  - a Grid spacing field in the Preferences dialog.
- **`tp-ui`**: tokens for rulers, guides, the grid and alignment lines.
