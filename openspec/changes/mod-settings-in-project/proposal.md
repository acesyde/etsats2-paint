## Why

The mod's information (its name, author, version, description, price and pictures) describes the project, but today it can only be seen and changed inside Export Mod…. The sidebar shows the Name and Version read only, and every change has to go through an export. The redesign (`new-design`) gives the Project screen a "Mod information" column, the natural home for these settings. The Export dialog can then be what the mockup asks for: a destination, the checks and one button.

## What Changes

Depends on `new-design`, which creates the Project screen and its "Mod information" column. Part of roadmap #2 (`polishing`).

- **Mod information is edited in the Project screen.** The column holds every mod setting of the mod-export spec: Name, Author, Version, Description, Price, Unlock level and the internal name (still derived from the Name until edited). It also holds the two pictures with their preview, Choose Image… (or dropping a PNG/JPEG on the preview) and Use Generated: the shop icon (256×64) and the Mod Manager image (276×162).
- **Each edit is its own undo step.** Fields behave like Game versions today: Enter or leaving the field records the change as one undo step, "Edit Mod Settings", and Escape restores the previous value. The multi-line Description is recorded when it loses focus. Choosing a picture or going back to the generated one is one undo step. An unchanged field adds no step. Edits mark the project as having unsaved changes, like any other edit.
- **Game versions become chips.** Each version is a chip with a remove button, and "+ Add" takes a typed version in the game's own format (`1.56.*`). Adding or removing a version is one "Edit Game Versions" step. A badly written version is marked on its chip. The versions every vehicle supports are still shown under the list.
- **Problems are shown where they are fixed.** A setting that would block the export (empty Name, forbidden characters, internal name too long, Price 0, bad game version) is flagged under its field in the Project screen. The rules of "Checks before export" are unchanged.
- **BREAKING (mod-export): the Export Mod dialog no longer edits anything.** It keeps the summary of the mod, the list of problems (each problem about a setting takes the user to its field in the Project screen), the destination and Export. Export… no longer records settings: "Editing them and exporting is one undo step" and the "Edit Mod Settings" step made on confirm go away. Cancel has nothing to restore, and exporting never changes the project or its undo history.
- **BREAKING (mod-export): the destination is shown in the dialog.** The dialog shows the proposed path (the game's mod folder and `<Name>.scs`, by the current rules) with Change…, which opens the native save dialog. Export writes there, and still asks before replacing an existing file.
- **Docs:** when it ships, `docs/roadmap.md` ("The mod" and "Game versions") records that settings are edited in the Project screen, one undo step per edit.

Non-goals: the mockup's export scope (current texture / current vehicle / whole fleet), "Enable in Mod Manager", the "Last export" line, and DDS settings under "Advanced". These are new features, not part of this change.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `mod-export`: the Export Mod dialog loses the settings and the pictures and gains a visible destination with Change…. Settings are recorded per edit, not on export. Cancel and export never change the project. Problems link to their field.
- `project-screen` (created by `new-design`): the Mod information column makes Name, Version and every other mod setting and picture editable, one undo step per edit, and shows Game versions as chips.

## Impact

- **tp-core:** none expected. `ModSettings` and `Project::game_versions` already hold everything. Field-level validation reuses the existing problem rules, exposed per setting.
- **tp-app:**
  - `ui/mod_export_dialog.rs` loses its settings form and picture controls, and gains the destination row;
  - the settings form, the picture previews (rendered on a worker thread, as today) and the chips move to the Project screen's Mod information column (built by `new-design`, today `ui/workspace/panels/vehicle.rs`);
  - `mod_export.rs`: `set_mod_settings` becomes a per-field edit, and the export no longer records settings;
  - `game_versions.rs`: add/remove one version;
  - strings in en/fr/es/de.
- **tp-file:** none. The settings are already saved in format 1.
- **Tests:** the mod-export undo and Cancel scenarios are rewritten. The export tests that rely on settings being recorded on confirm change.
