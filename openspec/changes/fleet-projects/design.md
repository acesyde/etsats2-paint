## Context

**tp-core:**
- `Project` holds `vehicle: Option<VehicleRef>`, which records one variant through `variant_id`. Its `surfaces: Vec<Surface>` list is flat.
- Each `Surface` has an optional `SurfaceTemplate { texture_id, asset, layout_version, opacity, visible, status }`.
- `Project::new(name, TextureResolution)` creates the blank project: one "Main texture" surface. It is also used by about 15 unit tests in tp-app and by tp-render.
- `Snapshot` captures every surface's metadata (name, size, template), objects, guides, the vehicle and the assets. `restore` keeps template opacity and visibility, matched by `texture_id`.

**tp-app:**
- **`vehicle_project.rs`:** `vehicle_project`, `plan`, `apply_update`, `dismiss_layout_change`, all for the single vehicle and variant.
- **`Workspace`:** keeps `viewports: Vec<Option<Viewport>>`, indexed by surface index. `set_active_surface` and `swap_view` exchange them.
- **Panel and tabs:** the Vehicle panel (`panels/vehicle.rs`) shows the one vehicle and its textures. The texture tabs (`canvas_with_tabs`) list every surface.
- **`state.rs`:** `open_update_dialog` uses `project.vehicle`, and `VehicleLibrary::update_for` requires the project's variant.
- **New Project wizard (`ui/dialogs.rs`):** `NewProjectDraft.vehicle: Option<(id, version, variant)>`. `None` means Blank texture, which leads to `WizardOutcome::Create(Project)` with a resolution choice.

**Rendering:** only the active surface is drawn. Image, gradient and geometry caches evict what isn't used each frame. A project with many surfaces costs memory only for its document (`Arc`-shared objects and compressed asset bytes), so no on-demand loading is needed.

**Tests:** `tests/common::create_project` drives New Project → Next → Create on Blank texture, and 17 UI test files use it. `AppState::with_prefs` builds an empty `VehicleLibrary::default()` with no folder, so it can't install anything.

**tp-file:** `v1.rs` is unreleased. It stores `resolution`, `vehicle: Option<FileVehicle { …, variant_id }>` and `FileTemplate { texture_id, … }`.

## Goals / Non-Goals

**Goals:**
- **Fleet model:** a document model where every surface is keyed by (vehicle, variant, texture), and a project holds several vehicles with chosen variants, all of one game.
- **Single undo steps:** adding or removing vehicles and variants, and updating one vehicle, are each one undo step, built on the existing snapshot mechanism.
- **Navigation:** a tree in the Vehicles panel, tabs scoped to the active variant, and a breadcrumb in the status bar.
- **Vehicle-only tests:** every UI test runs on a vehicle project, and the blank project is gone from the app.

**Non-Goals:**
- **Custom vehicle…:** the escape hatch for vehicles without a package is the next change, `custom-vehicle`.
- **Shared textures:** textures shared between variants come with `shared-textures`. The surface key leaves room for them (D1).
- **Brand kit:** palette, symbols, copy from variant (`brand-kit`).
- **Mod export:** `mod-export`.
- **Stable surface ids** and per-surface views kept across structural changes (see D3).
- **Removing `TextureResolution` / `Project::new` from tp-core:** it stays as a test constructor. Removing it would touch about 40 test call sites for no user-visible gain.

## Decisions

### D1. The surface key lives on the template
`SurfaceTemplate` gains `package_id` and `variant_id` next to `texture_id`, and `key() -> TextureKey { package_id, variant_id, texture_id }`.

`Project.vehicle` becomes `vehicles: Vec<ProjectVehicle>`:
- `ProjectVehicle { package_id, version, name, brand, kind, game, variants: Vec<VariantRef { id, name }> }`, which is `VehicleRef` renamed and extended;
- the variant names are stored so the panel reads the same without the package (spec: Templates travel with the project).

**Why on the template:** every surface of a project has one, which is the new document-model invariant. A texture that left the package keeps its template record with status `Removed`, which `apply_update` already does today, so the key is never lost. Moving the key to a separate `Surface.slot` would be cleaner, but would rewrite `plan`, `apply_update`, restore and the file format for no behavior change.

**tp-core helpers:**
- `Project::game()`, which takes the first vehicle's game;
- `vehicle(&id)` and `vehicle_of(surface_index)`;
- `variant_range(package, variant) -> Range<usize>`;
- `variant_names(key)`, which feeds the breadcrumb and export names.

**Later, for `shared-textures`:** a template could carry a list of variant ids, or `variant_id` could become optional for a shared texture. The key type is where that happens.

### D2. Surface order
Surfaces are grouped by vehicle in project order. Within a vehicle, they're grouped by variant in the order the variants are chosen, and within a variant they follow the package's texture order.

- **Add Vehicle:** appends at the end.
- **Add variant:** inserts after that vehicle's last surface.
- **New texture from an update:** inserts after its variant's last surface.

Tabs, the tree and export all rely on these contiguous ranges (`variant_range`).

