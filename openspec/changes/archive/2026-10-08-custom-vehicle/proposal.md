## Why

Every project now needs a vehicle, but a player can only pick a vehicle someone has packaged. A truck just released, or a community mod nobody has packaged, leaves them stuck unless they write a `vehicle.json` and run `tpv`. **Custom vehicle…** is the escape hatch the roadmap promises (`docs/roadmap.md`, change 1): the player drops the game's template files, describes the paint job the way the game does, and TruckPaint packs and installs the package.

## What Changes

**Custom Vehicle dialog (new):**
- Reached from:
  - **Custom vehicle…** in the Vehicle step of New Project;
  - **Custom vehicle…** in Add Vehicle…, where the game is locked to the project's game;
  - **Custom Vehicle…** in the Vehicle Library.
- The player enters the vehicle's name, brand, kind (truck or trailer), game, game path and, optionally:
  - the supported game versions (default: any);
  - the alternate UV set and colour picker flags.
- **Templates:** the player drops template files (PNG, DDS or SVG) on the dialog or picks them with Add Templates…. Each file becomes a texture row with:
  - a name, from the file name;
  - a role: main texture or accessory;
  - its game ids: cabin internal names or accessory ids;
  - its size, from the image.

  Rows can be renamed, re-assigned, given another template or removed.
- **Game data is required, as `tpv` requires it:**
  - the game path, always;
  - cabin internal names when there are several main textures;
  - accessory ids on every accessory.

  The dialog explains each missing or invalid field before Create is enabled. A trailer has exactly one main texture.
- **Create** builds the package the way `tpv` does: DDS templates are converted to PNG, and the package goes through the same validation. TruckPaint installs it in the library and goes back to the dialog it came from, with the new vehicle selected.
- Custom vehicles get the id `custom.<brand>.<name>` and version 1.0.0.

**New Version… (new):**
- In the Vehicle Library, a custom vehicle offers **New Version…**: the same dialog, filled in from its newest version.
- The id, game and kind are locked. The version defaults to the next minor version.
- Every texture whose template the player replaces gets a higher layout version, so Update Template flags it in projects, as for any package.

**Vehicle Library:**
- **Export…** on any installed version saves its `.tpv` file, ready to share. A reminder says that templates from the base games belong to SCS.
- **Custom Vehicle…** and **New Version…**, as above.

**Packer:** `tp-pack` packs from files in memory as well as from a folder. `tpv` and the app share one packing and validation path. `tpv` behaves as before.

## Capabilities

### New Capabilities
- `custom-vehicles`: the Custom Vehicle dialog (vehicle fields, template rows, roles, game ids and sizes, inline validation), building and installing the package, the `custom.` ids, and New Version… with the automatic layout versions.

### Modified Capabilities
- `vehicle-packages`: the Vehicle Library dialog offers Custom Vehicle…, New Version… for custom vehicles and Export… for every installed version, and shows "any version" for a vehicle without a game-version limit.
- `start-screen`: the New Project Vehicle step offers Custom vehicle…, also when no vehicle is installed, and selects the created vehicle.
- `vehicle-projects`: Add Vehicle… offers Custom vehicle…, locked to the project's game, and selects the created vehicle.

## Impact

- **tp-pack:**
  - packing split into `pack_entries` (in memory, DDS conversion, validation), used by `pack_folder`;
  - a `custom` module that builds the manifest and the package from a custom vehicle description;
  - a DDS header probe that checks the format without decoding;
  - unit tests.
- **tp-vehicles:** the `is_valid_id` and `is_valid_game_name` helpers, already public, are reused for inline validation. No format change.
- **tp-app:**
  - depends on `tp-pack`;
  - new `ui/custom_vehicle.rs` (dialog and form state);
  - `Modal::CustomVehicle`, which remembers the dialog it came from;
  - packing on a background thread, as for exports;
  - dropped template files go to the open dialog;
  - `FileDialogs::pick_templates` and `save_package`;
  - the Custom vehicle… buttons in the wizard (`dialogs.rs`), Add Vehicle… and the Vehicle Library (`vehicle_dialogs.rs`), with Export… and New Version…;
  - UI tests in `tests/vehicles.rs`.
- **tp-i18n:** new messages in the 4 languages (de, en, es, fr).
- **docs:**
  - `vehicle-package-format.md`: custom vehicles and the `custom.` ids;
  - `roadmap.md`: the decisions (game data required, New Version…) and, once shipped, the Shipped list.
