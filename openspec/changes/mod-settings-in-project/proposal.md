## Why

The mod's information (its name, author, version, description, price and pictures) describes the project, but it can only be changed inside Export Mod…. Since `new-design`, the Project space's Mod information column shows the pictures, Name, Author, Version and Description as plain text and sends the player to the dialog with **Edit in Export Mod…**; every change goes through an export. That column is the natural home for these settings, and the Export dialog can then be what the mockup asks for: a summary, the checks, a destination and one button.

## What Changes

Builds on `new-design` (Project space, Mod information column, Export Mod dialog with Advanced) and `texture-status` (Before exporting warnings), both shipped. Part of roadmap #2 (`polishing`), listed as 2b.

- **Every mod setting is edited in the Project space.** The Mod information column turns its read-only values into fields, labels above, and adds the settings it didn't show: Name, Author, Version, Game versions, Description, Price, Unlock level and, under a collapsed **Advanced** section, the internal name (still derived from the Name until edited; Advanced opens by itself while a problem concerns it). The two pictures stay at the top, each with its preview, **Choose Image…**, dropping a PNG or JPEG on the preview, and **Use Generated** (shown only for a chosen picture): the shop icon (256 × 64) and the Mod Manager image (276 × 162). **Edit in Export Mod…** goes away.
- **Each edit is its own undo step.** Enter or leaving a field records the change as one "Edit Mod Settings" step; Escape restores the previous value and records nothing; an unchanged value adds no step. The multi-line Description is recorded when it loses focus. Choosing, dropping or un-choosing a picture is one step. Edits mark the project as having unsaved changes, like any other edit.
- **Game versions become chips.** Each version is a chip with a remove button; **+ Add** takes a version typed in the game's own format (`1.56.*`). Adding or removing versions is one "Edit Game Versions" step. A badly written version is marked on its chip. The versions every vehicle supports are still shown under the chips.
- **Problems are shown where they are fixed.** A problem about a setting (empty or invalid Name, invalid Version or Author, bad internal name, Price 0, badly written or unsupported game version) is shown under its field. The column's **Before exporting** section now lists the blocking problems first, each taking the player to its field (or, for a problem about a vehicle, to its card), then the texture warnings of `texture-status`. The rules of "Checks before export" are unchanged.
- **BREAKING (mod-export): the Export Mod dialog edits nothing.** It keeps the title, the summary of the mod, the problems listed above the buttons (each one closes the dialog and takes the player to its field, or its vehicle, in the Project space), the texture warnings, and gains the **destination**. Its settings form, Advanced section and pictures go away.
- **BREAKING (mod-export): the destination is shown in the dialog.** The dialog shows the proposed file (the game's mod folder and `<Name>.scs`, by the current rules) with **Change…**, which opens the native save dialog. Export writes there, and still asks before replacing an existing file.
- **BREAKING (mod-export): exporting never changes the project.** "Editing them and exporting is one undo step" and the "Edit Mod Settings" step recorded on confirm go away. Cancel has nothing to restore; exporting changes neither the project, nor its undo history, nor its save state.
- **Docs:** when it ships, `docs/roadmap.md` ("The mod", "Game versions" and the next-changes table) records that settings are edited in the Project space, one undo step per edit, and that the Export dialog only checks and writes.

Non-goals: the mockup's export scope (current texture / current vehicle / whole fleet), "Enable in Mod Manager", the "Last export" line and the export folder in the Project column, and DDS settings under "Advanced". These are new features, not part of this change.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `project-screen`: "Mod information column" makes every mod setting and both pictures editable, adds Advanced with the internal name, drops Edit in Export Mod…, and lists the blocking problems under Before exporting; "Game versions field" becomes chips with + Add. New requirements describe how a setting is edited and recorded, how pictures are chosen or dropped, and how problems are shown under their field and lead to it.
- `mod-export`: "Export Mod dialog" loses the settings, Advanced and pictures, and its problems lead to their field; "Mod settings" are recorded per edit, not on export; "Mod images" are chosen in the Project space; "Checks before export" are also shown in the Project space; "Destination" is shown in the dialog with Change…; "Background export with progress" no longer records anything.

## Impact

- **tp-core:** none expected. `ModSettings` and `Project::game_versions` already hold everything, and snapshots already compare them (an unchanged edit records no step).
- **tp-app:**
  - `ui/workspace/spaces/project.rs`: the Mod information column gets the fields, the picture controls and drop zones, Advanced, the problems under fields and in Before exporting; Edit in Export Mod… is removed;
  - `ui/workspace/panels/vehicle.rs`: `game_versions_field` becomes the chips;
  - `ui/mod_export_dialog.rs` loses its settings form, Advanced, pictures and preview job, and gains the destination row, the replace confirmation and the problem links;
  - `mod_export.rs`: `set_mod_settings` is split into per-setting and per-picture edits; each problem tells where it is fixed;
  - `game_versions.rs`: add and remove versions;
  - `state.rs`: a file dropped on a picture's drop zone becomes that picture instead of being placed on the canvas;
  - strings in en/fr/es/de.
- **tp-file:** none. The settings are already saved in format 1.
- **Tests:** the mod-export undo, Cancel and "settings recorded on confirm" tests are rewritten; the project-space tests of the read-only column and Edit in Export Mod… are replaced; the game-versions field tests move to chips.
