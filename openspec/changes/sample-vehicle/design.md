## Context

**tp-vehicles:**
- reads and validates packages from bytes with `Package::read`, with no filesystem access;
- has a `sample` module that generates throwaway packages for tests (64 px PNGs, id `scs.sample.truck`).

**tp-app:**
- installs packages through `VehicleLibrary::install_bytes`;
- shows `EmptyState` in the Vehicle Library when nothing is installed;
- shows the New Project `vehicle_step`, which always lists Blank texture and then the installed vehicles.

**Rendering and DDS:**
- SVG templates are rendered with `tp_render::svg_options()`, which loads the bundled fonts, so `<text>` labels in a template render the same everywhere.
- `tp-vehicles` parses SVGs with the default options, but only to read their size.
- `texpresso` is already a dependency of tp-render, used for DDS export. `image`'s `dds` feature is only a dev-dependency there.

**CI:** `ci:changes` treats everything outside `*.md`, `openspec/` and `.claude/` as code, so changes under `examples/` run the full CI.

## Goals / Non-Goals

**Goals:**
- **One validation:** the packer and the app share it, so a package `tpv` accepts always installs.
- **Reproducible sample:** sources in, identical package out. The check doesn't depend on ZIP byte details.
- **Cross-platform tooling:** no Python and no external converter.

**Non-Goals:**
- **No GUI packer:** authoring inside the app is out of scope.
- **No marketplace:** publishing and downloading are for `vehicle-marketplace`.
- **No DDS in the package format:** templates in packages stay PNG and SVG only.
- **No other DDS formats:** BC4–BC7, cube maps and arrays aren't supported. SCS paint templates don't use them.
- **No game-file reading:** extracting templates from the game's `.scs` archives is out of scope.
- **No preview image:** the sample doesn't need one.

## Decisions

### D1. A new crate `tp-pack` (library and `tpv` binary)
`tp-vehicles` stays "bytes in, no filesystem". `tp-pack` holds the filesystem side.

**Library:**
- `pack_folder(path) -> Result<Packed, PackError>`, where `Packed` holds the package bytes, the manifest, the ignored files and the converted files;
- `check(bytes)`;
- `summary(&Manifest) -> String`.

**Binary:** `src/bin/tpv.rs` parses `pack <folder> [-o <file>]` and `check <file>` by hand, with no clap dependency for two subcommands.

**Messages:** they're in English. This is a developer tool, so it's not localized. `PackageError` values are turned into English sentences inside tp-pack, independently of the app's Fluent messages.

**Alternatives considered:**
- **An `xtask`:** it isn't usable by package authors outside the repo.
- **A Python script:** it duplicates validation, and Python isn't in mise.
- **A subcommand of tp-app:** it would drag the GUI into a CLI.

### D2. Packing pipeline
1. **Read the manifest:** read `vehicle.json` and parse it as `serde_json::Value`, keeping unknown fields such as `export`. Also parse it as a `Manifest` for typed access.
2. **Collect the referenced files:** each texture's `template` and the `preview`. A path is resolved inside the folder only after `tp_vehicles::is_safe_path`, so a manifest can't pull in `../secret`.
3. **Convert DDS templates:** for a `.dds` template, decode it (D3) to RGBA, encode it as PNG and store it under the same path with `.png`. The `template` field in the `Value` is rewritten to that path. If two textures reference the same DDS, it is converted once.
4. **Write the ZIP:**
   - entries in sorted path order, `vehicle.json` first;
   - the manifest pretty-printed from the edited `Value`;
   - every entry with timestamp 1980-01-01 00:00 and the same Deflate options;
   - PNGs re-encoded only when they come from DDS, so authored PNGs are copied as-is.
5. **Validate:** run `Package::read` on the produced bytes. Its `PackageError` becomes the reported failure.
6. **Report:** list ignored files, meaning folder files that aren't referenced. Hidden files and `.DS_Store` are skipped silently.

Templates are read from disk with a size cap: `MAX_UNCOMPRESSED`, checked before reading.

### D3. DDS decoding: our own header parser and texpresso
We parse the 128-byte DDS header and, if present, the DX10 header ourselves (about 100 lines):
- **Compressed formats:**
  - FourCC `DXT1`, `DXT3` and `DXT5`;
  - DXGI `BC1`, `BC2` and `BC3`, UNORM or SRGB.
  - These are decompressed with `texpresso::Format::{Bc1,Bc2,Bc3}.decompress`, at the top mip level only.
- **Uncompressed formats:**
  - 32-bit RGB with alpha bit masks, as BGRA or RGBA;
  - 24-bit RGB;
  - DXGI `R8G8B8A8_UNORM` and `B8G8R8A8_UNORM`.
- **Anything else** returns `PackError::UnsupportedDds { texture, detail }`.

**Why not `image`'s `dds` feature:** it decodes only DXT1, DXT3 and DXT5, with no DX10 header and no uncompressed formats. It would also add a feature to the shared `image` dependency of every crate. texpresso is already compiled for tp-render.

**Tests:** fixtures are generated in tests by compressing a known image with texpresso and writing a header. No binary fixture is needed, and the round trip is compared with a BC tolerance.