### D3. Undo, restore and views
`Snapshot.vehicle` becomes `vehicles`. `restore` matches the kept opacity and visibility by the full `TextureKey`, since "cabin" now exists in several variants.

**Structural changes:** adding or removing vehicles or variants, or an update that adds textures, changes the surface list. The index-based viewports would then point at the wrong surfaces. After any edit or undo/redo, `Workspace` compares the surface keys before and after. When they differ, it clears `viewports` and fits the active surface (spec: views are kept "for as long as the list of surfaces doesn't change").

**Alternative considered:** stable surface ids with a viewport map. It's the right long-term shape, but it touches every place that indexes surfaces. It was deferred because fitting again after a rare structural edit is acceptable.

### D4. Operations (`vehicle_project.rs`, `Workspace`)
**Building and editing:**
- **`fleet_project(name, &Package, &[variant_id]) -> Option<Project>`:** replaces `vehicle_project`, which stays as a one-variant wrapper used by tests.
- **`Workspace::add_vehicle(&Package, &[variant_id], now) -> bool`:**
  - refused when the game differs, when the vehicle is already in the project, or when no variant is given;
  - undo label `undo-add-vehicle`.
- **`Workspace::set_variants(package_id, &Package, &[variant_id], now)`:**
  - adds the missing variants' surfaces and removes the unchecked ones;
  - undo label `undo-change-variants`;
  - needs the package at the project's recorded version.
- **`Workspace::remove_vehicle(package_id, now)`:**
  - refused for the last vehicle;
  - undo label `undo-remove-vehicle`.

**Helpers:**
- `Project::has_artwork(range)`: whether any surface in the range has objects, which drives the confirmation dialogs;
- after removing surfaces, template assets with `asset_usage == 0` are dropped (`remove_asset`). Snapshots carry the assets, so undo restores them.

**Active surface:**
- after adding a vehicle, the first new surface becomes active;
- after removing the active surface's vehicle or variant, the first surface of the project becomes active.

### D5. Update Template per vehicle
**Planning:** `plan(project, package)` keeps its signature. It finds the vehicle with `package.manifest.id` and plans every surface of that vehicle:
- **variant and texture found:** `Replaced { layout_changed, resized }`;
- **texture or whole variant missing from the new version:** `Removed`;
- **package texture of a chosen variant not in the project:** `Added`, inserted after the variant's range.

When the vehicle has several variants, `TextureChange` names include the variant ("High roof › Cabin").

**Applying:** `apply_update` does the same per vehicle and records the new version on that `ProjectVehicle` only.

**Finding the update:** `VehicleLibrary::update_for(&ProjectVehicle)` returns the newest installed version newer than the recorded one. It no longer requires the variant: a missing variant is reported as Removed in the plan.

**Opening the dialog:** `AppState::open_update_dialog(package_id: Option<&str>)`. `None`, used by the Vehicle › Update Template… command, means the vehicle of the active surface. The panel notice passes its vehicle. The `update_available` command context checks the active surface's vehicle.

### D6. Vehicles panel
`PanelKind::Vehicle` keeps its persisted name. Only its title message changes, to `panel-vehicles` "Vehicles", so saved layouts still load.

**The panel:**
- **Header:** "ETS2 fleet" / "ATS fleet" (`vehicles-fleet-game`).
- **One collapsing section per vehicle:**
  - name, brand · kind, and package version with game versions;
  - the update notice;
  - **Variants…** and **Remove from Project** as two small buttons (a `⋯` menu was considered, but plain buttons are easier to find and to test), the latter disabled with a reason on the last vehicle.
- **Under each vehicle:** one collapsing section per variant, holding selectable texture rows ("Cabin · 4096"). The active row is highlighted, and the badges and Dismiss stay as today.
- **After the tree:**
  - **Add Vehicle…** (command `AddVehicle`);
  - the active template's opacity slider and visibility toggle.
- **Defaults:** sections open on first show; the active vehicle and variant are always open.

The dialogs go through `state.modal`, like the library and update dialogs:
- **`Modal::AddVehicle(AddVehicleDialog { filter, choice: Option<(id, version, Vec<variant>)> })`:**
  - the filter is the existing `VehicleFilter`, with its game locked to the project's game and vehicles already in the project hidden;
  - variants appear as checkboxes;
  - Add stays disabled until a vehicle and a variant are chosen.
- **`Modal::Variants { package_id, checked: Vec<String>, confirm: bool }`:**
  - the checkboxes come from the installed package at the recorded version;
  - when that version is missing, the dialog explains it and offers Update Template… instead;
  - unchecking variants that hold artwork switches to an inline confirmation naming them, as with Remove in the library.
- **`Modal::RemoveVehicle { package_id }`:** a confirmation when the vehicle holds artwork, otherwise immediate.

### D7. Tabs and status bar
**Tabs:** `canvas_with_tabs` shows only `variant_range(active)`. The segment indices map back to project indices, and the tabs are hidden when that range holds one surface.

**Status bar:** shows `vehicle › variant › texture` from `variant_names`, truncated with an ellipsis when too wide, with the full text on hover.

