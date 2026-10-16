## Context

See proposal.md for motivation and scope. `tp-pack` already builds deterministic packages and validates them through `tp-vehicles`; the application validates and installs arbitrary `.tpv` bytes in `VehicleLibrary`. The prototype in `experiments/man-tgx-game/extract.py` validates MAN/DAF feasibility but hardcodes archive lists, metadata, cabin sets and model-folder filters. Its local reports are under `target/prototypes/{man-tgx-game,daf-ngd-game}/run-1/`. `docs/game-extraction-plan.md` records later user confirmation in game, without an exhaustive configuration matrix.

## Goals / Non-Goals

**Goals:** Keep extraction in the authoring layer, make refusals diagnosable, preserve the native package/install/export chain, and replace profiles with traceable definition-derived inputs.

**Non-Goals:** No new package format, general SII editing framework, converter replacement, extraction UI, persistent generation service, resumable batch feature or automatic template version diff. Unsupported cases fail visibly instead of gaining speculative shader rules.

## Decisions

### Reuse tp-pack and the existing CLI
Add an extraction module to `tp-pack` and an `extract` arm to `tpv`. Keep game filesystem/process work out of domain types. Feed generated manifests and SVGs into native packing and validation; do not introduce a second ZIP/validation implementation. Retain pack/check and custom authoring. A new crate or separate Python tool would add another public surface without serving this scope.

### Resolve a source view before interpreting vehicle definitions
Use ConverterPIX for archive access and model conversion. Restrict its mounts to official archives in the specified installation, never the user mod directory. Inventory the base archives and installed DLC needed to resolve the truck definition/model graph; do not infer truck coverage from DLC filename profiles. A file from a trailer/map archive may be a shared dependency, but no trailer becomes an extraction target.

Record the mount order and winning source of each resolved file. Establish archive precedence against the local installation and ConverterPIX behavior with fixtures before relying on it; sorting filenames alone is not evidence of game-equivalent precedence. Reject unresolved conflicting overrides. Parse the SII subset used by the dependency graph with quoted values, arrays, units and relative/absolute includes, cycle detection and path containment. Unknown fields can be ignored only when they cannot affect paint selection or coverage; unsupported relevant constructs fail with source context. Regex scanning of top-level files alone would miss includes and shared models.

### Separate game-derived metadata from geometry selection
Discover trucks from resolved truck definitions and derive cabin names/ids (names translated from the game's English localization, as Paintjob Packer shows them), exterior models (including UK alternatives), looks, variants and accessory compatibility. Resolve shared model references without the prototype's truck-folder filter. Treat independent paint overrides and painted geometry as two inputs to reconcile: an override targeting an installed definition without painted coverage fails, while a reference with no installed definition is excluded and diagnosed; painted geometry omitted by the override list must still be assigned to its compatible main texture or cause a refusal.

Select the look and variant exactly once, filter visible parts, map material aliases, then select the semantic paint UV alias. Preserve ConverterPIX image coordinates, finite representable coordinates, vertex/triangle counts and index validation. Support the truckpaint, tsnmap and tsnmapuv families with optional airbrush/flipflake/altuv flavors and a/asafew.a alpha-test suffixes (which do not change paint UV bindings), rejecting unknown/duplicate flavors and multiple normal-map variants. Use `_TEXCOORD1` for every painted material, including `altuv` flavors. On DAF XF105, Actros 2014, Magnum, Premium, Scania R 2009, Streamline and FH16 2009 the two channels differ: the game's colour paint jobs, Paintjob Packer and the community template packs use the ordinary layout, while SCS DLC liveries set `alternate_uvset: true`. The ordinary layout keeps our templates compatible with existing community work. Record shaders, actual selected aliases, coordinate bounds and overflow counts per model. Preserve finite out-of-bounds UVs in the SVG: its viewport clips the wireframe, without changing coordinates, wrapping or choosing another UV channel. Reject non-finite values or values outside the edge quantization range. SCS documents that flipflake adds no exported geometry data. Packages record `game.alt_uv: false`; never mix both layouts into one template. Unavailable/ambiguous `_TEXCOORD1` aliases fail. Resolve cabin `detail_model` and its UK variants as well as legacy model/exterior fields.

Build each cabin's template from compatible painted contributors. Independently assigned accessories use their own template and stable game-derived part ids. Detect collisions introduced by part-id normalization. Cabin dimensions stay 4096-square; accessory dimensions use the validated paint-job crop names. When the game declares several sizes for one accessory (Scania R/S 2016 rear fenders: 1024×512 and 2048×512, both real DDS sizes), use the largest: paint UVs are normalized, so the template fits either and the larger keeps more detail. For the same reason, sizes below the package minimum (Scania 2016 mudflaps at 32×32, sun shields at 1024×32) are scaled up by a whole factor to 64 on the short side. An accessory overridden only by unsized shared fill masks (Scania S 2024 Electric's `electric_a` livery uses a 4×4 fill) gets no independent texture and stays on the main texture, as in the game's colour paint jobs that do not override it; a fill image never supplies a template resolution. Record both cases in `dimension_diagnostics`, with masks and their definition/archive provenance. Deduplicate and order SVG edges deterministically as in the prototype.

