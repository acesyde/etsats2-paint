# Vehicle package format

A **vehicle package** describes one truck or one trailer for TruckPaint:
- its **paint job**, structured as the game structures it;
- the *template* of each texture, the image showing where each part of the vehicle lands on the texture.

Painters install packages, and TruckPaint never ships game assets.

A package is a ZIP file with the `.tpv` extension:

```
volvo_fh16-1.3.0.tpv
├── vehicle.json          the manifest (below)
├── preview.png           optional picture shown in the library
└── templates/
    ├── globetrotter.png  one template per texture (PNG or SVG)
    ├── globetrotter_xl.png
    └── mirrors.svg
```

Packages contain only data, JSON and images, never code.

## How the game structures a paint job

In Euro Truck Simulator 2 and American Truck Simulator, a paint job is:
- **one or more main textures:**
  - a truck whose cabins have different layouts has one main texture per cabin layout, each used by the cabins listed for it;
  - a truck whose cabins all share one layout, and a trailer, have a single main texture;
- **accessory textures,** shared by the whole vehicle. Each covers a group of accessories: mirrors, side skirts, bumpers, or, for a trailer, its bodies and lengths.

A painter's project paints the main textures they choose and the accessories they choose. TruckPaint's mod export will turn each part into the game's definitions.

## Manifest

A truck whose cabins have two layouts:

```json
{
  "format": 1,
  "id": "scs.volvo.fh16_2012",
  "version": "1.3.0",
  "name": "Volvo FH16 2012",
  "brand": "Volvo",
  "kind": "truck",
  "authors": ["Jane Doe"],
  "license": "CC-BY-4.0",
  "homepage": "https://github.com/jdoe/truckpaint-volvo-fh16",
  "description": "Templates for the Volvo FH16 2012, all cabins.",
  "preview": "preview.png",
  "game": {
    "id": "ets2",
    "versions": ">=1.50, <1.54",
    "path": "volvo.fh16",
    "requires": []
  },
  "paint_job": {
    "main": [
      {
        "id": "globetrotter",
        "name": "Globetrotter",
        "game_ids": ["globetrotter", "globetrotter_8x4"],
        "texture": { "size": 4096, "template": "templates/globetrotter.png", "layout_version": 2 }
      },
      {
        "id": "globetrotter_xl",
        "name": "Globetrotter XL",
        "game_ids": ["globetrotter_xl"],
        "texture": { "size": 4096, "template": "templates/globetrotter_xl.png", "layout_version": 1 }
      }
    ],
    "accessories": [
      {
        "id": "mirrors",
        "name": "Mirrors",
        "game_ids": ["mirror.painted", "s_mirror.painted"],
        "texture": { "size": 1024, "template": "templates/mirrors.svg", "layout_version": 1 }
      }
    ]
  }
}
```

A truck whose cabins all share one layout has a single main texture. Without `game_ids`, it is painted on every cabin:

```json
{
  "format": 1,
  "id": "community.jdoe.mighty_hauler",
  "version": "1.0.0",
  "name": "Mighty Hauler",
  "brand": "JDoe",
  "kind": "truck",
  "game": { "id": "ats", "versions": ">=1.50", "path": "jdoe.hauler", "alt_uv": true },
  "paint_job": {
    "main": [
      {
        "id": "cabin",
        "name": "Cabin",
        "texture": { "size": 4096, "template": "templates/cabin.png", "layout_version": 1 }
      }
    ]
  }
}
```

A trailer has a single main texture, and its bodies and lengths are accessories:

```json
{
  "format": 1,
  "id": "community.jdoe.curtainsider",
  "version": "1.0.0",
  "name": "Curtainsider",
  "brand": "JDoe",
  "kind": "trailer",
  "game": { "id": "ets2", "versions": ">=1.50", "path": "jdoe.curtain" },
  "paint_job": {
    "main": [
      {
        "id": "base",
        "name": "Base",
        "texture": { "size": 2048, "template": "templates/base.png", "layout_version": 1 }
      }
    ],
    "accessories": [
      {
        "id": "body_13_6",
        "name": "Curtain body 13.6 m",
        "game_ids": ["body.curtain_136"],
        "texture": { "size": 4096, "template": "templates/body_13_6.png", "layout_version": 1 }
      }
    ]
  }
}
```

