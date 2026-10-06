## Context

After `text-and-images`, a `Project` (tp-core) holds `name`, `resolution`, `surfaces` (each an `Arc<Object>` tree), `active_surface`, `palette`, `assets: BTreeMap<AssetId, Arc<Asset>>` and a private `next_id` counter shared by objects and assets. Objects carry `kind`, `frame`, fill/stroke/opacity, flags, `children`, and `text: Option<TextBlock>` (content, `CharStyle`, cached `layout_size`, `scale`). History stores Arc-shared `Snapshot`s, so capturing the document is cheap. `Workspace.save_state` is set to `Unsaved` by every recorded change and never back. In `tp-app`, Open/Save/Save As exist as `NotYet` commands; the home screen shows recent entries (`Prefs.recent: Vec<RecentProject { name, path, last_opened }>`) disabled; `AppDirs.data` exists ("later autosave data"); `rfd` is already a dependency (Place…). Quit sends `ViewportCommand::Close` directly, and New Project replaces an open project without asking. User decisions: autosave writes **recovery copies only** (never the user's file); fonts are stored **by family name only**.

## Goals / Non-Goals

**Goals:**
- A stable, versioned, self-contained file format that can evolve (vehicle templates, multiple surfaces, new object kinds) without breaking old files.
- Saving never corrupts the previous file and never blocks the UI on large projects (8K surfaces with several MB of assets).
- No silent data loss: unsaved-changes prompt everywhere a project can go away, plus crash recovery.

**Non-Goals:**
- Exports (PNG/DDS/mod), thumbnails, embedded fonts, several open projects, file watching, cloud sync, saving undo history.

## Decisions

### D1. Container: ZIP with RON document and raw assets
`.truckpaint` is a ZIP archive:
- `mimetype` (stored, first entry): `application/x-truckpaint` — cheap identification.
- `project.ron` (deflated): the document.
- `assets/<id>.<png|jpg|svg>` (PNG/JPG stored, SVG deflated): original bytes, verbatim.

RON matches the preferences format already in use, is human-readable for debugging, and diffs reasonably. Assets stay as separate entries so large logos are not base64-inflated and can be read lazily later.
*Alternatives:* a single RON/JSON file with base64 assets (33 % larger, slow for big images); a folder bundle (awkward to share and on Windows); SQLite (overkill, opaque).

### D2. Separate, versioned file model in a new `tp-file` crate
`tp-file` defines serde DTOs (`FileProject`, `FileSurface`, `FileObject` with a tagged `FileKind`, `FileText`, `FileAsset` metadata) independent of tp-core's in-memory types. This lets tp-core types change freely (Arc trees, caches) while the file format stays stable. Object and asset ids are written as-is; tp-core gains `Project::from_parts(...)` that rebuilds a project and sets `next_id` to one past the largest id found. `layout_size` is saved (bounds stay meaningful for any reader) but the app relays out every text after opening.

**Versioning and migrations.**
- `project.ron` starts with `format: u32` (`FORMAT_VERSION = 1` for this change). It is a top-level field read first, before the rest of the document, through a tiny `Header { format }` parse, so the version is known even when the rest no longer matches the current DTOs. The `mimetype` entry never changes; it only identifies the file as a TruckPaint project.
- Each released format keeps a frozen DTO module (`v1`, later `v2`, …). Reading parses the file with the module of its own version, then applies migration functions in sequence (`v1 → v2 → … → current`), each a pure `fn(vN::FileProject) -> vN+1::FileProject` that adds defaults, renames or drops data. Writing always uses the current version.
- When migrations ran, the app opens the project with `saved = None` (unsaved), so the user's file is only rewritten in the new format on an explicit save.
- `format > FORMAT_VERSION` → `NewerVersion`. Unknown optional fields are ignored (`#[serde(default)]`), so a later minor addition that older readers can safely skip does not need a version bump. Any removal, rename or change of meaning does need a bump plus a migration.
- Rule for contributors, enforced by tests: every format change bumps `FORMAT_VERSION`, freezes the previous module, adds a migration, and adds a fixture file `tests/fixtures/vN.truckpaint` that must keep opening forever (a fixture round-trip test per version).

### D3. Errors
`tp-file::Error { NotAProject, Damaged(String), NewerVersion { found, supported }, Io(io::Error) }` with user-facing messages ("ace.truckpaint is damaged and cannot be opened", "… was created with a newer version of TruckPaint"). Invalid references (an image whose asset is missing) make the file `Damaged` rather than silently dropping objects.

### D4. Atomic, background saving
Saving captures `Project` (Arc-shared, cheap clone) and sends it to a save worker thread, which writes `<name>.truckpaint.tmp-<pid>` in the same directory, `sync_all`s it, then renames it over the destination (`std::fs::rename`, atomic on the same volume on all three OSes; on Windows it replaces existing files). The workspace keeps `saving: Option<PendingSave>`; the status bar shows "Saving…"; completion (`Ok(path)` or `Err`) is polled each frame (worker calls `request_repaint`). On success the snapshot captured at save time becomes the saved state (edits made during the save stay unsaved). A second Save while one is running is queued (latest wins).

### D5. Saved state derived from snapshots
Replace the `save_state` flag by `saved: Option<Snapshot>` in `Workspace` (set on save and on open, `None` for new projects). `save_state()` returns `Saved` iff `saved` exists and `snapshot().same_document(saved)` — mostly pointer comparisons thanks to Arc sharing, cheap enough per frame. Undoing back to the saved state therefore shows "Saved" with no extra bookkeeping in `History`.

### D6. Unsaved-changes prompt as a guarded action
A `PendingAction { CloseProject, Quit, NewProject, Open(PathBuf), OpenDialog, RestoreRecovery(id) }` is stored when an action would discard unsaved work, and a new `Modal::UnsavedChanges(action)` shows Save / Don't Save / Cancel (Enter = Save, Escape = Cancel). Save runs Save (or Save As) and continues the action when the save completes successfully; a failed or cancelled save keeps the modal context closed and the project open. Window close: in `AppState::show`, `ctx.input(|i| i.viewport().close_requested())` with unsaved changes sends `ViewportCommand::CancelClose` and opens the prompt with `Quit`; Quit then sends `Close` with a flag allowing it through.

### D7. File dialogs behind a trait
`AppState.file_dialogs: Box<dyn FileDialogs>` with `open_project() -> Option<PathBuf>`, `save_project(suggested_name) -> Option<PathBuf>` and the existing `pick_images()`; the default uses `rfd` (filters `truckpaint`), tests inject fakes returning temp paths. Save As appends `.truckpaint` when the user omits it. Dialogs are blocking, run between frames when the command is dispatched.

### D8. Crash recovery
`AppDirs.recovery()` = `<data>/recovery/`. Each running session writes `<session-uuid>.truckpaint` (same format) plus `<session-uuid>.ron` metadata `{ name, original: Option<PathBuf>, saved_at }`. The workspace tracks `last_recovery: f64` and the snapshot last written; every frame, if unsaved and 120 s have passed since the last recovery write and the document differs from the last recovery snapshot, the project is sent to the save worker with a recovery destination (same atomic write). The copy and metadata are deleted on successful save, on closing the project, and on normal quit. At launch, `recovery::scan()` lists metadata files not belonging to the current session. Each running session holds an exclusive OS lock (`File::try_lock`, std ≥ 1.89) on its `<session-uuid>.lock` file for its whole lifetime; the OS releases it when the process ends, even on a crash. A scan offers a session's copy only if it can take that session's lock (owner gone) and skips it otherwise (another instance is running). The home screen shows a "Recovered projects" section; Restore opens the project with `saved = None` (unsaved) and `path = original`, then deletes the copy once the project is saved or closed like any other session copy (it becomes this session's copy).

### D9. Recent list and opening
Opening (dialog, recent entry or restore) goes through `AppState::open_file(path)`: guarded by D6, reads with `tp-file` on the UI thread (files are local; a progress spinner is not needed for typical sizes — reading is I/O + RON parse, decode of assets stays lazy in `ImageCache`), builds a `Workspace` with `path = Some(path)`, `saved = Some(snapshot)`, relayouts texts, then updates `Prefs.recent` (move to front, dedup by canonical path, cap `MAX_RECENT`). Opening the path of the already open project does nothing. Errors open a small error modal (`Modal::Message`) rather than a canvas hint, because they can happen on the home screen.

### D10. Commands and UI
`OpenProject` (Cmd/Ctrl+O, Global), `Save` (Cmd/Ctrl+S, Global, enabled with a project), `SaveAs` (Cmd/Ctrl+Shift+S, Global) become available. The status bar shows "Saving…" with a spinner icon during D4. Window title is unchanged (`<name> — TruckPaint`).

### D11. Testing
- `tp-file`: a `v1` fixture file committed under `tests/fixtures/` opens and matches the expected document (the start of the per-version fixture suite); the header version is read before the body; round trip of a rich project (groups, rotated/scaled texts, images with assets, palette, hidden/locked flags) equal after reload; ids preserved and `next_id` beyond them; newer version rejected; non-zip and truncated files → `NotAProject`/`Damaged`; missing asset reference → `Damaged`; atomic write leaves the old file intact when the writer fails (simulated with an unwritable temp dir).
- `tp-app` kittests with fake dialogs and temp dirs: first save asks location and shows "Saved"; save again without dialog; undo back to saved state shows "Saved"; open replaces with prompt; prompt Save/Don't Save/Cancel for close, new project, window close (`close_requested` injected); recent list updated and recent entry opens; damaged file message; recovery written after 120 s of harness time, deleted on close, restored at next launch (new `AppState` on the same data dir), discard removes it.

## Risks / Trade-offs

- [Rename is not atomic across volumes] → the temp file is created next to the destination, so the rename stays on the same volume.
- [Saving while editing could capture an inconsistent document] → the snapshot is taken on the UI thread between frames; pending live edits are committed first (`commit_pending`), and an active text session is included as it stands without being ended.
- [Format lock-in] → independent DTOs and an explicit version with migrations; the first version is reviewed against the planned vehicle-template data (multiple surfaces already modelled).
- [Large assets duplicated in recovery copies every 2 minutes] → only rewritten when the document changed; assets are written stored (no recompression), so cost is mostly disk I/O in a background thread.
- [Two running instances and recovery] → per-session OS file locks, released by the OS on exit or crash; locked sessions are skipped.

## Migration Plan

No existing project files. Preferences keep their format; recent entries recorded before this change (none exist in practice) simply open if their file is a valid project.

## Open Questions

- Whether to show a thumbnail in recent entries — deferred until a canvas renderer for exports exists (`export-and-preview`).
