## 1. File format (tp-file)

- [x] 1.1 Create the `tp-file` crate (deps: tp-core, serde, ron, zip with deflate only) with versioned DTOs (`FileProject`, `FileSurface`, `FileObject`/`FileKind`, `FileText`, `FileAsset`) and `FORMAT_VERSION = 1`; add `Project::from_parts` in tp-core restoring ids and `next_id`; verify unit tests converting a rich project to DTOs and back (groups, texts, images, palette, flags, ids preserved, `next_id` beyond them)
- [x] 1.2 Implement `write(project, path)` (mimetype entry, `project.ron`, `assets/<id>.<ext>`, temp file + `sync_all` + rename) and `read(path)`; verify round-trip tests through a temp dir and that a failed write leaves the previous file byte-identical
- [x] 1.3 Implement version handling: `Header { format }` read first, frozen `v1` module, migration chain entry point (identity for v1) and a "migrated" flag returned by `read`; commit a `tests/fixtures/v1.truckpaint` fixture; verify tests that the fixture opens with the expected content, that the header version is read even when the body does not parse, and that writing records the current version
- [x] 1.4 Implement errors (`NotAProject`, `Damaged`, `NewerVersion`, `Io`) with user-facing messages; verify tests for a non-ZIP file, a truncated file, a newer version and an image referencing a missing asset

## 2. Save state and saving (tp-app)

- [x] 2.1 Replace the `save_state` flag by `saved: Option<Snapshot>` and `path: Option<PathBuf>` in `Workspace`, deriving Saved/Unsaved; verify unit tests: new project unsaved, saved after marking, edit → unsaved, undo back → saved
- [x] 2.2 Add the save worker (background thread, atomic write, completion polling with repaint, queued second save) and the "Saving…" status; verify a unit test saving through the worker to a temp dir and the status text while pending
- [x] 2.3 Add `FileDialogs` (rfd default; fakes in tests) and make Save / Save As / Open commands available with their shortcuts (Save and Save As global); verify kittests "First save", "Save after changes" and "Undo back to the saved state"
- [x] 2.4 Report save failures in a message modal, keeping the project unsaved; verify the "Save to a read-only location" kittest (destination in a read-only temp dir)

## 3. Opening

- [x] 3.1 Implement `open_file(path)`: read, rebuild the workspace (fitted view, empty history, saved — or unsaved when the file was migrated), relayout texts, update the recent list (front, dedup, cap); same-path open does nothing; verify kittests "Round trip" (save, close, reopen and compare documents) and "Open from the menu"
- [x] 3.2 Show read errors in a message modal without touching the open project; verify kittests "Damaged file" and "Newer format"
- [x] 3.3 Enable Open Project and recent entries on the home screen (missing files stay disabled); verify kittests "Open from the home screen", "Open a recent project" and "Saved project appears in recent list"

## 4. Unsaved changes prompt

- [x] 4.1 Add `PendingAction` and the Save / Don't Save / Cancel modal (Enter = Save, Escape = Cancel) guarding Close, New Project, Open and Quit; Save continues only after a successful save; verify kittests "Cancel keeps working", "No prompt when saved" and Don't Save closing to the home screen
- [x] 4.2 Intercept window close (`close_requested` → `CancelClose` + prompt, then `Close`); verify the "Quit with unsaved changes" kittest by injecting a close request

## 5. Crash recovery

- [x] 5.1 Add the recovery directory, per-session copy + metadata written through the save worker every 120 s while unsaved and changed, deleted on save, close and normal quit; verify kittests advancing harness time ("Recovery copy while editing", "Normal close removes the copy")
- [x] 5.2 Scan recovery copies at launch (skip the current session and copies of other running sessions), show the "Recovered projects" section with Restore / Discard (damaged copies: Discard only); verify kittests "Restore unsaved work", "Discard a recovered project" and "Recovery shown first" using a second `AppState` on the same data dir

## 6. Integration

- [x] 6.1 Screenshot review of the save prompt, the error message, the home screen with recovered and recent projects, and the "Saving…" status at 100% and 200% UI scale
- [x] 6.2 Run `openspec validate persistence --strict` and `mise run ci`; verify both pass locally and the CI matrix is green on the PR
