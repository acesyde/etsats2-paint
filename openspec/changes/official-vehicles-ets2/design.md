## Context

**The catalog today:**
- `catalog/ets2/daf.2021.json` and `krone.coolliner.json` follow the entry format of the first brick: `game_data` is rewritten by `tpv catalog sync`, while `main`, `accessories` and `left_out` are made by hand.
- They are embedded through `tp_vehicles::catalog::ENTRIES`, a hand list of `include_str!`, and a test checks that every file under `catalog/` is listed.
- `catalog::entries(game)` returns the complete entries in `ENTRIES` order, and `ui/vehicle_dialogs.rs::vehicle_rows` lists them in that order.
- `tp_pack::catalog` has `find_pack`, `build`, `check`, `sync` and `read_catalog`. `tpv.rs` dispatches `catalog check|sync` by hand.

**The data,** surveyed for this change:
- **Paintjob Packer** at `a7a7ed3` (the commit already recorded in both entries) has 66 official ETS2 vehicles: 26 `truck` and 40 `trailer_owned`. All have `alt_uv: false`.
- **The ETS2 pack** (1.59, 2026-05-28) has 63 vehicle folders:
  - 49 folders are named exactly as a Paintjob Packer vehicle, once accented names are compared in the same Unicode form. On the macOS disk, `Kässbohrer`, `Kögel` and `Schwarzmüller` are decomposed, while Paintjob Packer has them composed.
  - Shared folders: `Scania R & S`, `Scania R 2009 & Streamline`, `Kässbohrer K.SCS & K.SCX` and `Krone Profi Liner & Paper Liner`. `Feldbinder TSA` serves the TSA ADR and the TSA LM.
  - `Scania S 2024e` is the Scania S BEV (`scania.s_2024e`).
  - `SCS Livestock` has no Paintjob Packer vehicle.
  - Some folders hold macOS resource files (`DAF XD/Accessories/._Sun Visor.png`).
