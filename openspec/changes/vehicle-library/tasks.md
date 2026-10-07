## 1. Package format (tp-vehicles)

- [x] 1.1 Create `crates/tp-vehicles` (deps: serde, serde_json, semver, zip, image, usvg) with the `Manifest`, `Kind`, `Game`, `Variant`, `Texture` and `Requirement` types. Unknown fields are ignored and optional fields default. Verify with a unit test parsing a full manifest and one with an unknown field.
- [x] 1.2 Implement `Package::read` and `read_manifest`, validating in the order of D1, with a `PackageError` enum. Add the `sample_package` test helper. Verify with unit tests for each invalid case: not a zip, no manifest, bad JSON, bad id, bad version, bad range, newer format, empty variant, duplicate texture id, bad size, missing template, wrong image type, unsafe path, oversized package, oversized image.
- [x] 1.3 Write `docs/vehicle-package-format.md` with a complete example manifest. Verify with a unit test that the example manifest in the doc parses.

## 2. Document model and file format

- [x] 2.1 tp-core: add `VehicleRef`, `SurfaceTemplate`, `TemplateStatus`, `Project.vehicle` and `Surface.template`, plus `template_assets()`, and leave template assets out of `asset_usage` and the asset list. Verify with `cargo test -p tp-core`.
- [x] 2.2 tp-core: extend `Snapshot` with surface metadata and the vehicle reference. Restore rebuilds the surface list and keeps the current template opacity and visibility. Verify with unit tests: adding a surface undoes, and opacity survives undo.
- [x] 2.3 tp-file: reset the format to v1. Make the current v3 structures the only module, `v1`, plus vehicle and template. Set `FORMAT_VERSION = 1`. Remove the `v1`/`v2`/`v3` modules, the migrations and the old fixtures, and add `Error::Unsupported` for development formats 2 and 3. Regenerate `tests/fixtures/v1.truckpaint` with a three-texture vehicle project. Verify with `cargo test -p tp-file`:
  - round trip;
  - fixture opens;
  - format 3 refused as a development format;
  - format 2 refused as a development format;
  - a format above that is reported as newer.

## 3. Library and vehicle projects (tp-app)

- [x] 3.1 Add `vehicles.rs`: `VehicleLibrary` (index, install, remove, newest per id) on `AppDirs.data/vehicles`, created in `AppState`, plus the translated `PackageError` messages. Verify with unit tests in temporary folders: install, reinstall the same version, two versions, remove, invalid file left out.
- [x] 3.2 Add `vehicle_project(name, package, variant)` building surfaces, template assets and the vehicle reference. Verify with a unit test of three textures with sizes, names and templates.
- [x] 3.3 Add surface switching: `set_active_surface` with per-surface viewports, selection and session cleanup, and viewport swap on undo/redo across surfaces. Verify with unit tests: "Paint on the chassis" and "Undo across textures".
- [x] 3.4 Add `UpdatePlan::compute` and `apply_update` (replace templates, scale resized surfaces, add new surfaces, mark removed ones, record the version, one undo step), plus Dismiss. Verify with unit tests: layout changed, resized, added and removed textures, and undo.

## 4. UI (tp-app)

- [x] 4.1 Draw the template overlay in `paint.rs` with opacity and visibility, and add the `ShowTemplate` command (View, Shift+T). Verify with a kittest "Hide the template", and a tp-render test that templates are never exported.
- [x] 4.2 Add the surface tab bar above the canvas when there are 2 or more surfaces. Verify with a kittest that clicking a tab switches the Layers panel and the status bar.
- [x] 4.3 Add the New Project wizard Vehicle step (list, search, filters, variants, Blank texture, Install…, Next/Back), with Name & resolution adapting to vehicle or blank. Verify with kittests: "Creating a project for a vehicle", "Back to the vehicle", blank project unchanged.
- [x] 4.4 Add the Vehicle Library dialog (filters, list, versions, Install…, Remove with confirmation, empty state), the `VehicleLibrary` and `VehicleInfo` commands, and drag and drop of `.tpv` files. Verify with kittests: install from the dialog, two versions, remove a version keeps open projects intact, and an invalid package shows a message.
- [x] 4.5 Build the Vehicle panel (vehicle info, textures, template opacity and visibility, Layout changed badge with Dismiss, update notice) and the `UpdateTemplate` command with its confirmation dialog. Verify with kittests: "Newer version available", "Patch that changed the cabin layout", "Undo an update", and opening without the package.
- [x] 4.6 Make export use the surface size, propose the vehicle file name, and exclude templates. Verify with kittests: "Exporting a smaller texture", and a PNG of a vehicle project with a visible template contains only the artwork.
- [x] 4.7 Add every new text in en/fr/es/de. Verify that the tp-i18n completeness tests and `no_screen_shows_a_raw_message_id` pass, extended to the Vehicle Library dialog, the wizard Vehicle step and a vehicle project.

## 5. Verification

- [x] 5.1 Add screenshots `render_vehicle_project` (tabs, template overlay, Vehicle panel) and `render_vehicle_wizard` with sample packages, in English and German. Review them visually with `mise run screenshots`.
- [x] 5.2 Run `mise run fmt:check`, `mise run lint` and `mise run test`, plus the stress run (4 parallel × 5 rounds of panels, localization, gradients and vehicles kittests), and confirm everything passes.
