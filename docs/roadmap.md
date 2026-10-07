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
mod nobody has packaged), the player will use **Custom vehicle…**
(`custom-vehicle`, after `paint-job-model`):
1. drop the template files (DDS or PNG);
2. pick the game and the kind (truck or trailer), and describe the paint job
   the way the game does (see "A paint job is a main texture plus
   accessories" below): cabins and their main textures for a truck, the main
   texture for a trailer, and the accessory textures.

TruckPaint builds a local package from them, with the same packing and
validation as `tpv`. That package can later be exported as a `.tpv` and
shared.

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

So TruckPaint's "variant" is really a **truck cabin**. Trailers have no
variants, and accessory textures belong to the vehicle, not to a cabin.

The vehicle model that fits both:

```
vehicle { game, kind (truck | trailer), game path (e.g. "scania.r_2016"),
          name, mod author?, alt_uv, colour_picker }
  cabins[]              trucks only: name, internal names (cabins sharing one
                        layout), and their main texture, or one shared main
                        texture when every cabin has the same layout
  main texture          trailers
  accessory groups[]    name, game accessory ids, texture
texture { id, name, size, template }
```

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
don't redraw". Variants of the same truck often share most of their layout,
so **copy from variant** (same coordinates) is offered. The 3D preview will
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
  (vehicle → variant → texture), the only place to switch textures, with
  Next/Previous Texture on the keyboard; the template settings are in the
  Properties panel.
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
| 1 | `paint-job-model` | Packages and projects follow the game's paint job structure: trucks have cabins (each with its main texture, or one shared main texture), trailers have a main texture, and both have accessory textures shared by the whole vehicle. Packages record the data the mod export needs (game path, cabin internal names, accessory ids, `alt_uv`, `colour_picker`). Projects check cabins for trucks; trailers need no choice. The sample vehicle is redone. Replaces the former `shared-textures`. | `custom-vehicle` and `mod-export` build on it, and the format must be right before others publish packages. |
| 2 | `custom-vehicle` | Custom vehicle… in New Project and Add Vehicle…: pick the game and the kind (truck or trailer), describe cabins or the main texture and the accessories, drop template files (DDS or PNG); TruckPaint packs a local package, which can be exported as a `.tpv`. | The escape hatch for vehicles without a package, now that every project needs a vehicle. |
| 3 | `brand-kit` | Project palette, symbols with instances, shared styles, copy from cabin. | Makes a fleet a shared identity rather than separate drawings. |
| 4 | `mod-export` | Exports the whole fleet as one ready-to-install mod. | The deliverable of the app. Needs the paint job model. |
| 5 | `vehicle-marketplace` | Browses and installs community packages from a GitHub-hosted index. | Once the format is stable. |
| 6 | `distribution` | Release builds and installers. | Once a player can go from vehicle to mod. |

## Shipped

The changes already merged are archived in `openspec/changes/archive/`, and
their requirements are in `openspec/specs/`. The most recent ones are:
- `fleet-projects`: no blank project; a project holds several vehicles of one
  game with their chosen variants, the Vehicles panel tree, Add Vehicle…,
  Variants… and Update Template per vehicle;
- the sample vehicle with the `tpv` packer;
- the vehicle library (vehicle packages, projects and versioned templates).
