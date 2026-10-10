## Context

- **Edit Swatch… today** (`brand_ops.rs`, `panels/colors.rs::edit_swatch_popup`): a modal that calls `Workspace::preview_swatch_color` on each change. That runs `live_edit("undo-edit-swatch", project.set_swatch_color)`, so the document is recolored live. OK commits the pending edit, Cancel calls `cancel_pending()`. State lives in `ws.panels.editing_swatch: Option<SwatchEdit>`. The modal is drawn once per frame by the workspace, whichever list opened it (popover or Brand card menu).
- **Core** (`tp-core/src/brand.rs`):
  - `set_swatch_color(id, color)` recolors the palette entry, the looks of both style kinds, every linked paint on every surface (through `update_surfaces`), then `refresh_instances()`;
  - `redefine_graphic_style(id, from)` sets `style.look = Look::of(o)` and re-applies the style to every follower;
  - `Look { fill, fill_swatch, stroke, opacity }`; objects carry `style: Option<StyleId>`, texts carry `text.style_id`;
  - `symbols.rs::instance_count(id)` walks every surface's tree. Nothing else counts usage.
- **Thumbnails** (`surface_thumbnails.rs`): `SurfaceThumbnails` renders stale surfaces on a worker thread at `SIDE = 128`. Staleness is a pointer comparison of the surface's top-level `Arc<Object>`s. Nothing starts while a pointer button is down. One instance lives in `Workspace.thumbnails`.
- **Brand space** (`ui/workspace/spaces/brand.rs`): a left index panel plus a scrolling central panel with cards (`card`, `card_text`, `card_menu`). Palette cards show name and hex. Symbol cards show "N instances".
- **Mockup** (TruckPaint 5, artboard 07):
  - a 340-point panel on the right of the Brand space, with "Edit color" and the name, the SV square, a hue bar, the Before / After blocks, the hex field, an impact box ("Impact: 11 textures, 38 objects", a 6-column grid of texture tiles, a note line), and Cancel / Apply to Fleet at the bottom;
  - cards show "11 textures · 38 objects".

## Goals / Non-Goals

**Goals:**
- One pure usage query in tp-core for swatches, styles and symbols, cached in the workspace and recomputed only when the project changes.
- A single before/after editor model and UI shared by swatches and graphic styles. It never touches the document before Apply to Fleet, and Apply is one undo step.
- Live thumbnails of the affected textures, rendered off the UI thread from a modified copy of the project.

**Non-Goals:**
- Text style editor (text styles keep Redefine from Selection).
- Anything about the personal library (decided out of scope).
- Editing gradients, dashes, caps or joins in Edit Style…: those parts of the look are kept as they are.
- Usage in the Workshop's Resources tab rows (only the Brand cards and the color popover tooltip show it; the Resources tab keeps "N instances").

## Decisions

### 1. Usage: one walk, cached by pointer identity

Add `tp-core/src/usage.rs` with `Project::usage(&self) -> Usage`:

```rust
pub struct Count { pub objects: usize, pub surfaces: Vec<usize> } // textures() = surfaces.len()
pub struct Usage {
    swatches: HashMap<SwatchId, Count>,
    styles: HashMap<StyleId, Count>,   // graphic and text
    symbols: HashMap<SymbolId, Count>, // objects = instances
}
impl Usage { pub fn swatch(&self, id) -> Count; pub fn style(&self, id) -> Count; pub fn symbol(&self, id) -> Count }
```

How the walk counts:
- **Symbol content first:** for each symbol, collect the set of swatches linked anywhere in its content (fill, stroke, gradient stops, recursively).
- **Then each surface:**
  - every non-group object adds itself once to each swatch it links (a `HashSet<SwatchId>` per object, so fill and stroke on the same swatch count once) and to its graphic style or text style;
  - an `Instance { symbol }` adds one to that symbol, and one to each swatch of its symbol's set;
  - groups recurse;
  - textures are counted through a per-surface "seen" set.

