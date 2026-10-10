## Why

A fleet reaches 30 to 40 textures, and nothing tells the player which ones are done. A texture never painted ships in the mod as a transparent DDS, so the vehicle keeps the game's base color, and nobody notices until they drive it. A texture whose layout changed after Update Template is only marked by a warning icon on its row and a notice in the inspector with nothing selected, easy to miss once the player is on another texture.

The redesign (`new-design`, shipped) gave the project its own space and the Workshop a Textures tab. This change gives each texture a state, shown wherever textures are listed, so the player sees what is left to do before exporting.

## What Changes

- **Each texture has a state**, computed from the project, never typed by the player, and not stored:
  - **Empty:** the texture has no object, or every object is hidden. Nothing is rendered, so it is exported transparent and the game's base color shows;
  - **Modified:** at least one visible object;
  - **To check:** an update flagged it ("Layout changed" or "Not in this version", the existing template status). It wins over Empty and Modified.
- **To check is cleared as today:** the inspector's Dismiss becomes **Mark as Checked** (one undo step). Opening the texture doesn't clear it. "Not in this version" can't be cleared: the texture stays To check until it is removed from the project (it is already left out of the mod).
- **Shown where textures are listed:** a 6 px dot with the state's label on each texture of the Project space's vehicle cards; a state marker on each row of the Workshop's Textures tab, replacing its warning icon. A state never relies on color alone.
- **Project space header:** state counts, "2 vehicles · 9 textures · 4 modified · 1 to check", and a segmented filter **All**, **To do** (Empty or To check) and **To check**. A filter hides texture rows, never vehicles: a vehicle with no matching texture shows "Nothing to do".
- **"On this texture" in the inspector:** with nothing selected, under the texture's properties, the number of objects (top level, as in Layers), of symbol instances, and of **off-palette colors**: the distinct solid colors and gradient stops of the texture's own fills and strokes that aren't linked to a palette swatch. Colors inside symbol instances belong to the symbol and aren't counted on the texture; a color equal to a swatch's value but not linked counts. Clicking the count selects the objects that use them.
- **"Before exporting"** at the bottom of the Project space's Mod information column, and the same lines among Export Mod…'s checks: each texture To check ("Curtain body 13.6 m: layout changed", with Open in the Project space), and the Empty textures ("4 textures empty, exported with the game's color", each openable). They are **warnings**: they don't disable Export…, unlike the existing problems (name, internal name, price, game path, game data, game versions), which are unchanged.

Non-goals:
- a state the player sets by hand ("Done"), or per-object review flags (see `template-update-impact`);
- a "symbols and palette up to date" check: the library is copied, not linked, so there is nothing to compare yet;
- changing what an Empty texture exports, or leaving it out of the mod;
- filters in the Textures tab, or a persisted filter.

## Capabilities

### New Capabilities
- `texture-status`: the three states and how they are computed, Mark as Checked, the off-palette colors of a texture, and the "Before exporting" warnings shared by the Project space and Export Mod.

### Modified Capabilities
- `project-screen`: the Vehicles header gains state counts and the All / To do / To check filter (Project space layout); each texture shows its state instead of the update's warning (Texture rows); the Mod information column ends with "Before exporting" (Mod information column).
- `vehicle-projects`: the Textures tab's rows show the state marker instead of the warning icon (Textures tab); the inspector notice reads To check with Mark as Checked (Template overlay as a view setting).
- `properties-panel`: with nothing selected, the texture's properties say To check with Mark as Checked and are followed by "On this texture" (Selection summary, Inspector sections, a new On this texture summary requirement).
- `mod-export`: the dialog lists non-blocking warnings for To check and Empty textures, which don't disable Export… (Export Mod dialog, Checks before export).

## Impact

- **tp-core:** a texture state derived from a `Surface` (its objects' visibility and its template's `TemplateStatus`), and the off-palette colors walking fills, strokes and gradient stops (`fill_swatch`, `StrokeStyle::swatch`, `ColorStop::swatch`), skipping instances. Pure and unit-tested. No new stored data.
- **tp-file:** none. The flag is the existing `TemplateStatus` in format 1.
- **tp-app:**
  - `vehicle_project.rs`: `dismiss_layout_change` keeps its behavior, under the label Mark as Checked;
  - `mod_export.rs`: a warnings list beside `Problem`, which stays the set that blocks the export;
  - `ui/workspace/spaces/project.rs` (header, filter, tiles, Before exporting), `ui/workspace/panels/vehicle.rs` (Textures tab rows, inspector notice), `ui/workspace/inspector.rs` (On this texture), `ui/mod_export_dialog.rs`;
  - strings in en/fr/de/es.
- **Docs:** `docs/roadmap.md` records `texture-status` as shipped under `polishing`.
