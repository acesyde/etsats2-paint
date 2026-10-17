## 1. Preserve evidence and establish source resolution

- [x] 1.1 Inventory prototype assumptions and preserve portable MAN/DAF regression fixtures plus historical in-game evidence before removing the prototype; verify each fixture identifies its source version, cabin/accessory coverage and tested selection rule without committing game assets.
- [x] 1.2 Establish ConverterPIX archive discovery/access and override precedence for the installed base, vehicle DLC and tuning DLC; verify conflicting/overlaid fixture files resolve as the game source view does and the report can identify the winning source.
- [x] 1.3 Add the extraction CLI with required converter and SemVer, vehicle filter and output parent; verify help and argument tests reject missing/invalid versions, ATS installations, unknown vehicles and trailer targets without producing packages.

## 2. Discover and resolve game definitions

- [x] 2.1 Implement the needed SII reader and include resolution with source paths, arrays, quoted values, relative/absolute references, cycle detection and containment; verify synthetic fixtures cover nested includes, missing files, cycles and conflicting definitions.
- [ ] 2.2 Discover official ETS2 trucks and derive names, brands, cabins and exterior model/look/variant references without vehicle profiles; verify no-game fixtures include vehicle DLC, UK models and shared model paths while excluding user mods, ATS and trailers.
- [ ] 2.3 Resolve tuning accessories, independent paint assignments, cabin compatibility, export flags and crop-derived dimensions; verify coverage fixtures include tuning DLC and reject missing/conflicting dimensions or unsupported compatibility data.

## 3. Select painted geometry and build native packages

- [ ] 3.1 Invoke ConverterPIX with batch-local inputs/logs and validate exits, diagnostic failures and required output files; verify a fake converter covers failed exits, error output with successful exit and missing conversion products.
- [x] 3.2 Parse converted model/material traits and select looks, variants, visible painted faces and semantic UV aliases; verify MAN/DAF-derived tests, hidden parts, alias/tag differences and missing/ambiguous selections.
- [x] 3.3 Validate geometry counts, finite UV bounds and triangle indices, and explicitly reject unsupported paint shaders/alternate mappings; verify malformed and unsupported fixtures refuse the vehicle with source context.
- [ ] 3.4 Reconcile all cabins and painted accessories, generate deterministic SVG unions of compatible options and stable collision-free part ids; verify uncovered/empty geometry and id collisions fail, cabin sizes are 4096-square and accessory dimensions match fixtures.
- [x] 3.5 Generate the native manifest with official identity, explicit package version, game compatibility and export metadata, then reuse native packing/validation; verify autonomous packages pass `tpv check`, contain all templates and produce identical bytes from identical fixture inputs.

## 4. Publish batches and reports

- [x] 4.1 Reserve unique output batch directories and separate `target/` workspaces, publishing only validated packages through temporary writes and rename; verify repeated/concurrent reservations do not overwrite batches and failed writes leave no partial `.tpv`.
- [ ] 4.2 Continue after per-vehicle failures and compute the final exit result; verify fake-converter batches preserve successful packages, refuse failed vehicles and return nonzero for any incomplete batch or report/publication failure.
- [ ] 4.3 Publish structured JSON reports with source precedence, versions, per-texture contributors/coverage, errors and limits; verify report fixtures distinguish technical validation from game evidence and explicitly mark unknown converter metadata and vehicles needing game checks.

- [x] 4.4 Record missing accessory definitions, painted surfaces and game files in `report.json`, with referencing sources when available and explicit unknown DLC ownership; verify that diagnostics preserve refusal for required files and defined accessories and publish no Markdown or partial package.

- [x] 4.5 Exclude paint overrides with no installed accessory definition, retaining their source diagnostics on successful vehicles; verify unchanged package bytes, continued refusal for defined accessories and required files, then regenerate MAN and validate its manual app workflow.

## 5. Remove the catalog chain and preserve manual workflows

- [x] 5.1 Remove `catalog/`, vehicle/packer catalog modules and catalog CLI commands, updating exports and dependent tests; verify reference searches contain no live source-chain dependency and pack/check/custom/sample tests still pass.
- [x] 5.2 Simplify shared New Project/Add Vehicle selectors to installed vehicles and remove official installation state, workers, dialogs and events; verify headless tests cover empty lists, filters, keyboard selection, multiple vehicles and texture choices without catalog offers.
- [x] 5.3 Remove template pack preferences/controls and obsolete credits, updating all four locales; verify old preference files retain supported settings, removed fields are ignored and French/Spanish/German screens fit, especially German labels.
- [x] 5.4 Add explicit application regression tests for manual MAN/DAF `.tpv` installation, library listing, project creation/save/reload and mod export; verify cabin/accessory counts, DDS dimensions and exported game/TOBJ references against the generated packages.
- [x] 5.5 Verify custom vehicles, original sample packages, ATS/manual trailer packages and embedded project templates still work; run the existing targeted tests without depending on the removed catalog.