Workspace keeps `usage: UsageCache { key: Vec<*const Object>, symbols: Vec<*const Object>, value: Usage }`. The key holds the top-level `Arc` pointers of each surface and of each symbol's surface, the same trick as `surface_thumbnails::Key`. `Workspace::usage()` recomputes when the key differs. Palette and style edits always rewrite objects (links) or don't change counts, so pointer identity is enough. A deleted swatch simply has no entry.

*Alternative considered:* a revision counter bumped by `edit`/`live_edit`/undo. Rejected: there is no such counter today, and every mutation path would need to bump it.

Formatting goes in `tp-app` (`brand_ops::usage_label(Count) -> String`) through fluent plurals: `brand-usage = { $textures } { $textures -> [one] texture *[other] textures } · …`, `brand-unused`, `brand-symbol-usage`, `brand-impact`.

### 2. Editor model: a pending value on a cloned project

Replace `SwatchEdit` with:

```rust
pub enum BrandEditTarget { Swatch(SwatchId), Style(StyleId) }
pub struct BrandEdit {
    pub target: BrandEditTarget,
    pub name: String,
    pub before: EditValue,   // Color(Rgba) | Look(Look)
    pub after: EditValue,
    pub hex: String, pub hsv: Hsv,      // picker state (swatch, or the look's fill/stroke being edited)
    pub look_target: ColorTarget,       // which color of the look the picker edits
    pub preview: Option<Project>,       // project with `after` applied, rebuilt when `after` changes
    pub affected: Vec<usize>,           // surfaces holding counted objects, from Usage at open time
}
```

- `ws.panels.brand_edit: Option<BrandEdit>` replaces `editing_swatch`.
- `start_swatch_edit(id)` and `start_style_edit(id)` fill `before = after` from the project and compute `affected` from `Count::surfaces` of the cached usage.
- `set_edit_value(v)` stores `after` and rebuilds `preview`:
  - `preview = project.clone()`, which is cheap since surfaces are `Vec<Arc<Object>>`;
  - then, on that clone, `set_swatch_color` or the new `set_graphic_style_look`.
  - The live document is never touched, so the canvas, the panels and the history stay as they are. This satisfies "nothing changes before Apply" by construction.
- `apply_brand_edit(now)`: if `after != before` or the name changed, one `edit("undo-edit-swatch" | "undo-edit-style", …)` runs `set_swatch_color` or `set_graphic_style_look`, plus the rename. Otherwise it just closes.
- `cancel_brand_edit()`: drops the state. Nothing to restore.
- `preview_swatch_color`, `finish_swatch_edit` and `cancel_swatch_edit` are removed.

New core function `Project::set_graphic_style_look(id, look) -> bool`. It is `redefine_graphic_style` without the source object: set `style.look = look`, re-apply to followers through `update_surfaces`, then `refresh_instances()`. `redefine_graphic_style` is rewritten on top of it.

Swatch links in an edited look: when the user changes the fill color in Edit Style…, `fill_swatch` is cleared unless the new color equals the linked swatch's color. This is the "Linked, not copied" rule that `relink()` already applies after the edit.

*Alternative considered:* keep the live edit on the document and render the canvas as the preview. Rejected: the spec requires the canvas and history to be untouched, and in the Brand space the canvas isn't visible anyway.

### 3. Preview thumbnails: a second `SurfaceThumbnails`

- `SurfaceThumbnails` gets a `name` prefix (texture debug names), and an `only: Option<&[usize]>` filter in `update_with`, so that only the affected surfaces are rendered.
- The editor owns a `preview_thumbs: SurfaceThumbnails` (in `Workspace`, reset when an edit starts). Each frame, while an edit is open, it is updated with `preview` and `affected`.
- Unaffected surfaces are never rendered. Because the preview project shares the untouched `Arc`s, only the surfaces whose objects actually changed become stale between two values.
- **Throttling** comes for free:
  - one job at a time;
  - no new job while a pointer button is down, so the thumbnails catch up when the drag ends, which matches the spec's "MAY lag, SHALL show the last value when the drag ends";
  - the cost is bounded to affected surfaces at 128 px.
