## Context

See proposal.md (Why). Today:
- `tp-vehicles::Manifest` holds `variants: Vec<Variant{id, name, textures}>`, and `Package.templates` is keyed by `(variant id, texture id)`.
- `tp-core::SurfaceTemplate` carries `package_id`, `variant_id` and `texture_id`, and `ProjectVehicle` carries `variants: Vec<VariantRef>`.
- The surfaces of a variant are contiguous (`Project::variant_range`), and the vehicle operations of `tp-app/src/vehicle_project.rs` (`fleet_project`, `add_vehicle`, `set_variants`, `plan`, `apply_update`) all iterate variants.
- `tp-file` v1 stores vehicles with their variants. It still reads two development shapes: blank files (refused through `has_vehicle`), and single-vehicle files (the legacy `vehicle` and `variant_id` fields, which are converted).

Nothing is released. Package format 1 and project format 1 change in place, as decided with the user.

The game's model comes from the Paintjob Packer analysis (`docs/roadmap.md`). A check of its 183-vehicle database (99 trucks, 72 of them with cabins that have separate paint jobs, and 84 trailers) found:
- every game path and cabin internal name is a dot-separated set of words in `[a-z0-9_]`, of at most 12 characters;
- accessory ids use the same characters but are sometimes longer (`fenders_a.parlok_small_p`);
- no accessory id appears in two groups of a vehicle.

## Goals / Non-Goals

**Goals:**
- One data model, from manifest to project file to UI, that maps one to one onto what a mod writes:
  - a main texture is a paint job, with `suitable_for` (its game ids) when it has game ids, and for every cabin otherwise;
  - an accessory group is one `simple_paint_job_data` with its `acc_list`.
- Packages record every piece of game data the mod export needs, and validation guarantees it is well formed.
- The surface order inside a vehicle is always main textures, then accessories, each in package order. Navigation and the sidebar rely on it.

**Non-Goals:**
- Copying the game data (game path, game ids, `alt_uv`, `colour_picker`) into project files. `mod-export` decides whether exporting needs the package installed or the data embedded in the project.
- Prefilling packages from the Paintjob Packer database, and editing game data in the app (`custom-vehicle`).
- Copy from cabin, and sharing artwork between cabins (`brand-kit`).
- Reading the previous shapes of the formats: development files are refused, not migrated.

## Decisions

### 1. A manifest in sub-objects, one shape for every part, no rule per kind
The user asked for a coherent format without needless special cases, using sub-objects wherever they make it simpler or easier to extend. The manifest has three layers:

```json
{
  "format": 1, "id": "community.truckpaint.sample_truck", "version": "1.1.0",
  "name": "TruckPaint Sample Truck", "brand": "TruckPaint", "kind": "truck",
  "authors": ["TruckPaint contributors"], "license": "MIT",
  "game": {
    "id": "ets2", "versions": ">=1.56", "path": "truckpaint.sample",
    "alt_uv": false, "colour_picker": false, "requires": []
  },
  "paint_job": {
    "main": [
      { "id": "standard", "name": "Standard cab", "game_ids": ["standard"],
        "texture": { "size": 4096, "template": "templates/standard_cabin.svg", "layout_version": 2 } },
      { "id": "high_roof", "name": "High roof", "game_ids": ["high_roof"],
        "texture": { "size": 4096, "template": "templates/high_roof_cabin.svg", "layout_version": 1 } }
    ],
    "accessories": [
      { "id": "chassis", "name": "Chassis", "game_ids": ["chassis.sample"],
        "texture": { "size": 4096, "template": "templates/chassis.svg", "layout_version": 1 } }
    ]
  }
}
```

- **Identity** (`id`, `version`, `name`, `brand`, `kind`, credits) stays at the top: the library and the UI read it.
- **`game`** holds everything aimed at the game: `id`, `versions`, `path`, `alt_uv`, `colour_picker` and `requires`. The former top-level `game`, `game_versions` and `requires` move into it. `mod-export` reads only `game` and `paint_job`, and new game data (a UV set, a DLC requirement, …) goes in `game` without touching anything else.
- **`paint_job`** holds two lists of **parts** with one shape, `{id, name, game_ids, texture}`:
  - `main`, 1..N: one main texture for a trailer or a truck whose cabins share a layout, one per layout otherwise;
  - `accessories`, 0..N.

  `game_ids` means "what the game calls the things this part covers": cabin internal names for a main texture, which become `suitable_for`, and accessory ids for an accessory, which become `acc_list`. An empty `game_ids` on a main texture means "every cabin", so no `suitable_for`.
- **`texture`** is a sub-object (`size`, `template`, `layout_version`): what TruckPaint draws, apart from what the game targets. It can grow later (several template layers, a mask) without changing parts.

`kind` drives no rule. It only names the game folder at export time (`truck` or `trailer_owned`) and filters the library. A trailer is simply a paint job with one main texture.

