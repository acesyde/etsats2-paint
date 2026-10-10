## Context

See proposal.md for the motivation, and the specs for the behavior (`texture-status`, and the deltas of `project-screen`, `vehicle-projects`, `properties-panel`, `mod-export`).

What exists today:
- **tp-core** `project.rs`: a `Surface` holds `objects: Vec<Arc<Object>>` and `template: Option<SurfaceTemplate>`, whose `status: TemplateStatus { Current, LayoutChanged, Removed }` is written by Update Template and saved in format 1 (`tp-file/src/v1.rs`). Every surface of a project has a template (it is how a surface belongs to a vehicle texture); a symbol's surface has none.
- **Objects** (`document/object.rs`): `visible`, `locked`, `children` (groups and instances), `kind: ShapeKind` (Rectangle, Ellipse, Polygon, Path, Group, Text, Image, Instance). Swatch links are `Object::fill_swatch` (solid fill), `StrokeStyle::swatch` (solid stroke) and `ColorStop::swatch` (each gradient stop, `document/paint.rs`). `Project::relink` (`brand.rs`) drops a link whose value no longer equals its swatch, so a link always points to an existing swatch with the same value. An instance's `children` are the symbol's content expanded through its placement.
- **tp-app** `vehicle_project.rs`: `Workspace::dismiss_layout_change(index, now)` clears `LayoutChanged` as one undo step labelled `undo-dismiss-layout`. `mod_export.rs`: `problems_with(project, settings) -> Vec<Problem>` is what disables Export…, and `plan()` leaves `Removed` textures out of the mod and renders the others with a transparent background (`background: None`).
- **UI**: `ui/workspace/spaces/project.rs` (Vehicles header with `project-vehicles-count`, vehicle cards, `texture_tile` showing `vehicle::flag(...)` with `icons::WARNING` in `color::WARNING`, Mod information column); `ui/workspace/panels/vehicle.rs` (`texture_row` of the Textures tab with the same warning icon, `flag()`, and `texture_notices()` called by the inspector with Dismiss); `ui/workspace/inspector.rs` (`nothing_selected`); `ui/mod_export_dialog.rs` (problems listed with `dialogs::problem`, `color::ERROR`). `tp-ui` has `SegmentedControl` (`widgets/segmented.rs`).

## Goals / Non-Goals

**Goals:**
- One pure function per question in tp-core (state of a surface, off-palette colors of a list of objects), unit-tested, used by every view so the Project space, the Textures tab, the inspector and Export Mod never disagree.
- No new stored data, no file format change, no new undo kind.

**Non-Goals:**
- Caching states across frames (see Decision 6).
- A filter in the Textures tab, or a persisted filter.
- Listing the blocking problems in the Project space (they concern the mod settings, edited in Export Mod…; `mod-settings-in-project` will bring settings and checks to the Project space together).

## Decisions

