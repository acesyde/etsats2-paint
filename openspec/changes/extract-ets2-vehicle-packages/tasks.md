## 1. Preserve evidence and establish source resolution

- [ ] 1.1 Inventory prototype assumptions and preserve portable MAN/DAF regression fixtures plus historical in-game evidence before removing the prototype; verify each fixture identifies its source version, cabin/accessory coverage and tested selection rule without committing game assets.
- [ ] 1.2 Establish ConverterPIX archive discovery/access and override precedence for the installed base, vehicle DLC and tuning DLC; verify conflicting/overlaid fixture files resolve as the game source view does and the report can identify the winning source.
- [ ] 1.3 Add the extraction CLI with required converter and SemVer, vehicle filter and output parent; verify help and argument tests reject missing/invalid versions, ATS installations, unknown vehicles and trailer targets without producing packages.

## 2. Discover and resolve game definitions

- [ ] 2.1 Implement the needed SII reader and include resolution with source paths, arrays, quoted values, relative/absolute references, cycle detection and containment; verify synthetic fixtures cover nested includes, missing files, cycles and conflicting definitions.
- [ ] 2.2 Discover official ETS2 trucks and derive names, brands, cabins and exterior model/look/variant references without vehicle profiles; verify no-game fixtures include vehicle DLC, UK models and shared model paths while excluding user mods, ATS and trailers.
- [ ] 2.3 Resolve tuning accessories, independent paint assignments, cabin compatibility, export flags and crop-derived dimensions; verify coverage fixtures include tuning DLC and reject missing/conflicting dimensions or unsupported compatibility data.

## 3. Select painted geometry and build native packages

- [ ] 3.1 Invoke ConverterPIX with batch-local inputs/logs and validate exits, diagnostic failures and required output files; verify a fake converter covers failed exits, error output with successful exit and missing conversion products.
- [ ] 3.2 Parse converted model/material traits and select looks, variants, visible painted faces and semantic UV aliases; verify MAN/DAF-derived tests, hidden parts, alias/tag differences and missing/ambiguous selections.
- [ ] 3.3 Validate geometry counts, finite UV bounds and triangle indices, and explicitly reject unsupported paint shaders/alternate mappings; verify malformed and unsupported fixtures refuse the vehicle with source context.
- [ ] 3.4 Reconcile all cabins and painted accessories, generate deterministic SVG unions of compatible options and stable collision-free part ids; verify uncovered/empty geometry and id collisions fail, cabin sizes are 4096-square and accessory dimensions match fixtures.
- [ ] 3.5 Generate the native manifest with official identity, explicit package version, game compatibility and export metadata, then reuse native packing/validation; verify autonomous packages pass `tpv check`, contain all templates and produce identical bytes from identical fixture inputs.

## 4. Publish batches and reports

- [ ] 4.1 Reserve unique output batch directories and separate `target/` workspaces, publishing only validated packages through temporary writes and rename; verify repeated/concurrent reservations do not overwrite batches and failed writes leave no partial `.tpv`.
- [ ] 4.2 Continue after per-vehicle failures and compute the final exit result; verify fake-converter batches preserve successful packages, refuse failed vehicles and return nonzero for any incomplete batch or report/publication failure.
- [ ] 4.3 Publish structured JSON reports with source precedence, versions, per-texture contributors/coverage, errors and limits; verify report fixtures distinguish technical validation from game evidence and explicitly mark unknown converter metadata and vehicles needing game checks.

## 5. Remove the catalog chain and preserve manual workflows

- [ ] 5.1 Remove `catalog/`, vehicle/packer catalog modules and catalog CLI commands, updating exports and dependent tests; verify reference searches contain no live source-chain dependency and pack/check/custom/sample tests still pass.
- [ ] 5.2 Simplify shared New Project/Add Vehicle selectors to installed vehicles and remove official installation state, workers, dialogs and events; verify headless tests cover empty lists, filters, keyboard selection, multiple vehicles and texture choices without catalog offers.
- [ ] 5.3 Remove template pack preferences/controls and obsolete credits, updating all four locales; verify old preference files retain supported settings, removed fields are ignored and French/Spanish/German screens fit, especially German labels.
- [ ] 5.4 Add explicit application regression tests for manual MAN/DAF `.tpv` installation, library listing, project creation/save/reload and mod export; verify cabin/accessory counts, DDS dimensions and exported game/TOBJ references against the generated packages.
- [ ] 5.5 Verify custom vehicles, original sample packages, ATS/manual trailer packages and embedded project templates still work; run the existing targeted tests without depending on the removed catalog.

## 6. Replace prototype documentation and affected media

- [ ] 6.1 Replace prototype extraction usage with the maintained command and retain its validated evidence in durable notes/fixtures; verify no active instruction calls the removed Python extractor or advertises it as the supported tool.
- [ ] 6.2 Update English player/author documentation, then matching French/Spanish/German pages, covering extraction, manual installation, reports, unsupported cases and retirement without migration; verify no removed Fluent ids or obsolete Carsmaniac workflow remain, and preserve the English-only package-format fallback.
- [ ] 6.3 Update affected screenshot/doc-media fixtures and regenerate the affected sections with `mise run doc:media <section>` plus applicable screenshots; verify all four languages show installed-package workflows and the site has no dangling removed media references.
- [ ] 6.4 Update contributor/agent references and extraction-plan notes affected by catalog removal, following their writing conventions; verify reference searches and `mise run doc:build` pass.

## 7. Validate against the installed game and complete checks

- [ ] 7.1 Generate fresh MAN/DAF packages from the local ETS2 installation with the maintained command and an explicit version; compare cabin ids, accessory assignments, dimensions and geometry/UV evidence with the validated runs, explaining any tuning-DLC additions or source changes in the delivered report.
- [ ] 7.2 In an isolated application library, manually install the fresh MAN/DAF packages, create projects and export mods; verify the actual UI workflow completes, exported files match the packages and technical checks are recorded separately from historical game evidence.
- [ ] 7.3 Generate the full discovered ETS2 truck batch under a unique `dist/` subdirectory; verify successful packages individually, record every refused vehicle and deliver the report with the list of new vehicles/configurations still needing in-game checks. An incomplete extraction batch is reported as such, not treated as a successful full batch.
- [ ] 7.4 Delegate `mise run ci` to the rust-ui-tester as required by project apply guidance, resolve failures and explicitly run/confirm the app build and documentation build; verify format, lint, tests and build pass, reporting any GPU/game-only checks that could not be completed.
