## Why

The vehicle library is merged, but nobody can try it end to end. There's no package to install, and SCS templates can't be redistributed. Package authors also have no tool to build a `.tpv`; today they would zip a folder by hand and only learn it's invalid when the app refuses it. Real SCS templates are DDS files, which packages don't accept.

## What Changes

- **A sample vehicle we own, "TruckPaint Sample Truck":**
  - **Templates:** hand-made SVGs that look like real UV layouts. No game asset is involved.
  - **Variants and textures:** two variants (Standard cab and High roof), each with cabin 4096, chassis 2048 and accessories 1024.
  - **Versions:** 1.0.0 and 1.1.0. Version 1.1.0 changes the Standard cabin layout, enlarges the chassis to 4096 and adds a side-skirts texture, so every path of Update Template can be tried.
  - **In the repo:** the sources sit in `examples/vehicles/sample-truck/<version>/`, next to the built `examples/vehicles/community.truckpaint.sample_truck-<version>.tpv`. A test fails when a committed package is out of date with its sources.
- **A packer CLI, `tpv`, in a new crate `tp-pack`:**
  - `tpv pack <folder> [-o file.tpv]` builds a package from a folder holding a `vehicle.json` and its templates, and validates it with the reader the app uses.
  - `tpv check <file.tpv>` validates an existing package.
  - **DDS templates:** the packer converts them to PNG and rewrites their paths in the manifest. The package format still accepts only PNG and SVG.
  - **Contents:** the packer only includes the files the manifest references, and its output is deterministic.
- **mise tasks:**
  - `mise run pack -- <folder>` packs a folder.
  - `mise run sample-vehicles` rebuilds the committed sample packages.
- **In the app:**
  - When no vehicle is installed, the empty Vehicle Library and the New Project Vehicle step offer **Install the sample vehicle**.
  - That button installs the sample's newest version, which is built into the application.
- **Docs:** the package format guide points to the sample as the reference example and explains how to pack, including from DDS.

## Capabilities

### New Capabilities
- `vehicle-authoring`: the `tpv` packer (pack, check, DDS conversion) and the sample vehicle (contents, versions, and the up-to-date check).

### Modified Capabilities
- `vehicle-packages`: a new requirement for the built-in sample vehicle, offered by the empty Vehicle Library and by the New Project Vehicle step when no vehicle is installed. It adds to the existing empty-state and Install… requirements without changing them.

## Impact

- **New crate `crates/tp-pack`:** a library with the `tpv` binary. It depends on `tp-vehicles`, `zip`, `image` (with `dds`, already used by tp-render) and `serde_json`.
- **New `examples/vehicles/`:** the SVG and JSON sources and two built `.tpv` files of a few tens of KB. These count as code for `ci:changes`.
- **`crates/tp-app`:**
  - embeds the newest sample package with `include_bytes!`;
  - adds the button to the empty states in `ui/vehicle_dialogs.rs` and `ui/dialogs.rs` (`vehicle_step`);
  - adds new message ids in the four locales.
- **`mise.toml`:** new `pack` and `sample-vehicles` tasks.
- **Docs:** `docs/vehicle-package-format.md`; README mention.
- No change to the package format, the project file format or the existing package validation.
