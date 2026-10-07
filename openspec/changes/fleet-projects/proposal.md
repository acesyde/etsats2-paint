## Why

`docs/roadmap.md` records two decisions:
- **Every project has a vehicle.** TruckPaint designs paint jobs for ETS2 and ATS vehicles. A drawing with no vehicle has no texture names, no template and no place in a mod, so the blank-texture project leads nowhere.
- **A project is a fleet.** It holds several trucks with the variants the player chooses, and several trailers. It shares one identity and later exports as one mod.

Today a project holds at most one vehicle and one variant, and New Project still offers Blank texture. Every later step, `brand-kit` and `mod-export` among them, builds on the fleet model, so it comes first.

## What Changes

- **BREAKING (unreleased):**
  - **No blank-texture project:** New Project no longer offers Blank texture or a resolution choice.
  - **Files without a vehicle:** project files without a vehicle, written by development builds, are refused as a development format.
  - **Vehicle step:** a vehicle and at least one variant must be chosen. With no vehicle installed, the step offers Install… and Install the sample vehicle, and Next stays disabled.
- **Fleet projects:**
  - **Contents:** a project holds one or more vehicles (trucks and trailers). For each vehicle it holds one or more chosen variants, and each variant brings one surface per texture.
  - **One game:** a project belongs to one game, ETS2 or ATS, set by its first vehicle. Only vehicles of that game can be added.
  - **Each vehicle once:** a vehicle appears at most once in a project and records its own package version.
- **Variant picking:** New Project and **Vehicle › Add Vehicle…** both list vehicles with **checkboxes for the variants**. The vehicle's row in the Vehicles panel offers:
  - **Variants…:** add or remove variants. Removing one that has artwork asks for confirmation.
  - **Remove from Project:** with confirmation. The last vehicle can't be removed.

  These are all single undo steps.
- **Navigation:**
  - **Sidebar:** the Vehicle panel leaves the right column. A sidebar on the left of the canvas holds a read-only **Project** section (name; version and game versions empty until a later feature) and a **Vehicles** section: a tree, Vehicle › Variant › Texture, with + to add a vehicle and a ⋯ menu per vehicle. It collapses to a strip (View › Sidebar, F5) and remembers its state and width.
  - **No texture tabs:** the tree is the only place to switch textures, with Next/Previous Texture (Cmd/Ctrl+Page Down/Up) as the keyboard path.
  - **Template settings:** they move to the Properties panel when nothing is selected, together with the update flags and Dismiss.
  - **Status bar:** names the active texture as "Vehicle › Variant › Texture".
- **Update Template:**
  - It works **per vehicle**: the Vehicles panel shows the update notice on the vehicle concerned.
  - It covers every chosen variant of that vehicle. Textures of a variant missing from the new version are kept and marked "Not in this version".
- **View mode:** opening or creating a project shows its canvas. When the remembered view mode is 3D, which is still a placeholder, the project opens in 2D.
- **Export:** the proposed file name includes the vehicle and variant, "<project> - <vehicle> - <variant> - <texture>", so two cabins can't collide.
- **Templates travel with the project:** variant names are stored too, so the panel reads the same without the package installed.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `vehicle-projects`:
  - a project from one or more vehicles and variants;
  - adding and removing vehicles and variants;
  - one game per project;
  - the Vehicles panel tree;
  - tabs for the active variant;
  - Update Template per vehicle;
  - templates and names traveling with the project.
- `start-screen`: the New Project Vehicle step requires a vehicle with variant checkboxes, and the second step has a name only.
- `document-model`: every surface belongs to a vehicle's variant texture. There is no blank surface or "Main texture".
- `project-files`: files store the list of vehicles with their variants, and each template's vehicle and variant. Files without a vehicle are refused.
- `texture-export`: the proposed file name includes the vehicle and variant.
- `workspace-layout`: the sidebar on the left with its Project and Vehicles sections (new requirement), the Vehicle panel leaving the right column, and projects opening in a view that shows the canvas.
- `properties-panel`: with nothing selected, the panel also shows the active texture's template settings.

## Impact

- **tp-core (`project.rs`):**
  - `Project.vehicle: Option<VehicleRef>` becomes `vehicles: Vec<ProjectVehicle>`, each with its chosen variants (id and name);
  - `SurfaceTemplate` gains `package_id` and `variant_id`, so a surface knows which vehicle texture it paints;
  - snapshots and restore follow, with template settings matched by the full key.
  - `Project::new` remains as a single-surface constructor for unit tests only.
- **tp-file (`v1.rs`):** the v1 format, still unreleased, changes in place:
  - it stores `vehicles` and the template keys;
  - files written by development builds with a single `vehicle` are converted when opened;
  - files without a vehicle are refused.
- **tp-app:**
  - `vehicle_project.rs`: building from several variants, add and remove a vehicle or variant, update plan and apply per vehicle;
  - `vehicles.rs`: `update_for` per project vehicle;
  - `ui/dialogs.rs`: the wizard without blank, with variant checkboxes;
  - a new Add Vehicle and Variants dialog in `ui/vehicle_dialogs.rs`;
  - the tree in `ui/workspace/panels/vehicle.rs`;
  - tabs filtered to the active variant, the status bar breadcrumb and export naming;
  - new commands `AddVehicle`;
  - messages in the four locales;
  - viewports reset when the surface list changes shape.
- **Tests:**
  - `tests/common::create_project` creates a project from the built-in sample vehicle (Standard cab), so every UI test runs on a vehicle project;
  - the blank-project tests (resolution choice) are removed or rewritten;
  - the tp-file fixture is regenerated with two vehicles.
- **Docs:** the roadmap marks `fleet-projects` as shipped. `custom-vehicle` (Custom vehicle…) becomes the next change.
- **Unchanged:** the vehicle package format, the rendering and the caches. Only the active surface is drawn and caches drop what isn't used, so no on-demand loading is needed.
