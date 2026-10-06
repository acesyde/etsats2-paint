## Why

Everything a user creates in TruckPaint is lost when the window closes: Open, Save and Save As are disabled, recent projects cannot be opened, and nothing protects work from a crash. Now that liveries contain shapes, groups, lettering and imported logos, saving and reopening projects is the main missing piece before the editor is usable day to day.

## What Changes

- **`.truckpaint` project file**: one self-contained file holding the whole project — surfaces, the object tree (shapes, groups, texts with their character style, images), palette, and the imported asset files (PNG, JPG, SVG bytes). Fonts are referenced by family name only (bundled fonts are always available; a missing system font falls back to Inter with the existing warning). The file carries a format version; files from a newer TruckPaint are refused with a clear message.
- **Save, Save As, Open**: File › Save (Cmd/Ctrl+S), Save As… (Cmd/Ctrl+Shift+S) and Open… (Cmd/Ctrl+O) become available, with native file dialogs filtered to `.truckpaint`. Saving is atomic (a failed or interrupted save never corrupts the previous file) and does not freeze the editor. Read and write errors are reported with the file name and reason.
- **Save state follows the history**: after saving, the status bar shows "Saved"; undoing back to the saved state shows "Saved" again.
- **Unsaved changes prompt**: closing the project, quitting, closing the window, creating a new project or opening another one with unsaved changes asks "Save changes to “<name>”?" with Save, Don't Save and Cancel.
- **Recent projects open**: Open Project and recent entries on the home screen become enabled; opening or saving a project moves it to the top of the recent list.
- **Crash recovery (autosave)**: while a project has unsaved changes, a recovery copy is written every 2 minutes to the application data folder — never over the user's file. Recovery copies are deleted when the project is saved or closed normally. After a crash, the home screen offers to restore or discard each recovered project.

Not in this change: exporting PNG/DDS or game mods (`export-and-preview`), embedding font files, project thumbnails, cloud or version history, opening several projects at once.

## Capabilities

### New Capabilities

- `project-files`: the `.truckpaint` format, Save / Save As / Open, atomic writes, versioning, errors, and the unsaved-changes prompt.
- `crash-recovery`: periodic recovery copies, their cleanup, and restoring them after an abnormal exit.

### Modified Capabilities

- `start-screen`: Open Project and recent entries become enabled and open projects; the "Open Project availability" placeholder requirement is removed; recovered projects are offered on the home screen.
- `undo-history`: the save state returns to "Saved" when undo/redo reaches the saved state, and saving marks the current state as saved.

## Impact

- New crate `tp-file`: versioned serde data structures for the file format (independent of `tp-core` types), ZIP container (`zip` crate) with `project.ron` and `assets/`, reading/writing with atomic replace. `tp-core` gains constructors to rebuild a project with given ids (`Project::from_parts`) and access to its id counter.
- `tp-app`: Open/Save/Save As commands enabled; `rfd` dialogs; background save thread; save marker in `History`; unsaved-changes modal; window close interception (`close_requested` / `CancelClose`); recent list updates; recovery writer and recovery banner on the home screen; texts relaid out after opening.
- `tp-app::paths`: `recovery` directory under the data directory.
- New dependency: `zip` (deflate only).