### 1. The state lives in tp-core, derived from a `Surface`
```rust
pub enum TextureState { Empty, Modified, ToCheck(CheckReason) }
pub enum CheckReason { LayoutChanged, NotInVersion }
impl Surface { pub fn state(&self) -> TextureState }
pub fn is_drawn(o: &Object) -> bool // visible && (no children kind || children.iter().any(is_drawn))
```
`ToCheck` comes first from `template.status`; then `Modified` if `objects.iter().any(is_drawn)`, else `Empty`. `is_drawn` recurses into groups and instances (an instance's expanded children), so a group whose children are all hidden is not drawn. Opacity 0 still counts as drawn: the user asked for hidden objects only, and an invisible-by-opacity object is a deliberate edit, rare enough not to matter.
- *Alternative:* store a state per surface. Rejected: it would duplicate what the objects and the template already say, need a file format change, and go stale on undo.
- *Alternative:* put it in tp-app. Rejected: tp-core is where it can be unit-tested without egui, and the mod export, the Project space and the inspector all need it.

`CheckReason` keeps the version out: the UI reads it from `project.vehicle_of(index)` as `texture_notices` does today.

### 2. Off-palette colors are a walk over the texture's own objects
```rust
pub struct OffPalette { pub colors: Vec<Rgba>, pub objects: Vec<ObjectId> }
pub fn off_palette(objects: &[Arc<Object>]) -> OffPalette
```
It walks drawn objects (Decision 1), into groups, **not** into instances. For each Rectangle, Ellipse, Polygon, Path and Text: the fill when `fill_swatch` is `None` (solid) or each stop whose `swatch` is `None` (gradient); the stroke likewise with `StrokeStyle::swatch` and the stops. Images and groups are skipped (an image's `fill` field is unused). A color with `a == 0` is skipped. Colors are deduplicated by exact `Rgba` in first-seen order (`Vec` with `contains`, the lists are short); `objects` lists every object holding at least one, in tree order, innermost objects themselves (not their groups).
- A color equal to a swatch's value but not linked counts, by construction: only the link is looked at. "Link to Swatch" can come later.
- The function takes a list of objects rather than a `Surface`, so the inspector calls it on the symbol's surface while a symbol is edited ("In this symbol").
- *Alternative:* count colors inside instances on the texture. Rejected (user decision): they belong to the symbol, and would be counted once per instance.

### 3. Mark as Checked is the existing Dismiss
`dismiss_layout_change` keeps its code; only its labels change: the button reads **Mark as Checked** (`vehicle-panel-dismiss` → `texture-mark-checked`, its accessible name "Mark <vehicle › texture> as checked"), and the undo step `undo-dismiss-layout` reads "Mark as Checked". The Fluent ids are renamed in all four languages so no stale "Dismiss" string survives. `Removed` keeps no action. Nothing clears the flag on opening (no code to remove: nothing does today).

### 4. Where the state is drawn
One helper in `panels/vehicle.rs` replaces `flag()`: `state_label(state) -> String` and `state_tooltip(state, version) -> String` (the reason), used by both lists, the inspector and Before exporting.

| State | Project space tile (dot 6 pt + label, caption size) | Textures tab marker (right end, before the size) |
|---|---|---|
| Empty | dot `BORDER_STRONG`, label `TEXT_MUTED` | hollow ring 6 pt, stroke `TEXT_MUTED` |
| Modified | dot `TEXT_SECONDARY`, label `TEXT_PRIMARY` | filled dot 6 pt, `TEXT_SECONDARY` |
| To check | dot and label `SIGNAL` | `icons::WARNING` in `SIGNAL` |

The mockup's hex values (`#66666c`, `#3a3a3f`, `#a9a9b0`, `#f05252`) map to these tokens; the app's theme is lighter than the mockup's. The Textures tab draws no label (the row is narrow), so the states differ by shape: ring, dot, warning icon; the tooltip and the row's accessible name carry the label. The warning color moves from `color::WARNING` to `SIGNAL` (red = signal in the design system). The tile's state line takes the third line that the update flag uses today, so `TILE_TEXT` doesn't grow. `contrast.rs` covers `TEXT_MUTED` and `SIGNAL` on the panel and raised surfaces but not on `SURFACE_3`, the fill of the hovered and active tiles and rows, where `TEXT_MUTED` misses 4.5:1 (design of `new-design`, Decision 5). So on a hovered or active tile the Empty label uses `TEXT_SECONDARY`, and the contrast test gains `SIGNAL` on `SURFACE_3` (lifting `SIGNAL` is not in scope; if it fails, the To check label on those fills uses `TEXT_PRIMARY` and keeps the red dot and the tooltip).

### 5. Project space header and filter
- Counts: a new Fluent message `project-texture-states` with `$modified` and `$check`, each part left out when zero, appended to `project-vehicles-count` with " · ". Counting walks `project.surfaces` with `Surface::state()`.
- Filter: `TextureFilter { All, ToDo, ToCheck }` with `fn matches(self, TextureState) -> bool`, a field `texture_filter` of `Workspace` (beside `space`), reset to All by `Workspace::new` and not saved. A `SegmentedControl` in the header, left of Add Vehicle…. `textures()` filters `items` before deciding whether to draw a heading; when no texture of the vehicle matches, the card draws "Nothing to do" (`TEXT_MUTED`) in place of its textures.
- *Alternative:* collapse the vehicle card when nothing matches (the proposal's first wording). Rejected: cards have no collapsed state in the Project space today, and keeping the header (update notice, actions) is more useful than a new state.

### 6. No cache
`Surface::state()` stops at the first drawn object, so an all-Modified fleet of 40 textures costs 40 short walks per frame; Empty surfaces have few or no objects. `off_palette` runs only for the active surface, in the inspector, with nothing selected. Both are far below the canvas's per-frame cost. If a profile ever shows them, they can be keyed on the surface's `Arc` pointers, as `surface_thumbnails.rs` does, without changing callers.

### 7. Warnings beside problems
```rust
pub enum Warning { ToCheck { surface: usize, reason: CheckReason }, Empty { surfaces: Vec<usize> } }
pub fn warnings(project: &Project) -> Vec<Warning>
```
in `mod_export.rs`, next to `problems`. `Problem` is unchanged and still the only thing that disables Export… and that `plan()` returns. Texture names come from `project.surface_names(i)`; the vehicle prefix is added only when another surface has the same name.
- **Project space:** a "Before exporting" caps heading at the bottom of the Mod information column, then one row per warning: dot (state colors of Decision 4), wrapped text, and a small **Open** button (`set_active_surface(i)` and `space = Workshop`). The Empty line is a `CollapsingHeader`-like row listing each Empty texture with Open, or carries Open itself when there is one. With no warning: "Nothing to check" in `TEXT_MUTED`.
- **Export Mod dialog:** after the problems, the warnings with `icons::WARNING` in `color::WARNING` (problems keep `color::ERROR`), through a new `dialogs::warning(ui, text)` beside `dialogs::problem`. No Open in the dialog: it is modal and Cancel discards the edited settings, so leaving it from a warning would lose them silently. The dialog's height budget (`24.0 * problems.len()`) counts the warnings too.

### 8. "On this texture" in the inspector
A section after the texture's properties in `nothing_selected`, three rows label / value (value in mono 12, right-aligned, as the mockup):
- Objects: `surface.objects.len()` (top level, as the Layers header);
- Symbol instances: instances anywhere in the tree (`tree` walk, not into instances);
- Off-palette colors: `off_palette(&surface.objects).colors.len()`; above zero, in `SIGNAL` and clickable. Click: `ws.selection` set to the unlocked ids of `objects`, then `normalize_selection()` (hidden ones are already left out by Decision 2); the Layers tab expands their ancestors as it does for any selection. Locked objects left out are counted in the tooltip.

While a symbol is edited, the heading reads "In this symbol" and the instances row is left out. The section is labelled for assistive technologies with its heading, and the count button with "Select objects with off-palette colors".

### 9. Strings
New or renamed Fluent messages in en/fr/de/es (`panels.ftl`, `dialogs.ftl`, `canvas.ftl`): state labels (Empty, Modified, To check), reasons (layout changed in { $version }, not in this version), the header's state counts, the three filter options, "Nothing to do", "Before exporting", the warning lines (with plural forms), "Nothing to check", Open, "On this texture", "In this symbol", Objects, Symbol instances, Off-palette colors, the count's accessible name and tooltip, Mark as Checked and its undo label. French follows the mockup (Vide, Modifiée, À vérifier, Tout / À faire, Avant d'exporter, Sur cette texture, Couleurs hors palette); the localization test checks every key exists in every language.

## Risks / Trade-offs

- [A texture painted entirely by hidden objects reads Empty although the player worked on it] → That is what the mod will hold (transparent); the label says "Empty", and the warning says it is exported with the game's color, which is the point.
- [An object at 0 % opacity counts as Modified although nothing shows] → Accepted (Decision 1); rare, and hiding is the usual way to remove an object from the render.
- [Off-palette count of 0 can hide unlinked colors inside symbols] → The symbol's own count shows them while it is edited ("In this symbol"); a Brand-space count can come with `brand-edit-preview`.
- [Renaming Fluent ids breaks a translation silently] → The localization test fails on any missing key; old ids are removed, not left behind.
- [Header and Before exporting walk every surface each frame] → Early exit and short lists (Decision 6); `screenshots.rs` renders a 9-texture fleet, and a unit test times nothing but keeps the walk allocation-free for `state()`.
- [The "To check" red next to the "Update available" red on the same card] → Both are signals by design; each has its own text and icon.

## Migration Plan

None: no data change. Files from earlier builds already carry `TemplateStatus`; their textures get their state when they open. Rollback is reverting the change.