### D8. New Project wizard
**Draft:** `NewProjectDraft.vehicle` becomes `Option<(id, Version, Vec<variant_id>)>`.
- **Choosing a vehicle** checks its first variant.
- **Variants:** a checkbox per variant, shown even when there's only one, so the choice is explicit.
- **No blank or resolution:**
  - the Blank texture row, the resolution cards, `TextureResolution` in the draft and `WizardOutcome::Create(Project)` are removed;
  - `WizardOutcome::CreateVehicle` carries the list of variants.
- **Next:** disabled until a vehicle with at least one variant is chosen. Enter does nothing until then.
- **No vehicle installed:** the existing hint and Install the sample vehicle button stay, and the sample install selects it with its first variant.
- **Name step:**
  - `new-project-step-name` becomes "Name" in all locales;
  - the step lists each chosen variant's textures under the variant's name.

### D9. Export name
`export_dialog` proposes `"{project} - {vehicle} - {variant} - {texture}"`. Vehicle and variant names come from packages and may contain characters that file systems refuse, such as "Scania R/S". The app has no file name sanitizing yet, so a small `file_stem_safe` helper replaces `/ \ : * ? " < > |` and control characters with `-`, trims spaces and dots, and is unit-tested.

### D10. File format: v1 changed in place
v1 hasn't been released, as with the earlier reset.

**Writing:**
- `FileProject` writes `vehicles: Vec<FileVehicle>`, where `FileVehicle` has `variants: Vec<FileVariant { id, name }>`;
- `FileTemplate` writes `package_id` and `variant_id`;
- `resolution` is still written (the largest texture side) and ignored on read.

**Reading development-build files:**
- **The old single `vehicle` field:** it's kept as `#[serde(default)] vehicle: Option<FileVehicle>`, with an optional legacy `variant_id`. It's converted to one vehicle whose single variant has id `variant_id` and, since its name wasn't stored, `variant_id` as its name.
- **Templates without keys** get the legacy vehicle's package and variant.
- **No vehicle at all** (blank-texture files): `Error::Unsupported { found: 1 }`, which shows the existing "development format" message.

**Validation on load:** each surface must have a template whose key names a project vehicle and one of its variants, and all vehicles must share one game. Otherwise `Error::Damaged`.

**Fixture:** `tests/fixtures/v1.truckpaint` is regenerated with two vehicles, one of them with two variants. A legacy single-vehicle file is kept as a second fixture to test the conversion.

### D11. Tests on vehicle projects
**Shared helper:** `tests/common`:
- `sample_library() -> VehicleLibrary` opens a library in a temporary folder that is kept alive for the rest of the test process (`TempDir::keep`), with the sample installed;
- `harness_with` installs it in the `AppState`;
- `create_project` clicks New Project, "TruckPaint Sample Truck" (Standard cab checked), Next and Create. The active surface is then the 4096 px Cabin, the size the blank default had, so most canvas tests keep their coordinates.

**Unit tests** keep `Project::new` with no vehicles. They test editing behavior, not the fleet invariant, and the invariant is enforced by the wizard, the file loader and the vehicle operations, not by `tp-core` constructors.

**Blank-specific tests** are rewritten or removed:
- `ui.rs` resolution tests;
- the "blank project" kittest in `vehicles.rs`;
- the "For a blank-texture project…" panel state.

**New tests:**
- `tests/vehicles.rs` kittests:
  - two variants at creation;
  - Add Vehicle;
  - variants added and removed, with confirmation and undo;
  - removing a vehicle, and the last vehicle disabled;
  - the tree switching surfaces and the tabs scoped to the variant;
  - the status bar breadcrumb;
  - update of one vehicle only;
  - the game filter;
  - Next disabled without a vehicle.
- **Unit tests in `vehicle_project.rs`:** surface ordering, asset cleanup and undo, and the plan with a missing variant.
- **tp-file:** round trip, legacy conversion, blank refused, mixed games refused.

## Risks / Trade-offs

- **Some UI test churn:** 17 test files move to the sample vehicle, so the canvas starts at the 4096 Cabin with a visible template. → The template is excluded from hit testing and the eyedropper, so few assertions should change. The helper can hide templates if a pixel test needs it.
- **Views are lost on structural edits.** → They're rare and explicit, and fitting is predictable. Stable surface ids can be added later without a spec change.
- **Projects with dozens of surfaces** make the tree long. → Sections can be collapsed, and the active vehicle and variant stay open. Scrolling the panel column is enough at fleet sizes of 30 to 40 textures.
- **Development files with one vehicle** get variant names equal to their ids. → They're cosmetic and corrected by a later update. Only the user's own test files are affected.
- **Adding a variant needs the recorded version installed.** → The dialog says so and offers Update Template…, and projects never silently mix versions within one vehicle.

## Migration Plan

There's no released version, so no migration is needed. Development-build files with one vehicle are converted when opened, and blank-texture development files are refused with the development-format message. Rolling back means reverting the change, since the file format change only adds fields.
