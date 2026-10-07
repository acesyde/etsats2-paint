## Context

**Surfaces today:**
- `Project` (tp-core) holds `surfaces: Vec<Surface { name, size, objects, guides }>`, an `active_surface` index and `assets: BTreeMap<AssetId, Arc<Asset { kind: Raster | Svg, bytes, size }>>`.
- Every project has exactly one surface, and nothing in the UI switches surfaces. Only export reads `active_surface`.
- `Snapshot` captures objects and guides per surface, the active surface, the palette, assets and the selection. It assumes the surface list never changes.

**Reserved hooks:**
- The New Project wizard is already a step sequence (`NEW_PROJECT_STEPS`, `StepIndicator`).
- The Vehicle menu has `ChooseVehicle`, `VehicleInfo` and `MirrorToOtherSide` as `NotYet` commands.
- The Vehicle panel shows an empty state.
- `AppDirs.data` holds logs and recovery copies.

**Building blocks already there:**
- `ImageCache` already draws raster and SVG assets on the canvas: rasters capped at 2048 px, SVGs re-rendered per zoom.
- `tp-file` is at format v3, with frozen modules and a migration chain.
- `zip` and `image` are workspace dependencies.
- All UI text goes through `tp-i18n` (en/fr/es/de).

**User decisions:**
- templates come from imported packages, one package per truck or trailer;
- versioned packages with game-version ranges;
- one surface per texture;
- a locked overlay that is never exported;
- templates embedded in projects, with the package id and version recorded;
- Update Template is offered, not automatic;
- the GitHub marketplace is a separate, later change.

## Goals / Non-Goals

**Goals:**
- A small, documented, forward-compatible package format that community authors can produce by hand, and that the marketplace and mod export can extend.
- Vehicle projects that stay self-contained and can move to a newer package version safely, as one undoable step.
- Multi-texture projects in the editor: tabs, a per-surface view, and undo across surfaces.

**Non-Goals:**
- The marketplace: registry, download, checksums, update notifications.
- Mod export, and choosing the target game version.
- Mirror data and Mirror to Other Side.
- 3D models, localized vehicle names in packages, and package signing.

## Decisions

### D1. Crate `tp-vehicles`: format, validation, reading from bytes
New crate, with no filesystem access (the crate rule from app-shell).
- **Types:** `Manifest` (serde, `#[serde(default)]` on optional fields, unknown fields ignored), `Kind { Truck, Trailer }`, `Game { Ets2, Ats }`, `Variant`, `Texture { id, name, size, template, layout_version, export: Option<serde_json::Value> }`, `Requirement`.
- **Versions:** `version: semver::Version` and `game_versions: semver::VersionReq`. Game versions like "1.53" are compared as `1.53.0`.
- **Reading:** `Package::read(bytes: &[u8]) -> Result<Package, PackageError>` opens the zip and parses `vehicle.json`. It validates in a fixed order, so the first problem is reported deterministically:
  - format version, id pattern, semver, and the version range;
  - variants and textures: unique texture ids per variant, allowed sizes;
  - every zip entry name is relative with no `..` (checked on all entries, even unused ones);
  - total uncompressed size ≤ 512 MB, checked from the central directory before reading anything;
  - each template exists, is a PNG (decoded header for dimensions ≤ 16384) or an SVG (parsed with usvg).
- **Manifest-only reading:** `Package::read_manifest(bytes)` parses only the manifest, which is how the library is listed quickly.
- **Errors:** `PackageError` is an enum carrying the offending field or texture. tp-app turns it into translated messages, the same pattern as `OperandProblem`.
- **Templates:** returned as `Template { bytes, kind: Raster | Svg, size }` per (variant, texture).

### D2. The library on disk (tp-app `vehicles.rs`)
- **Location:** packages are stored unchanged as `<data>/vehicles/<id>/<version>.tpv`.
- **Install:** read the file, run `Package::read` (full validation), then `tp_file::write_atomic` to the target. The same version is replaced, and other versions are untouched.
- **Index:** `VehicleLibrary` is built at startup and after each change by listing the directory and calling `read_manifest`. It returns, per id, the installed versions sorted, the newest first. Unreadable files are skipped and logged.
- **Remove:** deletes `<version>.tpv`, and the folder when it becomes empty.
- **Threading:** installing and reading templates for a new project run on the UI thread. Packages are a few MB, and validating a 4096 PNG takes tens of milliseconds. A background thread can come with the marketplace.
- **Tests:** the library takes a root path, so tests use temporary folders.