### Identity

| Field | Required | Meaning |
|---|---|---|
| `format` | yes | Package format version, currently `1`. |
| `id` | yes | Stable identifier: lowercase words separated by dots, at least two (`a`–`z`, `0`–`9`, `_`, `-`). Use `scs.<brand>.<model>` for vehicles of the base games and `community.<author>.<name>` for community vehicles. `custom.<brand>.<name>` is used by vehicles made in TruckPaint (see [Custom vehicles](#custom-vehicles)). Never change it between versions. |
| `version` | yes | [Semantic version](https://semver.org) of the package (`1.3.0`). Increase it for every release. |
| `name`, `brand` | yes | Shown in the library. |
| `kind` | yes | `truck` or `trailer`. Filters the library and chooses the game folder of the mod. |
| `authors`, `license`, `homepage`, `description` | no | Shown in the library. |
| `preview` | no | Image shown in the library. |

### `game`: what the package targets in the game

| Field | Required | Meaning | In the mod |
|---|---|---|---|
| `id` | yes | `ets2` or `ats`. | Which game the mod is for. |
| `versions` | yes | Game versions the templates match, as a version range (`>=1.50, <1.54`, `^1.53`). | The manifest's compatible versions. |
| `path` | yes | The vehicle's path in the game's definitions (`scania.r_2016`): words of `a`–`z`, `0`–`9` and `_` separated by dots. | `def/vehicle/<truck\|trailer_owned>/<path>/paint_job/…` |
| `alt_uv` | no (`false`) | The paint job uses the vehicle's alternate UV set (some ATS trucks). | `alternate_uvset: true` |
| `colour_picker` | no (`false`) | The paint job lets the player pick a base color (mostly bus mods). | The paint job's color settings. |
| `requires` | no | Mods the vehicle depends on: `{ "name": "…", "version": ">=2.1" }`. | The mod's dependencies. |

### `paint_job`: the parts

`paint_job.main` lists **one or more** main textures, and `paint_job.accessories` lists **zero or more** accessory groups. Every part, main texture or accessory, has the same fields:

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | Unique within the vehicle, stable across versions (`globetrotter`, `mirrors`). Projects follow textures by this id. |
| `name` | yes | Shown in the sidebar and the dialogs. |
| `game_ids` | see below | What the game calls the things this part covers, as words of `a`–`z`, `0`–`9` and `_` separated by dots. |
| `texture.size` | yes | Square size in pixels: 256, 512, 1024, 2048, 4096 or 8192. |
| `texture.template` | yes | Path of the template image inside the package, PNG or SVG. |
| `texture.layout_version` | yes | Positive integer. **Increase it whenever the texture's layout changes**, for example after a game update. Projects updated to your new version flag this texture so painters check their artwork. |

The rules for `game_ids`:

| | `game_ids` holds | Required | In the mod |
|---|---|---|---|
| Main texture | The internal names of the cabins that share its layout (`highline`, `highline_8x4`). | When there are several main textures. A single main texture without `game_ids` is painted on every cabin. | `suitable_for[]` of the paint job |
| Accessory | The accessory ids it covers (`mirror.painted`, `sideskirt.4x2_p`). | Always, at least one. | `acc_list[]` of its `simple_paint_job_data` |

A game id appears at most once among the main textures, and at most once among the accessories.

Unknown fields are ignored, so a package can already carry data for later TruckPaint versions.

## Limits

- Entry paths are relative and never contain `..`.
- At most 512 MB uncompressed in total; template images at most 16384 px on a side.
- Template images are drawn stretched to the texture's square size: make them square.

## Packing a package

The `tpv` command builds a package from a folder holding a `vehicle.json` and the files it references. It checks the package exactly as TruckPaint does when installing. In this repository, run it through mise:

```sh
mise run pack my-truck/                 # writes <id>-<version>.tpv
mise run pack my-truck/ -o my-truck.tpv
cargo run -p tp-pack --bin tpv -- check my-truck.tpv
```

- **Only referenced files are packed:** the manifest, each part's `texture.template` and the `preview`. Other files in the folder (notes, `.psd` sources) are listed as ignored. Hidden files are skipped silently.
- **DDS templates are converted to PNG.** Point `template` at the DDS file as SCS distributes it (`templates/cabin.dds`). The package gets `templates/cabin.png` with the same pixels, and its manifest is updated. Your folder is never modified.
  - Supported: BC1, BC2 and BC3 (DXT1/3/5, also with a DX10 header), and uncompressed 24 or 32-bit RGB(A).
  - Other formats are refused, and the message names the texture.
- **Reproducible:** packing the same folder twice gives identical bytes.
- **Exit status:** 0 on success. On failure: 1, with the reason, and no file written.

Templates of the base games belong to SCS Software. Pack them for your own use, but check the license before sharing a package.

## Custom vehicles

When no package exists for a vehicle, a painter can make one in TruckPaint, without writing a manifest: **Custom vehicle…** in New Project or in Add Vehicle…, or **Custom Vehicle…** in the Vehicle Library.

1. Fill in the vehicle: name, brand, kind (truck or trailer), game, game path and, optionally, the supported game versions (empty: any version), the alternate UV set and the colour picker.
2. Drop the template files on the dialog, or use Add Templates…: PNG, SVG, or DDS in the formats `tpv` converts. Each file becomes one texture. Its name comes from the file name and its size from the image's width. The first one is a main texture, the others are accessories. Change any of them as needed.
3. Enter the game ids, as for any package:
   - for a truck with several main textures, the internal names of the cabins of each one;
   - for each accessory, the accessory ids it covers.

   These names come from the vehicle's definitions under `def/vehicle/truck/<game path>/` (or `trailer_owned/`), in the game's or the mod's archives. They are required: a mod can't be exported without them.
4. Click **Create**. TruckPaint packs the package the way `tpv pack` does, DDS templates included, checks it the same way, and installs it.

A custom vehicle has the id `custom.<brand>.<name>` and version 1.0.0. Each word is the brand or the name, lowercased, with other characters replaced by `_`. Its textures get ids made from their names the same way.

**After a game update,** use **New Version…** on the vehicle in the Vehicle Library. The dialog opens with the newest version's data and templates, and proposes the next minor version (1.0.0 gives 1.1.0):
- replace the templates that changed (Replace…, or drop a file on a texture). Each replaced template gets a higher `layout_version`, so Update Template flags it in projects;
- a texture keeps its id when it is renamed. Removed textures are left out of the new version, and new ones get new ids.

The id, kind and game of a custom vehicle can't change.

**Sharing:** **Export…** in the Vehicle Library saves any installed version as a `.tpv` file, unchanged. Another painter installs it like any package. Templates from the base games belong to SCS Software: check their license before sharing.

## Reference examples

[examples/vehicles/](../examples/vehicles/) holds two fictional vehicles with original SVG templates. Use them as a starting point.
- **TruckPaint Sample Truck** (`community.truckpaint.sample_truck`) has two main textures (two cabin layouts) and accessories, in versions 1.0.0 and 1.1.0. Version 1.1.0 changes a cabin layout, enlarges an accessory and adds one, which shows how to version a package.
- **TruckPaint Sample Trailer** (`community.truckpaint.sample_trailer`) has a single main texture and its bodies as accessories.

## Versions and game updates

When SCS updates a game and a vehicle's textures change:

1. update the templates that changed and increase their `layout_version`;
2. adjust `game.versions`;
3. release the package with a higher `version`.

Painters install the new version alongside the old one. Their projects keep working with the templates they embed. **Vehicle › Update Template…** moves a project's vehicle to the new version:
- it flags the textures whose layout changed;
- it offers the textures that are new in that version.
