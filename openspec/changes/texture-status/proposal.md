## Why

A fleet reaches 30 to 40 textures, and nothing tells the player which ones are done. A texture never painted ships in the mod as a transparent DDS, so the vehicle keeps the game's base color, and nobody notices until they drive it. A texture whose layout changed after Update Template is only marked by a warning icon in the sidebar and a "Layout changed" line in the Properties panel, easy to miss once the player is on another texture.

The redesign (`new-design`) gives the project its own screen and the Workshop a Textures tab. This change gives each texture a state, shown wherever textures are listed, so the player sees what is left to do before exporting.

## What Changes

- **Each texture has a state**, computed, never typed by the player:
  - **Empty:** the texture has no object. It is exported transparent, so the game's base color shows;
  - **Modified:** it has at least one object;
  - **To check:** an update flagged it. This is the existing "Layout changed" flag (and "Not in this version"), renamed in the interface. It wins over Empty and Modified.
- **To check is cleared as today:** the Dismiss action (renamed **Mark as Checked**) in the inspector, one undo step. Opening the texture doesn't clear it. "Not in this version" can't be cleared: the texture stays To check until it is removed from the project.
- **Shown where textures are listed:** a dot and a label on each row of the Project screen's texture list and of the Workshop's Textures tab, replacing the warning icon of the sidebar tree.
- **Project screen header:** counts, "2 vehicles · 9 textures · 4 modified · 1 to check", and the filters **All**, **To do** (Empty or To check) and **To check**. A filter hides rows, never vehicles: a vehicle with no matching texture shows as collapsed with "nothing to do".
- **"On this texture" in the inspector:** with nothing selected, under the texture settings, a summary of the active texture: the number of objects (top level, as in Layers), of symbol instances, and of **off-palette colors** (distinct solid colors and gradient stops of the texture's objects that aren't linked to a palette swatch). Clicking the off-palette count selects the objects that use them.
- **"Before exporting" on the Project screen,** and the same lines in Export Mod…'s checks: each texture To check ("Curtain body 13.6 m: layout changed", with Open), and the Empty textures ("4 textures empty, exported with the game's color", each openable). These are **warnings**: they don't disable Export…, unlike the existing problems (name, internal name, price, game path, game data, game versions), which are unchanged.

Non-goals:
- a state the player sets by hand ("Done"), or per-object review flags (see `template-update-impact`);
- a "symbols and palette up to date" check: the library is copied, not linked, so there is nothing to compare yet;
- changing what an Empty texture exports, or leaving it out of the mod.

Open questions (to settle in design):
- Does a texture whose objects are all hidden count as Empty (nothing is rendered) or Modified? Proposed: Empty, since the export is transparent.
- Do colors inside symbol instances count as off-palette on the texture, or only in the symbol? Proposed: in the symbol only, counted once in the Brand space.
- Does a color equal to a swatch's value but not linked count as off-palette? Proposed: yes, with "Link to Swatch" as a later fix.

## Capabilities

### New Capabilities
- `texture-status`: the three states and how they are computed, the dot and label in texture lists, the Project screen counts and filters, the "On this texture" summary and its off-palette count, and the "Before exporting" list.

### Modified Capabilities
- `vehicle-projects`: "Layout changed" and "Not in this version" read as "To check", Dismiss becomes Mark as Checked, and texture rows show the state instead of the warning icon (Vehicle panel requirement, Template overlay requirement).
- `properties-panel`: the nothing-selected summary adds "On this texture".
- `mod-export`: Checks before export gain non-blocking warnings for Empty and To check textures.

The texture lists, the Project screen and the inspector are placed by `new-design`. The specs above are those that exist today; the deltas will target whatever `new-design` leaves them as.

## Impact

- **Depends on** `new-design` (Project screen, Workshop Textures tab, inspector).
- **tp-core:** a texture state derived from a `Surface` (objects, `TemplateStatus`), and the off-palette count walking fills, strokes and gradient stops. Pure and unit-tested. No new stored data.
- **tp-file:** none. The flag is the existing `TemplateStatus` in format 1.
- **tp-app:**
  - `vehicle_project.rs`: `dismiss_layout_change` keeps its behavior under its new label;
  - `mod_export.rs`: a warnings list beside `Problem`, which stays the set that blocks the export;
  - the Project screen list, header and filters, the Workshop Textures tab rows, the inspector summary, and `mod_export_dialog.rs`;
  - strings in en/fr/es/de.
- **Docs:** `docs/roadmap.md` records the states under `polishing`.
