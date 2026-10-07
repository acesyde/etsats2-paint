## 1. Document model (tp-core)

- [x] 1.1 Replace `Project.vehicle: Option<VehicleRef>` with `vehicles: Vec<ProjectVehicle>` (VehicleRef renamed, plus `variants: Vec<VariantRef { id, name }>`). Add `package_id` and `variant_id` to `SurfaceTemplate`, with `key() -> TextureKey`. Verify that `cargo build -p tp-core` succeeds.
- [x] 1.2 Add the helpers of D1: `game()`, `vehicle()`, `vehicle_of(index)`, `variant_range(package, variant)`, `variant_names(key)` and `has_artwork(range)`. Verify with unit tests on a three-vehicle project: ranges, game, names, artwork detection.
- [x] 1.3 Update `Snapshot` and `restore`: `vehicles` instead of `vehicle`, and opacity and visibility matched by `TextureKey`. Verify with unit tests:
  - two variants both have a "cabin" texture, and each keeps its own opacity across undo;
  - adding a surface undoes.

## 2. File format (tp-file)

- [x] 2.1 Change v1 in place (D10):
  - write `vehicles` with their variants, and `package_id` and `variant_id` on templates;
  - read the legacy single `vehicle` and keyless templates;
  - refuse files with no vehicle as `Error::Unsupported { found: 1 }`;
  - refuse unknown keys or mixed games as `Damaged`.
  
  Verify with `cargo test -p tp-file`: fleet round trip, legacy conversion, blank refused, mixed games refused.
- [x] 2.2 Regenerate `tests/fixtures/v1.truckpaint` (two vehicles, one with two variants) and add a legacy single-vehicle fixture. Verify that the fixture tests open both.

## 3. Fleet operations (tp-app)

- [x] 3.1 Add `fleet_project(name, package, variants)` following the D2 order, and keep `vehicle_project` as a one-variant wrapper. Verify with unit tests:
  - two variants give six surfaces of the sample in order;
  - the first one is active;
  - the vehicle and variant names are recorded.
- [x] 3.2 Add `Workspace::add_vehicle`, `set_variants` and `remove_vehicle` (D4), with undo labels, unused template assets dropped, active surface rules, and refusals (other game, duplicate, last vehicle, no variant). Verify with unit tests for each operation, its undo, its asset count and each refusal.
- [x] 3.3 Reset viewports when the surface keys change after an edit, undo or redo (D3). Verify with a unit test: a variant added, then undone, resets views without panicking, and a plain edit keeps views.
- [x] 3.4 Make Update Template work per vehicle (D5):
  - `plan` and `apply_update` find the vehicle by package id;
  - a missing variant gives Removed;
  - new textures are inserted after their variant;
  - `update_for(&ProjectVehicle)` no longer requires the variant;
  - `open_update_dialog(Option<&str>)`, and the command context uses the active surface's vehicle.
  
  Verify with unit tests: only one vehicle updated, missing variant, sample 1.0.0 to 1.1.0 with two variants (Side skirts inserted after each variant), undo.

## 4. New Project without blank (tp-app)

- [x] 4.1 Rework the wizard (D8):
  - remove the Blank texture row, the resolution cards and `WizardOutcome::Create`;
  - variant checkboxes, with the first one checked;
  - Next and Enter disabled without a vehicle and variant;
  - a Name step listing the textures of each variant;
  - `CreateVehicle` with a list of variants;
  - the sample install selects the sample with its first variant.
  
  Verify with kittests: "Creating a project" (sample), "Two variants at once", "No vehicle installed", "Back to the vehicle" keeps the checked variants, "Empty name".
- [x] 4.2 Move `tests/common` to vehicle projects (D11): `sample_library()`, `harness_with` installing it, and `create_project` through the sample. Fix or rewrite the blank-specific tests in `ui.rs`, `vehicles.rs` and `screenshots.rs`. Verify that `cargo test -p tp-app` passes.

## 5. Vehicles panel, dialogs and navigation (tp-app)

- [x] 5.1 Rebuild the panel as a tree (D6):
  - the fleet game header;
  - vehicle and variant sections;
  - texture rows that select;
  - badges and Dismiss;
  - the update notice per vehicle;
  - the `⋯` menu with Variants… and Remove from Project (disabled on the last vehicle, with a reason);
  - Add Vehicle…;
  - template opacity and visibility.
  
  Retitle the panel "Vehicles" and keep `PanelKind::Vehicle`. Verify with kittests: "Switching from the tree", "Newer version available" on the right vehicle, last vehicle disabled.
- [x] 5.2 Add the `AddVehicle` command (Vehicle menu) and `Modal::AddVehicle` (game locked, vehicles already in the project hidden, variant checkboxes). Verify with kittests: "Adding to an ETS2 project" lists only ETS2, and "A fleet of two trucks and a trailer".
- [x] 5.3 Add `Modal::Variants` (checkboxes from the recorded version, the missing-version message with Update Template…, inline confirmation when artwork would be removed) and `Modal::RemoveVehicle` (confirmation when there's artwork). Verify with kittests: "Removing a variant with artwork" then Undo, removing a vehicle without artwork asks nothing, and a missing recorded version shows the message.
- [x] 5.4 Scope the tabs to the active variant and show the `vehicle › variant › texture` breadcrumb in the status bar (D7). Verify with the kittest "Tabs follow the active variant".
- [x] 5.5 Name exports "<project> - <vehicle> - <variant> - <texture>" with `file_stem_safe` (D9). Verify with a unit test of the helper (`/`, `:`, trailing dots) and the kittest "Exporting a smaller texture".

## 5b. Workspace follow-ups (review feedback)

- [x] 5.6 Move the fleet tree to a Vehicles sidebar on the left (D6b): collapsible to a strip, View › Vehicles (F5), Vehicle Information opens it, width and state remembered, `PanelKind::Vehicle` removed from the column, older preferences still loading. Verify with the kittest `the_vehicles_sidebar_hides_and_comes_back`, the layout sanitizing unit test, and the fleet screenshots.
- [x] 5.7 Open projects in a view showing the canvas (D6c). Verify with the kittest `projects_open_showing_the_canvas`.
- [x] 5.8 Stop the "No match for 'sans-serif' font-family" warnings: the package reader sizes SVG templates with a font resolver that looks for no font. Verify with `tp-vehicles/tests/svg_logs.rs` (91 warnings for one read of the sample before, none after).

## 6. Messages, docs and checks

- [x] 6.1 Add and update messages in en, fr, es and de:
  - `panel-vehicles`, `vehicles-fleet-game`, `cmd-add-vehicle`;
  - the Add Vehicle and Variants dialog texts;
  - the confirmations;
  - `undo-add-vehicle`, `undo-change-variants`, `undo-remove-vehicle`;
  - the disabled reasons;
  - `new-project-step-name` = Name.
  
  Remove the unused blank and resolution messages. Verify that the tp-i18n checks and `no_screen_shows_a_raw_message_id` pass.
- [x] 6.2 Render the new screens with a screenshot test (fleet tree, Add Vehicle, Variants, wizard checkboxes) in English and German, and check them visually. Verify that `render_fleet_screens` produces the images and that they show no clipping.
- [x] 6.3 Update `docs/roadmap.md`: mark `fleet-projects` shipped, add `custom-vehicle` as the next change, and record the one-game-per-project decision. Update the README's feature note. Verify by reading the docs.
- [x] 6.4 Run `mise run fmt:check`, `mise run lint` and `mise run test`, and verify that all three pass.
