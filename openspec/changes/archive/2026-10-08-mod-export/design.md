## Context

See proposal.md for the motivation. The pieces this change builds on:

- **Rendering and encoding:**
  - `tp_render::render(project, surface_index, RenderOptions, fonts, progress)` already renders any surface, not only the active one, without the template.
  - `encode_dds` writes BC3 with a full mip chain.
  - `tp_file::write_atomic` writes beside the target, then renames.
  - `ExportJob` (`tp-app/src/export.rs`) runs one surface on a worker thread, with per-mille progress, an `AtomicBool` cancel and an mpsc outcome. It is the model for the mod job.
- **The project doesn't know the game data.**
  - `ProjectVehicle` holds id, version, name, brand, kind and game.
  - `SurfaceTemplate` holds `package_id`, `texture_id`, `part`, layout version and the template asset.
  - The game path, game ids, `alt_uv`, `colour_picker`, `versions` and `requires` are only in the installed package (`VehicleLibrary::load`).
- **The project file is format 1** and grows by optional fields (`#[serde(default, skip_serializing_if)]`), with no format bump: mirrored flag, symbols. Older builds ignore unknown fields.
- **Already in place:**
  - `CommandId::ExportMod` exists, disabled with `NotYet("reason-soon-mods")`, on Cmd+Shift+E, in the Export menu, and in `needs_texture()`.
  - The sidebar's Project section (`panels/vehicle.rs`) shows a hard-coded empty Version.
- **The file format is from Paintjob Packer** (MIT, `Carsmaniac/paintjob-packer`, `scripts/ModGeneration.gd`), which has shipped working ETS2 and ATS paint job mods for years:
  - `accessory_paint_job_data` units, one `_settings.sui` per vehicle and paint job, and `paint_job/accessory/<unit>.sii` with `simple_paint_job_data` overrides;
  - a binary `.tobj` that is a fixed header plus the DDS path;
  - a 256×64 DXT5 shop icon behind a `ui` material;
  - a `manifest.sii` with a 276×162 `icon.jpg`;
  - unit names limited to 12 characters, the SII token length. When a truck splits its paint job per cabin, the name is `<internal>_<letter>`, so the internal name gets at most 10.

## Goals / Non-Goals

**Goals:**
- One pure function from a project to the mod's file tree, so the structure is unit-tested without rendering or writing anything.
- One background job that renders, encodes, zips and writes atomically, with progress and cancel shared across every texture.
- Projects that export on any computer, with game data recorded at the point where the package is in hand.

**Non-Goals:**
- Changing `tp-vehicles` or the package format: every field the mod needs is already there.
- Reading SCS archives or validating game ids against the game's definitions. TruckPaint trusts the package.

## Decisions

### Game data lives in the project, recorded where the manifest is in hand
`ProjectVehicle` gets the game data: `game_path`, `game_versions` (a string, as in the manifest), `alt_uv`, `colour_picker` and `requires`. It is an `Option<GameData>`, so "missing" is explicit. `SurfaceTemplate` gets `game_ids: Vec<String>`.

`vehicle_project::project_vehicle(&Manifest)` fills the vehicle's data, and the places that build templates from a `Part` (`fleet_project`, add vehicle, `set_textures`, Update Template) fill `game_ids`. Update Template already snapshots the whole project for undo, so restoring the old data comes for free.

Older files are backfilled in `project_io::open_file` and `restore`, right after `tp_file::read`. For each vehicle without data, `VehicleLibrary::recorded(vehicle)` gives the installed version; its manifest fills the vehicle and the game ids of its templates, matched by `texture_id`. A file that got any data opens unsaved (`saved = !migrated && !filled`).

- *Alternative: read the installed package at export time only.* Rejected. A project copied to another computer, or whose package version was removed, couldn't be exported, although it holds everything else. The format is versioned so that a project stands alone.
- *Alternative: embed the whole manifest.* Rejected. Most of it is library metadata, and a second copy of `paint_job` would drift from the surfaces, which already say what is painted.

