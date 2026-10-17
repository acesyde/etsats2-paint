## Why

Official packages currently depend on the Carsmaniac template pack and Paintjob Packer catalog. The validated MAN TGX and DAF NGD experiments demonstrate that installed ETS2 definitions and models can supply autonomous vehicle packages; a maintained Rust command should replace that prototype and the old source chain.

## What Changes

- Add `tpv extract <installation> --converter <executable> [--vehicle <identifier>] [--output <directory>] --package-version <version>` for official ETS2 trucks, including installed vehicle and tuning DLC, excluding user mods, ATS and trailers.
- Discover vehicles, cabins, accessory assignments and model references from game definitions; resolve inclusions and select looks, variants, painted faces and UVs before generating SVG templates and manifests.
- Produce one validated, autonomous `.tpv` per complete vehicle, named `official.ets2.<vehicle>`, with an explicitly supplied SemVer. Cabin textures are 4096 × 4096; accessory dimensions follow the validated definition-derived method and ambiguous cases fail.
- Write each batch into a unique directory under `dist/` by default, with packages and a JSON report; keep intermediates under `target/`. Continue after vehicle failures, publish no partial vehicle package and return failure for an incomplete batch.
- **BREAKING:** Remove the official catalog, its CLI commands and modules, Carsmaniac import, template pack preferences, dependent UI offers and obsolete credits. Keep installed-package selection, manual `.tpv` installation, custom vehicles and samples.
- **BREAKING:** Keep the existing official identities, including `official.ets2.daf.2021`; old Carsmaniac packages are retired with no migration or placement compatibility promised. No automatic deletion of user libraries or projects is introduced.
- Replace the extraction prototype with the maintained command and update tests, four-language UI/documentation and affected generated media. Separate technical checks from recorded in-game evidence.

Non-goals: marketplace, upload, in-app extraction, ATS extraction, trailer extraction, user-mod extraction, migration of old packages, and automatic layout comparison between generated versions.

## Capabilities

### New Capabilities
- `ets2-game-extraction`: archive discovery, definition resolution, supported geometry extraction, batch outputs, reports and isolated failures.

### Modified Capabilities
- `official-vehicles`: retire the catalog and template pack chain; official packages use manual installation.
- `vehicle-authoring`: expose the maintained extraction command alongside pack and check; remove catalog commands.
- `start-screen`: New Project lists installed vehicles only and retains manual installation, samples and Custom Vehicle.
- `vehicle-projects`: Add Vehicle lists installed vehicles only, preserving fleet and texture operations.
- `app-preferences`: remove template pack folders and their controls, tolerating old preference fields.
- `user-documentation`: replace the template pack workflow with extraction and manual installation instructions.

## Impact

- `tp-pack`: extraction module, `tpv` CLI, fake-converter integration tests; reuse `pack_entries`/`pack_folder` and native package validation.
- `tp-vehicles`: remove the catalog module; retain package format, validation and samples.
- `tp-app`: remove `official`, template pack dialogs/state/preferences and catalog-dependent selector rows; preserve library installation, project creation and mod export.
- Remove `catalog/` and prototype extraction code after preserving relevant test evidence. Update `docs/game-extraction-plan.md`, contributor references, locales, UI tests, screenshot fixtures and documentation media.
- ConverterPIX remains an external executable. Game assets, generated packages and intermediates are not committed.
- Existing `first-run` and `template-update-impact` proposals remain separate; this change does not implement their marketplace or update-impact behavior.
