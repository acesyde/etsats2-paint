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
  Prefilling them from Paintjob Packer's database will lift the burden
  (planned in `paintjob-importer`).
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
so **copy from cabin** (same coordinates) is offered.

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

The placeholder Object › Mirror to Other Side is gone, and copying a design
to the other flank of a truck is dropped for good: copy/paste and Flip cover
it, and it would need packages to record where each side lies on the
texture.

### Game versions (shipped in `game-versions`)
The project's **Game versions** (Project space, Mod information) are the list the
mod's manifest gets as `compatible_versions[]`, typed as the game writes them
(`1.56.*, 1.57.*`). Export Mod copies them as they are: after a game update,
the author adds the new version and exports a new version of the mod. The
list starts empty (no `compatible_versions`).

Since `mod-settings-in-project`, each version is a chip with a remove
button, and **+ Add** takes versions typed as the game writes them; adding
or removing versions is one undo step. A badly written version is marked on
its chip.

Under the chips, the Project space shows the versions every vehicle
supports, computed from the packages' ranges. Export Mod blocks a badly
written version, a version a vehicle's package doesn't support, and a fleet
whose vehicles have no version in common; the problems about the versions
are also shown under the field.

### No 3D preview (decided after `mod-export`)
The 3D preview was only ever a placeholder, and it is dropped: TruckPaint
won't render vehicle models. The workspace shows the canvas only; the 2D /
3D / Split view modes and their shortcuts are gone (removed in
`remove-3d-preview`).

### One game per project
A project belongs to one game, ETS2 or ATS, set by its first vehicle: ETS2
and ATS mods are separate mods, so one project exports one mod. A player with
fleets in both games has two projects, sharing the personal library.

### The personal library (shipped in `shared-library`)
A company's identity is the same on every fleet, in both games, so it lives
in **one personal library** per user, in the application's data folder
(`library.tplib`), shared by every project. It holds symbols, swatches,
graphic styles and text styles, with the images its symbols use.
- **Add to Library** (context menu of a symbol, a swatch or a style) copies
  the element with everything it uses: a symbol brings its images, the
  swatches it links to and the styles it follows. When the element came
  from the library, the item reads **Update in Library** and replaces the
  library's copy.
- **Import from Library…** (Object menu, the Brand space header and the
  Resources tab) imports checked elements with what they use, as one undo
  step. **Remove from Library** deletes an entry; projects keep their copy.
- **Copies with an origin, not links:** each element remembers the library
  entry it came from (saved in the project, not part of the undo history).
  Editing the library doesn't change projects; editing a project doesn't
  change the library until Update in Library. Live links and "Update from
  Library" may come later.
- **Reuse, no duplicates:** an import reuses what the project already has
  (the same library entry, or a swatch or style with the same name and
  value), never adding "Company red 2" next to "Company red". A symbol is
  reused only by its library entry. A new element whose name is taken gets
  a numbered name ("Logo 2").
- **Paste across projects follows the same rules:** objects pasted into
  another project bring their images, swatches, styles and symbols, even
  after the source project is closed.
- An unreadable library is set aside as a backup and the library starts
  empty, said once. Not done: several libraries, sharing a library file.

### The mod (shipped in `mod-export`)
Export › Export Mod… writes the whole fleet as **one `.scs` file**: one paint
job, in one mod, in the format Paintjob Packer (MIT) has shipped for years.
- **Ready to install:** the Export Mod dialog proposes `<Name>.scs` in the
  game's mod folder when it exists (`Documents/<game>/mod` on Windows, the
  user's data folder on macOS and Linux), else in the last export folder of
  the session, else in Documents; **Change…** picks another file, and an
  existing file is only replaced once the player confirms.