### D4. Sample sources and layout
```
examples/vehicles/
├── README.md                          what it is, how to rebuild, license (MIT, ours)
├── sample-truck/
│   ├── 1.0.0/vehicle.json
│   ├── 1.0.0/templates/{standard_cabin,high_roof_cabin,chassis,accessories}.svg
│   ├── 1.1.0/vehicle.json
│   └── 1.1.0/templates/{standard_cabin,high_roof_cabin,chassis,accessories,side_skirts}.svg
├── community.truckpaint.sample_truck-1.0.0.tpv
└── community.truckpaint.sample_truck-1.1.0.tpv
```

**Self-contained versions:** each version folder can be packed on its own, as the packer expects. Unchanged SVGs are duplicated between versions. They're small text files, and duplicating them keeps the packer free of "inherit from" logic.

**Templates:**
- hand-written SVGs with `viewBox` equal to the texture size;
- dark outlines and light grey panel fills at low alpha, on a transparent background;
- `<text>` labels in `font-family="sans-serif"`, which the app maps to the bundled font;
- a different silhouette for each cab (the High roof cabin has a taller roof panel);
- in the 1.1.0 Standard cabin, the doors and grille move, so the layout change is visible;
- the 1.1.0 chassis is redrawn at 4096.

The variants share the chassis, accessories and side-skirts templates by referencing the same file.

**Manifest data:**
- brand "TruckPaint", license "MIT", authors ["TruckPaint contributors"];
- `game_versions`: 1.0.0 uses `>=1.50, <1.56` and 1.1.0 uses `>=1.56`.

### D5. Up-to-date check compares contents, not bytes
A test in `tp-pack` (`tests/sample.rs`) packs each `examples/vehicles/sample-truck/<v>/` folder and compares it with the committed `.tpv`. It compares the sorted list of entry names and each entry's uncompressed bytes. ZIP framing, the compression level and the zip crate version are ignored, so a dependency bump never fails it spuriously.

On mismatch, the test names the package and says to run `mise run sample-vehicles`. Installing both committed packages through `VehicleLibrary::install_bytes` is covered by a tp-app unit test (D8).

### D6. mise tasks
- **`pack`:** `cargo run -q -p tp-pack --bin tpv -- pack {{arg(name='folder')}}`. Extra args such as `-o` are forwarded through `--`.
- **`sample-vehicles`:** runs `tpv pack` for each version folder, with `-o examples/vehicles/community.truckpaint.sample_truck-<v>.tpv`.

### D7. In-app sample
**Embedding:** `tp-app/src/vehicles.rs` gains:
- `pub const SAMPLE: &[u8] = include_bytes!("../../../examples/vehicles/community.truckpaint.sample_truck-1.1.0.tpv")`;
- `VehicleLibrary::install_sample()`, which calls `install_bytes(SAMPLE)`.

Only the newest version is embedded. Painters who want to try Update Template install 1.0.0 from the repo.

**UI:**
- **Vehicle Library:** `EmptyState` gets a primary action, **Install the sample vehicle**, in addition to the dialog's existing Install….
- **Vehicle step:** when the library is empty, a short hint and the same button appear below the Blank texture row.
- **Messages:** both places reuse `install_named`'s result messages, so the confirmation and error wording is unchanged.
- **Message ids:** `vehicles-install-sample` and `new-project-no-vehicles-hint`, in the four locales.

**Alternative considered:** installing the sample automatically on first launch. It was rejected because it's surprising and puts data in the user's library unasked.

### D8. Tests
**tp-pack (unit tests):**
- packing produces identical bytes twice;
- ignored files are listed;
- a missing template fails and no file is written (temp dir);
- an unsafe path is refused;
- DDS BC1, BC3, uncompressed and DX10 are converted, and the manifest path is rewritten;
- an unsupported DDS fails, naming the texture;
- `export` and unknown fields are preserved;
- `check` summary.

**tp-pack (integration tests):**
- the sample is up to date (D5);
- the CLI exit codes, run through `env!("CARGO_BIN_EXE_tpv")`.

**tp-app (kittest):**
- with an empty library, clicking Install the sample vehicle in New Project lists "TruckPaint Sample Truck 1.1.0";
- the button is hidden when a vehicle is installed.

**tp-app (unit test):** a project from sample 1.0.0 Standard cab, with an update planned against 1.1.0, gives:
- Cabin: LayoutChanged;
- Chassis: resized;
- Side skirts: added.

The existing `sample` module in tp-vehicles stays as it is for fast unit tests.

## Risks / Trade-offs

- **SVG text rendering differs with system fonts.** Labels rely on the bundled sans-serif fallback. → The labels are decorative, so a font difference doesn't change the layout.
- **The binary `.tpv` files grow the repo with each sample revision.** → They're tens of KB, and the sample changes rarely.
- **A developer edits the sources and forgets to rebuild.** → The D5 test fails in CI with the exact command to run.
- **Real SCS DDS variants we haven't seen** (e.g., L8, or R5G6B5 without alpha). → The packer fails clearly, naming the texture. More formats can be added to the decoder later, with no spec change.
- **`include_bytes!` path from tp-app into `examples/`.** It ties the app build to the repo layout. → Acceptable: the app is built from this workspace, and the D5 test keeps the file valid.
- **Large DDS (8192², 256 MB RGBA) in memory.** Decoding and PNG encoding hold the full image. → Acceptable for a developer tool. The 16384 px limit still applies through validation.
