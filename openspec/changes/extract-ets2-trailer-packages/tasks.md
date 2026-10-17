## 1. Discover owned trailers

- [ ] 1.1 Introduce the vehicle kind (definition root, data unit header, painted groups, manifest kind) and route the existing truck build through it; verify all existing extraction tests pass and a truck package built from the fake-converter fixture is byte-identical to before.
- [ ] 1.2 Extract `/def/vehicle/trailer_owned` in the inventory and discover trailer ids from their `data.sii`; refuse an id defined both as truck and trailer; let `--vehicle` select a trailer; verify with fake-converter fixtures (trailer discovered, filtered trailer, truck/trailer collision refused, ATS still refused) and that `tpv --help` mentions trailers.

## 2. Build trailer packages

- [ ] 2.1 Read trailer identity from `accessory_trailer_data` and name trailers with the existing localization rules plus the empty-brand "SCS <Series>" rule; verify fixtures for an SCS trailer ("SCS Box") and a branded one ("Krone Cool Liner").
- [ ] 2.2 Walk `body/`, `chassis/` and `accessory/` units and build the single 4096² `trailer` main texture from every painted part without an independent size; verify a fixture trailer produces `kind: "trailer"`, one main texture with no game ids, and passes `tpv check`.
- [ ] 2.3 Group trailer accessory ids by identical override signature into one texture with all their game ids, sizes from the shared masks and a superposed template; record grouped ids and masks in the report; verify fixtures for bodies sharing a mask in every paint job (one texture) and ids differing in one paint job (separate textures).
- [ ] 2.4 Refuse a trailer whose paint jobs define no main mask and no override, and confirm size rules, exclusions, missing-model refusal and `--previous` layout versions apply to trailers; verify with fixtures for each, including a trailer rebuilt with `--previous` keeping layout version 1.

## 3. Validate in the app and against the game

- [ ] 3.1 Add an application test that installs a generated trailer package, adds it to an ETS2 project and exports a mod; verify the `trailer_owned` paint job and one override per accessory texture listing all its game ids.
- [ ] 3.2 Run a full batch on the local ETS2 installation with `--previous` set to the last truck batch; verify the 26 truck packages are byte-identical to that batch, every trailer package passes `tpv check`, and list refused trailers with their reasons.
- [ ] 3.3 Compare trailer names, main/accessory textures and grouped game ids with Paintjob Packer's `vehicles/ets/scs/*.json` trailer data; record agreements, differences and which side matches the game in `docs/game-extraction-plan.md`, with the trailers to check in game first.

## 4. Documentation and checks

- [ ] 4.1 Update the official vehicles documentation page in English, then French, Spanish and German, to cover trailers (discovery, single main texture, grouped accessories, SCS names); verify `mise run doc:build` passes and German text fits.
- [ ] 4.2 Run `mise run ci` and verify format, lint, tests and build pass; report any GPU or game-only checks that could not run.