- Before the first render, a tile shows the workspace's existing thumbnail of that surface (the Before state). So the grid is never empty.

### 4. Where the editor is drawn

- **Brand space:** a right side panel `Panel::right("brand_edit")` of 340 points, inside `brand::show`, drawn before the central panel. The sections stay visible, and the edited card gets the selected fill (the same "fill only, no bar" selection as elsewhere).
- **Workshop** (Edit Swatch… from the fill/stroke popover): the same body function `brand_editor::body(ui, env)`, inside `dialogs::modal("brand_edit_modal")` at 340 points wide. One body, two hosts.
- Escape is consumed by the editor in both hosts. Opening another element's editor replaces the state.
- Changing `ws.space` while an edit is open calls `cancel_brand_edit()`. This is done in the space switch path in `state.rs`, which already handles leaving a space.

Body layout, top to bottom, following the mockup:
1. Caption ("Edit color" / "Edit style") and the name field.
2. **Swatch:** SV square (full width, 160 high), hue bar, alpha bar.
   **Style:** a Fill / Stroke segmented toggle for the picker, a stroke width field with a "None" toggle, and an opacity field. A gradient fill shows "Gradient" and its picker is disabled, with a tooltip pointing to Redefine from Selection.
3. Before / After blocks. For a style, each block is a `paint_look` preview.
4. Hex field (the color of the picker's target).
5. Impact box: the impact line or "Not used yet", a 6-column grid of tiles (`affected` thumbnails, scrolls after 4 rows), and the note line.
6. Footer: Cancel (secondary), then Apply to Fleet (primary, semibold). Apply is disabled while the name is empty, with `colors-swatch-name-empty` shown above.

Controls reuse `panels/colors.rs` widgets (`sv_square`, `hue_slider`, `alpha_slider`, the hex parsing), made `pub(crate)`.

### 5. Usage in the lists

- **Palette cards:** a third text line, `TEXT_MUTED`, small.
- **Style cards:** the detail line becomes the usage, and a text style keeps its size before it ("200 px · 3 textures · 9 objects").
- **Symbol cards:** "14 instances · 9 textures".
- **Color popover:** the swatch tooltip gains the usage line.
- `card_text` grows an optional third line, and `PALETTE_BLOCK` / card heights are adjusted so the grid doesn't jump.

### 6. Strings

New keys in en/fr/de/es:
- `brand-usage`, `brand-unused`, `brand-symbol-usage`;
- `brand-edit-color`, `brand-edit-style`, `brand-before`, `brand-after`;
- `brand-impact`, `brand-impact-none`, `brand-impact-note`, `brand-apply-to-fleet`;
- `styles-edit` (menu item "Edit Style…");
- `undo-edit-style`, `brand-gradient-fill`.

## Risks / Trade-offs

- **Cloning the project per value change.** Surfaces are `Vec<Arc<Object>>`, so the clone is shallow. But `set_swatch_color` on the clone rewrites the linked objects' `Arc` path, and `refresh_instances()` runs too. On a 40-texture fleet this happens at most once per frame while dragging, without rendering. Mitigation: rebuild `preview` only when `after` actually changes, and measure in a test fleet of 40 surfaces (task 2.4). If too slow, rebuild at most every 50 ms while the pointer is down.
- **Usage key misses a change.** For example, a palette-only change that alters counts without rewriting an object. Deleting a swatch calls `relink()`, which rewrites the formerly linked objects, so their `Arc`s change. Mitigation: the cache key also includes the palette and style ids, and a unit test covers delete/undo.
- **Two thumbnail workers at once** (the workspace's and the preview's) on large fleets. Acceptable: the preview worker only renders affected surfaces, and the main one is idle while nothing is edited.
- **Behavior change for existing users:** Edit Swatch… no longer recolors the canvas live in the Workshop. This is intended (the spec), and the thumbnails replace that feedback.
- **kittest timing:** preview thumbnails repaint from a worker. Tests that open the editor must use `settle_renders`, extended to wait on `preview_thumbs.is_rendering()`.
