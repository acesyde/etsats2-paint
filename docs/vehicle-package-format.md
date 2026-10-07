# Vehicle package format

A **vehicle package** describes one truck or one trailer for TruckPaint: its
textures and their *templates*, the images showing where each part of the
vehicle lands on the texture. Painters install packages; TruckPaint never
ships game assets.

A package is a ZIP file with the `.tpv` extension:

```
volvo_fh16-1.3.0.tpv
├── vehicle.json          the manifest (below)
├── preview.png           optional picture shown in the library
└── templates/
    ├── cabin.png         one template per texture (PNG or SVG)
    ├── chassis.png
    └── accessories.svg
```

Packages contain only data, JSON and images, never code.

## Manifest

```json
{
  "format": 1,
  "id": "scs.volvo.fh16_2012",
  "version": "1.3.0",
  "name": "Volvo FH16 2012",
  "brand": "Volvo",
  "kind": "truck",
  "game": "ets2",
  "game_versions": ">=1.50, <1.54",
  "authors": ["Jane Doe"],
  "license": "CC-BY-4.0",
  "homepage": "https://github.com/jdoe/truckpaint-volvo-fh16",
  "description": "Templates for the Volvo FH16 2012, all cabins.",
  "preview": "preview.png",
  "requires": [],
  "variants": [
    {
      "id": "globetrotter_xl",
      "name": "Globetrotter XL",
      "textures": [
        {
          "id": "cabin",
          "name": "Cabin",
          "size": 4096,
          "template": "templates/cabin.png",
          "layout_version": 2
        },
        {
          "id": "chassis",
          "name": "Chassis",
          "size": 2048,
          "template": "templates/chassis.png",
          "layout_version": 1
        },
        {
          "id": "accessories",
          "name": "Accessories",
          "size": 1024,
          "template": "templates/accessories.svg",
          "layout_version": 1,
          "export": { "note": "reserved for the mod export" }
        }
      ]
    }
  ]
}
```

| Field | Required | Meaning |
|---|---|---|
| `format` | yes | Package format version, currently `1`. |
| `id` | yes | Stable identifier: lowercase words separated by dots, at least two (`a`–`z`, `0`–`9`, `_`, `-`). Use `scs.<brand>.<model>` for vehicles of the base games and `community.<author>.<name>` for community vehicles. Never change it between versions. |
| `version` | yes | [Semantic version](https://semver.org) of the package (`1.3.0`). Increase it for every release. |
| `name`, `brand` | yes | Shown in the library. |
| `kind` | yes | `truck` or `trailer`. |
| `game` | yes | `ets2` or `ats`. |
| `game_versions` | yes | Game versions the templates match, as a version range (`>=1.50, <1.54`, `^1.53`). |
| `authors`, `license`, `homepage`, `description` | no | Shown in the library. |
| `preview` | no | Image shown in the library. |
| `requires` | no | Mods the vehicle depends on: `{ "name": "…", "version": ">=2.1" }`. |
| `variants` | yes | At least one. A variant is a cabin, chassis or body with its own set of textures. |

Each texture of a variant:

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | Unique within the variant, stable across versions (`cabin`). |
| `name` | yes | Shown on the texture tabs. |
| `size` | yes | Square size in pixels: 256, 512, 1024, 2048, 4096 or 8192. |
| `template` | yes | Path of the template image inside the package, PNG or SVG. |
| `layout_version` | yes | Positive integer. **Increase it whenever the texture's layout changes**, for example after a game update: projects updated to your new version flag this texture so painters check their artwork. |
| `export` | no | Reserved for the mod export; kept as-is. |

Unknown fields are ignored, so a package can already carry data for later
TruckPaint versions.

## Limits

- Entry paths are relative and never contain `..`.
- At most 512 MB uncompressed in total; template images at most 16384 px on a side.
- Template images are drawn stretched to the texture's square size: make them square.

## Versions and game updates

When SCS updates a game and a vehicle's textures change:

1. update the templates that changed and increase their `layout_version`;
2. adjust `game_versions`;
3. release the package with a higher `version`.

Painters install the new version alongside the old one. Their projects keep
working with the templates they embed, and **Vehicle › Update Template…**
moves a project to the new version, flagging the textures whose layout
changed.
