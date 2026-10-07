## 1. Package format (tp-vehicles)

- [x] 1.1 Reorganize the manifest (design D1):
  - `game: Game` (`id`, `versions`, `path`, `alt_uv`, `colour_picker`, `requires`), replacing the top-level `game`, `game_versions` and `requires`;
  - `paint_job: PaintJob { main, accessories }` replacing `variants`;
  - `Part { id, name, game_ids, texture: Texture }`, and `Texture { size, template, layout_version }` without `export`;
  - `Role`, `PaintJob::parts()` (package order) and `PaintJob::part(id)`;
  - `Package.templates` keyed by part id, and `Package::template(id)`.

  Verify that `cargo build -p tp-vehicles` succeeds.
- [x] 1.2 Rewrite `validate` with the general rules of D2, using one part-validation function for both lists: at least one main texture, unique part ids, game path and game id syntax (no length limit), game ids required on accessories and on main textures when there are several, game ids unique among main textures and among accessories. Verify with unit tests in `src/tests.rs`, one per error, plus:
  - a truck with two main textures;
  - a truck with one main texture without game ids;
  - a trailer;
  - an accessory id longer than 12 characters accepted;
  - an unknown field ignored.
- [x] 1.3 Update the `sample` test helpers (`package`, `truck_textures`, plus a trailer builder) to the new manifest. Verify that `cargo test -p tp-vehicles` passes, including `tests/svg_logs.rs`.

## 2. Packer (tp-pack)

- [x] 2.1 Make the DDS template-path rewriting walk `paint_job.main[].texture` and `paint_job.accessories[].texture`, and update `message()` for the new errors. Verify with `src/tests.rs`: a DDS main-texture template and a DDS accessory template are converted and their paths rewritten.
- [x] 2.2 Make `summary` and `describe` print the kind, game path, `alt_uv` and `colour_picker` when set, then the main textures, then the accessories, each with its game ids and texture size. Verify with `tests/cli.rs`: `tpv check` on a truck and on a trailer prints those lines and exits 0.

## 3. Sample vehicles

- [x] 3.1 Extend the scratchpad generator and regenerate the sample truck's SVGs:
  - 1.0.0: main textures Standard cab and High roof (4096), and accessories Chassis (2048) and Cab accessories (1024);
  - 1.1.0: Standard cab at layout 2, Chassis at 4096, and Side skirts (1024) added.

  Write both `vehicle.json` files in the new shape, with `game.path` `truckpaint.sample` and fictional game ids. Verify by rendering the SVGs to PNG and checking the labels and footers visually.
- [x] 3.2 Create `examples/vehicles/sample-trailer/1.0.0` (`community.truckpaint.sample_trailer`) with main texture Base (2048) and the accessories Curtain body 13.6 m (4096), Curtain body 10.5 m (4096) and Mudflaps (512), with original SVGs. Verify by rendering them and checking visually.
- [x] 3.3 Make `mise run sample-vehicles` pack every `examples/vehicles/<sample>/<version>/` folder to `community.truckpaint.<name>-<version>.tpv`, taking the id from its manifest. Run it, then make `tp-pack/tests/sample.rs` check all three committed packages against their sources. Verify that `cargo test -p tp-pack` passes, and that editing a source makes it fail with the package's name.
- [x] 3.4 Embed both samples in the app: `SAMPLES` and `install_samples()` in `tp-app/src/vehicles.rs`, used by the empty Vehicle Library and the wizard ("Install the sample vehicles", one confirmation naming both). Verify with the `vehicles.rs` unit tests (the embedded bytes equal the committed packages; installing gives two vehicles) and a `tests/vehicles.rs` UI test.

## 4. Document model (tp-core)

- [x] 4.1 Replace `SurfaceTemplate.variant_id` with `part: TexturePart` (`Main | Accessory`), make `TextureKey` `{package_id, texture_id}`, remove `ProjectVehicle.variants`, `VariantRef` and `variant_range`, and make `surface_names` return `(vehicle, texture)`. Verify that the tp-core unit tests are updated and pass: snapshot restore by key, vehicle range, names.

## 5. File format (tp-file)

- [x] 5.1 Change v1 in place (D7): `FileTemplate.part`, required `package_id`, and no variants or legacy `vehicle`. Add the head pre-parse that returns `Unsupported` for files with no vehicle, a legacy `vehicle`, or `variants`. Update `check_fleet`: unique `(package, texture)`, and every vehicle has a surface. Verify with `cargo test -p tp-file`: round trip with parts, refused duplicate texture, refused vehicle without surface, refused mixed games.
- [x] 5.2 Fixtures:
  - rename the current `v1.truckpaint` to `dev-variants.truckpaint` and test that it is refused as a development format;
  - rename `v1-single-vehicle.truckpaint` to `dev-single-vehicle.truckpaint` and test that it is refused too;
  - update `rich_project` (a truck with two main textures and an accessory, and a trailer with its Base texture) and regenerate `v1.truckpaint` with `write_current_fixture`.

  Verify that `v1_fixture_opens` checks the parts and that the test suite passes.

## 6. Fleet operations (tp-app)