### Mod settings on `Project`, edited through one undo step
`Project.mod_settings: ModSettings { name, version, author, description, internal_name: Option<String>, price: u32, unlock_level: u32, icon: Option<AssetId>, image: Option<AssetId> }`.

- **Defaults:** `ModSettings::for_project(name)` gives the defaults of the spec (Name = project name, Version `1.0`, Price 5000).
- **Name and internal name:**
  - `name` is stored, not derived: the Name follows the project name only until the first export changes it, as with any project setting.
  - `internal_name` is `None` while it follows the Name. It is then derived when read, by a pure `derive_internal_name(name, max_len)` in tp-core, with the limit of the vehicles in the project at that time (10 or 12). Storing a derived name would go stale when a truck with several cabins is added.
- **Images:** chosen images are ordinary `Asset`s, so they are saved, zipped and deduplicated like imported images, and survive asset cleanup because they are referenced.
- **Editing in the dialog:** the dialog edits a copy. On Export… it compares it with the project's settings. If they differ, `Workspace::set_mod_settings(settings, now)` takes a snapshot, assigns, and `record`s `undo-edit-mod-settings`. Cancel drops the copy.

- *Alternative: undo step per field, live.* Rejected. The dialog is modal, and a dozen steps for one export is noise.
- *Alternative: prefs.* Rejected by the player (decision in the proposal): the settings belong to the fleet, and the mod version is the project version the sidebar was waiting for.

### `mod_export::plan` builds the file tree, `ModJob` produces it
`plan(&Project) -> Result<ModPlan, Vec<Problem>>` needs no fonts and no files.

- **Problems:** the spec's checks, each a `Problem` enum variant with what the dialog needs to name it. The dialog shows the same list.
- **The plan:**
  - `files: BTreeMap<String, Entry>`, sorted archive paths;
  - `Entry::Text(String)` for `.sii`, `.sui`, `.mat` and `.txt`;
  - `Entry::Bytes(Vec<u8>)` for `.tobj`;
  - `Entry::Texture { surface: usize, size: u32 }` for a texture;
  - `Entry::Icon` and `Entry::ModImage` for the two images.

  The plan is pure and unit-tested on the sample truck and trailer, with exact file lists and contents.

`ModJob::start(project_clone, plan, fonts, path, notify)` follows `ExportJob`:
- it walks the plan in path order;
- it renders each `Texture` with `tp_render::render` (transparent background), then `encode_dds(Bc3)`, adding it to an in-memory `zip::ZipWriter<Cursor<Vec<u8>>>`;
- it checks the cancel flag between textures and between mip levels;
- it calls `write_atomic` once at the end.

Progress weights each texture by its pixel count, since 4096² dominates. A texture is released once zipped, so the peak memory is one 8192² render plus the archive. The archive is about the DDS bytes: 4096² BC3 with mips is about 22 MB, so a 40-texture fleet is under 1 GB at worst.

Zip options:
- copied from `tp-pack::write_zip`: Deflate, fixed timestamp, `0o644`;
- DDS entries are Stored: BC3 barely deflates, and Stored saves seconds on a fleet;
- `zip` becomes a regular dependency of tp-app.

- *Alternative: stream to a temp file instead of memory.* Possible later if memory is a problem. `write_atomic` takes bytes today, and the export dialog does the same for one texture.
- *Alternative: a separate crate for the mod writer.* Not needed. The plan needs tp-core only, and the job needs tp-render. A `mod_export.rs` module next to `export.rs` keeps the pattern. Split it if it grows past one file.