- **Settings live in the project:** Name (shop and Mod Manager), Version,
  Author, Description, Price, Unlock level and an internal name derived from
  the Name (at most 12 characters, or 10 when a truck has several main
  textures, whose paint jobs are `<name>_a`, `<name>_b`…). Since
  `mod-settings-in-project` they are edited in the Project space's Mod
  information column (the internal name under Advanced), each committed
  edit being its own undo step. The Export Mod dialog edits nothing: it
  shows a summary, the destination with Change…, the problems (each with
  Show, which leads to the field or vehicle card where it is fixed) and the
  texture warnings, and never changes the project nor its undo history.
- **Pictures are generated, and can be replaced:** the shop icon (256×64)
  and the Mod Manager image (276×162) are rendered from the first main
  texture, or made from a PNG or JPEG the player chooses (Choose Image…
  or dropped on the preview, in the Project space), stored in the project;
  each change is one undo step.
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
- **Navigation:** textures are switched from the Workshop's Textures tab
  (vehicle → Main textures and Accessories → texture), from the Project
  space, or with Next/Previous Texture on the keyboard; the template is shown
  and dimmed from the status bar.
- **Update Template:** works per vehicle, since each vehicle in a project has
  its own package version.

### The redesign (shipped in `new-design`)
The new interface follows the design spec "TruckPaint 5" (claude.ai design
project *TruckPaint éditeur de livrées*): three spaces, **Project**,
**Workshop** and **Brand**, an Export… button always visible, a thin chrome
around the canvas, and one accent color per meaning (white for the main
action, red for signals and selection, blue for what is linked to the
brand).
- **Constant functionality:** `new-design` moves every existing function to
  its new place and adds none. The mockup's new features are small separate
  changes, done one at a time after it (see "Next changes").
- **Brand space and the personal library:** the Brand space shows the
  project's elements; the personal library is reached through Import from
  Library… and Add to / Update in Library, with no library browser (a
  library view can be its own change later).
- **Where the mockup is not followed:** the main texture "one per cabin / one
  for all" comes from the package and is shown read only; Export Mod always
  writes the whole fleet, with no "current texture / current vehicle" choice
  and no "enable in the mod manager" option; Custom vehicle keeps DDS, PNG
  and SVG templates and typed game data.

## Open questions

None at the moment.

## Next changes, in order

| # | Change | What it does | Why now |
|---|---|---|---|
| 2 | `polishing` | Fixes, ergonomics, wording and performance across the app, with no new feature. | Smooths what the first players will meet, on the new design. |
| 2f | `drop-shadow` | Drop shadow as an appearance property. A new feature, optional and low priority. | Common on livery lettering. |
| 3 | `paintjob-importer` | Prefills the game data of a custom vehicle (game path, cabin internal names, accessory ids) from Paintjob Packer's database. | Custom vehicle asks for game data most players don't know. |
| 4 | `distribution` | Release builds and installers. Includes a macOS `.app` bundle (so the app menu's bold name reads "TruckPaint", not the binary's name), and the manual checks of the drawn title bar on Windows, Linux X11 and Wayland (moving, double-click, resize grips, window controls, menus, the system title bar option and `TRUCKPAINT_SYSTEM_TITLE_BAR=1`). | A player can go from vehicle to mod without building the app. |
| 5 | `marketplace` | Browses and installs community packages from a GitHub-hosted index. | Once players have the app and can make packages easily. |
| 5a | `template-update-impact` | A template update says what it changes and which projects it touches; affected textures become "to check" and affected objects are flagged. Nothing moves automatically. Works with local packages too; builds on the To check state of `texture-status` (shipped). | Updates become frequent once packages come from the marketplace. |
| 5b | `first-run` | A three-step first launch: language and game, vehicles to install from the catalog, first project. | Needs the marketplace catalog. |

## Shipped

