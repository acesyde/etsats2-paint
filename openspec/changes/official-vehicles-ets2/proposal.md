## Why

The first brick of `official-vehicles` proved the whole path, from picking a vehicle to a mod loaded in the game. It did so on two vehicles, the DAF NGD and the Krone Cool Liner, and its in-game check passed (user, 2026-10-11). Every other official vehicle still goes through Custom vehicle, with ids nobody knows. The ETS2 template pack (game 1.59) and Paintjob Packer (commit `a7a7ed3`) already hold everything needed for the 64 other official ETS2 vehicles. Mapping them by hand the way the first two were mapped (several thousand accessory ids) would be slow and error-prone. So this change adds a tool that proposes the mapping, then uses it to bring in **every official ETS2 vehicle** at once.

## What Changes

- **`tpv catalog match`.** For each official vehicle of Paintjob Packer that the catalog doesn't have yet, it:
  - finds its folder in the pack, including shared folders (`Scania R & S`, `Kässbohrer K.SCS & K.SCX`, `Krone Profi Liner & Paper Liner`, `Feldbinder TSA`) and renamed ones (`Scania S 2024e` for the Scania S BEV);
  - proposes the main textures:
    - trucks: `Cabin A (XG+).png` and `Cabin A Aero (Aero Globetrotter XXL).png` are matched to a cabin by its name, `Cabin 8x4 (GigaSpace).png` by its code `8`, and a single file named after the vehicle covers every cabin;
    - trailers: see the multi-body point below;
  - proposes the accessory textures by matching file names to Paintjob Packer's groups ("Main Mirrors - Camera" ↔ "Mirrors": `mirror.cam_semi`). It also splits a group across several files when the variants show in the names ("(4x)", "(6x Long)", "Standard" / "Camera");
  - leaves out, with the reason "no template in the pack", the groups that no file matches.

  It writes a draft entry per vehicle and prints a report: what it is sure of, the **doubtful matches** to check by hand (each with its reason), the ids left uncovered and the pack files nothing uses. It never changes an existing entry. An entry with uncovered ids stays incomplete, so the app doesn't list it and the tests refuse it until someone resolves it.
- **Trailers with several bodies.** Most ETS2 trailers have several body templates: `Body.png` and `Body (Long).png`, `2 Axles.png` and `3 Axles.png`, `Curtainsider Long.png`… The game paints each body through its own accessory ids, as Paintjob Packer does. The first body, the one matching Paintjob Packer's first "Body" group, becomes the trailer's main texture and also paints that group's ids (`also_paints`), as the Krone's body does. Each other body becomes an accessory texture covering its own group's ids. This keeps the rule "a trailer has one main texture" and needs no format change.
- **The rest of the ETS2 catalog.** Every official ETS2 vehicle of Paintjob Packer gets an entry, matched by the tool and then reviewed by hand. That's 25 more trucks (DAF, Iveco, MAN, Mercedes-Benz, Renault, Scania, Volvo) and 39 more trailers (Feldbinder, Kässbohrer, Kögel, Krone, Schmitz Cargobull, Schwarzmüller, SCS, Tirsan, Wielton), DLC vehicles included. Every entry is checked against the pack with `tpv catalog check`, and it builds a valid package in the tests. A sample covering each kind of mapping is checked in the game.
- **A longer Official list.** With 66 vehicles, the official vehicles are sorted by brand, then by name, in New Project and Add Vehicle…. The existing search and the Trucks / Trailers filter keep working on them.
- **Pack folder names with accents** (`Kässbohrer`, `Kögel`, `Schwarzmüller`) are found however the unzip tool encoded them (composed or decomposed Unicode). macOS resource files (`._Sun Visor.png`) are ignored by `tpv catalog check` and `match`.

**Non-goals:**
- ATS vehicles: they need the ATS pack and come in the next batch, with the same tool.
- The pack's vehicles that Paintjob Packer doesn't know, such as the SCS Livestock: without game data they can't be exported.
- Detecting which DLCs the player owns.
- Thumbnails.
- Changing which textures are checked by default when a vehicle is checked: all accessories, as today.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `official-vehicles`:
  - the catalog holds every official ETS2 vehicle, with trailers of several bodies, and is sorted by brand and name;
  - `tpv catalog match` is added;
  - `tpv catalog check` ignores macOS resource files and finds accented folders whatever their Unicode form;
  - the sync scenario no longer names vehicles that are now in the catalog.

## Impact

- **tp-pack:**
  - `catalog::match` (folder finding, name matching, draft entries, report) and the `tpv catalog match` command;
  - Unicode-insensitive folder lookup in `find_pack`, `build` and `check`; `._*` files skipped.
- **tp-vehicles:**
  - the 64 new entries in `catalog::ENTRIES`;
  - official entries sorted by brand and name.
- **tp-app:** the Official rows follow that order. No new UI and no new strings.
- **Data:** 64 new files under `catalog/ets2/`, each at version 1.0.0, validated with the pack "1.59 (2026-05-28)" and Paintjob Packer `a7a7ed3902ca3fe6f9ff3eb4afd2d92db3f5db2d`.
- **Tests:**
  - match on a fixture pack built from the shapes seen in the real pack;
  - every entry complete, every entry builds from placeholder PNGs;
  - the ignored end-to-end test (`TRUCKPAINT_ETS2_PACK`) installs every ETS2 vehicle;
  - the in-game check of a sample is manual.
- **Size:** about 5–20 KB of JSON per vehicle embedded in the app; no template is redistributed.
- **docs:** `docs/roadmap.md` (3: ETS2 shipped, ATS next).