- [x] 6.1 Rewrite creation (D5):
  - `fleet_project(name, package, chosen)`, with a single main texture implied and package order;
  - `default_textures(manifest)`;
  - `FleetError::BadTextures`;
  - remove `vehicle_project` and update `test_project()`.

  Verify with unit tests: sample truck with two main textures and every accessory gives 5 surfaces in order; trailer gives Base plus 3; an unchecked accessory is left out; no main texture is refused; an unknown id is refused.
- [x] 6.2 Implement the insertion of D4 and use it in `Workspace::add_vehicle(package, chosen)` and `Workspace::set_textures(package, chosen)` (replacing `set_variants`), which re-adds a missing single main texture. Verify with unit tests:
  - an accessory added between two others;
  - a main texture added before the others;
  - insertion next to a surface marked "Not in this version";
  - the active index shifted;
  - removal drops unused assets;
  - Undo restores;
  - the last main texture can't be removed.
- [x] 6.3 Update Template (D6): `plan` keyed by texture id with `new` textures, plain change names, and `apply_update(package, added)` that inserts the chosen new textures and a new single main texture always. Verify with unit tests:
  - sample 1.0.0 to 1.1.0 flags Standard cab, scales Chassis and adds Side skirts after the accessories when added;
  - Side skirts is not added when unchecked;
  - a removed texture is marked "Not in this version";
  - only one vehicle changes;
  - Undo restores.

## 7. Workspace UI (tp-app)

- [x] 7.1 Sidebar tree (D9): Main textures and Accessories headings for every vehicle, and the ⋯ menu's **Textures…** (`VehicleRequest::Textures`). Verify with `tests/vehicles.rs`: a truck and trailer project shows the headings and rows of the Vehicle panel scenario, and clicking "Mudflaps" activates it.
- [x] 7.2 Shared `texture_checkboxes` widget and `TexturesDialog` (replacing `VariantsDialog`): the last checked main texture locked (a single one shows checked and disabled), a confirmation naming the removed textures with artwork, and the "version missing" explanation offering Update Template…. Verify with UI tests: adding Cab accessories later inserts it between Chassis and Side skirts; removing High roof with artwork confirms, and Undo restores.
- [x] 7.3 New Project wizard and Add Vehicle… use `texture_checkboxes` (first main texture and every accessory checked, Next or Add disabled without a main texture), and the Name step lists the chosen textures. Verify with the start-screen UI tests: the default sample truck project, the trailer with Base checked and disabled, no main texture disables Next, Back keeps the choices.
- [x] 7.4 Update dialog lists the new textures (accessories checked, main textures unchecked, a single main texture always added) and passes `added` to `apply_update`. The Vehicle Library entry shows the main textures and the accessory count. Verify with UI tests: sample truck 1.0.0 to 1.1.0 shows Side skirts checked; the Library shows "Standard cab, High roof" and 3 accessories.
- [x] 7.5 Status bar breadcrumb "<vehicle> › <texture>" and export name "<project> - <vehicle> - <texture>". Verify with the status bar and `export.rs` unit tests, and the Next Texture scenario ("TruckPaint Sample Truck › Chassis").
- [x] 7.6 Update `tests/screenshots.rs` and `tests/common` (`sample_library()` with both samples, and `create_project`) to the new model. Verify that `cargo test -p tp-app` passes.

## 8. Localization

- [x] 8.1 Replace the variant messages with Textures…, Main textures, Accessories, the new texture checkboxes of the update dialog, Install the sample vehicles, the library summary and the new `pkg-*` errors, in en, fr, es and de. Remove the unused ids. Verify that the tp-i18n tests (same ids in every language, no unused id) pass.

## 9. Documentation

- [x] 9.1 Rewrite `docs/vehicle-package-format.md` for the new manifest (identity, `game`, `paint_job` and its parts; examples of a truck with several cabin layouts, a truck with one shared layout and a trailer; where each field goes in a mod), and update `examples/vehicles/README.md` (both samples, the Update Template walkthrough with the new texture checkbox). Verify by reading both against the vehicle-packages spec.
- [x] 9.2 Update `docs/roadmap.md`: `paint-job-model` moves to Shipped, the "vehicle model" block shows the final shape (`game`, `paint_job.main` 1..N, `paint_job.accessories`, uniform parts), and the "Navigation" bullet describes the Main textures and Accessories tree. Verify by reading the file.
- [x] 9.3 Update the Purpose of `openspec/specs/vehicle-projects/spec.md` and `openspec/specs/vehicle-authoring/spec.md`, which mention variants and a single sample. Verify that `openspec validate --specs --strict` passes.

## 10. Verification

- [x] 10.1 Run `mise run fmt:check`, `mise run lint` and `mise run test`, and verify that all three pass.
- [ ] 10.2 Run the app and walk through it:
  - install the sample vehicles;
  - create a truck project with both main textures;
  - add the trailer;
  - use Textures… to remove and re-add an accessory;
  - with only truck 1.0.0 installed, create a project, then install 1.1.0 and run Update Template;
  - save, reopen, and export a texture.

  Verify that the tree, the breadcrumb and the export name match the specs.