The changes already merged are archived in `openspec/changes/archive/`, and
their requirements are in `openspec/specs/`. The most recent ones are:
- `title-bar-menus` (under `polishing`): one 40 px bar at the top of the
  window. On macOS the menus are in the system menu bar (with the app menu:
  About, Settings…, Hide, Quit) and the top bar sits in the window's
  title strip after the traffic lights; the app menu's bold name reads
  "TruckPaint" only from a `.app` bundle, which `distribution` will make.
  On Windows and Linux the app draws the title bar: the TruckPaint mark,
  the menus, the space switcher (centered when it fits), Export… and the
  window controls, with moving, double-click and resize grips; Preferences
  has a Use the system title bar option (applied live), and
  `TRUCKPAINT_SYSTEM_TITLE_BAR=1` forces it as a rescue. The Export items
  are under File, and the breadcrumb is only in the canvas. Windows' Snap
  Layouts flyout and vertically centered traffic lights are left out;
- `brand-edit-preview` (under `polishing`): the Brand space's cards say
  where each element is used (textures and objects for swatches and styles,
  instances and textures for symbols, Unused otherwise); Edit Swatch… and
  Edit Style… open a before/after editor for swatches and graphic styles
  (the Brand space's right panel, a dialog over the Workshop): name, picker
  (fill or stroke, width, opacity for a style), Before / After, hex code,
  and the impact with live thumbnails of the affected textures; nothing is
  applied before Apply to Fleet (one undo step), Cancel or Escape leaves
  the document untouched. Text styles have no editor yet, and the relation
  to the library is left for later;
- `command-palette` (under `polishing`): Cmd/Ctrl+K opens a command palette
  on every screen, even while a field has the focus (also View › Command
  Palette…); it lists every command with its shortcut, menu path (or group:
  Tools, Colors, Symbol), check mark and disabled reason, and the project's
  textures with their state; fuzzy search ignoring case and accents, best
  matches first; the commands recently run from it, for the session; the
  Textures tab's Search textures field opens it limited to textures. The
  menus' structure became data shared by the menu bar and the palette.
  Convert to Symbol still has no shortcut;
- `mod-settings-in-project` (under `polishing`): every mod setting and both
  pictures are edited in the Project space's Mod information column (labels
  above the fields, the internal name under Advanced, pictures chosen or
  dropped), one undo step per committed edit (Enter or leaving commits,
  Escape restores); Game versions as chips with + Add; the problems that
  block the export shown under their fields and first in Before exporting,
  each with Show; the Export Mod dialog only checks and writes: summary,
  destination with Change…, problems with Show, warnings, one Export button,
  and exporting records nothing;
- `texture-status` (under `polishing`): each texture is Empty, Modified or
  To check (flagged by Update Template), computed from the project and never
  stored, shown on the Project space's tiles (dot and label) and in the
  Textures tab (ring, dot or warning icon); the Vehicles header counts the
  Modified and To check textures and filters All / To do / To check; Dismiss
  became Mark as Checked; the inspector with nothing selected shows On this
  texture (objects, symbol instances, off-palette colors, which select the
  objects using them); Before exporting, in the Mod information column and
  among Export Mod's checks, warns about the textures To check and the Empty
  ones without blocking the export. `template-update-impact` can now build
  on To check;
- `new-design`: the interface redone after the "TruckPaint 5" design spec,
  with constant functionality: Geist and JetBrains Mono, one accent per
  meaning; the Project, Workshop and Brand spaces (Cmd/Ctrl+1/2/3) with an
  always-visible Export…; the tool rail, tool options bar and status bar
  (template, snapping, grid, guides); the Textures, Layers and Resources tabs;
  an inspector following the selection, with color and stroke popovers; a
  one-page New Project, a home screen with project thumbnails, and redone
  Export Mod and Vehicle Library dialogs; several shortcuts moved;
- `shared-library`: the personal library shared by every project of both
  games (Add to Library / Update in Library, Import from Library…, Remove
  from Library), elements imported with what they use and without
  duplicates, and paste into another project bringing images, swatches,
  styles and symbols;
- `game-versions`: the project's Game versions, edited in the sidebar and
  copied into the manifest's `compatible_versions[]`, with the versions every
  vehicle supports shown under the field and checked by Export Mod;
- `remove-3d-preview`: the 3D preview placeholder and the view modes are
  gone;
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