### Keep explicit versions and established identities
Use `official.ets2.<game path>` and the supplied SemVer, including `official.ets2.daf.2021`. Derive game compatibility from the extracted version without promising future layout compatibility; start with the prototype's extracted minor-version range and report the exact source build. Initial extracted layouts use layout version 1; this tool does not automatically infer later layout revisions, so publishers must not claim unchanged layouts merely by incrementing the package version.

Old Carsmaniac packages are retired, with no migration. The user confirmed retaining the identity because the old packages will be removed. This change removes the production chain and documents manual retirement; it does not scan/delete user libraries or rewrite projects. Reusing an installed id/version retains the existing installer replacement behavior, which the authoring documentation must explain.

### Publish only complete vehicle outputs
Validate CLI arguments and installation before starting a batch. Atomically reserve a unique output subdirectory under `--output` (default `dist/`); put its working inputs/logs under a distinct `target/` extraction workspace. Process vehicles sequentially first, with a result per vehicle and no added concurrency infrastructure. Reuse converted models within the batch where safe; never treat stale files from another game build as proof of successful conversion.

Build/validate each package completely, then write a temporary non-`.tpv` file and rename it on successful completion on the output filesystem. Clean up unsuccessful temporary writes. A batch may contain successful vehicles even when another fails. Publish the final structured report similarly, and return failure for any vehicle, report or publication failure. No resume or overwrite flag is needed.

### Use one report for provenance and evidence boundaries
Keep a versioned JSON report structure with batch inputs, mounted sources, versions, per-vehicle stage/result, published filename, template coverage and contributors, diagnostic log references and supported-case limits. Record converter executable identity and version if exposed; explicitly record unavailable values. Missing references are recorded in per-vehicle `missing_dependencies`, with their kind, identifier/path and referencing source definitions when available. `expected_dlc: null` and `dlc_origin: "unknown"` explicitly distinguish unknown ownership from the installed DLC containing the reference. Each entry has a `disposition`: `ignored_reference` for paint overrides without an installed accessory definition, `blocking` for required missing files, `excluded` for defined accessories whose selected model has no truckpaint material (Renault T `s_guard.renault_pa` uses a base-colour-only `paint` flavor: it cannot show a livery whatever its override says). Successful vehicles retain their ignored references in the report. No Markdown sidecar is generated. Local technical checks and in-game evidence are separate fields. MAN/DAF's historical checks remain attributed to their tested configurations; adding tuning DLC does not automatically inherit that evidence.

### Remove catalog-dependent UI at the shared selector
Simplify the shared selector to installed library entries, preserving game/kind/search filters, newest-version selection and per-texture choices. Remove `tp-vehicles::catalog`, `tp-pack::catalog`, `catalog/`, catalog CLI commands, `tp-app::official`, template pack dialog/state/events, remembered pack paths and obsolete credits. Preserve the command-menu catalog, which is unrelated.

Keep manual package installation in the library and project flow, custom vehicles, samples, embedded project templates and mod export. Old preference files with removed pack fields still load their supported settings. Update affected Fluent ids in all four languages and replace obsolete documentation/media scenes with installed-package scenes. Keep the English-only package-format fallback; translating it is outside scope.

## Risks / Trade-offs

- Archive overlays or includes differ from the two prototypes -> establish precedence and source resolution fixtures, refuse conflicts, record provenance.
- Unsupported shader, compatibility or dimension rules reduce batch coverage -> refuse whole vehicles and report why; do not mask omissions as success.
- Same official identity can look like an old-package update -> remove old packages manually for trials and promise no migration; do not add destructive cleanup.
- Game updates change layouts while the explicit package version alone cannot describe them -> report exact sources and document author responsibility for layout versions.
- Historical MAN/DAF checks cover only tested configurations -> retain baseline evidence and mark new configurations/vehicles for game review.
- Converter errors may appear in diagnostics despite successful process exit -> check both exit result and recognized failure diagnostics; fake-converter tests cover both.

## Migration Plan

1. Preserve portable MAN/DAF selection/geometry regression fixtures and the recorded historical evidence before removing the prototype.
2. Implement the maintained command and validate MAN/DAF outputs against the baseline and native package checks.
3. Remove the catalog chain, update selectors/docs/translations/media and prove manual install, project creation and mod export through application tests.
4. Generate a fresh batch from the local ETS2 installation; retain outputs/report under ignored directories and list vehicles needing in-game checks. Retire old test packages manually, without deleting unrelated user data.
5. Run format, lint, tests and build plus the documentation build and affected media generation. Rollback is a code revert and removal of newly installed test packages; no project migration needs reversal.