### Unit names and paths
- **Unit names:**
  - `<id>` when the package has one main texture;
  - `<id>_<letter>` otherwise, the letter being the main texture's position in the package (`a`, `b`…).
  - Unpainted main textures aren't surfaces, so the surfaces alone can't give either fact. The game data therefore also records `GameData.main_count` (the package's number of main textures), and each main texture's template records `SurfaceTemplate.main_index` (its position in the package; `None` for accessories). `main_count > 1` also sets the 10-character limit.
- **Texture paths:** `vehicle/<type>/upgrade/paintjob/<id>/<game path>/<texture id>.dds`.
  - The paths stay ASCII: the game path, texture id and internal name are all `[a-z0-9_.-]`.
  - Paintjob Packer uses folders named after the paint job and vehicle names. Those need diacritics stripped and file name checks, which are avoided here.
- **`.tobj`:** the Paintjob Packer header bytes, then the path length and the absolute path (`/vehicle/...dds`). It lives in `mod_export::tobj(path)`, tested against a known-good file from Paintjob Packer's output.
- **SII strings:** the Name, Author and Version are written as typed, inside `"…"`. The checks forbid `"`, `\` and line breaks, so no escaping is needed. The Description goes to `description.txt`, never into SII.

### Images
- **Generated images:** `tp_render::render_cover(project, surface 0, w, h, background)` renders the surface at a size that covers W×H (for 4096² into 276×162, a 276 px render) and crops the center.
- **Chosen images:** decoded with `image`, then resized to cover with `FilterType::Lanczos3` and center-cropped.
- **Encoding:** the icon goes through `encode_dds(Bc3)`, and the Mod Manager image is encoded as JPEG at quality 90 (`image` already has the `jpeg` feature).
- **Previews:** the dialog renders both on a background thread, like the Export Texture preview.

### Destination
- **The save dialog:** `FileDialogs::save_mod(suggested, start_dir)` uses `rfd` with `set_directory`. `ScriptedDialogs` queues `mod_exports` for tests.
- **`game_mod_folder(Game) -> Option<PathBuf>`:**
  - it uses `directories::UserDirs::document_dir()` on Windows, so a OneDrive-redirected Documents works;
  - it uses `BaseDirs::data_dir()` on macOS and Linux;
  - it returns the folder only if it exists.
- **Fallback:** the last mod export folder is kept in `AppState` for the session, like `export_settings`.

### Command and dialog
- **The command:** `ExportMod` becomes `When(has_project && !gesture_active, …)`, and `needs_texture()` keeps it disabled while a symbol is edited. `reason-soon-mods` is removed from the four locales.
- **The dialog:** `ui/mod_export_dialog.rs` follows `export_dialog.rs`: a `Modal::ExportMod(Box<ModExportDialog>)`, the take/put-back in `dialogs.rs`, `show_hint` on success and `Modal::Message` on failure.
- **The sidebar:** the Version field reads `project.mod_settings.version`.

## Risks / Trade-offs

- **[The game reads the alpha channel differently than assumed]** In-game, transparent areas could show black instead of the base color. → Task 7 checks a real export in ETS2. If transparency misbehaves, the fix is a render background (white, or the base color), which changes no structure.
- **[Unit names change when a package reorders its main textures]** Players who bought the paint job would lose it on that truck after a mod update. → Letters follow package order, which package authors are told to keep stable (ids are stable already). The risk is recorded in the package format doc.
- **[An internal name collides with another mod's]** Two mods defining `ace.<path>.paint_job` override each other. → The derived name is readable rather than random, and the player can edit it. The dialog's help text says it should be unique to them.
- **[Memory on large fleets]** The whole archive is in memory. → Measured in the job test with a 4096² fleet. Switch to a temp file plus rename if it goes past about 1 GB.
- **[Steam Proton installs on Linux]** The Linux mod folder isn't under `~/.local/share`. → The save dialog still works, and the session remembers the folder.

## Migration Plan

- The new fields are optional, the format stays 1, and the `v1.truckpaint` fixture is unchanged.
- Older files are backfilled when their package is installed.
- Older builds opening a newer file ignore the new fields, and those fields are lost if that build saves it, as for any optional field so far.
- Rollback is reverting the change: files saved in the meantime still open.
