## 1. Discover owned trailers

- [x] 1.1 Introduce the vehicle kind (definition root, data unit header, painted groups, manifest kind) and route the existing truck build through it; verify all existing extraction tests pass and a truck package built from the fake-converter fixture is byte-identical to before.
- [x] 1.2 Extract `/def/vehicle/trailer_owned` in the inventory and discover trailer ids from their `data.sii`; refuse an id defined both as truck and trailer; let `--vehicle` select a trailer; verify with fake-converter fixtures (trailer discovered, filtered trailer, truck/trailer collision refused, ATS still refused) and that `tpv --help` mentions trailers.

## 2. Build trailer packages

- [x] 2.1 Read trailer identity from `accessory_trailer_data` and name trailers with the existing localization rules plus the empty-brand "SCS <Series>" rule; verify fixtures for an SCS trailer ("SCS Box") and a branded one ("Krone Cool Liner").
- [x] 2.2 Walk `body/`, `chassis/` and `accessory/` units and build the single 4096² `trailer` main texture from every painted part without an independent size; verify a fixture trailer produces `kind: "trailer"`, one main texture with no game ids, and passes `tpv check`.
- [x] 2.3 Group trailer accessory ids by identical override signature into one texture with all their game ids, sizes from the shared masks and a superposed template; record grouped ids and masks in the report; verify fixtures for bodies sharing a mask in every paint job (one texture) and ids differing in one paint job (separate textures).
- [x] 2.4 Refuse a trailer whose paint jobs define no main mask and no override, and confirm size rules, exclusions, missing-model refusal and `--previous` layout versions apply to trailers; verify with fixtures for each, including a trailer rebuilt with `--previous` keeping layout version 1.

## 3. Validate in the app and against the game

- [x] 3.1 Add an application test that installs a generated trailer package, adds it to an ETS2 project and exports a mod; verify the `trailer_owned` paint job and one override per accessory texture listing all its game ids.
- [x] 3.2 Run a full batch on the local ETS2 installation with `--previous` set to the last truck batch; verify the 26 truck packages are byte-identical to that batch, every trailer package passes `tpv check`, and list refused trailers with their reasons.
- [x] 3.3 Compare trailer names, main/accessory textures and grouped game ids with Paintjob Packer's `vehicles/ets/scs/*.json` trailer data; record agreements, differences and which side matches the game in `docs/game-extraction-plan.md`, with the trailers to check in game first.

## 5. Address first-batch trailer refusals

- [x] 5.1 Ignore comments after a SII unit header (`… .chs_r_l #(100cm)`); verify a parser test and that the six affected trailers parse.
- [x] 5.2 Give a trailer whose parts all have their own texture a 64×64 main texture with a blank template; verify a fixture trailer and package validation.
- [x] 5.3 Report colour-only trailers as skipped (`not_paintable`), without package and without failing the batch; exclude definitions with no model field at all as `accessory_without_model`; verify fixtures, the exit code and the updated missing-model test.
- [x] 5.4 Rerun the trailers on the local game, validate the packages and record the remaining refusals.
- [x] 5.5 Accept a look or variant repeated with identical content as one (Schwarzmüller Food Tank `cage.pmd`); keep refusing differing duplicates; verify a geometry test and that the trailer builds and passes `tpv check` on the local game.

## 4. Documentation and checks

- [x] 4.1 Update the official vehicles documentation page in English, then French, Spanish and German, to cover trailers (discovery, single main texture, grouped accessories, SCS names); verify `mise run doc:build` passes and German text fits.
- [x] 4.2 Run `mise run ci` and verify format, lint, tests and build pass; report any GPU or game-only checks that could not run.
