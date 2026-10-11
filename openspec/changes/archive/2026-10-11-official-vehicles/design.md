## Context

**Vehicles today:**
- Packages are `.tpv` ZIPs read and validated by `tp_vehicles::Package::read`. Manifests are deserialized only; tp-vehicles has no writer.
- `tp_pack::pack_entries(manifest: Value, files)` packs in memory, converts DDS, writes a reproducible ZIP and validates it. Custom vehicle (`tp_pack::custom::CustomVehicle::pack`) already goes through it.
- The installed library is `tp_app::vehicles::VehicleLibrary`, stored as `<data>/vehicles/<id>/<version>.tpv`. `install_bytes` validates and replaces the same version. The sample vehicles are embedded with `include_bytes!` (`vehicles::SAMPLES`).
- Ids starting with `custom.` are custom vehicles (`tp_pack::custom::is_custom`); that's the only origin notion.

**The dialogs:**
- New Project (`ui/new_project.rs`): the vehicle list is hand-painted rows from `filtered(library, filter)`, with no sections and no thumbnails. Picking only selects an installed vehicle (`VehicleChoice::of(&InstalledVehicle)`).
- Add Vehicle (`ui/vehicle_dialogs.rs::add_vehicle`) uses `vehicle_list`, plain selectable rows.
- Both open Custom vehicle through `custom_vehicle::Origin { NewProject, AddVehicle, Library }`, which comes back with the new vehicle chosen (`Origin::created`). That is the pattern to reuse for the pack dialog.
- Long work runs on a named thread with an mpsc result and `request_repaint` (`custom_vehicles::PackJob`, `mod_export::ModJob` with progress and cancel).
- File pickers sit behind `trait FileDialogs` (rfd natively, `ScriptedDialogs` in tests). There is no folder picker.
- Dropped files are handled in `state.rs::after_frame`; dropped folders are not handled anywhere.

**Preferences:** `Prefs` (RON, `#[serde(default)]`), dialog `ui/dialogs.rs::preferences(ctx, &mut Prefs, title_bar) -> bool`, which has no access to `state.dialogs`.

**Links and credits:** no `open_url` anywhere yet. egui 0.36 offers `ctx.open_url(OpenUrl::new_tab(..))` through eframe. About is a small dialog with no notices.

**Mod export** (`mod_export.rs::plan`): per vehicle, one `<unit>.sii` per painted main texture, and an `accessory/<unit>.sii` with one `simple_paint_job_data` per painted accessory, written only when accessories are painted.

**Data:**
- Paintjob Packer's JSON (`vehicles/ets/scs/daf.2021.json`): `name`, `path`, `type`, `mod`, `alt_uv`, `colour_picker`, `separate_paint_jobs`, `cabins[{code, designation, name, internal_name[]}]`, `accessories{group: [ids]}`.
- The ETS2 pack (`templates/`) has one folder per vehicle and `Info and links.txt` ("current for game version 1.59, updated 2026-05-28").
  - `DAF NGD/`: `Cabin A (XG+).png`, `Cabin B (XG).png`, `Cabin C (XF).png` (4096²); `Accessories/Sun Visor.png` (2048 × 256), `Sideskirts (4x).png` (2048 × 1024), `Main Mirrors - Standard.png` (2048²), `Main Mirrors - Camera.png` (2048 × 1024); a PSD.
  - `Krone Cool Liner/`: `Krone Cool Liner.png` (8192 × 1024); `Accessories/Chassis Front - Pallet Box.png` (1024 × 256), `Chassis Front - Side Skirt.png` (1024 × 256), `Chassis Front - Simple Side Skirt.png` (1024 × 128), `Chassis Mid - Simple Side Skirt.png` (256 × 512), `Chassis Rear - Pallet Box.png` (512²); a PSD.
- Paintjob Packer's Krone groups: Body (`body.reefer`, `body.reef_13ct`), Sideskirts (16 ids), Rear Bumper (`r_bumper.bumper01p`, `r_bumper.bumper02p`).
  - The pack has no body file apart from the main one. Other trailers of the pack (Krone Dry Liner, Kögel Cool) name their root files `Body.png` and `Body (Long).png`.
  - So the root file is the body, and the body accessories are painted with it.

