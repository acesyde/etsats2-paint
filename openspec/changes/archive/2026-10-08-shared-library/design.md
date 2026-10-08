## Context

See proposal.md for the motivation. What exists:
- **References between elements:** an object refers to other elements of its project through project-local ids:
  - `ShapeKind::Image { asset }` → `AssetId`;
  - `fill_swatch`, `StrokeStyle.swatch` and gradient `ColorStop.swatch` → `SwatchId`;
  - `Object.style` → graphic `StyleId`, and `TextBlock.style_id` → text `StyleId`;
  - `ShapeKind::Instance { symbol, .. }` → `SymbolId`.

  A style's `Look` can link to swatches. Symbols hold an object tree and don't nest. Fonts are family names. All ids come from one counter in `Project` (`fresh_id`).
- **Links hold by value:** `Project::relink()` drops a link whose value no longer matches its source. `refresh_instances()` rebuilds each instance's content from its symbol.
- **Assets are deduplicated:** `add_asset` returns the existing asset with the same bytes (blake3).
- **The clipboard is broken across projects:** it is `AppState.clipboard: Vec<Object>`. It survives closing a project and is pasted with `Workspace::paste` → `Project::add_copies`, which keeps every reference id. That is the cross-project paste bug.
- **Undo already has an exception:** template opacity and visibility are kept across `Project::restore` and ignored by `same_document`.
- **Files:** projects are `tp-file` format 1 (a ZIP with a RON document and asset entries). `into_project` checks the fleet (vehicles and templates). Preferences show how app data is stored atomically with a backup of an unreadable file (`prefs.rs`).
- **Panels:** the Symbols, Colors and Styles panels already have per-element context menus built from `MenuRow` (`panels/symbols.rs`, `colors.rs`, `styles.rs`).

## Goals / Non-Goals

**Goals:**
- One import mechanism, pure and tested, used for library → project, project → library and paste across projects.
- No duplicate swatches and styles when the same identity comes back several times.
- A library that can never be half-written, and never blocks the app when it is unreadable.

**Non-Goals:**
- Live links and updating projects from the library.
- Several libraries, and sharing a library file.

## Decisions

### `LibraryKey` and an `origin` on library-able elements
`LibraryKey(String)`, 32 hex characters, identifies a library entry for good. `Swatch`, `GraphicStyle`, `TextStyle` and `Symbol` get `origin: Option<LibraryKey>`:
- in the library, every element has one: its own key;
- in a project, an element has one when it was imported from the library or added to it.

Keys are made in tp-app (tp-core has no clock) as the first 16 bytes of `blake3(time in nanoseconds ‖ process id ‖ counter)`, so no new dependency.

**Origins aren't part of the undo history,** following the template opacity precedent. `Project::restore` keeps the current origin of each swatch, style and symbol that still exists (matched by id), and `Snapshot` keeps origins out of the undo comparison. They are compared for the saved state, so recording one marks the project as having unsaved changes. Without this, undoing an edit made before Add to Library would forget the link, and the next Add would create a second library entry.

- *Alternative: make Add to Library an undo step.* Rejected. Undo would remove the link but not the library entry, which is confusing, and Add to Library doesn't change what the project draws.

### The library is a `Project` without vehicles
In memory, the library is a `tp_core::Project` with no vehicles and one empty placeholder surface. It holds the palette, styles, symbols and assets. With the same container on both sides, one import function serves every direction:
- library → project (Import from Library);
- project → library (Add to Library);
- project → project (paste).

The placeholder surface is never shown.

- *Alternative: a dedicated `Library` struct.* Rejected for now. It would duplicate the brand kit, symbols and assets code, and every direction of the import would need its own variant.

### `tp_core::import`: closure, then copy with reuse
`Picks { symbols, swatches, graphic_styles, text_styles, objects: Vec<Object> }` names what to bring. `objects` is for paste.

**Closure:** `closure(from, picks) -> Closure` walks the picks and every object tree they contain, collecting the symbols, styles (and the swatches their looks link to), swatches and assets they reference. Symbols don't nest, so one pass over each symbol's tree is enough.

