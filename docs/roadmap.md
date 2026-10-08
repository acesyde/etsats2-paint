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

### One game per project
A project belongs to one game, ETS2 or ATS, set by its first vehicle: ETS2
and ATS mods are separate mods, so one project exports one mod. A player with
fleets in both games has two projects (sharing a symbol library, later).

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

- **Project version and game versions:** the sidebar's Project section shows
  them empty and read only; a later feature stores and edits them (they will
  feed the mod's manifest).
- **Symbol library across projects:** reusing a logo in several projects, or
  in projects for both games. Probably after `brand-kit`.

## Next changes, in order

| # | Change | What it does | Why now |
|---|---|---|---|
| 1 | `brand-kit` | Project palette, symbols with instances, shared styles, copy from cabin. | Makes a fleet a shared identity rather than separate drawings. |
| 2 | `mod-export` | Exports the whole fleet as one ready-to-install mod. | The deliverable of the app. Needs the paint job model. |
| 3 | `vehicle-marketplace` | Browses and installs community packages from a GitHub-hosted index. | Once the format is stable. |
| 4 | `distribution` | Release builds and installers. | Once a player can go from vehicle to mod. |

## Shipped

The changes already merged are archived in `openspec/changes/archive/`, and
their requirements are in `openspec/specs/`. The most recent ones are:
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