### D3. Document model (tp-core)
- **Vehicle reference:** `Project.vehicle: Option<VehicleRef { package_id, version: String, variant_id, name, brand, kind, game }>`. Plain strings keep tp-core free of `semver`.
- **Templates:** `Surface.template: Option<SurfaceTemplate { texture_id, asset: AssetId, layout_version: u32, opacity: f32, visible: bool, status: TemplateStatus { Current, LayoutChanged, Removed } }>`.
- **Template images are assets**, so they are stored, saved and cached like logos. `Project::template_assets()` lists the asset ids that templates use. The Assets panel and `asset_usage` leave those out, so templates aren't listed or placeable as images.
- **Snapshots:**
  - `Snapshot` now captures each surface's `SurfaceMeta` (name, size, template) and the vehicle reference, besides objects and guides.
  - `restore` rebuilds the surface list from the snapshot, which allows adding surfaces during an update.
  - Opacity and visibility are taken from the current surfaces with the same `texture_id`, so undo never reverts them, as the spec requires.
- **Unsaved state:** changing opacity or visibility marks the project as having unsaved changes through a `settings_changed` flag on the workspace, without a history entry.

### D4. Creating a vehicle project
`vehicle_project(name, &Package, variant_id) -> Project` in tp-app:
- one `Surface::new(texture.name, texture.size)` per texture, in order;
- each template added as an asset and set as a `SurfaceTemplate` with opacity 0.6, visible, status Current;
- `project.vehicle` filled in, and `active_surface = 0`.

The project's `resolution` is set to the largest texture size, used for new-project defaults only.

### D5. Surface tabs and views
- **Tabs:** a tab bar above the canvas (tp-ui `SegmentedControl`-style tabs) shows the surface names when there are 2 or more.
- **Switching:** `Workspace::set_active_surface(i)` clears the selection, the points and the text session. It swaps in the surface's own viewport from `viewports: Vec<Option<Viewport>>`, and a surface not shown yet fits to screen on first display.
- **Undo across surfaces:** undo and redo already restore `active_surface` from the snapshot. The workspace swaps viewports when the restored index differs, so the undone change is in view.

### D6. Template overlay on the canvas
- **Drawing:** `paint.rs` draws the active surface's template right after the objects, before the grid, guides and selection.
  - The texture comes from `ImageCache::texture(asset, screen_px)`: SVG templates stay sharp, and rasters are capped at 2048 for display.
  - It is drawn as a textured quad over the artboard, tinted with `Color32::from_white_alpha(opacity)`.
- **Not interactive:** hit testing, the eyedropper (`tree::sample`) and snapping only consider objects, so the template is automatically ignored.
- **Not exported:** `tp-render` draws `draw_list(objects)` only, so the template is never exported. A test covers this.
- **Show Template:** a new command `ShowTemplate` (View menu, Shift+T, enabled when the active surface has a template) toggles visibility.

### D7. Update Template
- **Plan:** `UpdatePlan::compute(project, &Package) -> UpdatePlan` lists, per texture: `Replaced { layout_changed, resized: Option<(old, new)> }`, `Added` and `Removed`. The confirmation dialog shows this plan in plain words.
- **Apply:** one `ws.edit("undo-update-template", …)`.
  - Templates are replaced: new assets are added, and old template assets no longer used are removed.
  - On a size change, the surface's objects are scaled with `transform::resize` from the top-left corner (the factor is new size over old size), and guides are scaled the same way.
  - New textures become new surfaces. Removed textures get `status = Removed`, `template = None` for drawing, and their artwork is kept.
  - `vehicle.version` is updated.
- **Availability:** "an update is available" is computed per frame from the library index: the newest installed version is greater than `vehicle.version` and has `variant_id`. The command and the panel notice use it.
- **Dismiss:** Dismiss on a "Layout changed" badge sets the status to Current, as an undoable step `undo-dismiss-layout`.

### D8. File format reset to v1
TruckPaint has never been released, so no user file needs migrating.
- **One module:** the current `v3` module becomes the only module, `v1`, extended with:
  - `FileProject.vehicle: Option<FileVehicle>`;
  - `FileSurface.template: Option<FileTemplate { texture_id, asset, layout_version, opacity, visible, status }>`.