## Goals / Non-Goals

**Goals:**
- An official vehicle is a normal installed package, built locally: nothing downstream (projects, Update Template, mod export, Vehicle Library) needs to know where it came from, except one new manifest field.
- The catalog is reviewable data in the repository, with its mapping separate from the data that `tpv catalog sync` rewrites.
- One pack folder per game, asked for once, in the flow where it's needed.

**Non-Goals:**
- `tpv catalog match` and the other vehicles (next bricks).
- ATS: the code is per game, but the catalog has no ATS entry, so ATS lists and Preferences show nothing new.
- Reading PSDs; thumbnails before installation; Vehicle Library changes; editing `also_paints` in Custom vehicle.
- Checking the pack's files beyond their pixel size (no hashes: a re-saved PNG of the same size is fine).

## Decisions

### 1. Catalog entry format: one JSON file per vehicle, mapping separate from game data

`catalog/<game>/<path>.json`:

```json
{
  "format": 1,
  "game": "ets2",
  "path": "daf.2021",
  "name": "DAF NGD",
  "brand": "DAF",
  "kind": "truck",
  "version": "1.0.0",
  "game_versions": ">=1.59",
  "folder": "DAF NGD",
  "source": { "paintjob_packer": "<commit>", "pack": "1.59 (2026-05-28)" },
  "game_data": {
    "alt_uv": false,
    "colour_picker": false,
    "cabins": [{ "code": "a", "name": "XG+", "internal_names": ["xg_plus"] }],
    "accessories": { "Sun Visor": ["sunshield.paint_xf", "…"] }
  },
  "main": [
    { "id": "xg_plus", "name": "XG+", "file": "Cabin A (XG+).png", "size": [4096, 4096],
      "layout_version": 1, "cabins": ["xg_plus"] }
  ],
  "accessories": [
    { "id": "mirrors_camera", "name": "Mirrors (camera)", "file": "Accessories/Main Mirrors - Camera.png",
      "size": [2048, 1024], "layout_version": 1, "ids": ["mirror.cam_semi"] }
  ],
  "left_out": [{ "ids": ["r_bumper.bumper01p", "r_bumper.bumper02p"], "reason": "no template in the pack" }]
}
```

- `game_data` is the only part `tpv catalog sync` writes. `main`, `accessories` and `left_out` are hand-made and reviewed in PRs.
- A main texture may add `"also_paints": [ids]` (the Krone's Body).
- Completeness (Requirement "Official vehicle catalog") is computed from `game_data` against `main[].cabins`, `accessories[].ids`, `also_paints` and `left_out`. A main texture without `cabins` covers every cabin, which is allowed only when there is one main texture.
- Ids and versions: package id `official.<game>.<path>`; the package version is the entry's `version`, raised by hand when the mapping or a template changes (with that texture's `layout_version`).

*Alternatives:*
- One big catalog file: harder to review and to diff per vehicle.
- Storing Paintjob Packer's group names as accessory parts: rejected, because the pack splits groups (DAF mirrors) and its files can share a group (Krone sideskirts).

### 2. The DAF NGD and Krone Cool Liner mappings

