## 1. Packing from memory (tp-pack)

- [x] 1.1 Split `pack_folder` into `pack_entries(manifest, files)` (design D1): DDS conversion and path rewriting, reproducible ZIP, `Package::read` validation. `pack_folder` reads the referenced files and calls it, then lists the ignored files. Verify:
  - `cargo test -p tp-pack` passes unchanged, including `tests/sample.rs` (byte-identical sample packages) and `tests/cli.rs`;
  - a new unit test packs a manifest and a BC3 DDS from memory and gets the PNG entry and its rewritten path.
- [x] 1.2 Add `dds::probe(bytes) -> Result<(u32, u32), String>`, sharing the header checks of `dds::decode` without decoding blocks. Verify with unit tests: it accepts BC1, BC3, DX10 BC3 and uncompressed 32-bit, and refuses BC7 (DX10 format 98) and a cube map with the same reasons as `decode`.

## 2. Custom vehicle model (tp-pack `custom` module)

- [x] 2.1 Add `CustomVehicle`, `CustomPart`, `TemplateFile`, `probe` and `TemplateError` (design D2):
  - PNG and SVG dimensions;
  - DDS through `dds::probe`;
  - the size default: the width if allowed, else the nearest allowed size, ties to the larger;
  - the role default: main while there is none.

  Verify with unit tests:
  - a 4096 PNG gives size 4096;
  - a 3000 px PNG gives 2048, and 3072 gives 4096;
  - a 4096×2048 image is reported as not square;
  - a BC7 DDS gives `UnsupportedDds`;
  - a text file gives `NotAnImage`.
- [x] 2.2 Implement `slug`, `vehicle_id(brand, name)` and the part ids:
  - kept rows keep their base id;
  - new rows are unique against the form and the base version's ids.

  Verify with unit tests:
  - "Scania" + "R 2024" gives `custom.scania.r_2024`;
  - "Ä" gives `custom.vehicle.…`;
  - duplicate names give `cabin` and `cabin_2`;
  - a new row named like a removed texture doesn't reuse its id.
- [x] 2.3 Implement `problems(installed)` with one `Problem` code per rule of the "Describing the paint job" requirement, plus "id already installed" and "version not higher". Verify with one unit test per code, and with valid cases:
  - a truck with one main texture without game ids;
  - a trailer with one main texture and accessories.
- [x] 2.4 Implement `manifest_json()` and `pack()`:
  - `game.versions` is `"*"` when empty;
  - templates are at `templates/<id>.<ext>`;
  - every row has layout version 1 for a new vehicle;
  - the carried fields are put back.

  Verify with unit tests: the packed custom truck passes `Package::read`, its DDS template becomes a PNG of the same dimensions, and `tpv check` (the `summary` function) prints its game path, main textures and accessories.
- [x] 2.5 Implement `from_package(&Package)` for New Version…:
  - the next minor version;
  - each row with `base` set, its stored template and `replaced: false`;
  - the authors, license, description and `game.requires` carried over.

  Implement the layout version rule: +1 when replaced, kept otherwise, 1 for new rows. Verify with a round-trip unit test: build 1.0.0, then `from_package`, replace one template, add a row, remove a row, and pack 1.1.0. The replaced texture is at layout 2, the kept one at 1, the new one at 1 with a fresh id, and the removed id is absent.

## 3. App plumbing (tp-app)

- [x] 3.1 Add the `tp-pack` dependency to `tp-app`. Extract `package_error_reason(&PackageError)` from `install_error_message`, and add a localized message for `PackError` and `TemplateError` (design D4). Verify that `cargo build -p tp-app` succeeds and that the existing install-error tests still pass.
- [x] 3.2 Extend `FileDialogs` with `pick_templates()` (PNG, DDS, SVG) and `save_package(suggested)` for the native dialogs, with queues in `ScriptedDialogs`. Verify with `cargo build -p tp-app` and the scripted queues used in the tests of 4.x.
- [x] 3.3 Add `PackJob` (design D4): a thread running `CustomVehicle::pack`, an atomic count of converted templates, an `mpsc` result, and cancellation by dropping the job. Verify with unit tests: a completed job returns the package, and a dropped job leaves nothing installed.

## 4. Custom Vehicle dialog (tp-app)