- **Removed:** the `v1`, `v2` and `v3` modules, `v3::from_v2`, `v2::from_v1`, the old fixtures and their migration tests.
- **Kept:** the dispatch in `lib::migrate`, which now handles only format 1, so the next format bump adds `v2` + `from_v1` as before. The "frozen once released" rule now applies from the first release.
- **Version:** `FORMAT_VERSION = 1`. A file with format 2 or 3, written by a development build, would be read as "created with a newer version". A dedicated `Error::Unsupported` reports it as an unreleased development format instead.
- **Assets:** template assets are ordinary entries in `assets/`.
- **Fixture:** `tests/fixtures/v1.truckpaint` is regenerated with a three-texture vehicle project, alongside the existing rich content.

### D9. Commands, wizard, panel, dialog
- **Commands:**
  - `ChooseVehicle` becomes `VehicleLibrary` ("Vehicle Library…", App scope, always enabled).
  - `VehicleInfo` reveals the Vehicle panel (needs a project).
  - New `UpdateTemplate` is enabled when an update is available, with a translated reason otherwise.
  - New `ShowTemplate` (View menu, Shift+T).
  - `MirrorToOtherSide` stays `NotYet`.
- **Wizard:** `NewProjectDraft` gains `vehicle: Option<(package_id, version, variant_id)>`, a search query and filters. `NEW_PROJECT_STEPS = ["new-project-step-vehicle", "new-project-step-name"]`, with Next and Back buttons. The Vehicle step is a scrollable list with a "Blank texture" row first, then the vehicles, each expanding into variants when it has several.
- **Vehicle Library dialog:** a modal with filters, a list, Install…, and Remove with confirmation.
- **Vehicle panel:** replaces the empty state as described in the spec.
- **Drag and drop:** dropped files ending in `.tpv` are installed. Other dropped files keep their current handling (image import).
- **Messages:** installing shows a hint "Installed Volvo FH16 2012 1.3.0", and failures open a message modal with the translated `PackageError`.
- **Translations:** every new text gets a message id in en/fr/es/de.

### D10. Export
- `ExportSettings::size` uses `project.surface().size` instead of the project resolution.
- For vehicle projects, the suggested name is "<project> - <texture>.<ext>".
- Mod export, later, will export every surface.

### D11. Documentation and sample package
- **Format docs:** `docs/vehicle-package-format.md` documents the manifest with a full example, the size limits, and the rule that `layout_version` must increase when a layout changes.
- **Test fixture:** a test helper writes a sample package (`sample_package(version, layouts)`) with small generated PNGs. It is used by tp-vehicles, tp-app and kittests, so no game asset is committed.

### D12. Notes from implementation
- **Template assets and `asset_usage`:** `Project::asset_usage` *counts* template references rather than leaving them out, so a template's image can never be removed as "unused". The Assets panel is what leaves template assets out.
- **Wizard outcomes:** the New Project wizard returns `CreateVehicle` and `Install` outcomes, which `show_modal` handles because they need the library and file dialogs. Install results are shown inside the wizard and the Vehicle Library dialog.
- **Dropped packages:** these are installed from the dropped bytes. Results appear as a workspace hint or in the open library dialog, and failures open a message.
- **Snapshot restore:** restoring a snapshot on another surface swaps the views through `Workspace::swap_view`, so undo always shows the change it reverts.
- **Display only:** texture names come from packages and are not translated. The "Layout changed" flag also shows as a warning icon on the texture tab.

## Risks / Trade-offs

- **Project size:** embedding templates grows project files by the PNG size, typically 1–5 MB per texture. This is the price of self-contained projects.
- **Display quality:** an 8192 px raster template is downscaled to 2048 for display. It's fine as a guide, and SVG templates stay sharp.
- **No signatures:** packages are unsigned. That's acceptable for local files, and the marketplace adds checksums.
- **Layout changes:** when a layout really moves, artwork does not move with it, since there's no UV remapping. The "Layout changed" flag tells the user to check. Automatic remapping would need per-texture mapping data, which the format can grow later through unknown fields.
- **Snapshot cost:** snapshots now hold surface metadata, which is a few small structs, so the cost is negligible.
- **Development files:** project files saved by development builds (formats 2 and 3) no longer open. That is acceptable before the first release; they can be recreated.
- **Startup time:** the library index reads every package's manifest at startup. That's fast for tens of packages, and can be cached when the marketplace brings hundreds.