| Vehicle | Texture | Pack file | Covers |
|---|---|---|---|
| DAF NGD | main XG+ / XG / XF | `Cabin A (XG+).png` / `Cabin B (XG).png` / `Cabin C (XF).png` | `xg_plus` / `xg` / `xf` |
| | Sun Visor | `Accessories/Sun Visor.png` | 3 `sunshield.*` |
| | Sideskirts | `Accessories/Sideskirts (4x).png` | 6 `sideskirt.*` |
| | Mirrors (standard) | `Accessories/Main Mirrors - Standard.png` | `mirror.semi` |
| | Mirrors (camera) | `Accessories/Main Mirrors - Camera.png` | `mirror.cam_semi` |
| Krone Cool Liner | main Body, `also_paints` `body.reefer`, `body.reef_13ct` | `Krone Cool Liner.png` | every cabin (trailer) |
| | Chassis front pallet box | `Accessories/Chassis Front - Pallet Box.png` | `chs_f_{l,r}.pallet01p` |
| | Chassis front side skirt | `Accessories/Chassis Front - Side Skirt.png` | `chs_f_{l,r}.side02{p,sp,tp,tsp}` |
| | Chassis front simple side skirt | `Accessories/Chassis Front - Simple Side Skirt.png` | `chs_f_{l,r}.side01p` |
| | Chassis rear pallet box | `Accessories/Chassis Rear - Pallet Box.png` | `chs_r_{l,r}.pallet01{p,sp}` |
| | left out | none | `r_bumper.bumper01p`, `r_bumper.bumper02p` ("no template in the pack") |

- `Chassis Mid - Simple Side Skirt.png` matches no id of Paintjob Packer and stays unused (`tpv catalog check` lists it).
- The in-game check (task 7.2) confirms or corrects the side skirt mapping and the Body choice before the PR is merged.

### 3. Where the code lives

- **`tp-vehicles::catalog`:**
  - entry types (serde), embedded entries (`ENTRIES: &[&str]` of `include_str!("../../../catalog/ets2/….json")`, with a test that every file under `catalog/` is listed);
  - `entries(game)`, `completeness(&Entry) -> Vec<Problem>`;
  - `manifest(&Entry) -> serde_json::Value`, the package manifest, with template paths `templates/<id>.png`.
- **`tp-pack::catalog`:**
  - `find_pack(folder, game) -> Option<PathBuf>`: the folder or a direct subfolder holding an entry's `folder`;
  - `build(entry, pack) -> Result<Packed, CatalogError>`: reads each file, checks the PNG header size against the entry (`custom::probe`), then calls `pack_entries`;
  - `sync(ppk_folder, commit, catalog_dir) -> Report`, `check(pack, game, catalog_dir) -> Report`.
  - `tpv.rs` gets a `catalog` arm with `sync` and `check`, in the existing hand-rolled style.
- **`tp-app::official`:** an `InstallJob` modelled on `PackJob`, with a cancel flag and file-count progress. It builds on a thread, then `VehicleLibrary::install_bytes` on the UI thread.
- **Listing:**
  - a helper returns the official rows of a game, each `{entry, installed: Option<&InstalledVehicle>}`, followed by the installed rows whose id doesn't start with `official.`;
  - New Project's `list()` and Add Vehicle's `vehicle_list` draw section headers when both sections have rows;
  - the row's Package column is "Official", plus the version when installed.

*Alternative:* a separate catalog crate. Rejected, because tp-vehicles already owns the package types and tp-pack already owns packing.

### 4. Picking an official vehicle: install on pick, through an origin

- A click on an official row follows this decision:
  - **installed at ≥ the entry's version:** choose it (`VehicleChoice::of`);
  - **else, pack folder known and the vehicle folder present:** start the `InstallJob`. The dialog shows "Installing <name>…" with a progress bar and Cancel, the list is disabled, and the previous choice is kept;
  - **else:** open the template pack dialog with `Origin::NewProject(draft)` or `Origin::AddVehicle(dialog)` (the existing `custom_vehicle::Origin`, moved to a shared module) and the picked entry. Cancel returns to the origin unchanged. An accepted folder is saved in `prefs.template_packs`, then the install starts back in the origin.
- When the install finishes:
  - `install_bytes`, then the existing `Origin::created`-like path chooses it;
  - a refusal (missing file, other size, validation) shows its message in the dialog's existing `messages` list.

*Alternatives:*
- Installing on Create Project: the fleet panel would need the textures before install, from the catalog. That's a second data path for one panel. Install on pick keeps `VehicleChoice` an installed vehicle.
- A separate progress modal: it's heavier, and the origin dialog already has a message area.

### 5. Pack folder: picker, drop, prefs

