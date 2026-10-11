## Why

To paint an official truck today, a player goes through Custom vehicle:
- they find the templates;
- they type the game path, the cabins' internal names and every accessory id;
- they hope it all matches the game.

Almost nobody knows those ids. The marketplace was meant to solve this with community packages, but it needs hosting, moderation and a release first. It is postponed.

What exists already is enough for the official vehicles:
- **Paintjob Packer** (Carsmaniac, MIT, on GitHub) holds the game data of every official ETS2 and ATS truck and trailer. That's cabins, internal names, accessory ids, `alt_uv` and the colour picker, in `vehicles/{ets,ats}/scs/*.json`.
- **Carsmaniac's template packs** (forum threads for ETS2, t=272386, and ATS, t=288778): every official vehicle as PNG and PSD, one folder per vehicle named as in Paintjob Packer. Each is about 1–1.2 GB. The ETS2 pack is current for game 1.59 (2026-05-28), as its `Info and links.txt` says.

The packs have no license for redistribution, and we don't ask for one. So the player downloads the pack once and tells TruckPaint where it is. TruckPaint embeds a catalog that knows, for each official vehicle, its game data and which pack file is which texture.

## What Changes

This is the first brick: the whole path from choosing a vehicle to a mod loaded in the game, for **one truck and one trailer**:
- the **DAF NGD**, with three cabin layouts (XG+, XG, XF), the sun visor, the sideskirts and the mirrors (standard and camera);
- the **Krone Cool Liner**, a trailer whose body texture is 8192 × 1024, with its chassis accessories.

The other vehicles come in later batches (see Later bricks).

- **Official vehicles in New Project and Add Vehicle.**
  - Each game lists its official vehicles (name, brand, truck or trailer) above the player's own vehicles, marked "Official".
  - Only supported vehicles are listed, with no "coming soon" entries.
  - There are no thumbnails, as for every vehicle in these lists today. (The vehicle images in Paintjob Packer's repository are likely game renders and aren't covered by its license.)
