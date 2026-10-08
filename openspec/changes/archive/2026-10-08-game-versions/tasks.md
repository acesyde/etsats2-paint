## 1. Model and file (tp-core, tp-file)

- [x] 1.1 Add `Project.game_versions: Vec<String>` (empty in `Project::new`), with it in `Snapshot`, `restore` and `same_document`. Verify with a unit test: a change is a document change, and restoring brings back the previous list.
- [x] 1.2 Add `FileProject.game_versions` (`#[serde(default, skip_serializing_if = "Vec::is_empty")]`), written and read in `from_project` / `into_project`. Verify in `tests/format.rs`: a round trip keeps `1.56.*, 1.57.*` in order; the v1 fixture opens with none; a project without game versions writes no `game_versions` key.

## 2. Game versions logic (tp-app `game_versions`)

- [x] 2.1 Add `Bound`, `Interval`, `of_req(&VersionReq)` (one design table row per `Op`), `of_listed(&str) -> Option<Interval>`, `intersect`, `is_empty`, `contains` and `text`. Verify with unit tests:
  - each table row, checked against `VersionReq::matches` on versions just inside and outside each bound (1.55.9, 1.56.0, 1.57.0, 1.57.9, 1.58.0, 2.0.0);
  - `of_listed` accepts `1.56`, `1.56.*`, `1.56.2`, `1.56.2.*` and `1.56.2.3`, and refuses `1.56.x`, `1`, `*`, `1.*.2` and an empty string;
  - `>=1.53, <1.58` ∩ `^1.56` gives `>=1.56, <1.58`;
  - `>=1.56` ∩ `<1.55` is empty;
  - `*` gives "any version";
  - `>=1.56` contains `1.56.*` but not `1.55.*`.
- [x] 2.2 Add `fleet(&Project) -> FleetVersions` (`NoData`, `Common(Interval)`, `Conflict { first, second }`, each side holding the vehicle name and its recorded range), skipping vehicles without game data and treating an unparseable range as `*`. Verify with unit tests:
  - the sample truck alone, or with the trailer, gives `>=1.56`;
  - the sample truck plus a vehicle at `<1.55` gives a conflict naming both with their ranges;
  - vehicles without game data give `NoData`.

## 3. Sidebar

- [x] 3.1 Add `Workspace::set_game_versions(list, now)` recording `undo-edit-game-versions`, and nothing when the list is unchanged. Make the Project section's Game versions field editable:
  - a buffer committed on Enter or focus loss, split on commas, trimmed, empties dropped;
  - Escape restores the field through `remember_escape` / `take_escape`;
  - the guide line under the field shows `fleet(project)`.

  Verify with UI tests:
  - typing `1.56.*, 1.57.*` then Enter records the list, the status bar shows "Unsaved changes", and Undo empties it;
  - Escape while typing leaves the list unchanged;
  - a new sample-truck project shows ">=1.56" under the field, and "no common version" once a `<1.55` vehicle is added;
  - Name and Version stay read only.

## 4. Export

- [x] 4.1 Add `Problem::BadGameVersion`, `UnsupportedGameVersion` and `NoCommonGameVersion` to `mod_export::problems_with` (after `MissingGameData`), with their messages in the dialog. Verify with unit tests, one per problem, matching the spec scenarios (`1.56.x`; `1.55.*` with the sample truck; the sample truck with a `<1.55` vehicle), and with a UI test that the message appears and Export… is disabled.
- [x] 4.2 Copy `project.game_versions` into `manifest.sii` as `compatible_versions[]` lines, after `description_file`. Verify with unit tests: `1.56.*, 1.57.*` gives both lines in order; an empty list gives none.
- [x] 4.3 Add the strings `undo-edit-game-versions`, `project-game-versions-supported` (the guide line), `project-no-common-version`, `mod-problem-bad-game-version`, `mod-problem-unsupported-game-version` and `mod-problem-no-common-game-version` in en/fr/es/de. Verify with the i18n tests and the localization screen test.

## 5. Docs and checks

- [x] 5.1 Update the docs:
  - `docs/roadmap.md`: replace the Game versions open question with the decision (a project property copied into `compatible_versions`, checked against the packages, export blocked when the vehicles have no version in common), and replace the "it will come back with a left/right mapping" paragraph with the decision to drop it (copy/paste and Flip cover it);
  - `docs/roadmap_light.md`: mark both items as done.

  Verify by reading the diff.
- [x] 5.2 Run `mise run ci`. Verify that format, lint, tests and builds pass.
