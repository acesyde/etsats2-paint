## 1. A main texture that also paints accessories

- [x] 1.1 Add `also_paints` to `tp_vehicles::Part` with its validation (design Decision 6); verify with tp-vehicles tests: vehicle-packages "Main texture that also paints accessories" and "Accessory id painted twice", an accessory with `also_paints` refused, an invalid id refused.
- [x] 1.2 Record `also_paints` in `SurfaceTemplate` (from the package, refreshed by Update Template, undone with it) and save it in `FileTemplate` (additive, format 1); verify with tests: vehicle-projects "Accessories painted by a main texture" (round-trip through save and open), an older project file without the field still opens, and Update Template replaces the ids and Undo restores them.
- [x] 1.3 Write the extra `simple_paint_job_data` in the mod export, and the accessory file even when no accessory is painted (design Decision 6); verify with a mod-export test: "Main texture applied to accessories", plus the existing "Trailer" and "Truck with two cabin layouts" scenarios unchanged.
- [x] 1.4 Document `also_paints` in `docs/vehicle-package-format.md`; verify with a tp-pack test that packs the doc's example with the field and passes `Package::read`.

## 2. The catalog

- [x] 2.1 Add `tp_vehicles::catalog`: entry types, embedded entries, `entries(game)`, completeness check, and `manifest(&Entry)` producing a package manifest with id `official.<game>.<path>` (design Decisions 1, 3); verify with unit tests on an inline entry: a complete entry gives a valid manifest, an uncovered cabin, an uncovered accessory id, an id covered twice and a covered id missing from the game data are each reported.
- [x] 2.2 Write `catalog/ets2/daf.2021.json` and `catalog/ets2/krone.coolliner.json` from Paintjob Packer's data at its current commit and the mapping of design Decision 2, plus `catalog/LICENSE-paintjob-packer.txt`; verify with tests: official-vehicles "Complete entries only" (every embedded entry is complete), every file under `catalog/` is embedded, and each entry builds a valid package from placeholder PNGs of its sizes.

## 3. Building from the pack and the `tpv catalog` commands

- [x] 3.1 Add `tp_pack::catalog::find_pack` and `build(entry, pack)`, which checks each file's presence and pixel size and packs with `pack_entries` (design Decision 3); verify with tests on a temporary pack of small PNGs: folder found directly and one level down, a wrong folder refused, a missing file and a file of another size each refused with a message naming the file, and a built package that passes `Package::read` with the entry's game ids and `also_paints`.
- [x] 3.2 Add `tpv catalog sync <folder> --commit <sha>` (design Decision 1: only `game_data` and the source commit change); verify with CLI tests on a fixture copy of Paintjob Packer's layout: official-vehicles "Accessory id added upstream" and "Vehicles not in the catalog", a missing vehicle failing with no entry changed, and an unchanged run printing "unchanged".
- [x] 3.3 Add `tpv catalog check <pack> [--game]`; verify with CLI tests on a temporary pack: official-vehicles "Uncovered id" (error exit), a missing file and a size mismatch reported, an unused PNG and a different `Info and links.txt` version listed as information, and success when the pack matches.
- [x] 3.4 Run `tpv catalog check ~/Downloads/templates` on the real ETS2 pack; verify it reports no problem and lists `Chassis Mid - Simple Side Skirt.png` as unused (official-vehicles "Pack matches the catalog"), and note the output in the PR.

## 4. Pack folder and preferences

- [x] 4.1 Add `FileDialogs::pick_folder` (rfd and `ScriptedDialogs`) and `Prefs.template_packs` with `#[serde(default)]` (design Decision 5); verify with prefs tests: a pack folder round-trips, a preferences file without it loads unchanged, and `reset_scaling` keeps it.
- [x] 4.2 Preferences: a Template packs section, shown for games with official vehicles, with the path, Choose… (checked with `find_pack`) and Clear, through an outcome returned by `preferences()`; verify with kittest: app-preferences "Pack folder survives restart", "Change the pack folder", "Refused pack folder", and no Template packs row for ATS.

## 5. Official vehicles in the dialogs

- [x] 5.1 List the official vehicles of the game under Official, then the other installed vehicles under Installed, in New Project and Add Vehicle…, with the Package column "Official" and the installed version (design Decision 3); verify with kittest: start-screen "Official vehicles first", "Official vehicle filtered by kind", "No vehicle installed" (ATS), and an installed official vehicle listed once.
- [x] 5.2 The template pack dialog, opened from an origin (New Project or Add Vehicle) with the picked entry: explanation, Open the download page, Choose the pack folder…, drop of a folder, Cancel back to the origin (design Decisions 4, 5, 7); verify with kittest: official-vehicles "First official vehicle", "Download page" (the opened URL), "Folder unzipped inside another", "Wrong folder", "Dropped folder", "Pack moved", and Cancel restoring New Project as it was.
- [x] 5.3 `InstallJob`: build on a thread with progress and Cancel, install on success, choose the vehicle in the origin, show refusals in the dialog's messages (design Decision 4); verify with kittest on a temporary pack: official-vehicles "Pick and install", "Installed without the pack", "Cancel the installation", "Missing file", "Pack of another version"; start-screen "Project from an official vehicle"; vehicle-projects "Official trailer added to a fleet".
- [x] 5.4 Strings in en/fr/de/es for the sections, the pack dialog, the installation, Preferences and About; verify with the i18n key tests and a screenshot of the pack dialog and the New Project list in German at 100 %, with no clipped text.

## 6. Credits

- [x] 6.1 About: the credits line and Licenses… showing Paintjob Packer's MIT notice (design Decision 7); verify with kittest: official-vehicles "Credits" and "No network needed" (the catalog listed with no network code involved).

## 7. End to end, checks and docs

- [x] 7.1 Add an ignored test that runs when `TRUCKPAINT_ETS2_PACK` points to a pack: it installs both vehicles from it, creates a project with the DAF NGD (XG+ and every accessory) and the Krone Cool Liner, and plans the mod; verify it passes locally on `~/Downloads/templates`, and that the mod plan holds the Krone's `also_paints` entry and every accessory id of design Decision 2.
- [ ] 7.2 In-game check (manual, ETS2 1.59): export a mod painting each texture of both vehicles with distinct colors and labels, load it in the game, and check every cabin, each accessory, the Krone's body and its chassis parts; verify by recording the result in the PR. If the body or a side skirt is wrong, correct the mapping in the entry, or stop and ask (design Risks).
- [x] 7.3 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --release --locked` and the screenshot suite; verify all pass.
- [x] 7.4 At ship time, update `docs/roadmap.md`: move `official-vehicles` (first brick) to Shipped, keep 3a for the next batches; verify `openspec validate official-vehicles --strict` passes.
