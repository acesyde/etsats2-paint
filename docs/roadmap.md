# Roadmap and product decisions

This page records what TruckPaint is for, the product decisions taken so far,
the questions still open, and the order of the next changes. Read it before
proposing a change. Update it when a decision is taken or a change ships.

## What TruckPaint is

TruckPaint is a **livery editor for Euro Truck Simulator 2 and American Truck
Simulator**. A player designs the paint jobs of their fleet (trucks, their
cabins and accessories, and trailers) and exports them as a game mod.

It is **not** a general vector editor. A drawing that isn't tied to a vehicle
has nowhere to go: it has no texture names or sizes, no template, and no place
in a mod. The vector tools exist to serve vehicle textures, and competing with
Illustrator or Inkscape is not a goal.

## Decisions

### Every project has a vehicle
The blank-texture project goes away. New Project starts by choosing a vehicle.

When no package exists for a vehicle (a truck just released, or a community
mod nobody has packaged), the player uses **Custom vehicle…** (shipped in
`custom-vehicle`):
1. drop the template files (DDS, PNG or SVG);
2. pick the game and the kind (truck or trailer), and describe the paint job
   the way the game does (see "A paint job is a main texture plus
   accessories" below): cabins and their main textures for a truck, the main
   texture for a trailer, and the accessory textures.

TruckPaint builds a local package from them, with the same packing and
validation as `tpv`. Export… in the Vehicle Library saves it as a `.tpv`,
to share.

- **The game data is required,** as `tpv` requires it: the game path, the
  cabin internal names when there are several main textures, and the
  accessory ids. A package without them couldn't be exported as a mod.
  Prefilling them from Paintjob Packer's database would lift the burden
  later.
- **New Version…** makes the next version of a custom vehicle (id
  `custom.<brand>.<name>`) after a game update: replacing a template raises
  its layout version, so Update Template works as for any package.

Trying the app without game files is covered by the built-in sample vehicle.

### A project is a fleet
A project holds **several trucks with the cabins the player chooses, and
several trailers**: the fleet of one company. It exports as one mod. A player
who drives two or three trucks and owns five trailers designs the identity
once, not once per vehicle.

Painting every cabin of a truck is not required. Cabins left out simply don't
get the paint job.

### A paint job is a main texture plus accessories
This is how ETS2 and ATS structure paint jobs (studied in Paintjob Packer,
MIT, `Carsmaniac/paintjob-packer`). A paint job is **one main texture plus
one texture per accessory group**, shipped together; the game uses whichever
applies to the vehicle as configured.

| | Truck | Owned trailer |
|---|---|---|
| Main texture | One **per cabin** when cabins have different layouts (`separate_paint_jobs`), else **one for every cabin** | **One** ("Base") |
| Accessory textures (deflectors, sideskirts, mirrors… / bodies, lengths, bumpers…) | One per group, **shared by every cabin** | One per group. Body types and lengths are accessory groups, **not variants** |
| What the player picks | Which cabins to paint | Nothing: everything ships together |
| In the mod | `def/vehicle/truck/<path>/paint_job/…`, `suitable_for` = cabin internal names | `def/vehicle/trailer_owned/<path>/paint_job/…` |

So what TruckPaint first called a "variant" was really a **truck cabin
layout**. Trailers have no variants, and accessory textures belong to the
vehicle, not to a cabin.

The vehicle model that fits both, with no rule specific to trucks or trailers
(shipped in `paint-job-model`; see `docs/vehicle-package-format.md`):

```
vehicle   { id, version, name, brand, kind (truck | trailer), credits }
  game      { id (ets2 | ats), versions, path (e.g. "scania.r_2016"),
              alt_uv, colour_picker, requires }
  paint_job
    main[]        1..N: one per cabin layout, or one for the whole vehicle
    accessories[] 0..N: accessory groups shared by the whole vehicle
part      { id, name, game_ids, texture { size, template, layout_version } }
```

A part's `game_ids` are the cabin internal names of a main texture (none:
every cabin) or the accessory ids of an accessory. A trailer is simply a paint
job with one main texture.

**Data the mod export will need, recorded in packages from now on:** the game
path, the kind, cabin internal names, accessory ids, `alt_uv` (some ATS
trucks) and `colour_picker` (mostly bus mods). Without them a mod can't write
`def/vehicle/…`, `suitable_for` or `acc_list`. Paintjob Packer's vehicle
database (MIT) lists them for about 180 SCS and community vehicles and can
prefill them later; its templates belong to SCS and are never shipped.

### What is shared: elements, not placement
Texture layouts (UVs) are fixed by each vehicle's 3D model and differ from one
vehicle to the next. The same design can't be copied to the same spot on
another vehicle. What a project shares:
- **brand palette:** the company colors, at project level;
- **symbols:** a logo, lettering or stripes, defined once and placed as
  instances on each texture. Editing the symbol updates every instance, and
  each instance keeps its own position, scale and rotation;
- **styles:** gradients, strokes, text styles.

What remains per vehicle is **placing** the elements. The promise is "reposition,
don't redraw". The cabin layouts of a truck often share most of their texture,
so **copy from cabin** (same coordinates) is offered. The 3D preview will
make placement easier later.

How sharing works (palette, styles and copy from cabin shipped in
`brand-kit`, symbols in `symbols`):
- **Linked, not copied:** a color picked from a palette swatch, and an object
  given a style, follow later edits of the swatch or the style on every
  texture. A link holds while the value still equals its source: changing an
  object's own look detaches it. A gradient's position stays the object's
  own, so moving an object keeps its style.
- **No partial overrides:** an object follows a whole style or none.
- **A symbol is edited in its own view** (like a temporary texture, with
  every tool), not in place through an instance's transform.
