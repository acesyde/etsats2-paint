## Context

See proposal.md for why. What exists today, after `new-design` and `texture-status`:

- **Project space** (`ui/workspace/spaces/project.rs`): a fixed 360 pt right panel, `mod_information`, inner margin 18 pt (324 pt of content). It shows the two pictures at their pixel size from `ws.mod_previews` (`mod_previews.rs`, rendered on a worker thread, keyed on the first texture's objects, the fonts and `mod_settings.icon/image`), the Name, Author/Version and Description as plain text through `spaces::value`, the **Edit in Export Mod…** button (`project-edit-in-export`, `CommandId::ExportMod`), `vehicle::game_versions_field` and `before_exporting` (warnings only).
- **Game versions** (`ui/workspace/panels/vehicle.rs::game_versions_field`): one `TextEdit` whose buffer lives in egui temp data while focused, committed on focus loss through `Workspace::set_game_versions` unless `take_escape` says Escape was pressed. `game_versions::parse_list` splits on commas.
- **Export Mod dialog** (`ui/mod_export_dialog.rs`): `ModExportDialog` holds a copy of `ModSettings`, two `Picture`s (`Generated | Asset | File`), its own preview job, the summary and the `ModJob`. `start()` calls `Workspace::set_mod_settings(settings, icon, image)` (one "Edit Mod Settings" step that adds new picture assets and removes the old ones), then asks `FileDialogs::save_mod(suggested, folder)` and starts `ModJob`. The replacement of an existing file is only confirmed by the native save dialog. The folder is `game_mod_folder(game)`, else `AppState::last_mod_folder`, else the dialog's default.
- **Checks** (`mod_export.rs`): `problems_with(project, settings)` returns `Problem`s in display order; `warnings(project)` the texture warnings; `problem_message` lives in the dialog module. `SamePath`, `MissingGameData` and `VehicleRange` carry vehicle names only.
- **Undo** (`workspace.rs`): `Workspace::edit(label, now, coalesce, f)` snapshots, applies and calls `record`, which skips the step when `Snapshot::same_document` sees no change; `same_document` already compares `mod_settings` and `game_versions`. Labels `undo-edit-mod-settings` and `undo-edit-game-versions` exist in all four languages.
- **Dropped files** (`AppState::after_frame`): packages are installed, files over the Custom Vehicle dialog become templates, everything else is placed on the active texture (at the pointer when over the canvas, else in the middle of the view), whatever the space.
- **Tokens and helpers**: `color::FIELD` (sunken fields), `color::CHIP`, `color::ERROR`, `color::WARNING`, `color::TEXT_SECONDARY/MUTED`, `radius::MD/LG`, `space::*`; `ui::dialogs::{labelled, field_label, text_field, mono_text_field, columns, problem, warning, footer, FIELD_HEIGHT (34)}`; `tp_ui::widgets::{NumericField, remember_escape, take_escape, secondary_button, primary_button, paint_focus_ring}`.

The mockup (artboard 03, "Informations du mod") orders the column picture, Name, Author | Version (96 pt), compatible game versions as chips with ✕ and "+ ajouter", Description; it also shows an Export section (folder, "Dernier export"), which stays out (proposal, Non-goals). The app's theme is lighter than the mockup: fields use `color::FIELD` with `BORDER`, labels above, 34 pt high.

## Goals / Non-Goals

**Goals:**
- One place that edits the mod settings, with the same commit rules for every field (Enter or leaving commits, Escape restores, nothing recorded when unchanged).
- The dialog reads the project and never writes to it; the export uses `ws.project` as it is.
- One mapping from a `Problem` to where it is fixed, shared by the field messages, Before exporting and the dialog's Show.

**Non-Goals:**
- No change to `ModSettings`, the file format, the problem rules or the mod's contents.
- No per-project memory of the destination (a "Last export" line or a saved path); the destination is computed when the dialog opens, by the rules of today.
- No Open on the dialog's texture warnings (they could have one now that leaving the dialog loses nothing, but that is not asked).
- No marking of unsupported (well-written) versions on their chip: only badly written ones, as asked; the unsupported problem is shown under the field.

## Decisions

### 1. A reusable "committed text field" for the column
The column needs the same behaviour on five text fields (Name, Author, Version, internal name, Description) that `game_versions_field` already implements by hand. Factor it into one helper in `ui/workspace/spaces/project.rs` (or `ui/dialogs.rs` next to `text_field`): `committed_field(ui, id, label, current: &str, kind: Single | Mono | Multiline) -> Option<String>`.
- The edit buffer is kept in egui temp data under `id` only while the field has focus; when it doesn't, the field shows `current`, so Undo/Redo show at once.
- `remember_escape` before drawing; on `lost_focus`: drop the buffer, and return `Some(buffer)` unless `take_escape` (Escape restores by simply dropping the buffer). Enter on a single-line `TextEdit` makes it lose focus, so Enter and leaving are the same path. A multi-line `TextEdit` keeps Enter as a new line and commits on focus loss only.
- Drawn like `dialogs::text_field_in` (34 pt, `Margin::symmetric(10, 0)`, centered, `TextStyle::Monospace` for Version), `FIELD` fill comes from the theme's text-edit visuals, labelled above with `field_label` and named for AccessKit with its label.
- Price and Unlock level use `NumericField` with `.fill().inset(FIELD_HEIGHT)` and an empty scrub label, as the dialog does; only `FieldEvent::Commit` is applied (no `Live`, so no `live_edit` / `commit_pending` dance), `Revert` does nothing.

Alternative considered: keep `ModSettings` in a column draft and commit it as a whole (like the dialog). Rejected: it reintroduces a "pending" state the user can lose, and per-field steps are what was asked.

### 2. Per-setting edits on `Workspace`
Replace `Workspace::set_mod_settings(settings, icon, image, now)` with:
- `set_mod_setting(now, f: impl FnOnce(&mut ModSettings))`: `self.edit("undo-edit-mod-settings", now, false, |p, _| f(&mut p.mod_settings))`. `record` already drops a step that changes nothing, so "unchanged value adds no step" needs no extra code; tests assert it.
- `set_mod_picture(which: ModPicture, picture: Picture, now)`: one "Edit Mod Settings" step that adds a `Picture::File` as an asset (`Picture::add_to`), sets `mod_settings.icon` or `.image`, and removes the previous asset when no longer used (as today). `ModPicture { Icon, Image }` replaces the dialog's private `Which`.
- The internal name commits `Some(typed)` only when `typed != settings.internal_name(limit)`, so leaving the field untouched keeps it following the Name.
- `set_game_versions` stays; add `add_game_versions(&[String], now)` (appends the parsed entries not already listed, one step, nothing when none is new) and `remove_game_version(index, now)` in `game_versions.rs`.

### 3. Where a problem is fixed: `Problem::place`
Add in `mod_export.rs`:
```rust
pub enum ModField { Name, Author, Version, Description, Price, UnlockLevel, InternalName, GameVersions }
pub enum ProblemPlace { Field(ModField), Vehicle(String /* package id */) }
impl Problem { pub fn place(&self) -> ProblemPlace }
```
The mapping is the table of the project-screen spec (Problems before exporting). To point at a card, `SamePath` and `MissingGameData` gain the `package_id` of the vehicle concerned and `VehicleRange` gains `package_id` (filled in `game_versions::vehicle_ranges`); the messages don't change. `problem_message` moves from `ui/mod_export_dialog.rs` to a shared place (`mod_export.rs`, like `Warning::message`) so the column and the dialog use the same text.

`problems_with(project, settings)` loses its only caller with a draft settings; keep `problems(project)` and make `problems_with` private (or drop it).

### 4. Going to a place: a one-shot request on the workspace
`Workspace` gets `reveal: Option<ProblemPlace>` (transient, not in snapshots). Show (in the dialog or in Before exporting) sets it and `space = Space::Project`; the dialog's Show also closes the dialog. On its next frame the Project space consumes it:
- `Field(f)`: if `f == InternalName`, open Advanced; the field whose `ModField` matches calls `response.scroll_to_me(Some(Align::Center))` and `request_focus()`. `GameVersions` focuses + Add (turning it into its text field).
- `Vehicle(id)`: the card whose `package_id` matches calls `scroll_to_me` on its header.
Advanced's open state lives in egui temp data under a column id, forced open while `problems` holds an internal-name problem (same rule as the dialog today).

### 5. Problems in the column
`mod_information` computes `problems(&project)` once per frame. Each field draws, under it, the problems whose place is that field with `dialogs::problem` (error icon + `ERROR` text, so never color alone). `before_exporting` lists all problems first, each a `problem`-style line with a small **Show** button built like `open_button` (same width logic, accessible name "Show <field label>" / "Show <vehicle>"), then the warnings exactly as today. "Nothing to check" only when both lists are empty. Strings: `project-show`, `project-show-named`.

### 6. Pictures: controls, drop zones and errors
`project.rs::picture` keeps its preview/placeholder drawing and adds, under the preview, the size (`"256 × 64"`, mono, `TEXT_MUTED`) and the buttons (`secondary_button`, `horizontal_wrapped` so German labels wrap instead of clipping): Choose Icon…/Choose Image… (`mod-choose-icon`, `mod-choose-image`) and, when chosen, Use Generated Icon/Image (`mod-generated-icon`, `mod-generated-image`). The placeholder text gains "or drop a PNG or JPEG here" (`project-picture-drop`).
- **Choose:** `state.dialogs.pick_mod_image()` can't be called from the panel (it borrows `AppState`); the panel sets `ws.picture_request = Some(ModPicture)` and `AppState::after_frame` runs the picker, reads the file with `PictureFile::read`, then `ws.set_mod_picture` (same pattern as `place_request`).
- **Drop:** the panel stores each preview's screen rect in `ws.picture_drop_zones: [Option<Rect>; 2]`, cleared at the start of every frame of the Project space (so they're `None` in other spaces). In `after_frame`, before placing dropped files, a drop whose pointer position is inside a zone takes the first file: `PictureFile::read` → `set_mod_picture`, and the files are not placed on the canvas. While `raw.hovered_files` isn't empty, both zones draw a dashed `BORDER_STRONG` outline, and the one under the pointer fills with `SURFACE_2` (as the Resources drop zone does).
- **Errors:** `ws.picture_error: Option<(ModPicture, String)>` (transient), shown under that picture with `dialogs::problem`, set on a read failure, cleared on the next successful change of either picture.
- Previews: `ModPreviews` already re-renders when `mod_settings.icon/image` change; nothing to add. The dialog's own `PreviewJob` and `start_previews` go away.

