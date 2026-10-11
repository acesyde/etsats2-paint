## 1. Names with accents and sorting

- [x] 1.1 Add `unicode-normalization` to the workspace and `tp_vehicles::fold` (NFD, combining marks removed, lowercase) (design Decision 5); verify with unit tests: `Kögel` composed and decomposed fold to the same string, `Schwarzmüller` folds to `schwarzmuller`.
- [x] 1.2 Sort `catalog::entries(game)` by folded brand then name; verify with a tp-vehicles test on inline entries (Kässbohrer before Kögel before Krone, Schwarzmüller before SCS), and the existing kittest "Official vehicles first" still passing.
- [x] 1.3 Resolve vehicle folders and files by folded name in `find_pack_of`, `build_with` and `check`, and skip `._*` files in `pngs()` (design Decision 5); verify with tp-pack tests on a temporary pack: a decomposed `Kögel Cargo/` folder found and built from a composed entry (official-vehicles "Accented folder in decomposed form"), and `._Sun Visor.png` neither read nor listed as unused ("macOS resource file").
- [x] 1.4 In `check`, list a file of a shared folder as unused only when no checked entry using that folder uses it; verify with a tp-pack test: official-vehicles "Shared folder" (check).

## 2. `tpv catalog match`

- [x] 2.1 Build a fixture pack and Paintjob Packer copy under `crates/tp-pack/tests/fixtures/match/` with small PNGs. It copies the real names of: the DAF XD, the Volvo FH6 (aero cabins), the New Actros (8x4), the Iveco S-Way (single file), `Scania R & S` (shared), the Kögel Cargo, the SCS Fuel Tank (`2 Axles.png`), the Wielton Curtain Master (split body), the Wielton Dropside Master (`835 mm`), a group with no file, and `SCS Livestock` (no vehicle). Verify the fixture loads with `read_catalog` and `ppk_official`.
- [x] 2.2 Add `catalog::matching` with folder finding, the shared-folder prefix split, truck main textures, trailer bodies (design Decisions 1, 3) and draft entries (Decision 2); verify with tests on the fixture: official-vehicles "Truck matched by cabin names", "Shared folder" (match), "Trailer with several bodies" (match), "Folder not found", the Volvo FH6 matched by name despite its codes, and the New Actros' "GigaSpace 8x4".
- [x] 2.3 Add accessory matching, group splitting and left-out groups (design Decision 3); verify with tests on the fixture: "Group without a template", "Id that no file names", "Cabin that can't be placed", the Wielton Curtain Master's body ids split as a doubt, `835 mm` matched as a doubt, and a run made twice giving identical drafts.
- [x] 2.4 Add the `tpv catalog match <pack> <ppk> --commit <c> [--game] [--vehicle <path>]… [--catalog <dir>]` command and `MatchReport::text()`, with the usage text updated; verify with CLI tests: "Existing entry kept", drafts written only for missing files, the report listing doubts, uncovered ids, unused files and folders without a vehicle, and an unknown option refused.

## 3. The ETS2 catalog

- [x] 3.1 Run `tpv catalog match ~/Downloads/templates <Paintjob Packer at a7a7ed3> --commit a7a7ed3902ca3fe6f9ff3eb4afd2d92db3f5db2d` and save its report for the PR; verify that it writes 64 drafts, leaves `daf.2021.json` and `krone.coolliner.json` unchanged, and lists `SCS Livestock` as a folder without a vehicle.
- [x] 3.2 Review and complete the trucks: DAF, Iveco, MAN, Mercedes-Benz and Renault (resolve each doubt, place or leave out each uncovered id, with a reason). Verify that `tpv catalog check --catalog catalog ~/Downloads/templates` reports no problem for them, and commit them.
- [x] 3.3 Review and complete the trucks: Scania (both shared folders, and the S BEV) and Volvo; verify with `tpv catalog check` as in 3.2, and commit them.
- [x] 3.4 Review and complete the trailers: Feldbinder, Kässbohrer, Kögel, Krone (Dry Liner, Paper Liner, Profi Liner, Profi Liner Building) and Schmitz Cargobull, applying design Decision 1; verify with `tpv catalog check` as in 3.2, and commit them.
- [x] 3.5 Review and complete the trailers: Schwarzmüller, SCS, Tirsan and Wielton; verify with `tpv catalog check` as in 3.2, and commit them.
- [x] 3.6 Add the 64 files to `catalog::ENTRIES`; verify with the tp-vehicles tests: every file under `catalog/` is embedded, every entry is complete (official-vehicles "Complete entries only"), and each entry builds a valid package from placeholder PNGs of its sizes.
- [x] 3.7 Run `tpv catalog check ~/Downloads/templates` with the embedded catalog; verify there's no problem, that the unused files include `Krone Cool Liner/Accessories/Chassis Mid - Simple Side Skirt.png` ("Pack matches the catalog"), and that each unused file is explained in the PR.

## 4. App, specs scenarios and end to end

- [x] 4.1 Update the kittest and unit tests that count or name the official vehicles; verify with kittest: official-vehicles "Every official ETS2 vehicle" (66 rows), "Sorted by brand and name", "Trailer with several bodies" (Schmitz Cargobull S.CS), "No network needed", and start-screen "Official vehicle filtered by kind" updated for 40 trailers.
- [x] 4.2 Update the CLI test for `tpv catalog sync`; verify official-vehicles "Vehicles not in the catalog": a fixture with a new ETS2 trailer and ATS vehicles lists only those.
- [ ] 4.3 Add a mod-export test for a trailer with several bodies; verify official-vehicles "A mod for a trailer with several bodies" (Kögel Cargo, from placeholder PNGs): the Body ids in the main texture's `also_paints` entry and the Body (Long) ids in its own accessory entry.
- [ ] 4.4 Extend the ignored test behind `TRUCKPAINT_ETS2_PACK` to install every ETS2 entry from the real pack and plan a mod for each; verify it passes locally on `~/Downloads/templates`.
- [ ] 4.5 Measure New Project → Create Project with the SCS Box Trailer and the Volvo FH6, every accessory checked (time and memory, release build); verify by recording the numbers in the PR, and report to the user if it's noticeably slow or heavy (design Risks).
- [ ] 4.6 Screenshot New Project in German at 100 % with the full Official list and a trailer with several bodies checked; verify that nothing is clipped and that the list scrolls.

## 5. In-game check, checks and docs

- [ ] 5.1 In-game check (manual, ETS2 1.59) of the sample in design Decision 6. Export one mod painting each texture of those vehicles with distinct colors and labels, then check each cabin, accessory and body in the game. Verify by recording the result in the PR. If a trailer's main body looks wrong, stop and ask the user (design Risks); for any other failure, fix the entry and the rule behind it, then re-run 3.7.
- [ ] 5.2 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --release --locked` and the screenshot suite; verify all pass.
- [ ] 5.3 At ship time, update `docs/roadmap.md`: add the ETS2 catalog and `tpv catalog match` to Shipped, and keep row 3 for the ATS batch; verify `openspec validate official-vehicles-ets2 --strict` passes.