## 6. Replace prototype documentation and affected media

- [x] 6.1 Replace prototype extraction usage with the maintained command and retain its validated evidence in durable notes/fixtures; verify no active instruction calls the removed Python extractor or advertises it as the supported tool.
- [x] 6.2 Update English player/author documentation, then matching French/Spanish/German pages, covering extraction, manual installation, reports, unsupported cases and retirement without migration; verify no removed Fluent ids or obsolete Carsmaniac workflow remain, and preserve the English-only package-format fallback.
- [x] 6.3 Update affected screenshot/doc-media fixtures and regenerate the affected sections with `mise run doc:media <section>` plus applicable screenshots; verify all four languages show installed-package workflows and the site has no dangling removed media references.
- [x] 6.4 Update contributor/agent references and extraction-plan notes affected by catalog removal, following their writing conventions; verify reference searches and `mise run doc:build` pass.

## 7. Validate against the installed game and complete checks

- [x] 7.1 Generate fresh MAN/DAF packages from the local ETS2 installation with the maintained command and an explicit version; compare cabin ids, accessory assignments, dimensions and geometry/UV evidence with the validated runs, explaining any tuning-DLC additions or source changes in the delivered report.
- [x] 7.2 In an isolated application library, manually install the fresh MAN/DAF packages, create projects and export mods; verify the actual UI workflow completes, exported files match the packages and technical checks are recorded separately from historical game evidence.
- [x] 7.3 Generate the full discovered ETS2 truck batch under a unique `dist/` subdirectory; verify successful packages individually, record every refused vehicle and deliver the report with the list of new vehicles/configurations still needing in-game checks. An incomplete extraction batch is reported as such, not treated as a successful full batch.
- [x] 7.4 Delegate `mise run ci` to the rust-ui-tester as required by project apply guidance, resolve failures and explicitly run/confirm the app build and documentation build; verify format, lint, tests and build pass, reporting any GPU/game-only checks that could not be completed.

## 8. Extend shared shader coverage

- [x] 8.1 Establish SCS shader/UV semantics, recognize truckpaint/tsnmap/tsnmapuv with optional airbrush/altuv and a/asafew.a alpha-test suffixes, derive one cabin UV mode and enforce it on accessories and package metadata; cover distinct UV streams, missing/duplicate aliases, unsupported flavors, conflicting cabins, detail_model and UK reuse with no-game fixtures.
- [x] 8.2 Regenerate Volvo FH 2024 and Mercedes Actros 2014 as TPV-only trials, validate any produced packages, and report precise further refusals and game-check requirements without publishing reduced packages.
- [x] 8.3 Replay cached MAN/DAF geometry against all 45 validated SVGs, update four-language extraction documentation and source notes, then pass CI and documentation build.

## 9. Address full-batch refusals

- [x] 9.1 Support the geometry-neutral flipflake flavor and preserve finite representable UV overflow with original channel/coordinates and reported bounds/counts; test combinations, duplicate flavors and invalid geometry without game assets.
- [x] 9.2 Diagnose all missing/conflicting accessory dimensions with mask and source provenance, retaining refusal for genuinely unresolved sizes and shared fill masks; test structured diagnostics, continued refusal for defined base-colour-only materials without paint-job UV coverage, and absence of partial packages.
- [x] 9.3 Replay MAN/DAF baselines, regenerate the local ETS2 batch, validate resulting packages and report remaining failures/game-check requirements; update source notes and four-language documentation and pass CI/doc build.

## 10. Resolve the last refusals

- [x] 10.1 Use the largest declared crop size for accessories with several sizes, scale sizes below the package minimum up keeping their aspect ratio, keep accessories overridden only by unsized fill masks on the main texture and exclude accessories without truckpaint material, all reported in `report.json`; test each case with no-game fixtures.
- [x] 10.2 Publish packages and reports through a temporary file and rename, and make batch reservations retry instead of failing on a name collision; test that a failed write leaves no partial file.
- [x] 10.3 Regenerate the local ETS2 batch, validate the packages and record the results in the source notes and four-language documentation; pass CI and the documentation build.

## 11. Align with Paintjob Packer

- [x] 11.1 Use the ordinary paint UVs (`_TEXCOORD1`) for every material, including `altuv` flavors, and record `game.alt_uv: false`; test that alternate-UV materials keep the ordinary layout.
- [x] 11.2 Name trucks, brands and cabins from the game's English localization, appending the short series when it tells trucks apart and the cabin id when cabins share a name; test with a no-game localization fixture.
- [x] 11.3 Regenerate the local ETS2 batch, compare names and layouts with Paintjob Packer, validate the packages and update the source notes; pass CI and the documentation build.
