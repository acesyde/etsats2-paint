## Why

The sidebar's Project section has shown an empty, read-only **Game versions** field since `fleet-projects`, waiting for "a later feature". The mod export writes a `manifest.sii` without `compatible_versions`, so the Mod Manager can't tell players which game versions the mod is made for.

Mod authors already manage this list in other mods: after a game update, they add the new version (`1.57.*`) and release a new version of their mod. TruckPaint should work the same way, with a project property that matches the manifest's field one to one. The export then simply copies it, and the mapping stays trivial.

There is also no check that the vehicles fit together. Projects record each vehicle's supported game versions (since `mod-export`). If one package's templates are for 1.56 and later and another's stop at 1.54, no game version shows both paint jobs correctly, and the mod exports anyway.

## What Changes

- **Game versions becomes an editable project property,** in the sidebar's Project section:
  - it's the list the manifest's `compatible_versions[]` gets, typed the way the game writes it: `1.56.*, 1.57.*`;
  - it's empty for a new project;
  - Enter or leaving the field records it as one undo step ("Edit Game Versions"), and Escape cancels the edit;
  - under the field, the section shows the versions every vehicle supports, computed from the packages (`>=1.56`, "any version", or "no common version"), to guide the author.
- **Export Mod copies it into the manifest:** one `compatible_versions[]: "<version>"` line per version, in the order typed. There are none when the list is empty. The dialog has no field for it.
- **Export Mod checks it.** Each of these blocks the export, with a message naming what to fix:
  - a version that isn't written like the game's (`1.56.*`, `1.56`, `1.56.2`, `1.56.2.*`);
  - a listed version that a vehicle's package doesn't support (e.g. `1.55.*` when a package needs `>=1.56`), naming the vehicle and its range;
  - vehicles that have no game version in common, naming two vehicles and their ranges, and asking to update or remove one.
- **The project file stores the game versions** as an optional field (format 1). Older files open with none.
- **Docs:**
  - `docs/roadmap.md`: the Game versions open question is resolved and the decision recorded. The paragraph saying that copying a design to the other side "will come back" is replaced by the decision to drop it (copy/paste and Flip cover it);
  - `docs/roadmap_light.md`: both items are marked done.

Non-goals: prefilling the list from the packages, checking it against the installed game, and making Name or Version editable in the sidebar.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `workspace-layout`: the Project section's Game versions field becomes editable, with the fleet's supported versions shown under it.
- `mod-export`: the manifest copies the project's Game versions into `compatible_versions[]`, and three new problems block the export.
- `project-files`: the file stores the project's game versions.

## Impact

- **tp-core:** `Project.game_versions: Vec<String>`, empty by default and part of undo snapshots.
- **tp-file:** an optional `game_versions` field on `FileProject`. Files without it open with an empty list, and the format stays at 1.
- **tp-app:**
  - a new `game_versions.rs` module: parsing a listed version, turning a package range or a listed version into an interval, the fleet's common interval, and its text. Pure and unit-tested;
  - `Workspace::set_game_versions`, one undo step;
  - the Project section: the editable field and the guide line;
  - `mod_export`: the problems, and the manifest's `compatible_versions[]`.
- **tp-i18n:** strings for the guide line, the undo label and the three problems, in en/fr/es/de.
- **Dependencies:** none (`semver` is already a dependency of tp-app).
