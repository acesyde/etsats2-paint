## Context

See proposal.md (Why) and the specs for the behavior. Today:
- `tp-pack::pack_folder` does three things in one function: it reads the referenced files from a folder, converts DDS templates to PNG (`dds::decode`, then rewriting the template paths in the manifest JSON), and writes a reproducible ZIP that it validates with `Package::read`. `tp-app` doesn't depend on `tp-pack`.
- `tp_vehicles::Manifest` is `Deserialize` only. Packages are built from JSON values. `is_valid_id` and `is_valid_game_name` are public.
- `Package` exposes each template's bytes, kind and natural size, so an installed version can fill in a form.
- `VehicleLibrary::install_bytes` validates and stores a package as `<root>/<id>/<version>.tpv`. `install_error_message` localizes every `PackageError`.
- `AppState.modal` holds a single `Modal`. New Project (`Modal::NewProject(NewProjectDraft)`), Add Vehicle (`Modal::AddVehicle`) and the Vehicle Library (`Modal::VehicleLibrary`) each keep their state in the modal value. `show_modal` takes the value out each frame and puts it back.
- `after_frame` installs dropped `.tpv` files. The workspace places other dropped images on the canvas.
- Long work runs on a thread that reports through `mpsc` and atomics (`export.rs`'s `ExportJob`).
- `FileDialogs` is a trait with native (`rfd`) and scripted (tests) implementations.

## Goals / Non-Goals

**Goals:**
- One packing and validation path for `tpv` and the app: a custom vehicle is exactly what `tpv pack` would build from the same files.
- Form rules that live next to the package rules, tested without UI, and localized by the app.
- The UI stays responsive while 4096² DDS templates are converted and PNG-encoded.

**Non-Goals:**
- Prefilling game data from the Paintjob Packer database (later).
- A preview image, and fields for authors, license, homepage, description and `requires`. New Version… carries them over from the previous version unchanged.
- Reordering texture rows. The package order is the main-texture rows, then the accessory rows, each in row order. To reorder, the player removes and re-adds a row.
- New Version… for packages that aren't custom (`custom.` ids), and changing a custom vehicle's id, kind or game.
- Changes to `tpv`'s behavior or to the package format.

## Decisions

### 1. Packing from memory in `tp-pack`, shared by `tpv` and the app
Split `pack_folder`:
- `pack_entries(manifest: serde_json::Value, files: BTreeMap<String, Vec<u8>>) -> Result<Packed, PackError>`:
  - converts every DDS entry referenced as a template to PNG and rewrites its path;
  - writes the ZIP as today;
  - validates it with `Package::read`.
- `pack_folder` reads the manifest and the referenced files, calls `pack_entries`, then lists the ignored files.

The committed sample packages, checked byte for byte by `tests/sample.rs`, guard the refactor.

`tp-app` depends on `tp-pack`, which already depends only on `tp-vehicles`, `image`, `texpresso`, `zip`, `serde_json` and `semver`.

**Alternatives:**
- Write a temporary folder from the app and call `pack_folder`. Rejected: temporary files to clean up, and an "ignored files" list that means nothing here.
- Re-implement packing in `tp-app`. Rejected: two paths to keep in sync, while the roadmap requires "the same packing and validation as `tpv`".

### 2. `tp_pack::custom`: the form's model, without UI
A plain-data description and its rules:

```rust
pub struct CustomVehicle {
    pub id: Option<String>,          // Some on New Version (locked)
    pub version: semver::Version,    // 1.0.0, or the New Version field
    pub name: String, pub brand: String,
    pub kind: Kind, pub game: Game,
    pub path: String, pub versions: String,   // "" = any ("*")
    pub alt_uv: bool, pub colour_picker: bool,
    pub carried: serde_json::Map<String, Value>, // authors, license… and game.requires from the base version
    pub rows: Vec<CustomPart>,
}
pub struct CustomPart {
    pub base: Option<(String, u32)>, // texture id and layout version kept from the base version
    pub replaced: bool,              // a new template was picked for a kept texture
    pub name: String, pub role: Role, pub game_ids: String, pub size: u32,
    pub template: TemplateFile,      // file name, bytes, format, width, height
}
```

- `probe(file_name, bytes) -> Result<TemplateFile, TemplateError>` reads only the dimensions:
  - PNG with `image::ImageReader::into_dimensions`;
  - SVG with `usvg`'s size;
  - DDS with a new `dds::probe` that runs the header checks of `dds::decode` without decoding any block.

  `TemplateError` is typed (`NotAnImage`, `UnsupportedDds { format }`, `TooLarge`), so the app can localize it.
- `problems(&self, installed: &[ManifestSummary]) -> Vec<Problem>` gives the inline checks of the "Describing the paint job" requirement, plus:
  - an id already installed (new vehicle);
  - a version not higher than the installed ones (New Version).

  `Problem` names the field or row index and a code. The app maps codes to messages. Game paths and game ids are checked with `tp_vehicles::is_valid_game_name`.
- `slug`, `vehicle_id(brand, name)`, and part ids:
  - a kept row keeps its `base` id;
  - a new row gets `slug(name)`, made unique with `_2`, `_3`… against every id of the form **and every id of the base version**. A new texture never takes the id of a removed one, which Update Template would match as the same texture.
- `layout_version`: `base.1 + 1` when `replaced`, `base.1` otherwise, and 1 for a new row.
- `manifest_json()` builds the JSON value:
  - `format: 1`;
  - `game.versions`: `"*"` when empty;
  - templates at `templates/<part id>.<png|svg|dds>`;
  - the `carried` fields put back.
- `pack()` calls `pack_entries`.
- `from_package(&Package) -> CustomVehicle` fills in New Version…:
  - the version is the next minor version;
  - each template becomes a row, with the stored PNG or SVG as its file and `replaced: false`.

**Why compare a "replaced" flag rather than the template bytes:** a DDS template is converted to a PNG whose bytes depend on the encoder, so comparing bytes would bump the layout version of unchanged textures, or miss changes. Replacing a template is the player's explicit statement that the layout moved. A size change alone keeps the layout version: Update Template scales the artwork.

### 3. The dialog is its own modal and remembers where it came from
`Modal::CustomVehicle(Box<CustomVehicleDialog>)`. The dialog holds `origin: Origin`:

```rust
enum Origin {
    NewProject(NewProjectDraft),
    AddVehicle(AddVehicleDialog),
    Library(LibraryDialog),
}
```

Opening the dialog moves the current modal into `origin`.

- **Closing:** the origin is put back in `state.modal`.
- **Success:**
  - New Project and Add Vehicle get `choice = VehicleChoice::of(new vehicle)`;
  - all three get the confirmation in their `messages`;
  - New Project clears its search and its game and kind filters when they would hide the new vehicle. Add Vehicle keeps its locked game, which is the vehicle's game anyway.

The dialog's game is the origin's locked game (Add Vehicle), else the wizard's game filter, else ETS2.

**Alternatives:**
- Embed the form as a sub-view of each of the three dialogs. Rejected: three layouts and keyboard flows to keep identical, and a 560 px dialog too narrow for the rows.
- A general modal stack. Rejected: this is the only nested dialog, and the `take()`/put-back pattern of `show_modal` handles one `origin` directly.

### 4. Building on a thread
`PackJob` follows `ExportJob`:
- a thread runs `CustomVehicle::pack()` on a clone of the form. The clone holds the template bytes, a few hundred MB at most, freed when the job ends;
- an `AtomicU32` counts the converted templates, so the dialog shows "Building the package… (3/8)";
- an `mpsc` channel returns `Result<Packed, PackError>`.

While the job runs, the form and Create are disabled. Cancel and Escape stay available: they drop the job and close the dialog, and the thread's result is ignored.

On `Ok`, the UI thread calls `state.vehicles.install_bytes(&packed.bytes)`. It validates again, which is cheap: it reads the dimensions only. Errors are mapped:
- `PackError::Package(e)` and `InstallError` go through `install_error_message`. Its `PackageError` match is extracted into `package_error_reason` and reused;
- `PackError::UnsupportedDds`, which is possible only for a truncated file that passed the header probe, maps to the localized "can't be read" message of its texture.

**Alternative:** pack on the UI thread. Rejected: PNG-encoding a 4096² image takes about a second, and a truck has up to a dozen textures.

### 5. Template rows and dropped files
- **Adding a file:** `probe` runs on the UI thread. It reads headers only, so it is instant. The probe's result gives the defaults:
  - the size: the image's width if it is in `SIZES`, else the nearest allowed size, ties going to the larger one;
  - the non-square warning;
  - the role: main texture while the form has none, else accessory.
- **Dropped files:** `after_frame` routes them to the dialog when `state.modal` is `CustomVehicle`:
  - `.tpv` files are still installed. The installation's messages go to the dialog, and no other modal opens;
  - every other file goes to the dialog. A single file dropped over a row's rectangle, which the dialog records each frame, replaces that row's template. Any other drop adds rows;
  - nothing reaches the canvas while the dialog is open.
- **File dialogs:** `FileDialogs` gains:
  - `pick_templates() -> Vec<PathBuf>`, filtered on PNG, DDS and SVG, used by Add Templates… and Replace…, with a single file kept for Replace…;
  - `save_package(suggested) -> Option<PathBuf>`.

  `ScriptedDialogs` gets a queue for each.

### 6. Vehicle Library additions
- **Export…** reads the installed file (`InstalledVersion.path`) and writes it with `tp_file::write_atomic` to the chosen path. The bytes are unchanged, so an exported package is exactly the installed one. The reminder about SCS templates is a line in the dialog under the version's actions, and is part of the save dialog's title.
- **New Version…** is shown when `id.starts_with("custom.")`. It loads the newest version (`VehicleLibrary::load`) and opens the dialog with `Origin::Library`.
- **"Any version":** `versions == VersionReq::STAR` is shown with a localized label wherever the supported game versions appear: the library entries and the vehicle tooltip in the sidebar.

### 7. Text
New `tp-i18n` messages in en, fr, de and es:
- the dialog's labels and hints. The game ids hint differs for a main texture and for an accessory;
- each `Problem` and `TemplateError` code;
- the progress line;
- Export…, New Version…, Custom vehicle… and the SCS reminder.

The `localization` test already checks that every key exists in every language.

## Risks / Trade-offs

- **[Two players create the same `custom.<brand>.<name>`]** Installing the other player's package of the same version replaces one's own. → The ids are documented as local. Create refuses an id that is already installed, so the player renames. Exported packages keep their id, and anyone sharing widely can still repackage with `tpv` under a `community.` id.
- **[Memory with many large DDS files]** The form keeps the raw bytes of each template: 16 MB for a 4096² BC3 file, 64 MB uncompressed. → Bytes only, never decoded pixels, until the job runs. The job converts one template at a time. The package limit of 512 MB still applies at validation.
- **[Players don't know cabin internal names or accessory ids]** Required game data can block the form. → The hints say what to enter, and the format doc lists where to find the names (the game's `def/vehicle/…` files). Prefilling from the Paintjob Packer database is the planned follow-up. The decision to require the data was made with the user: a package without it can't be exported as a mod.
- **[Refactor changes `tpv`'s output]** → `tests/sample.rs` compares the committed sample packages byte for byte, and the `tp-pack` CLI tests stay unchanged.
- **["Replaced" bumps the layout even when the same file is picked again]** → Accepted. Replacing is an explicit act. The player can dismiss "Layout changed" in the project.

## Migration Plan

Additive. Package and project formats are unchanged, and `tpv` behaves as before. Rollback is reverting the change. Custom packages already installed stay valid packages.