In Rust, `Manifest { …, game: Game, paint_job: PaintJob }`:
- `PaintJob { main: Vec<Part>, accessories: Vec<Part> }`;
- `Part { id, name, game_ids: Vec<String>, texture: Texture }`;
- `Texture { size, template, layout_version }`. `export` is dropped.

There is no `serde(flatten)`. `PaintJob::parts()` yields `(Role, &Part)` in package order (main, then accessories), with `Role = Main | Accessory`, and `PaintJob::part(id)` looks one up. `Package.templates` becomes `Vec<(part id, TemplateImage)>` and `Package::template(id)`.

*Alternatives:*
- **`cabins` for trucks plus a separate `main` for trailers and single-layout trucks:** the first draft. It had two concepts and three invalid combinations.
- **`cabins` only, 1..N, a trailer's base being its single "cabin":** the second draft. It needed trailer-only rules (exactly one entry, no game ids), and the format named a trailer's base a cabin.
- **Parts with flattened texture fields:** one level less, but it mixes what TruckPaint draws with what the game targets, and needs `serde(flatten)`, whose error messages are weaker.
- **A `separate_paint_jobs` flag** (Paintjob Packer's shape): the number of main textures already says it.
- **A top-level `textures` list referenced by id from the parts:** it would allow sharing one texture between parts, which no vehicle needs, at the price of dangling references to validate.

### 2. Validation: general rules only
`PackageError` drops `NoVariant`, `EmptyVariant`, `DuplicateVariant` and `DuplicateTexture{variant, texture}`, and gains:
- `NoMainTexture`: `paint_job.main` is missing or empty;
- `DuplicatePart(id)`;
- `MissingGameIds(name)`: an accessory without game id, or a main texture without game id when there are several (they would all claim every cabin);
- `BadGamePath(path)`;
- `BadGameId(id)`;
- `DuplicateGameId(id)`: twice among the main textures, or twice among the accessories.

A game id or game path is valid when it is words of `[a-z0-9_]` separated by dots, with **no length limit** (see Context: real accessory ids exceed 12 characters).

One function validates a part, whichever list it is in. Each error gets a message in `tp-app/src/vehicles.rs` (4 languages) and in `tp-pack`'s `message()`.

### 3. A project vehicle has no list of chosen textures; its surfaces are the list
`ProjectVehicle` loses `variants`. `SurfaceTemplate.variant_id` becomes `part: TexturePart` (`Main | Accessory`, in tp-core, which doesn't depend on tp-vehicles), and `TextureKey` becomes `{package_id, texture_id}`. What a vehicle paints is its surfaces: the sidebar groups them by `part`, and the Textures… dialog checks the ids present.

*Alternative:* keep chosen part ids on `ProjectVehicle`. They would duplicate the surfaces and could drift from them, for example after Undo, or when an update marks a texture "Not in this version".

`Project` helpers:
- `variant_range` is removed;
- `vehicle_range` stays;
- `surface_names(i)` returns `(vehicle, texture)`.

The status bar breadcrumb, `export::export_name` and the update plan's change names follow.

### 4. Canonical order inside a vehicle, kept by insertion
A new surface for texture `t` (from `set_textures` or `apply_update`) is inserted before the first surface of the vehicle whose texture comes after `t` in the manifest's order. If none does, it goes at the end of the vehicle's range. Surfaces whose texture isn't in the manifest (marked "Not in this version") keep their place. The active surface index is shifted when the insertion point is at or before it, as `set_variants` does today. Nothing is ever re-sorted, so a surface the user is looking at never jumps.

### 5. Choosing textures: one list of ids, a single main texture implied
Creating, adding and changing a vehicle take `chosen: &[String]`, the main texture and accessory part ids.
- A single main texture is always included, chosen or not, and `set_textures` adds it back if it is missing. This is the only special case, and it isn't about kind: it comes from "at least one main texture" when there is only one.
- Unknown ids, or no main texture chosen, give `FleetError::BadTextures`, which replaces `BadVariants`.
- `VehicleChoice{id, version, textures}` replaces `variants`. Its default, `default_textures(manifest)`, is the first main texture plus every accessory.

The functions are:
- `fleet_project(name, package, chosen)` (`vehicle_project` is removed);
- `Workspace::add_vehicle(package, chosen)`;
- `Workspace::set_textures(package, chosen)`;
- `Workspace::apply_update(package, added)`.

The checkbox widget `texture_checkboxes(ui, manifest, &mut chosen)` is shared by the wizard, Add Vehicle… and Textures…. It shows:
- a **Main textures** group, which won't uncheck the last checked one, so a single main texture shows as checked and disabled;
- an **Accessories** group.

### 6. Update Template offers new textures
`UpdatePlan` gains `new: Vec<NewTexture{id, name, part}>`, the manifest textures that the vehicle has no surface for. The Update dialog shows them as checkboxes:
- accessories checked;
- main textures unchecked;
- when the new version has a single main texture the vehicle doesn't paint, that texture as a disabled, checked row. This covers a main texture whose id changed, so the vehicle never ends up with no main texture.

`apply_update(package, added)` replaces and flags the existing surfaces as today, keyed by texture id instead of (variant, texture). It then inserts a surface for each id in `added`, plus an always-added single main texture, following decision 4.

The change names are plain texture names, since there is no variant to prefix.

### 7. Project file: the new shape only, development shapes detected first
In `tp-file` v1:
- `FileVehicle` drops `variants` and `variant_id`;
- `FileTemplate` drops `variant_id`, gains `part: FilePart`, and `package_id` becomes required;
- the legacy `vehicle` field of `FileProject` is removed.

`from_bytes` first parses a small head with serde, ignoring unknown fields: `{vehicles: [{variants: Option<IgnoredAny>}], vehicle: Option<IgnoredAny>}`. It returns `Error::Unsupported` (development format) when:
- there is no vehicle;
- there is a legacy `vehicle` field;
- any vehicle records `variants`.

Only then does it parse the full document. That way, development files are named as such instead of failing as damaged on the missing `part`.

`check_fleet` checks:
- one game;
- no duplicate vehicle;
- every surface has a template of a listed vehicle;
- `(package_id, texture_id)` is unique;
- every vehicle has at least one surface.

Fixtures:
- the current `v1.truckpaint` becomes `dev-variants.truckpaint`, the refused development file;
- `v1-single-vehicle.truckpaint` is deleted;
- `v1.truckpaint` is regenerated with `write_current_fixture`, from a `rich_project` that has a truck with two main textures and an accessory, and a trailer with its Base texture.

### 8. Samples and tooling
- `examples/vehicles/sample-truck/{1.0.0,1.1.0}` are rewritten as described in vehicle-authoring.
- `examples/vehicles/sample-trailer/1.0.0` is new.
- The SVGs come from the scratchpad generator (not committed, as before), extended with the trailer's layouts and the new labels.
- `mise run sample-vehicles` packs every `examples/vehicles/<sample>/<version>/` folder to `community.truckpaint.<id>-<version>.tpv`, taking the id from the manifest.
- `tp-pack/tests/sample.rs` checks every committed sample.
- `tp_app::vehicles` exposes `SAMPLES: [(file, bytes); 2]`, and `install_samples()` installs both in one call and returns both manifests for the confirmation.
- The DDS path rewriting in `tp-pack` walks `paint_job.main[].texture.template` and `paint_job.accessories[].texture.template`. `summary` and `describe` print the part structure and the game data.

### 9. UI
- **Sidebar** (`panels/vehicle.rs`): `variant_group` is removed. `vehicle_group` walks `vehicle_range` and emits a "Main textures" heading before the first `Main` surface and an "Accessories" heading before the first `Accessory`, for every kind of vehicle. Rows are unchanged: name, size, warning icon, highlighted when active.
- **Dialogs** (`vehicle_dialogs.rs`): `VariantsDialog` becomes `TexturesDialog`, and `VehicleRequest::Variants` becomes `VehicleRequest::Textures`. The removal confirmation lists the unchecked textures that hold artwork.
- **Vehicle Library** (`library`): shows the names of the main textures and the accessory count.
- **Wizard** (`dialogs.rs`): the Vehicle step uses `texture_checkboxes`, and the Name step lists the chosen textures with their sizes.
- **i18n:** `variants-*` and `pkg-*-variant` messages are replaced with `textures-*`, `main-textures`, `accessories` and the new `pkg-*` messages, in en, fr, es and de.

## Risks / Trade-offs

- **[Development files stop opening]** The user's own variant-era projects become unreadable. → TruckPaint is unreleased and the user chose this; the message names the reason.
- **[Default "every accessory checked" makes big projects]** A Scania R has 11 accessory groups. → The canvas only draws the active surface and its caches drop the rest (see fleet-projects). The player can uncheck groups at creation or later with Textures….
- **[`serde(flatten)` costs]** Error messages for a flattened struct are less precise, and unknown fields stay ignored, which is fine. → Validation reports the semantic errors itself. Serde errors only cover missing or mistyped fields, and the message still names the field.
- **[Order invariant broken by a bug]** The sidebar headings would repeat. → Unit tests on insertion (an accessory added between two others, a main texture added before the others, insertion next to a surface marked "Not in this version"). The sidebar doesn't assume more than "headings before the first of each part".
- **[Game id rules too strict for an exotic mod]** → The rule only checks characters, which the engine requires anyway, and the whole Paintjob Packer database passes it.

## Migration Plan

None: unreleased formats change in place. Old packages in a user's library no longer validate, and the library skips them (with a log warning) like any unreadable package. With nothing else installed, the library is empty again and offers Install the sample vehicles, which writes the new samples. The stale files stay in the library folder until removed by hand. They are harmless.

## Open Questions

- Whether `mod-export` needs the game data in the project file (see Non-Goals). It doesn't change this change's formats beyond adding fields later.