**Copy:** `import(from, picks, into, mode) -> Imported { maps, objects }` copies in dependency order, each step building an id map:
1. **Assets:** `add_asset` (same bytes → the existing asset).
2. **Swatches:** reused when `into` has one with the same `origin`, or the same name and color. Otherwise added with a fresh id, the source's origin and a unique name.
3. **Styles:** looks are remapped to the new swatch ids first. A style is reused on the same origin, or the same name and an equal look (and character style for a text style). Otherwise added.
4. **Symbols:** their content is remapped (assets, swatches, styles), with fresh object ids. A symbol is reused on the same origin only, since there's no cheap identity test for content. Otherwise added with a unique name.
5. **Objects** (paste): remapped the same way, instances pointing to the new symbol ids.

Then `relink()` and `refresh_instances()` run on `into`.

**Unique names:** "Logo", then "Logo 2", "Logo 3"… The first free one wins, among elements of the same kind.

`ImportMode::Import` never changes an element it reuses. `ImportMode::Publish` (Add to Library) replaces the content and name of an element reused **by origin** with the source's, keeping its id. It returns the keys to record in the source: new elements get a fresh key, and the source elements get theirs.

### Library storage
`AppDirs::library()` is `<data>/library.tplib`.

**The file:** `tp-file` gains a `library` module. A library file is a ZIP like a project:
- a `mimetype` of `application/x-truckpaint-library`;
- `library.ron`, holding a `FileProject`: format 1, no vehicles, one placeholder surface;
- `assets/…`.

**Reading and writing:**
- `into_project` is split so that the library path skips the fleet check: `into_document` builds the project, and `into_project` adds `check_fleet`.
- Writing goes through `write_atomic`.
- Unused assets are dropped before each save.

**`LibraryStore` in tp-app** (on `AppState`):
- it loads lazily, the first time the library is used;
- an unreadable file is renamed to `.bak-<timestamp>`, as for preferences, and the library starts empty with a one-time message;
- it saves after each change, on the background saver used for projects (`Saver`), so the UI never waits for the disk.

### UI
- **Context menus:** Add to Library or Update in Library (when the element's origin exists in the library), in the Symbols, Colors and Styles panels. It calls `LibraryStore::publish`. The project's new origins are set directly (outside the history) and mark it unsaved. A hint confirms: "Added to the library".
- **`CommandId::ImportFromLibrary`:** in the Object menu, after Convert to Symbol. It is disabled while a symbol is edited. It opens `Modal::ImportFromLibrary(LibraryDialog)`.
- **The dialog:**
  - four groups with checkboxes;
  - previews: color chips for swatches, a small `render_cover` of each symbol's surface, rendered on a worker like the mod pictures;
  - "In this project" for elements whose origin the project has;
  - Import runs `Workspace::import_from_library` (one `edit` step, `undo-import-from-library`);
  - Remove from Library asks for confirmation.
- **Empty panels:** the Symbols, Colors and Styles panels show an "Import from Library…" button next to their existing empty-state actions.

### Paste across projects
- **Each open workspace gets a `session: u64`** from a counter in `AppState`. It isn't saved.
- **The clipboard becomes `Clipboard { objects, source: Option<(u64, Arc<Project>)> }`.** Copy and Cut store the current project, cheap since objects and assets are `Arc`. The clipboard therefore keeps the dependencies as they were, even after the source closes.
- **Paste:**
  - from the same session, as today;
  - from another session, `Workspace::paste_from(source, objects)` runs `import` with `Picks { objects }` inside the same `edit` as the paste (`cmd-paste`), then selects the pasted objects.

  A project closed and reopened gets a new session, so pasting into it goes through the import. The reuse rules find its own swatches and styles by name and value, so nothing is duplicated.

## Risks / Trade-offs

- **[A symbol imported twice without an origin]** Two projects that each converted the same drawing to a symbol, then pasted between them, get "Logo 2". Symbols are only reused by origin. That's acceptable, since content comparison would be costly and fragile.
- **[The library file grows with images]** Unused assets are dropped on each save. Logos are small compared to textures.
- **[Clipboard keeps a whole project alive]** The snapshot holds `Arc`s shared with the open project. After the source closes, the memory stays until the next copy. Acceptable for one clipboard.
- **[Origins outside the history]** An origin recorded, then the element deleted and the deletion undone: the restored element has the origin it had in the snapshot. Undo restores the element as it was, which is the expected outcome.