- **The template pack, once per game.**
  - The first time a player picks an official vehicle of a game whose pack isn't known, a dialog explains where the templates come from: Carsmaniac's free pack, about 1.2 GB, downloaded once for every vehicle. It offers **Open the download page** (the forum thread of that game), says to unzip the pack anywhere, and offers **Choose the pack folder…**. A folder can also be dropped on the dialog.
  - The folder is checked (it holds the vehicle's folder with the expected files), then remembered for that game. It can be changed in Preferences.
- **Picking an official vehicle installs it.** TruckPaint builds the vehicle's package from the catalog and the pack's files, with the same packing and validation as Custom vehicle and `tpv`, and installs it like any package. That takes a few seconds, with a progress indicator.
  - The package holds its own templates, so installed vehicles keep working after the pack is moved or deleted.
  - The pack is only needed to install more vehicles, or a newer version of the catalog's entry.
- **From there, nothing new:** the vehicle is a normal installed package. The player chooses cabins, paints, and exports the mod with the right game data, typed by no one.
- **A main texture can also paint accessories.** On the Krone Cool Liner, the pack's only body template is the trailer's main texture, and the game paints the body through two accessory ids (`body.reefer`, `body.reef_13ct`). A package's main texture can now list accessory ids it is also applied to, and the mod export writes them. The player paints the body once.
- **The catalog is embedded in the app.** It is data only (a few KB per vehicle), in the repository (`catalog/ets2/…`), compiled in and checked by tests. For each vehicle it holds:
  - the game data from Paintjob Packer;
  - the validated mapping: which pack file is each main texture and accessory, which cabins and accessory ids each one covers, and the ids left out on purpose (with the reason, for example "no template in the pack");
  - where it comes from: the Paintjob Packer commit and the pack version.

  About credits Paintjob Packer with its MIT notice, and Carsmaniac for the templates.
- **`tpv catalog`, the tool that keeps it up to date** (in `tp-pack`, next to `tpv pack` and `tpv check`):
  - `tpv catalog sync` reads Paintjob Packer's official vehicles (`mod: false`) from a local copy at a given commit, updates the game data of the catalog's vehicles, and lists what changed: ids added or removed, cabins, vehicles the catalog doesn't have yet;
  - `tpv catalog check <pack folder>` checks that every file the catalog references exists at the expected size, that every cabin and accessory id is covered or left out on purpose, and lists the pack's files no entry uses.
- **The DAF NGD and Krone Cool Liner mappings are made and validated by hand** in this brick, including in the game. They become the examples for the matching tool of the next brick. The DAF's main textures match the pack by name ("Cabin A (XG+).png" is cabin A, XG+). Accessories don't map one to one: Paintjob Packer groups ids by game part ("Mirrors": `mirror.semi`, `mirror.cam_semi`), while the pack has one file per template ("Main Mirrors - Standard", "Main Mirrors - Camera").

**Later bricks** (not in this change):
- `official-vehicles` batches: `tpv catalog match`, which proposes a vehicle's mapping from the pack's names and marks doubtful cases to validate, then the other official vehicles of ETS2 and ATS, in batches.
- `first-run` can offer the official catalog at first launch, instead of the marketplace's.
- `template-update-impact` applies when a new pack comes with a game update: `tpv catalog check` tells which vehicles changed, and the player's projects get the "To check" flow.

Non-goals: ATS vehicles (none in this brick, so no ATS pack is asked for yet); thumbnails before installation; changes to the Vehicle Library; reading the PSD files; letting Custom vehicle set a main texture's extra accessory ids.

The roadmap already reflects this order (updated by `rectangular-textures`): `official-vehicles` replaces `paintjob-importer`, and the marketplace is postponed with no date.

## Capabilities

### New Capabilities
- `official-vehicles`:
  - the embedded catalog: what an entry holds, which vehicles are listed, where they come from, and the credits;
  - the template pack: the dialog with the download page, choosing and checking the folder, remembering it per game;
  - installing an official vehicle as a local package that doesn't need the pack afterwards;
  - the `tpv catalog sync` and `tpv catalog check` commands.

### Modified Capabilities
- `start-screen`: the New Project vehicle list shows the game's official vehicles above the installed ones, and picking one installs it.
- `vehicle-projects`: Add Vehicle… lists the official vehicles too; the game data recorded with a main texture includes the accessory ids it also paints.
- `vehicle-packages`: a main texture can list accessory ids it also paints (format and validation).
- `mod-export`: a main texture's extra accessory ids get their own entry, pointing to that texture.
- `app-preferences`: the template pack folder of each game is saved, shown and changeable in Preferences.

## Impact

- **New data:**
  - `catalog/ets2/daf.2021.json` and `catalog/ets2/krone.coolliner.json`, plus Paintjob Packer's license notice;
  - an embedded copy of the catalog, like the sample vehicles (`vehicles::SAMPLES`).
- **tp-vehicles:** the catalog types and loading; the optional `also_paints` of a main texture, validated.
- **tp-pack:** building a package from an entry and a pack folder; `tpv catalog sync` (reads Paintjob Packer's JSON from a local copy) and `tpv catalog check`.
- **tp-core / tp-file:** a main texture's extra accessory ids recorded in the project and saved.
- **tp-app:**
  - the official list in New Project and Add Vehicle;
  - the pack dialog (links to the two forum threads, folder picker, drop, check);
  - installing on a background thread through the existing vehicle library;
  - the pack folders in Preferences and in the saved preferences;
  - the mod export of extra accessory ids; the credits in About;
  - strings in en/fr/de/es.
- **Tests:**
  - catalog entries are complete and build valid packages from placeholder images;
  - a test that runs only when a pack is present on the machine (ignored in CI) installs both vehicles from it and exports a mod;
  - the in-game check of the mod is manual.
- **docs:** `docs/vehicle-package-format.md` (`also_paints`), `docs/roadmap.md` at ship time.
