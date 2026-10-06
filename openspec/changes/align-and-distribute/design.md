## Context

- **Selection**: `Workspace.selection` is ordered by selection time (Shift+click appends). Top-level selected items may be groups. `selected_objects()` returns them.
- **Moving**: `transform::translate(objects, delta, false)` moves objects exactly (paths included), and `ws.edit(label, …)` records one undo step.
- **Commands**: they're declared in `commands.rs` with labels, icons, shortcuts and an `Availability` predicate over `EditContext`, and dispatched in `state.rs`. Menus use `CommandUi::menu_item`, and `toggle_icon_button`/`icon_button` exist for panels.
- **Transform panel**: `panels/transform.rs` shows a 3-column grid of fields.
- **Icons**: the Phosphor set is embedded and already includes the `ALIGN_*` and `ARROWS_OUT_LINE_*` glyphs.

## Goals / Non-Goals

**Goals:**
- Pure, unit-testable geometry in `tp-core`.
- One command path for menus, shortcuts and panel buttons.

**Non-Goals:**
- Live spacing hints.
- Distributing to the artboard.
- Persisting the Align to choice across launches. It's session state, like the proportion lock.

## Decisions

### D1. Geometry in `tp-core::document::align`

```rust
pub enum Edge { Left, HCenter, Right, Top, VCenter, Bottom }
pub enum DistributeMode { Centers, Spacing }

pub fn align(objects: &[Object], edge: Edge, target: Rect) -> Vec<Object>;
pub fn distribute(objects: &[Object], axis: DistributeAxis, mode: DistributeMode) -> Vec<Object>;
```

- Bounds are `Object::bounding_box()`, the visible axis-aligned bounds that already account for rotation, curves and group children.
- Each object moves by a delta on one axis through `translate`, so groups, texts and paths stay exact.
- `distribute` sorts by the bounds' center along the axis (ties broken by the selection index, so the order is stable). It keeps the first and last objects, then places the inner objects:
  - **Centers:** at equal steps between the first and last centers.
  - **Spacing:** with an equal gap of `(span − Σ sizes) / (n − 1)`. The gap can be negative when objects overlap, and the result still keeps the outermost objects fixed.

`distribute` takes its own `DistributeAxis { Horizontal, Vertical }` (horizontal moves along x), rather than reusing the guide `Axis`, whose `Vertical` means "a line at an x position" and would read backwards here.

### D2. Target resolution (in `tp-app`)

`align_target(ws) -> Option<(Rect, Option<ObjectId>)>`:
- **Selection:** the union of the selected bounds when there are 2 or more objects, else the artboard.
- **Artboard:** `surface.bounds()`.
- **Key object:** the last id of `ws.selection`. Its bounds are the target, and it's excluded from the moved objects. With fewer than 2 objects, the command is disabled (`EditContext` gets `selection_count`).

### D3. Commands

- `CommandId::Align(Edge)` and `CommandId::Distribute(DistributeAxis, DistributeMode)`, so the set of commands is enumerable for the registry, menus and the conflict test.
- **Shortcuts:** Alt+A/H/D/W/V/S for the aligns, Alt+Shift+H/V for equal spacing. There are no existing Alt shortcuts, and the registry's conflict test covers the new ones.
- **Availability:**
  - align: `editable_selection && selection_count ≥ (key ? 2 : 1) && !editing_text`;
  - distribute: `selection_count ≥ 3`.

  Each has a reason string for its tooltip.
- **Undo labels** are the command labels ("Align Left", "Distribute Horizontal Spacing").

### D4. UI

- **Object › Align submenu:** six aligns, a separator, then the four distribute commands.
- **Transform panel:**
  - two rows under the grid, made of 24 pt icon buttons that go through `CommandUi`, so the tooltip, shortcut and disabled state come from the registry;
  - an `Align to` combo box (Selection, Artboard, Key object) stored in `PanelState.align_to`. This requires passing the `CommandUi` into the panel environment, the way the Layers panel already pushes commands.
- **Key object highlight:** when Align to is Key object and 2 or more objects are selected, the canvas draws the key object's outline with a 2 pt stroke on top of the regular selection outline.

## Risks / Trade-offs

- [Alt+letter on macOS types special characters] → Canvas shortcuts aren't active while typing in fields or editing text (the command system already filters them), so the characters only appear in text fields, where the shortcuts don't fire.
- [Panel width: 6 buttons and a combo box in a 280 pt column] → The combo box is compact (Selection / Artboard / Key). It's verified with the 100 % and 200 % UI scale screenshots.
- [Locked objects inside a selected group] → The group moves as a whole, as it does with the Move tool, which is consistent with existing transforms.
