## Why

TruckPaint models a vehicle as **variants**, each with its own full set of textures. ETS2 and ATS structure paint jobs differently (see "A paint job is a main texture plus accessories" in `docs/roadmap.md`):
- a truck has **cabins**, each with its own main texture, or one main texture shared by every cabin;
- an owned trailer has **one main texture** and no variants: its body types and lengths are accessory groups;
- both have **accessory textures** that belong to the whole vehicle.

The current model:
- duplicates the accessories under every variant;
- can't describe a trailer correctly;
- doesn't record what a mod needs: the game path, cabin internal names, accessory ids, `alt_uv` and `colour_picker`.

`custom-vehicle` and `mod-export` build on this model, and the package format has to be right before anyone else publishes packages. TruckPaint is unreleased, so the formats change in place.

## What Changes

**Package format (format 1, changed in place):**
- **BREAKING:** the manifest is reorganized into sub-objects, with one rule set for every kind of vehicle:
  - `game` holds everything aimed at the game: its id, the supported versions, the vehicle's `path`, `alt_uv`, `colour_picker` and `requires`;
  - `paint_job` replaces `variants`, with `main` (1..N main textures: one for a trailer or a truck whose cabins share a layout, one per layout otherwise) and `accessories` (the groups shared by the whole vehicle);
  - every part, main texture or accessory, has the same shape: `id`, `name`, `game_ids` (cabin internal names or accessory ids), and a `texture` sub-object (`size`, `template`, `layout_version`).
- Validation checks this structure, the game names and their uniqueness, without any rule specific to trucks or trailers. The `export` object of textures is dropped.

**Projects:**
- A vehicle in a project paints the **main textures the player checks** (a single main texture is always included, so a trailer needs no choice) and the **accessories the player checks**. Every accessory is checked by default.
- A surface belongs to one texture of one vehicle (no variant). It records whether it is a main texture or an accessory.
- **BREAKING:** project files of development builds that record variants are refused as a development format. The single-vehicle development files no longer open.

**Workspace:**
- The sidebar tree groups each vehicle's textures under **Main textures**, then **Accessories**, for every kind of vehicle.
- **Variants…** becomes **Textures…**, with checkboxes for main textures and accessories.
- New Project and Add Vehicle… show the same checkboxes.
- The status bar and exported file names drop the variant: "<vehicle> › <texture>".

**Update Template:**
- Matches textures by id within the vehicle.
- Offers the new version's new textures: new accessories checked, new main textures unchecked, a new single main texture always added.

**Samples:**
- The sample truck is redone with two main textures (two cabin layouts) and accessory groups.
- A new **sample trailer** (one Base texture plus body accessories) is added.
- "Install the sample vehicles" installs both.
- `tpv` packs, checks and describes the new structure.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `vehicle-packages`: the manifest's `game` and `paint_job` sub-objects (1..N main textures, accessories, uniform parts) replace variants and the top-level game fields; the validation rules that go with them; the Vehicle Library shows main textures and accessories; the built-in samples are a truck and a trailer.
- `vehicle-authoring`: `tpv check` describes main textures and accessories. The sample truck is redone with main textures and accessories. A sample trailer is added.
- `vehicle-projects`: a project paints chosen main textures (a single one always), plus chosen accessories. The sidebar tree has Main textures and Accessories sections. Textures… replaces Variants…. Update Template matches by texture and offers new textures. The status bar drops the variant.
- `document-model`: a surface belongs to one texture of a vehicle, as a main texture or an accessory (no variant).
- `project-files`: files store each vehicle and, per template, its texture and part. Variant-era development files are refused, and single-vehicle development files no longer open.
- `start-screen`: the New Project Vehicle step checks main textures and accessories instead of variants.
- `texture-export`: the proposed file name drops the variant.

## Impact

- **tp-vehicles:**
  - `Manifest`, with the new `Game`, `PaintJob`, `Part` and `Texture` types;
  - validation and `PackageError`;
  - `Package::template` keyed by texture id;
  - the `sample` test helpers.
- **tp-pack:** template path rewriting for DDS, `summary`, `describe`, the error messages, and the sample test with the trailer.
- **tp-core:** `ProjectVehicle` (no variants), `SurfaceTemplate` (`part` instead of `variant_id`), `TextureKey`, and the range and name helpers.
- **tp-file:** `FileVehicle`, `FileTemplate`, `check_fleet`, development-file detection, and the fixtures.
- **tp-app:**
  - `vehicle_project.rs` (creation, `set_parts`, update with new textures);
  - `vehicles.rs` (both samples);
  - `state.rs` and the sidebar `panels/vehicle.rs`;
  - `vehicle_dialogs.rs` (Textures…, Add Vehicle…, Update, Library) and the wizard in `dialogs.rs`;
  - `status_bar.rs`, `export.rs` and their tests.
- **tp-i18n:** variant messages replaced by cabin, accessory and texture messages, in 4 languages.
- **examples/vehicles:** new sample truck sources (1.0.0 and 1.1.0), sample trailer sources (1.0.0), rebuilt packages, and the README.
- **docs:** `vehicle-package-format.md` rewritten; the roadmap's shipped list.