### 7. Game versions chips
`game_versions_field` becomes: label, a wrapping row on a `FIELD` box (`BORDER` outline, `radius::MD`, 5 pt padding, as the mockup) holding one chip per version — `CHIP` fill, pill radius, version in `TextStyle::Monospace` at `typography::MONO`, an `icons::CLOSE` button with a 24 pt hit area named "Remove <version>" (`project-game-version-remove`) — then **+ Add** (`project-game-versions-add`, `TEXT_SECONDARY`). Clicking + Add swaps it for a small `TextEdit` (hint `1.56.*`, id stable so the reveal can focus it) using the committed-field helper; its commit calls `add_game_versions(parse_list(text))`. A chip whose `of_listed(version)` is `None` is drawn with an `ERROR` outline and `icons::WARNING`, hover text `mod-problem-bad-game-version`. The supported-versions hint stays under the box, unchanged.

### 8. The dialog: summary, destination, problems, warnings, buttons
`ModExportDialog` keeps `summary`, `job`, and gains `destination: PathBuf`, `confirmed: Option<PathBuf>` (a path whose replacement the native dialog confirmed) and `confirm_replace: bool`. It loses `settings`, `icon`, `image`, `picture_error`, previews and `advanced`.
- **Destination at open:** `mod_export::destination(project, last_folder)`: folder = `game_mod_folder(game)` else `last_mod_folder` else `UserDirs::document_dir()` else `home_dir()`; file = `suggested_name(&mod_settings.name)`. Computed in `CommandId::ExportMod` **after** committing a field being edited (see Risks), so the proposed name uses the committed Name.
- **Row:** `field_label("Destination")`, then a horizontal row: the path in monospace, `TEXT_PRIMARY`, elided from the start to the width left by Change… (lay out the galley, drop leading characters behind "…" until it fits), hover shows it in full; Change… a `secondary_button`. The path is plain text, not a sunken box: `new-design` decided read-only values are plain text so they aren't mistaken for fields (the mockup draws a box; documented deviation).
- **Change…:** `save_mod(file_name, Some(folder))`; on `Some(path)`: `destination = with_extension(path)`, `confirmed = Some(destination.clone())` when it exists, `last_mod_folder` updated.
- **Export:** if `destination.exists() && confirmed != Some(destination)`, set `confirm_replace`; the footer then shows "<file> already exists. Replace it?" (`mod-export-replace-question`) with Replace (primary, `mod-export-replace`) and Cancel. Otherwise `ModJob::start(ws.project.clone(), plan, fonts, destination, …)` and `last_mod_folder = parent`. No `set_mod_settings`, no history.
- **Problems:** `problems(&ws.project)` each frame, drawn with `dialogs::problem` plus a Show button (decision 4); clicking it sets `ws.reveal`, `ws.space = Project` and closes the dialog.
- **Buttons:** Export (`mod-export-start`, "Export", no ellipsis since it no longer opens a dialog; `export-start` stays for Export Texture) disabled while a problem remains; Cancel. Escape closes unless an export runs. Dialog width stays 640 pt; the body scroll area is no longer needed but kept for small windows.

