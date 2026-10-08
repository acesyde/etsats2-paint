## Why

A fleet carries the same logo, lettering and stripes on every truck and trailer, but each texture has its own layout (`docs/roadmap.md`, "What is shared: elements, not placement"). Today the player copies the artwork onto each texture. Changing the logo afterwards then means redoing it everywhere.

`brand-kit` shared the colors and the styles. **Symbols** share the drawings themselves, which is the roadmap's promise: "reposition, don't redraw".

## What Changes

**Symbols and instances (new):**
- A **symbol** is a named drawing owned by the project: any shapes, paths, texts and images.
- An **instance** places a symbol on a texture. Each instance has its own position, size, rotation, flip, opacity, visibility, lock and name.
- An instance has **no other override**. Editing the symbol updates every instance on every texture. For a variant, the player duplicates the symbol or detaches the instance (decided with the user).

**Creating and placing:**
- **Object › Convert to Symbol** turns the selection into a new symbol ("Symbol 1"…). The selection is replaced by the symbol's first instance, at the same place. Instances inside the selection are detached first: symbols don't nest in this change.
- A new **Symbols** panel lists the project's symbols with their number of instances. From it, the player can:
  - **Place** a symbol in the middle of the view, or drag it onto the canvas;
  - **Edit**, **Rename** or **Duplicate** it;
  - **Delete** it. When the symbol is used, a confirmation comes first, and each instance becomes a group with the same look.

**Editing a symbol in its own view:**
- **Edit Symbol** opens the symbol alone on the canvas, on its own artboard, with every tool. It is reached from:
  - a double-click on an instance;
  - Object › Edit Symbol;
  - the Symbols panel.
- An "Editing symbol …" bar with **Done** returns to the texture. Escape also returns, and so does choosing a texture in the sidebar.
- Every change shows at once on every instance, and Undo works across both views.

**Instances in the rest of the editor:**
- An instance is moved, resized, rotated, flipped, aligned, duplicated, copied, grouped and copied from cabin like any object.
- It is one row in the Layers panel, with a symbol icon.
- Its look can't be changed in place:
  - colors, strokes, styles, path editing, Convert to Path, Create Outlines and Combine don't apply to it, and their commands say why;
  - Properties shows the symbol's name with **Edit Symbol** and **Detach Instance**.
- Swatches and styles used inside a symbol follow their edits like any object.
- An image inside a symbol counts once in the Assets panel.
- Instances render, export and hit-test like groups of their content.

**Project files:** symbols are saved with their content. Instances are saved with their symbol and placement, and their content is rebuilt from the symbol when the file opens. Files without symbols are unchanged. The format version stays 1.

## Capabilities

### New Capabilities
- `symbols`: symbols and instances, Convert to Symbol, the Symbols panel, editing a symbol in its own view, Detach Instance, Duplicate and Delete, and how instances behave with selection, transforms, layers, colors, styles, assets, rendering and export.

### Modified Capabilities
- `workspace-layout`: the right panel stack includes the Symbols panel.
- `project-files`: the file stores the symbols and the instances.

## Impact

- **tp-core:**
  - `SymbolId` and `Symbol { id, name, surface }`, with the content stored as a `Surface` (objects, artboard size, guides);
  - `ShapeKind::Instance { symbol }`, whose frame is its placement and whose children are the content expanded from the symbol;
  - a `transform::place` function that maps objects from one frame to another;
  - the project's active surface can be a texture or a symbol;
  - convert, expand, detach, duplicate and delete operations;
  - the brand kit (swatch propagation, relink) and asset usage extended to symbols;
  - a review of every `is_group()` and `ShapeKind::Group` site, with an instance counted as group-like for rendering, bounds and hit testing only.
- **tp-file:** optional `symbols` field and an `Instance` object kind in v1, instances expanded on open, round-trip tests and the fixture.
- **tp-app:**
  - the workspace's symbol editing (view swap, bar, Done and Escape);
  - instance expansion after each recorded edit made in a symbol;
  - the commands Convert to Symbol, Edit Symbol and Detach Instance, with their enablement reasons;
  - the Symbols panel (`PanelKind::Symbols`);
  - the Layers, Properties and Colors panel adjustments;
  - the canvas double-click;
  - status bar "Symbol › Logo";
  - UI tests and screenshots.
- **tp-render:** no change expected, since instances render as their expanded children. To be checked.
- **tp-i18n:** new messages in en, fr, de and es.
- **docs:** `roadmap.md` records the decisions (no overrides, no nesting, deleting detaches) and adds `symbols` to the Shipped list once it ships.
