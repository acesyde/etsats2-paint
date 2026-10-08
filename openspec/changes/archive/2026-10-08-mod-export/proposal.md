## Why

TruckPaint promises "pick a vehicle, design its paint job, export a ready-to-install mod", but today a player can only export one texture at a time as PNG or DDS. Turning those files into a mod means writing SCS definition files by hand or going through Paintjob Packer. That is the whole deliverable of the app, and Export › Export Mod… has been shown disabled ("coming soon") since the app shell. The paint job model, custom vehicles, brand kit and symbols have shipped, so the data a mod needs is now in the packages.

## What Changes

- **Export › Export Mod… (Cmd/Ctrl+Shift+E) is enabled.** It opens a dialog with the mod settings, a summary of what the mod will contain, and the problems that block the export. Confirming asks for a destination and writes **one `.scs` file**: the whole fleet as one paint job, in one mod.
- **The save dialog opens in the game's mod folder** (ETS2 or ATS, as set by the project's game) when that folder exists, so exporting also installs the mod. Otherwise it opens in the last export folder.
- **Mod settings are saved in the project:**
  - Name, used both in the Mod Manager and in the shop;
  - Version (`1.0` by default), Author and Description;
  - Price and Unlock level;
  - an internal name, derived from the Name and editable;
  - the shop icon (256×64) and the Mod Manager image (276×162).

  Editing them and exporting is one undo step. The Project section of the sidebar shows the mod version in its Version field, which was empty until now.
- **Images are generated, and can be replaced.** By default both images are rendered from the first main texture of the fleet. The player can choose a PNG or JPEG for either one instead. The image is stored in the project and scaled to cover the size.
- **What the mod contains,** for each vehicle in the project and only for the textures it paints:
  - the paint job definitions under `def/vehicle/truck/<path>/` or `def/vehicle/trailer_owned/<path>/`;
  - one paint job per painted main texture. A truck with several main textures gets one per texture, limited to its cabins (`suitable_for`). Cabins left unpainted get no paint job;
  - accessory paint jobs (`acc_list`) for each painted accessory;
  - `alternate_uvset` and an unlocked base color when the package asks for them;
  - every texture rendered at full size as DDS (BC3 with mipmaps), with its `.tobj`;
  - the shop icon material, a `manifest.sii` and a description listing the vehicles.

  Texts written into the mod stay in English.
- **The export checks before it writes.** It refuses with a reason, shown in the dialog:
  - a Name that is empty or contains characters the game can't read;
  - an invalid internal name: `a`–`z`, `0`–`9`, `_`, at most 12 characters, or 10 when a truck has several main textures;
  - a price of 0;
  - two vehicles with the same game path;
  - a vehicle whose game data is missing.
- **Exporting runs in the background,** with progress and Cancel. It leaves no partial file, and it doesn't change the project's save state except for the mod settings.
- **Projects record the game data of their vehicles:** game path, supported game versions, alternate UV, colour picker and required mods, plus the game ids of each painted texture. Projects then export on a computer where the package isn't installed. This data is filled in when a vehicle is added, and replaced by Update Template. Older files get it from the installed package when they open, if that version is installed. A project opened elsewhere without it can still be edited; only the export reports which vehicle lacks it.

Non-goals:
- writing an unpacked folder;
- Steam Workshop upload files (`versions.sii`, workshop image);
- several paint jobs in one project;
- the colour-mask (non-airbrush) paint job mode and changeable mask colors;
- `compatible_versions` in the manifest, since mapping package version ranges to the game's patterns is lossy and the Game versions field stays a later feature;
- the 3D preview of the result.

## Capabilities

### New Capabilities
- `mod-export`: the Export Mod dialog, mod settings and their validation, the generated mod structure (definitions, textures, icon, manifest), the `.scs` output with default destination, and the background export with progress and cancel.

### Modified Capabilities
- `project-files`: the file also stores the mod settings with their images, and the game data of each vehicle and painted texture. Older files fill the game data from the installed package.
- `vehicle-projects`: the vehicle record includes the game data, filled in when a vehicle is added and replaced by Update Template.
- `workspace-layout`: the Project section's Version field shows the mod version.

## Impact

- **tp-core:** `ProjectVehicle` gets the game data (path, versions, alt_uv, colour_picker, requires). `SurfaceTemplate` gets `game_ids`. `Project` gets `mod_settings: ModSettings` (name, version, author, description, internal name, price, unlock level, optional icon and image asset ids).
- **tp-file:** optional fields on `FileVehicle`, `FileTemplate` and `FileProject`, without a format bump. Files without them still open, and the fixtures are unchanged.
- **tp-app:**
  - `vehicle_project.rs` fills the game data;
  - opening a file backfills it from `VehicleLibrary`;
  - a new `mod_export` module builds the mod's file tree and runs the job, reusing `tp_render::render`, `encode_dds` and `tp_file::write_atomic`;
  - a new `ui/mod_export_dialog.rs`, with `CommandId::ExportMod` enabled;
  - `FileDialogs::save_mod` and `pick_image`, with the game mod folder lookup per platform;
  - the Project section's Version field;
  - locale strings in en/fr/es/de. The now unused `reason-soon-mods` is removed.
- **tp-render:** a small helper that renders a surface or an image to cover a W×H rectangle, for the generated images. `encode_dds` is reused as is, and the icon gets a mip chain too.
- **Dependencies:** `zip` moves from dev-dependencies to dependencies in tp-app. Nothing new in the workspace.
- **Docs:** `docs/roadmap.md` records the decisions and marks `mod-export` shipped. `README.md` drops "Exporting a ready-to-install mod comes next". `docs/vehicle-package-format.md` says the mod export is done.