### 9. Committing a field before the dialog opens
A Cmd/Ctrl+E pressed while a column field has focus must export the typed value, but egui reports `lost_focus` only on a later frame, after the modal is built. So the committed-field helper also mirrors its buffer into the workspace while it has focus: `ws.mod_draft: Option<(ModField, String)>` (transient). `Workspace::commit_mod_draft(now)` applies it exactly as a commit would (same `set_mod_setting` / `add_game_versions`, so an unchanged value records nothing) and clears it. `CommandId::ExportMod` calls it, then surrenders keyboard focus, before building `ModExportDialog`, so the destination's name and the problems use the committed settings. When the field later sees `lost_focus`, its buffer equals the project value and nothing more is recorded.

Alternative considered: defer building the dialog by one frame so the field commits through `lost_focus`. Rejected: it adds a frame of state to the command system for one case, and tests would have to step frames to see the dialog.

## Risks / Trade-offs

- [Native save dialogs differ on overwrite confirmation (macOS and Windows ask; the Linux portal may not)] → `confirmed` is only set when the chosen path exists after the dialog returns, i.e. the user picked an existing file there; on platforms that don't ask, the user still saw the file and picked it. Acceptable; if a Linux tester finds it surprising, drop the `confirmed` shortcut and always ask.
- [A draft left in `ws.mod_draft` after its field disappeared (space switched while typing)] → the helper clears it on `lost_focus` and `commit_mod_draft` is also called when the space changes; test "Export while typing" and a space-switch test.
- [Dropping a file on the Project space anywhere but a picture still places it on the active texture, which the user can't see from there] → unchanged behaviour (out of scope); the drop highlight makes the picture zones obvious.
- [`Problem` variants gain `package_id`] → only `mod_export.rs`, `game_versions.rs` and their tests construct them; messages unchanged, so localisations don't move.
- [Column height grows (two pictures, eight fields, chips, Advanced, Before exporting)] → it already scrolls; Show scrolls to the target.
- [Destination default changes from "the native dialog's default" to the Documents folder when no game folder exists] → a path must be shown before any dialog opens; Documents is what the native dialogs open in on Windows and macOS anyway.
- [Screenshots in `tests/screenshots.rs` are `#[ignore]` (need a GPU)] → they're regenerated by hand for review; kittest tests in CI cover behaviour.

## Migration Plan

No data migration: `ModSettings` and the file format are unchanged. The undo history of a session is not persisted, so the removal of the "settings recorded on export" step has no stored trace. Rollback is reverting the change.
