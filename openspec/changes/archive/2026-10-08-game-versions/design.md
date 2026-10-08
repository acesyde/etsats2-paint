## Context

See proposal.md for the motivation. What exists:
- **The ranges:** each `ProjectVehicle.game_data` (since `mod-export`) holds `versions`, the package's game version range, stored as the text of a `semver::VersionReq` (`>=1.56`, `^1.53`, `>=1.50, <1.54`, `*`).
- **The field:** the Project section (`ui/workspace/panels/vehicle.rs::project_section`) shows three disabled `TextEdit`s. Game versions is a hard-coded empty string.
- **Editing pattern:** a text field edited in a panel keeps a buffer and is committed on Enter or focus loss, with Escape restoring the value through `tp_ui::widgets::remember_escape` / `take_escape`. The hex field in `panels/colors.rs` and layer renaming in `panels/layers.rs` work this way.
- **The export:** `mod_export::problems_with` lists the problems, `plan` writes `manifest.sii`, and "any version" already exists as `vehicles-any-version`.

`semver` gives each range as comparators (`Op` = `=`, `>`, `>=`, `<`, `<=`, `~`, `^`, wildcard; with optional minor and patch). It has no intersection.

## Goals / Non-Goals

**Goals:**
- One field, one list, copied as is: what the player types is what `compatible_versions[]` gets.
- One pure module that turns ranges and listed versions into intervals, used by the sidebar's guide line and the export checks.

**Non-Goals:**
- Pre-release versions: game versions don't use them, and they are ignored.
- Editing Name or Version in the sidebar.

## Decisions

### `Project.game_versions: Vec<String>`, a project property
The game versions belong to the project rather than to `ModSettings`. They are edited in the sidebar, and the export only copies them.

- They are part of `Snapshot`, so undo and the saved state cover them.
- `Workspace::set_game_versions(list, now)` records `undo-edit-game-versions`, and nothing when the list is unchanged.
- In the file they are `FileProject.game_versions`, with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so files without them are unchanged.
- The field's text is split on commas, each entry trimmed, and empty entries dropped. Entries are stored as typed, even badly written ones: the export reports those. That way the field never silently loses what the player typed.
- *Alternative: a Game versions setting in Export Mod.* Rejected by the player: a project property that the export copies is simpler to understand and maintain.

### Listed versions and package ranges as intervals
**A listed version** is `N.N`, `N.N.N` or `N.N.N.N`, with the last number possibly replaced by `*` (at least `N.N.*`). Its interval:
- `1.56.*` is `[1.56.0, 1.57.0)`;
- `1.56` is the same as `1.56.*` (the game version 1.56 with any patch);
- `1.56.2` and `1.56.2.*` are `[1.56.2, 1.56.3)`, since the game's fourth number is a build and semver has no room for it.

**A package range** becomes an interval comparator by comparator. A missing minor or patch counts as 0:

| Comparator | Interval |
|---|---|
| `>=1.56` | `[1.56.0, ∞)` |
| `>1.56` (partial) / `>1.56.2` | `[1.57.0, ∞)` / `(1.56.2, ∞)` |
| `<1.58` | `(-∞, 1.58.0)` |
| `<=1.58` (partial) / `<=1.58.1` | `(-∞, 1.59.0)` / `(-∞, 1.58.1]` |
| `=1.56` (partial) / `=1.56.2` | `[1.56.0, 1.57.0)` / `[1.56.2, 1.56.2]` |
| `~1.56` / `~1` | `[1.56.0, 1.57.0)` / `[1.0.0, 2.0.0)` |
| `^1.56` / `^0.3` | `[1.56.0, 2.0.0)` / `[0.3.0, 0.4.0)` |
| `1.56.*` / `1.*` | `[1.56.0, 1.57.0)` / `[1.0.0, 2.0.0)` |

**Combining intervals:**
- An intersection keeps the highest lower bound and the lowest upper bound. It is empty when the lower bound is above the upper one, or equal to it with either side exclusive.
- A listed version is supported by a vehicle when its interval lies inside the vehicle's range.

- *Alternative: test sample versions against every range.* Rejected. It needs a list of game versions that TruckPaint doesn't have, and it gives no range to show.

### Text of the guide line
The guide line under the field writes the fleet's interval in the packages' notation:
- `>=L` or `>L`, then `<U` or `<=U`, joined with ", ". A patch of 0 is left out (`1.56`, not `1.56.0`);
- `=V` for a single version;
- "any version" when there is no bound on either side;
- "no common version" when the intersection is empty;
- nothing when no vehicle has game data.

### Checks
They are added after `MissingGameData`, in this order:
- `BadGameVersion { version }` for each badly written entry;
- `UnsupportedGameVersion { version, vehicle, range }` for the first vehicle that doesn't support a well-written entry;
- `NoCommonGameVersion { first, second }`, naming the vehicle that sets the highest lower bound and the one that sets the lowest upper bound, each with its recorded range.

Vehicles without game data are left out (already reported as `MissingGameData`). A range that doesn't parse counts as "any version"; that can't happen for ranges recorded from validated packages.

### The manifest
`plan` writes one `\tcompatible_versions[]: "<version>"` line per entry after `description_file`. The entries are as typed: a badly written one can't reach this point, since the checks block the export first.

### Where the code lives
`tp-app/src/game_versions.rs` holds `Bound`, `Interval`, `of_req`, `of_listed`, `intersect`, `contains`, `fleet(project)` and `text`. tp-core has no `semver` dependency, and this is export and display logic, not document data.

## Risks / Trade-offs

- **[The fourth number of game versions]** `1.56.2.3` is checked as `1.56.2`. That is accurate enough to compare with package ranges, which don't go below the patch.
- **[Partial versions with `>` and `<=`]** Unit tests check each row of the table against `VersionReq::matches` on sample versions, so the table can't drift from semver's meaning.
- **[Badly written entries are stored]** The field keeps what was typed; the export reports it. A badly written list never reaches a manifest.