- [x] 4.1 Add `ui/custom_vehicle.rs` with `CustomVehicleDialog`, `Origin` and `Modal::CustomVehicle` (design D3). Implement:
  - the fields: name, brand, kind, game (locked from Add Vehicle), game path, game versions, alternate UV set and colour picker;
  - the SCS reminder;
  - Cancel and Escape returning to the origin unchanged.

  Verify with UI tests in `tests/custom_vehicles.rs`:
  - opened from Add Vehicle… of an ATS project, the game is ATS and can't be changed;
  - Escape restores the wizard with its previous choice.
- [x] 4.2 Add the template rows:
  - Add Templates… and drops add rows, Replace… and a drop on a row replace a template, Remove deletes a row;
  - the name, role, game ids and size can be edited;
  - the file name and dimensions are shown, with the non-square warning;
  - the unreadable-file message names the file.

  Route dropped files to the dialog in `after_frame` (design D5). Verify with UI tests:
  - dropping a BC3 `cabin.dds` and a BC1 `mirrors.dds` gives a main texture "cabin" of 4096 and an accessory "mirrors" of 1024;
  - a BC7 DDS is reported by name while the PNG of the same drop is added;
  - Replace… keeps the name, role and game ids;
  - a dropped image is not placed on the canvas of an open project.
- [x] 4.3 Show each `Problem` next to its field or row, with game-ids hints that differ for main textures and accessories, and enable Create only without problems. Verify with UI tests, one per spec scenario:
  - one cabin layout is accepted;
  - "Topline" without cabin names is refused;
  - an accessory without id is refused;
  - a trailer with two main textures is refused;
  - the game path `Scania R` is refused;
  - an id already installed points to New Version….
- [x] 4.4 Implement Create: start `PackJob` and show "Building the package… (n/N)" with the form disabled. On success:
  - install the package;
  - return to the origin with the vehicle selected (New Project, Add Vehicle) or listed (Library), and a confirmation;
  - in New Project, clear the filters that would hide the vehicle.

  On failure, keep the dialog with the reason. Verify with UI tests:
  - the spec's "Project from a custom vehicle" scenario, end to end to a workspace whose surface shows the template;
  - "Custom trailer added to a fleet";
  - "Created vehicle hidden by a filter";
  - the installed package holds a PNG with the DDS's pixels.

## 5. Entry points and Vehicle Library (tp-app)

- [x] 5.1 Add **Custom vehicle…** to the New Project Vehicle step, also in its no-vehicle state next to Install the sample vehicles, and to Add Vehicle…. Verify with the start-screen UI test "No vehicle installed": Install…, Install the sample vehicles and Custom vehicle… are offered.
- [x] 5.2 In the Vehicle Library, add:
  - **Custom Vehicle…**, also in the empty state;
  - **New Version…** for `custom.` ids, loading the newest version into the dialog with the version field, which must be higher than every installed version;
  - **Export…** per version, which copies the installed file through `save_package`, proposing `<id>-<version>.tpv`, with the SCS reminder.

  Verify with UI tests:
  - Export writes identical bytes;
  - the sample truck offers no New Version…;
  - New Version… with 1.0.0 entered disables Create with the "higher than 1.0.0" message;
  - Custom Vehicle… from the library lists the new vehicle.
- [x] 5.3 Show "any version" for packages whose game versions are `*`, in the Vehicle Library and in the sidebar's vehicle tooltip. Verify with a UI test on a custom vehicle created without game versions.
- [x] 5.4 End-to-end UI test of the spec's "Update Template on a custom vehicle" scenario:
  - create `custom.scania.r_2024` 1.0.0 and a project from it;
  - make 1.1.0 with New Version…, replacing the "cabin" template;
  - run Update Template.

  "cabin" is flagged "Layout changed", and the accessory isn't.

## 6. Text and docs

- [x] 6.1 Add every new message to `tp-i18n` in en, fr, de and es: the labels, hints, problem codes, template errors, progress, Export…, New Version…, Custom vehicle… and the SCS reminder. Verify that `cargo test -p tp-app --test localization` passes.
- [x] 6.2 Update `docs/vehicle-package-format.md` with a "Custom vehicles" section:
  - the `custom.` ids;
  - where to find cabin internal names and accessory ids in the game's `def/vehicle/…` files;
  - New Version… and its layout versions;
  - Export….

  Update `docs/roadmap.md`: record the decisions (game data required, New Version… for custom vehicles), move `custom-vehicle` to Shipped, and renumber the next changes. Verify by reading both pages against the specs.
- [x] 6.3 Add the Custom Vehicle dialog (filled in, with a problem shown) to `tests/screenshots.rs`, and run `mise run screenshots` to check it visually. Run `mise run ci` (fmt:check, lint with warnings denied, test, build). Verify that every step passes with no warnings.
