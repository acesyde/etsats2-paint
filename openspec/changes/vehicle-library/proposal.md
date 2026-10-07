## Why

TruckPaint can draw a livery but has no idea what it is painting on. A livery only works when the artwork lines up with the vehicle's texture layout (its UV template), and a truck has several textures (cabin, chassis, accessories). Painters need to pick a truck or trailer, see its template under their work, and keep projects valid when SCS updates the games.

Templates can't ship inside TruckPaint (SCS copyright, and there are many community vehicles), so vehicles come as **packages**: one per truck or trailer, versioned, declaring which game versions they support. The same package format will feed a community marketplace on GitHub (next change) and the mod export (after it).

## What Changes

- **Vehicle packages (`.tpv`).** A zip with a `vehicle.json` manifest and template images, describing exactly one vehicle:
  - an id (`scs.volvo.fh16_2012`, `community.<author>.<name>`), a semver version, a kind (truck or trailer) and a game (ETS2 or ATS);
  - a brand, a display name, authors and a license;
  - supported game versions as a range (`>=1.50, <1.54`);
  - optional requirements, such as a community mod the vehicle comes from;
  - one or more variants (cabins, chassis), each listing its textures. A texture has an id, a name, a square size, a template image (PNG or SVG) and a layout version;
  - an optional free-form `export` section reserved for the mod export.

  Packages are data only (JSON and images), never code. They are validated on install, with size limits, safe paths and readable images.
- **Vehicle library.** Install a package from a file (also by drag and drop onto the home screen or workspace), list the installed vehicles, and remove them. Several versions of the same vehicle can be installed side by side, and the newest is used for new projects. Packages are stored in the application data folder.
- **New Project from a vehicle.**
  - The New Project wizard gains a first **Vehicle** step: search and filter the library by game, kind and brand, then pick a vehicle and its variant, or choose **Blank texture** (today's behavior).
  - A vehicle project gets one surface per texture of the variant, each with the texture's size and name, and is named after the vehicle by default.
- **Textures and templates in the editor.**
  - With several textures, tabs above the canvas switch the active texture.
  - Each surface shows its template as a locked overlay that can be shown or hidden (View › Show Template, Shift+T) and given an opacity. The overlay is never part of the artwork and never exported.
- **Vehicle panel and menu.**
  - The Vehicle panel (an empty state today) shows the vehicle, game and package version, the supported game versions and the textures. It also holds the template controls and, when a newer version of the package is installed, an **Update Template** action.
  - The Vehicle menu gets **Vehicle Library…**, **Update Template…** and **Show Template**. Mirror to Other Side stays disabled, since it needs mirror data that packages don't have yet.
- **Versioned templates in projects.**
  - Projects embed the templates they use, so they stay self-contained, and record the package id, version and variant.
  - **Update Template** switches the project to a newer installed version. It replaces the templates, keeps the artwork, adds surfaces for new textures, keeps surfaces whose texture was removed (marked as such), and flags the textures whose layout version changed until the user dismisses them. It is one undo step.
- **Project file format reset to v1.** TruckPaint has never been released, so no user files exist. The frozen v1/v2/v3 modules and their migrations are removed, and format **1** becomes today's content plus the vehicle reference and the per-surface template, layout version and status. The versioning and migration mechanism stays for future releases. Files written by development builds (formats 2 and 3) are refused with a clear message.
- **Export:** Export Texture uses the active surface's own size and never includes the template.

Not in this change:
- the GitHub marketplace: browsing, downloading, checksums and update notifications (`vehicle-marketplace`);
- building game mods (`mod-export`);
- Mirror to Other Side, 3D preview models, and localized vehicle names;
- choosing which game version a project targets.

## Capabilities

### New Capabilities
- `vehicle-packages`: the package format, its validation, the local library (install, list, remove, coexisting versions) and the Vehicle Library dialog.
- `vehicle-projects`:
  - creating a project from a vehicle;
  - one surface per texture, with tabs to switch;
  - the template overlay;
  - the Vehicle panel and menu;
  - recording the package in the project;
  - Update Template.

### Modified Capabilities
- `start-screen`: the New Project dialog starts with the Vehicle step. The resolution choice only applies to blank-texture projects.
- `document-model`: a surface's size comes from its texture (the project resolution for blank projects). A surface may have a template that is not part of the artwork.
- `project-files`: files store the vehicle reference and each surface's template. The format restarts at version 1, and files from unreleased development formats are refused.
- `texture-export`: the output size follows the active surface's size, and the template is never exported.

## Impact

- **New crate `tp-vehicles`:** manifest types (serde JSON), validation, semver versions and game-version ranges (`semver`), and package reading from bytes (`zip`). It has no filesystem access.
- **tp-core:**
  - `Project.vehicle: Option<VehicleRef>`;
  - `Surface.template: Option<Template>` (asset, opacity, visibility, layout version, status);
  - snapshots cover the whole surface list and the vehicle reference, so Update Template can be undone.
- **tp-file:** one module `v1` (today's v3 structures plus vehicle and template), `FORMAT_VERSION = 1`, the `v1`/`v2`/`v3` modules, migrations and old fixtures removed, and a new v1 fixture with a vehicle project.
- **tp-render:** draws artwork only, so templates are never rendered. Export size comes from the surface.
- **tp-app:**
  - `vehicles.rs`: the library on disk, under `AppDirs.data/vehicles`;
  - the wizard's Vehicle step;
  - surface tabs;
  - the template overlay drawn on the canvas;
  - the Vehicle panel;
  - the Vehicle Library dialog;
  - commands and drag and drop of `.tpv` files;
  - translations of all new texts in en/fr/es/de.
- **Docs:** `docs/vehicle-package-format.md`, describing the manifest for package authors.
- **Dependencies:** `semver` and `serde_json` become direct dependencies; `zip` and `image` are already used.
