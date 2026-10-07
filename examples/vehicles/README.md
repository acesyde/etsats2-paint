# Sample vehicle

**TruckPaint Sample Truck** (`community.truckpaint.sample_truck`) is a
fictional truck. It lets you try vehicle projects without game files, and it
is the reference example for package authors. Its templates are original
drawings, not game assets, and are MIT licensed like the rest of the
repository.

| Version | Variants | Textures |
|---|---|---|
| 1.0.0 | Standard cab, High roof | Cabin 4096, Chassis 2048, Accessories 1024 |
| 1.1.0 | Standard cab, High roof | Cabin 4096 (Standard cab layout changed), Chassis 4096, Accessories 1024, Side skirts 1024 (new) |

The built packages sit next to their sources:

```
community.truckpaint.sample_truck-1.0.0.tpv
community.truckpaint.sample_truck-1.1.0.tpv
sample-truck/<version>/vehicle.json
sample-truck/<version>/templates/*.svg
```

## Trying Update Template

1. Install `community.truckpaint.sample_truck-1.0.0.tpv` (Vehicle › Vehicle
   Library… › Install…, or drop it on the window).
2. Create a project on the **Standard cab** and paint something.
3. Install `community.truckpaint.sample_truck-1.1.0.tpv`, then run
   **Vehicle › Update Template…**. The cabin is flagged "Layout changed", the
   chassis artwork is scaled to 4096, and a Side skirts texture is added.

TruckPaint also offers **Install the sample vehicle** (version 1.1.0) while
no vehicle is installed.

## Rebuilding

After editing a source, rebuild both packages:

```sh
mise run sample-vehicles
```

The test suite fails if a committed package no longer matches its sources.
See [the package format](../../docs/vehicle-package-format.md) to make your
own.