- **An instance has no override:** only its position, size, rotation, flip,
  opacity, visibility and lock are its own. A variant is a duplicated symbol,
  or a detached instance (a plain group).
- **Symbols don't nest:** converting a selection that holds an instance
  detaches it first.
- **Deleting a used symbol** turns its instances into groups that look the
  same.
- Files with symbols don't open in builds made before `symbols`.

### Flipping and the other side of the truck
Flip Horizontal and Flip Vertical (shipped in `flip-objects`) mirror the
selection across the center of its bounds, along the texture's axes; several
objects are mirrored as one block. **Images and texts are mirrored too**: a
logo faces the other way, and a text reads backwards, still editable.

The placeholder Object › Mirror to Other Side is gone. Copying a design to
the other flank of a truck needs to know where each side lies on the
texture, which packages don't record. It will come back only as a change
that adds that left/right mapping to packages.

### One game per project
A project belongs to one game, ETS2 or ATS, set by its first vehicle: ETS2
and ATS mods are separate mods, so one project exports one mod. A player with
fleets in both games has two projects (sharing a symbol library, later).

### The mod (shipped in `mod-export`)
Export › Export Mod… writes the whole fleet as **one `.scs` file**: one paint
job, in one mod, in the format Paintjob Packer (MIT) has shipped for years.
- **Ready to install:** the save dialog opens in the game's mod folder when
  it exists (`Documents/<game>/mod` on Windows, the user's data folder on
  macOS and Linux), else in the last export folder of the session.
- **Settings live in the project:** Name (shop and Mod Manager), Version,
  Author, Description, Price, Unlock level and an internal name derived from
  the Name (at most 12 characters, or 10 when a truck has several main
  textures, whose paint jobs are `<name>_a`, `<name>_b`…). Editing them and
  exporting is one undo step. The sidebar's Version is the mod version.
- **Pictures are generated, and can be replaced:** the shop icon (256×64)
  and the Mod Manager image (276×162) are rendered from the first main
  texture, or made from a PNG or JPEG the player chooses, stored in the
  project.
- **Projects record the game data** of their vehicles (game path, versions,
  alternate UV set, colour picker, required mods, and the game ids and
  main-texture position of each texture), so a project exports on a
  computer without its packages. Older files get it from the installed
  package version when they open; without it, only the export is blocked.
- **Main textures keep their order:** a truck's paint jobs are lettered by
  the position of their main texture in the package, so package authors
  must not reorder main textures between versions.
- Not done: an unpacked folder, Steam Workshop files, several paint jobs per
  project, colour-mask paint jobs, `compatible_versions` in the manifest.

### How a fleet works (shipped in `fleet-projects`)
- **Scale:** a fleet can reach 30 to 40 textures, mostly 4096². The canvas is
  vector-based and only draws the active texture, and its caches drop what is
  not shown, so no on-demand loading is needed.
- **Navigation:** the sidebar on the left shows the project and a tree
  (vehicle → Main textures and Accessories → texture), the only place to
  switch textures, with Next/Previous Texture on the keyboard; the template
  settings are in the Properties panel.
- **Update Template:** works per vehicle, since each vehicle in a project has
  its own package version.

## Open questions

- **Game versions:** the sidebar's Project section shows them empty and read
  only; a later feature stores and edits them, and could feed the manifest's
  `compatible_versions`. (The project version is the mod version since
  `mod-export`.)
- **Symbol library across projects:** reusing a logo, swatches or styles in
  several projects, or in projects for both games. Probably after `symbols`.

## Next changes, in order

| # | Change | What it does | Why now |
|---|---|---|---|
| 1 | `vehicle-marketplace` | Browses and installs community packages from a GitHub-hosted index. | Once the format is stable. |
| 2 | `distribution` | Release builds and installers. | A player can now go from vehicle to mod. |

## Shipped

The changes already merged are archived in `openspec/changes/archive/`, and
their requirements are in `openspec/specs/`. The most recent ones are:
- `mod-export`: Export Mod… writes the fleet as one `.scs` mod (definitions,
  DDS textures, shop icon, Mod Manager entry) into the game's mod folder;
  mod settings and pictures saved in the project; projects record the game
  data of their vehicles;
- `flip-objects`: Flip Horizontal (Shift+H) and Flip Vertical (Shift+V) in
  the Object menu, the canvas context menu and the Transform panel; images
  and texts can be mirrored (also by dragging a handle past the opposite
  side); Mirror to Other Side removed;
- `symbols`: Convert to Symbol, the Symbols panel (Place, Edit, Rename,
  Duplicate, Delete), a symbol edited in its own view with Done, Detach
  Instance; instances move, resize, rotate and flip like objects and follow
  every edit of their symbol on every texture;
- `brand-kit`: named palette swatches that fills, strokes and gradient stops
  link to (Edit Swatch… recolors the fleet); linked graphic and text styles
  in a new Styles panel; Copy From Cabin… between the main textures of a
  truck;
- `custom-vehicle`: Custom vehicle… in New Project, Add Vehicle… and the
  Vehicle Library builds and installs a package from template files (DDS,
  PNG or SVG) and the paint job described by the player; New Version… for
  custom vehicles; Export… of any installed version as a `.tpv`;
- `paint-job-model`: packages and projects follow the game's paint job
  structure (`game` and `paint_job` with main textures and accessories, and
  the game data the mod export needs); projects check main textures and
  accessories (Textures…); the sample truck is redone and a sample trailer
  added;
- `fleet-projects`: no blank project; a project holds several vehicles of one
  game with their chosen variants, the Vehicles panel tree, Add Vehicle…,
  Variants… and Update Template per vehicle;
- the sample vehicle with the `tpv` packer;
- the vehicle library (vehicle packages, projects and versioned templates).