- **Picker:** `FileDialogs::pick_folder(title) -> Option<PathBuf>`: rfd `pick_folder` natively, a queue in `ScriptedDialogs`.
- **Drop:** the pack dialog reads `raw.dropped_files` paths that are directories. `after_frame` leaves directories alone when the pack dialog is open.
- **Check:** `tp_pack::catalog::find_pack`. A folder is a pack of a game when it, or a direct subfolder, holds the `folder` of at least one entry of that game. The resolved folder (the subfolder when nested) is what is saved.
- **Prefs:** `template_packs: BTreeMap<String /* game code */, PathBuf>` with `#[serde(default)]`, so old preference files load unchanged. `reset_scaling` doesn't touch it.
- **Preferences dialog:**
  - a "Template packs" section with one row per game that has entries: the path shortened in the middle, its full text in a tooltip, **Choose…** and **Clear**;
  - `preferences()` returns an outcome (`Done`, `ChoosePack(Game)`); the caller, which owns `state.dialogs`, runs the picker and the check, and puts a refusal message back into the dialog.

### 6. `also_paints`, from the package to the mod

- **tp-vehicles:** `Part.also_paints: Vec<String>` (`#[serde(default)]`). Validation, after `validate_game_ids`:
  - an accessory with `also_paints` is refused;
  - each id must be a valid game name, unique across all main textures' `also_paints`, and not an accessory's game id (new `PackageError::BadAlsoPaints` and `DuplicateGameId`).
- **tp-core:** `SurfaceTemplate.also_paints: Vec<String>`, set by `template_for` from the part and refreshed by Update Template with the other game ids.
- **tp-file:** `FileTemplate.also_paints` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`. This is additive, so the format stays 1 and older project files load.
- **mod export:** for each painted main texture with `also_paints`, the accessory file of its unit gets one more `simple_paint_job_data` (`paint_job_mask` = the main texture's `.tobj`, `acc_list[]` = the ids). The accessory file is written per unit when either list is non-empty.
- **docs:** `docs/vehicle-package-format.md` documents the field. Custom vehicle neither shows nor writes it.

### 7. Credits and links

- **About:** a "Credits" line: vehicle data from Paintjob Packer by Carsmaniac (MIT), template packs by Carsmaniac. A **Licenses…** button shows Paintjob Packer's MIT notice in a scrollable text, from `catalog/LICENSE-paintjob-packer.txt` (embedded).
- **Pack dialog:** "Open the download page" calls `ctx.open_url(OpenUrl::new_tab(url))`, with the two forum URLs as constants per game. In kittest, the test reads the URL from the output commands.

### 8. Strings

New keys in en/fr/de/es for:
- the Official and Installed headers, "Official" and "Installing { $name }…";
- the pack dialog: title, explanation, steps, buttons, wrong folder, moved folder, missing file, other version;
- Preferences: Template packs, Not set, Choose…, Clear;
- About: credits and Licenses….

Vehicle and texture names come from the catalog, in English, as in the game's own names.

## Risks / Trade-offs

- **The Krone body hypothesis.** The pack's root file is used as the trailer's main texture and as the body accessories' texture. If the game shows the body texture wrongly on the trailer's own mask (the chassis), the in-game check (task 7.2) stops the change. The body then needs a different model, which goes back to the user before going further: for example, Body as an accessory texture with the main texture left blank.
- **Side skirt mapping by name.** `side01p` versus `side02*` and front versus mid are guessed from file names. The in-game check validates them; the catalog version starts at 1.0.0 after that check.
- **Pack updates.** A new pack can change sizes (refused with the "other version" message) or layouts at the same size (not detected). `tpv catalog check` on each new pack catches the first. The second is `template-update-impact`'s job (layout versions).
- **Install time and memory.** The DAF NGD reads about 12 MB of PNG and deflates it. That's a few seconds on a thread, with cancel; there's no image decoding beyond the size probe.
- **Ids under `official.`.** A player can still install a `.tpv` with such an id from a file (for example one exported from another computer). That's fine: it is the same package.
- **Legal.** No template is redistributed: packages are built on the player's machine from the pack they downloaded. Exporting such a package from the Vehicle Library already reminds that game templates aren't the player's to share.
