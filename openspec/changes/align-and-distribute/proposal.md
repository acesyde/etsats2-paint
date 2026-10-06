## Why

Liveries are full of elements that must line up: a row of sponsor logos at the same height, lettering centered on the cab, stripes evenly spaced along the trailer. Snapping helps while dragging, but lining up several existing objects, or centering one on the texture, still means typing coordinates into the Transform panel one object at a time.

## What Changes

- **Align commands**:
  - Align Left, Horizontal Centers, Right, Top, Vertical Centers and Bottom;
  - in a new Object › Align submenu, with Figma-style shortcuts (Alt+A/H/D/W/V/S).
- **Distribute and spacing commands**:
  - Distribute Horizontal Centers and Vertical Centers make the centers evenly spaced;
  - Distribute Horizontal Spacing and Vertical Spacing make the gaps equal.
  - Alt+Shift+H and Alt+Shift+V give equal spacing.
  - They need at least 3 objects and keep the outermost objects in place.
- **Align to**:
  - **Selection** (default): several objects align to their common bounds, and a single object aligns to the artboard;
  - **Artboard**: always align to the texture;
  - **Key object**: align to the most recently selected object, which doesn't move and is highlighted on the canvas.
- **Transform panel**: a compact row of align buttons with the Align to selector, and a row of the four distribute buttons. Buttons that can't apply are disabled.
- **Behavior**:
  - Selected objects move as wholes (groups included), measured by their visible axis-aligned bounds, so rotated objects and paths line up by what you see.
  - Each command is one undo step named after it.

Not in this change: snapping-based equal-spacing hints, distributing to the artboard, aligning path points, aligning text baselines.

## Capabilities

### New Capabilities

- `align-distribute`: the align, distribute and spacing commands, the Align to reference (selection, artboard, key object), the key object highlight, the menu and shortcuts, and when commands are enabled.

### Modified Capabilities

- `transform-panel`: the panel gains the align and distribute buttons and the Align to selector.

## Impact

- **`tp-core`**: pure functions `align(objects, edge, target)` and `distribute(objects, axis, mode)` returning moved copies, built on `translate`.
- **`tp-app`**:
  - new `CommandId::Align(Edge)` and `CommandId::Distribute(Axis, Mode)` with metadata, shortcuts and availability;
  - the Object › Align submenu;
  - `PanelState.align_to`;
  - the buttons in the Transform panel;
  - the key-object outline on the canvas.
- **`tp-ui`**: icon constants for alignment and distribution (from the Phosphor set already embedded).
