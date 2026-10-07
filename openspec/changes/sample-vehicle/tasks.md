## 1. Packer library (tp-pack)

- [x] 1.1 Create `crates/tp-pack` (deps: tp-vehicles, serde_json, zip, image (png), texpresso; dev: tempfile) with `PackError` and English messages for every `PackageError`. Verify that `cargo build -p tp-pack` succeeds.
- [x] 1.2 Implement the DDS decoder of D3:
  - parse the header and the DX10 header;
  - decode BC1, BC2 and BC3 with texpresso;
  - read the uncompressed 32-bit and 24-bit masks and DXGI RGBA/BGRA;
  - return `UnsupportedDds` for anything else.
  
  Verify with unit tests on fixtures generated in the tests: BC1, BC3, uncompressed BGRA and DX10 BC3 decode to the source image within BC tolerance, and BC7 or a truncated file gives an error.
- [x] 1.3 Implement `pack_folder` following D2:
  - manifest kept as a `Value`, with safe-path resolution;
  - DDS converted to PNG and the path rewritten;
  - a deterministic ZIP in sorted order with fixed timestamps;
  - validation with `Package::read`;
  - the list of ignored files.
  
  Verify with unit tests in temporary folders: identical bytes on two runs, unrelated files left out and listed, missing template, unsafe path, DDS path rewritten, `export` and unknown fields preserved.
- [x] 1.4 Implement `check` and `summary` (id, version, name, game, game versions, variants with textures and sizes). Verify with a unit test on a `tp_vehicles::sample` package.

## 2. CLI and tasks

- [x] 2.1 Add the `tpv` binary:
  - `pack <folder> [-o <file>]`, with the default output name `<id>-<version>.tpv`;
  - `check <file>`;
  - `--help` and usage errors;
  - exit codes 0 on success and 1 on failure;
  - on failure, no file written.
  
  Verify with integration tests through `CARGO_BIN_EXE_tpv`: pack success, pack failure leaves no file, check success, check of a corrupt file.
- [x] 2.2 Add the mise tasks `pack` and `sample-vehicles` (D6). Verify that `mise run pack -- <tempfolder>` and `mise run sample-vehicles` run.

## 3. Sample vehicle

- [x] 3.1 Write `examples/vehicles/sample-truck/1.0.0/`:
  - `vehicle.json`: `community.truckpaint.sample_truck`, Standard cab and High roof variants, Cabin 4096, Chassis 2048 and Accessories 1024, layout 1, MIT;
  - SVG templates for the standard cabin, the high-roof cabin, the chassis and the accessories, with labelled panels.
  
  Verify that `tpv pack` succeeds, and inspect a render of each template in the app.
- [x] 3.2 Write `examples/vehicles/sample-truck/1.1.0/`:
  - newer `game_versions`;
  - a redrawn Standard cabin at layout 2;
  - the chassis at 4096 in both variants;
  - a new `side_skirts` template (1024) in both variants.
  
  Verify that `tpv pack` succeeds.
- [x] 3.3 Build both packages with `mise run sample-vehicles`, commit them, and add `examples/vehicles/README.md`. Verify that `tpv check` passes on both.
- [x] 3.4 Add `crates/tp-pack/tests/sample.rs`: pack each version folder and compare the entries and uncompressed bytes with the committed `.tpv` (D5), with a failure message naming the package and the command to run. Verify that it passes, and that it fails after editing an SVG without rebuilding (then revert).

## 4. In-app sample (tp-app)

- [x] 4.1 Embed `SAMPLE` (the 1.1.0 package) in `vehicles.rs` and add `install_sample`. Verify with unit tests:
  - both committed packages install;
  - a project from 1.0.0 Standard cab, planned against 1.1.0, gives Cabin LayoutChanged, Chassis resized and Side skirts added.
- [x] 4.2 Add the message ids `vehicles-install-sample` and `new-project-no-vehicles-hint` in en, fr, es and de. Verify that the tp-i18n checks pass (all locales have the same ids).
- [x] 4.3 Add **Install the sample vehicle** to the empty Vehicle Library and, when the library is empty, to the New Project Vehicle step, both reusing `install_named` messages. Verify with kittests:
  - from an empty library, the wizard button installs and lists "TruckPaint Sample Truck";
  - the button is hidden once a vehicle is installed;
  - the Vehicle Library button installs.
  
  Also verify that `no_screen_shows_a_raw_message_id` still passes.

## 5. Docs and checks

- [x] 5.1 Update `docs/vehicle-package-format.md` with a "Packing a package" section (`tpv pack` and `check`, DDS conversion and supported DDS formats, ignored files), point to the sample as the reference example, and mention the sample in the README. Verify that the doc's example manifest test still passes.
- [x] 5.2 Run `mise run fmt:check`, `mise run lint` and `mise run test`, and verify that all three pass.
