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
mod nobody has packaged), the player will use **Custom vehicle…** (next
change, `custom-vehicle`):
1. drop the template files (DDS or PNG);
2. name each texture and pick the game.

TruckPaint builds a local package from them, with the same packing and
validation as `tpv`. That package can later be exported as a `.tpv` and
shared.

Trying the app without game files is covered by the built-in sample vehicle.

### A project is a fleet
A project holds **several trucks with the variants the player chooses, and
several trailers**: the fleet of one company. It exports as one mod. A player
who drives two or three trucks and owns five trailers designs the identity
once, not once per vehicle.

Painting every variant of a truck is not required. Variants left out simply
don't get the paint job.

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
- **Navigation:** the Vehicles panel shows a tree (vehicle → variant →
  texture); the tabs above the canvas list the active variant's textures.
- **Update Template:** works per vehicle, since each vehicle in a project has
  its own package version.

## Open questions

- **Textures shared between variants:** in ETS2, some cabins of a truck use
  the same texture (same layout) and others don't (Paintjob Packer's
  `separate_paint_jobs`). The package format can't express a texture shared
  by several variants yet, so the player would paint the same texture several
  times. This must be settled before packages are published on the
  marketplace.
- **Symbol library across projects:** reusing a logo in several projects, or
  in projects for both games. Probably after `brand-kit`.

## Next changes, in order

| # | Change | What it does | Why now |
|---|---|---|---|
| 1 | `custom-vehicle` | Custom vehicle… in New Project and Add Vehicle…: drop template files (DDS or PNG), name the textures and pick the game; TruckPaint packs a local package, which can be exported as a `.tpv`. | The escape hatch for vehicles without a package, now that every project needs a vehicle. |
| 2 | `shared-textures` | Variants can share a texture in the package format. | Fixes the format before others publish packages. |
| 3 | `brand-kit` | Project palette, symbols with instances, shared styles, copy from variant. | Makes a fleet a shared identity rather than separate drawings. |
| 4 | `mod-export` | Exports the whole fleet as one ready-to-install mod. | The deliverable of the app. Needs the fleet model and shared textures. |
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