- **Truck main files:**
  - `Cabin <letter>[ Aero] (<cabin name>).png`, or `Cabin 8x4 (<name>).png` for the cabin with code `8`;
  - with a vehicle prefix in shared folders: `R Cabin A (High Roof).png`, `Streamline Cabin B (Highline).png`;
  - a single file named after the vehicle for one-cabin trucks (`Iveco S-Way.png`).
  - Cabin letters don't always match Paintjob Packer's codes (the Volvo FH6's `Cabin A` is code `e`), but the names in parentheses always match.
- **Trailer root files are bodies.** They map to Paintjob Packer's groups whose names start with "Body":
  - `Body.png` / `Body (Long).png` ↔ "Body" / "Body (Long)";
  - `2 Axles.png` ↔ "Body (2 Axles)";
  - `Curtainsider Long.png` ↔ "Body - Curtainsider (Long)";
  - a file named after the trailer ↔ "Body".
  - Some don't map one to one: `Body (835 mm).png` ↔ "Body (825 mm)" (a typo on one side), and the K.SBT's two files `Body (2 Axles)`, `Body (3 Axles)` against one group "Body".
- **Accessory files** sit in `Accessories/`, named "<part> - <variant> (<axles>)". Paintjob Packer groups ids by part ("Rear Fender", 9 ids). One group often maps to several files ("Rear Fender (4x)", "(6x Long)", "(6x Short)").
- **Paintjob Packer's own export** gives each body group of a trailer its own texture through `simple_paint_job_data`, plus a main mask (`Base`) for the trailer. The pack has no `Base` template; the first brick used the body as the main mask on the Krone, and that passed the in-game check.

## Goals / Non-Goals

**Goals:**
- A matching tool good enough that a human reviews a report instead of building 64 entries by hand. It is never trusted blindly: anything not certain is reported, and the completeness check stops an entry that isn't fully placed.
- No change to the entry format, the package format or the mod export: every new vehicle fits what the first brick shipped (`also_paints` included).

**Non-Goals:**
- A general fuzzy matcher. The rules are tuned to the naming of Carsmaniac's packs, and the fixtures copy real names.
- Using `match` in the app. It stays a maintainer tool in `tpv`.
- Matching ATS (the code takes `--game`, but no ATS pack has been surveyed yet).

## Decisions

### 1. Trailers with several bodies: first body group as main, the others as accessories

- **The main texture** is the file matched to Paintjob Packer's first group whose name starts with "Body", in the order of `game_data.accessories` (by name: "Body" before "Body (Long)"). It has no `cabins` and `also_paints` that group's ids, exactly like the Krone. Its id is `body` and its name is the file's name ("Body", "Curtainsider B-Double").
- **Every other body file** is an accessory texture covering its group's ids.
- **A one-body trailer** whose file is named after it, like the Krone, gives the same shape.

*Alternatives:*
- **A blank generated `Base` main texture**, with every body as an accessory, as Paintjob Packer does. It needs a texture the pack doesn't have. The player would also get a main texture that paints nothing visible, and New Project would need a way to hide it. Rejected.
- **One main texture per body.** That breaks the rule that a trailer has one main texture, and the mod export writes one `<unit>.sii` per main texture with no cabins to tell them apart. Rejected.

### 2. Where the matcher lives, and what it writes

- **`tp_pack::catalog::matching`** (a new file) holds:
  - `match_vehicles(pack, ppk, game, commit, only: &[String], existing: &[Entry]) -> MatchReport`;
  - the draft entries (`Entry` values, written with `Entry::to_json`);
  - each vehicle's notes: `Doubt { what, why }`, uncovered ids, unused files, and the folders matched to no vehicle.
  - `tpv catalog match` writes each draft at `catalog/<game>/<path>.json` only when that file doesn't exist, then prints `MatchReport::text()`.
- **Doubts stay in the report, not in the JSON.** Entries keep `deny_unknown_fields`, the reviewer pastes the report into the PR, and nothing to clean up is left in the catalog.
- **A draft gets:**
  - version `1.0.0`;
  - `game_versions` `>=1.59`, from the pack's version, as for the first brick;
  - `source.pack` from `pack_version`;
  - `kind` `trailer` for `trailer_owned`;
  - `brand` as the first word of the name, except "Mercedes-Benz", "Schmitz Cargobull" and "SCS", which are written whole;
  - texture sizes from the PNG headers (`png_size`).
  - A file whose size isn't a valid texture size (powers of two from 64 to 8192) is reported as a problem and left out of the draft.
- **Texture ids and names:**
  - a truck's main texture: the cabin's first internal name as id, and its name as name. The `8` cabin's name gets " 8x4" added when another cabin has the same name (New Actros "GigaSpace" / "GigaSpace 8x4");
  - other textures: the file's name without extension as name, and its lowercase ASCII slug as id (`Sideskirts - Paint (4x)` → `sideskirts_paint_4x`), made unique with a number when two collide.

*Alternatives:*
- **A separate `catalog-tool` binary.** `tpv` already holds `catalog check` and `sync`. Rejected.
- **Writing doubts as a `"review"` field** that the tests refuse. That's cleaner to diff but needs a format change for a maintainer-only concern. Rejected.

### 3. Matching rules

All comparisons use `fold(name)`: NFC, lowercase, accents removed (Decision 5). Names are split into words on anything that isn't a letter or digit. A trailing plural "s" is dropped (`mirrors` → `mirror`, `sideskirts` → `sideskirt`).

- **Folder:**
  1. a pack folder whose name equals the vehicle's (certain);
  2. else a shared folder `<A> & <B>`, where `<B>` takes `<A>`'s first word as its brand (`Scania R & S` gives "Scania R" and "Scania S");
  3. else a folder whose name the vehicle's name starts with (`Feldbinder TSA`);
  4. else a folder whose words hold the words of the vehicle's game path model (`s_2024e` → `Scania S 2024e`).

  Steps 2 to 4 are doubts. A folder claimed by an exact match is never offered to another vehicle by step 3 or 4.
- **The vehicle's files in a shared folder:**
  - Root files are split by prefix: the vehicle's name without the brand ("R", "S", "R 2009", "Streamline").
  - Accessory files are shared, unless their name holds the other vehicle's prefix in parentheses (`Rear Bumper - Diva (R 2009)`).
  - In `Krone Profi Liner & Paper Liner` and `Feldbinder TSA`, root files have no prefix. They are matched against both vehicles' body groups, and every assignment is a doubt.
- **Truck main textures:**
  - `^(?:<prefix> )?Cabin (?<code>[A-Z]|8x4)(?: Aero)? \((?<name>.+)\)$`, matched by cabin name first, then by code (`8x4` ↔ `8`, a letter ↔ its lowercase code). A match by code alone is a doubt.
  - A single root file named after the vehicle (or its prefix) covers every cabin when the vehicle has one cabin; otherwise it's a doubt and its cabins stay uncovered.
- **Trailer bodies:** each root file is compared with each body group, ignoring the word "body". Equal word sets are certain. Otherwise the group sharing the most words wins, if any, and that's a doubt (`835 mm` ↔ `825 mm`). Root files left after this go to the accessory matching (`Trailer Rear - Flap Door.png`).
- **Accessories:** each file goes to the non-body group sharing the most words with it, as a doubt unless the group's words are all in the file's name. A tie is a doubt, decided for the first group in game data order.
- **Splitting a group** between several files:
  - variant words are the words of a file that aren't in the group's name nor in every file's name ("4x", "camera", "front", "left");
  - an id goes to the file whose variant words all appear in the id (a word may start an id word, `4x` ↔ `_4x2`, or begin with it, `camera` ↔ `cam_semi`), the most specific one;
  - otherwise to the file naming most of it alone, a place word counting when its initial is the id's first one-letter word (`front` ↔ `chs_f_l`), or the last one for left and right;
  - otherwise to the group's plainest file (fewest variant words, then "stock", "standard", "paint" or "main"), because Paintjob Packer paints a whole group with one texture: the group's ids share one layout (decided with the user while building the matcher; most ids, such as `sunshld.triangle`, name no file);
  - every split is a doubt.
- **Accessory group choice** also counts the file's words found in the group's ids (`Pallet Box` ↔ `chs_r_l.pallet01p`), and words of place count less than words of part.
- **Left out:** every group with no file is left out with "no template in the pack" (certain). Files with no group are listed as unused.

The rules are deterministic, so running the tool twice on the same pack gives the same drafts.

*Alternative:* string distance (Levenshtein) between names. It's opaque in the report and wrong on the pack's short variant words ("4x" against "6x"). Rejected for word sets, which can be explained in a doubt line.

### 4. Reviewing 64 drafts

- The drafts are generated once and then edited by hand in the PR: the doubts are resolved, the uncovered ids placed or left out with a reason.
- They're committed per brand, so each diff can be reviewed on its own.
- `tpv catalog check` on the real pack must pass with no problem before the PR is merged. Its unused-file list is pasted into the PR: each unused file is either known (no Paintjob Packer id) or a mistake.
- `match` stays useful for ATS and for vehicles that arrive with game updates.

### 5. Names with accents, sorting

- **`tp_vehicles::fold(&str) -> String`:** NFD, combining marks removed, lowercase. It uses the `unicode-normalization` crate, which is small, widely used and has no other runtime dependency. It goes into tp-vehicles, which tp-pack already depends on.
- **Folder lookup:** `find_pack_of`, `build_with` and `check` resolve `pack.join(folder)` through `resolve_dir(parent, name)`: the exact path when it exists, else the entry of `read_dir(parent)` whose `fold`ed name equals `fold(name)`. Files under the folder are resolved the same way, segment by segment. Entry files always store names in NFC.
- **Sorting:** `catalog::entries(game)` sorts by `(fold(brand), fold(name))`. Kässbohrer then sorts before Kögel and Krone, and "SCS" sorts after "Schwarzmüller". New Project, Add Vehicle… and the Preferences check all read `entries`, so they all follow.
- **`._*` files** are skipped by `pngs()` (check) and by `match`.

*Alternative:* a hand-written table of composed and decomposed Latin letters, which would avoid the dependency. Rejected: it's fragile, and the crate is the standard answer.

### 6. In-game sample

One vehicle per kind of mapping is checked in ETS2 1.59 with a mod painting each of its textures in a distinct color with a label:

| Kind of mapping | Vehicle |
|---|---|
| Aero cabins, many accessories, codes different from letters | Volvo FH6 |
| 8x4 cabin with the same name as another | Mercedes-Benz New Actros |
| Shared folder with prefixed cabins | Scania R |
| Single main texture for one cabin | Iveco S-Way |
| Several bodies, first body as main | Schmitz Cargobull S.CS, Kögel Cargo |
| Body files without "Body" in the name | SCS Fuel Tank |
| A group split between files | Wielton Curtain Master |

A failure corrects that vehicle's entry and the matching rule behind it. When the rule changes, every draft is regenerated or re-checked.

## Risks / Trade-offs

- **[A group's file chosen for ids it doesn't show]** The plainest file's guides may not draw the exact accessory a player has. → The layout is shared, so the paint lands right; the doubt names the other files, and a reviewer may pick a better template. The in-game sample checks a group texture.
- **[A wrong but plausible match passes review]** → Each doubt is listed, the completeness check refuses uncovered ids, `check` lists unused files, and the in-game sample covers each rule. A mistake found later is fixed by raising that entry's version, and installed players get the new package on their next pick, as the first brick allows.
- **[The first body as the trailer's main mask looks wrong on a trailer other than the Krone]** → The in-game sample includes two trailers with several bodies. If one fails, stop and come back to the user before choosing another model (for example the blank `Base` of Decision 1's alternatives).
- **[Large vehicles create many textures by default]** The SCS Box Trailer has 26 accessories, and all accessories are checked when a vehicle is checked. → The player can uncheck them in the fleet card. A task measures New Project → Create Project with the SCS Box Trailer and the Volvo FH6. If it's noticeably slow or heavy, it's reported to the user as a follow-up, and this change doesn't alter the default.
- **[DLC vehicles the player doesn't own are listed]** → Harmless: the mod only paints vehicles the game has. DLC detection is a non-goal.
- **[A big PR (64 JSON files)]** → Committed per brand, and the tool's output can be reproduced from the recorded commit and pack version.
- **[A pack update renames files]** → `tpv catalog check` reports missing files; `match --vehicle <path>`, run on a moved-aside entry, proposes the new mapping. `template-update-impact` covers the player's side.

## Migration Plan

Nothing to migrate. The DAF NGD and Krone Cool Liner entries stay as they are, at 1.0.0. Players who installed them keep them, and the new vehicles appear in the Official list after the update.
