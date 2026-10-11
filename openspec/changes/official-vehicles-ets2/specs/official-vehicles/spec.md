## MODIFIED Requirements

### Requirement: Official vehicle catalog
TruckPaint SHALL embed a catalog of official vehicles. It is data only and needs no network access. Each entry SHALL hold:
- the vehicle's game (ETS2 or ATS), name, brand, kind (truck or trailer) and game path (`daf.2021`);
- its game data, from Paintjob Packer: the cabins with their internal names, the accessory ids by group, whether it uses the alternate UV set and the colour picker;
- its mapping to the template pack: the vehicle's folder in the pack and, for each main texture and accessory, the pack file it comes from, its size, its name, and the cabins or accessory ids it covers. A main texture MAY also list accessory ids it is applied to (see the vehicle-packages capability). Several entries MAY share a folder of the pack (`Scania R & S` holds the Scania R and the Scania S);
- the cabins and accessory ids left out on purpose, each with a reason ("no template in the pack");
- the entry's version and each texture's layout version;
- its source: the Paintjob Packer commit its game data comes from, and the template pack version its mapping was validated with.

An entry SHALL be listed only when it is complete: every cabin is covered by exactly one main texture, and every accessory id of its game data is covered by exactly one texture or left out on purpose.

A trailer SHALL have one main texture. When the pack has several body templates for a trailer (`Body.png` and `Body (Long).png`), the main texture SHALL be the body of Paintjob Packer's first body group, and it SHALL also be applied to that group's accessory ids. Each other body SHALL be an accessory texture covering its own group's accessory ids.

The catalog SHALL hold, for ETS2, every official vehicle of Paintjob Packer that the ETS2 template pack has templates for. That's the 26 trucks and the 40 trailers of Paintjob Packer at commit `a7a7ed3902ca3fe6f9ff3eb4afd2d92db3f5db2d`, validated with the pack for game 1.59 (2026-05-28). Among them:
- the **DAF NGD** (`daf.2021`, truck): main textures XG+, XG and XF, one per cabin layout; accessories Sun Visor, Sideskirts, Mirrors (standard) and Mirrors (camera);
- the **Krone Cool Liner** (`krone.coolliner`, trailer): the main texture Body (8192 × 1024), also applied to the body accessories; accessories for the chassis front pallet box, front side skirt, front simple side skirt and rear pallet box. The rear bumper is left out: the pack has no template for it.

New Project and Add Vehicle… SHALL list the official vehicles of a game sorted by brand, then by name.

About SHALL credit Paintjob Packer by Carsmaniac for the vehicle data, with its MIT license notice, and Carsmaniac for the template packs.

#### Scenario: Complete entries only
- **WHEN** the catalog is built into the app
- **THEN** every entry covers each of its cabins with one main texture and each of its accessory ids with one texture or a reason, and an entry that doesn't is refused by the tests

#### Scenario: No network needed
- **WHEN** the computer is offline and the user opens New Project with the ETS2 game
- **THEN** the DAF NGD, the Krone Cool Liner and the other official ETS2 vehicles are listed

#### Scenario: Every official ETS2 vehicle
- **WHEN** the user opens New Project with the ETS2 game
- **THEN** the Official section lists 66 vehicles, including the Volvo FH6, the Scania S BEV, the Schmitz Cargobull S.CS and the SCS Box Trailer

#### Scenario: Sorted by brand and name
- **WHEN** the user opens New Project with the ETS2 game and the filter on Trucks
- **THEN** the Official section starts with the DAF NGD, DAF XD, DAF XF, DAF XF Electric and DAF XF105, and ends with the Volvo FH3, FH4, FH5 and FH6

#### Scenario: Trailer with several bodies
- **WHEN** the user checks the Schmitz Cargobull S.CS in New Project
- **THEN** its main texture is Body, and Body (Long), Body (Double) and Body - EcoFIX are among its accessories

#### Scenario: A mod for a trailer with several bodies
- **WHEN** the user creates a project from the Kögel Cargo, paints Body and Body (Long), and exports the mod
- **THEN** the mod paints the ids of Paintjob Packer's group Body with the Body texture, and those of the group Body (Long) with the Body (Long) texture

#### Scenario: Credits
- **WHEN** the user opens About
- **THEN** it credits Paintjob Packer by Carsmaniac with its MIT license notice, and Carsmaniac for the template packs

### Requirement: Template pack
Official vehicles SHALL be installed from the game's template pack by Carsmaniac, which the player downloads and unzips. TruckPaint SHALL remember one pack folder per game.

When the player picks an official vehicle of a game whose pack folder isn't known, a **template pack dialog** SHALL open. It SHALL:
- explain that official vehicles use Carsmaniac's free template pack for that game, about 1.2 GB, downloaded once for every vehicle;
- offer **Open the download page**, which opens the pack's forum thread in the web browser: `https://forum.scssoft.com/viewtopic.php?f=33&t=272386` for ETS2, `https://forum.scssoft.com/viewtopic.php?f=199&t=288778` for ATS;
- say to unzip the pack anywhere;
- offer **Choose the pack folder…**, and accept a folder dropped on the dialog;
- offer **Cancel**, which returns to the dialog it was opened from, unchanged.

A chosen folder SHALL be accepted as the pack when it, or a folder directly inside it, holds the folder of at least one official vehicle of that game. Otherwise the dialog SHALL say that the folder doesn't hold that game's template pack, and keep the pack folder unchanged. An accepted folder SHALL be remembered and the picked vehicle installed (see Installing an official vehicle).

A vehicle's folder and files SHALL be found whether their names with accents (`Kässbohrer`, `Kögel`, `Schwarzmüller`) are stored with composed or decomposed Unicode characters, as unzip tools differ.

When the remembered folder no longer holds the picked vehicle's folder (it was moved or deleted), the dialog SHALL open again, saying so.

#### Scenario: First official vehicle
- **WHEN** no ETS2 pack folder is known and the user picks the DAF NGD in New Project
- **THEN** the template pack dialog opens with Open the download page and Choose the pack folder…

#### Scenario: Download page
- **WHEN** the user clicks Open the download page in the ETS2 template pack dialog
- **THEN** the web browser opens `https://forum.scssoft.com/viewtopic.php?f=33&t=272386`

#### Scenario: Folder unzipped inside another
- **WHEN** the user chooses a folder that holds only `ETS2 templates/`, itself holding `DAF NGD/` and the other vehicle folders
- **THEN** `ETS2 templates/` is remembered as the ETS2 pack folder

#### Scenario: Wrong folder
- **WHEN** the user chooses their Downloads folder, which holds no vehicle folder of the pack
- **THEN** the dialog says the folder doesn't hold the ETS2 template pack, and no pack folder is remembered

#### Scenario: Dropped folder
- **WHEN** the user drops the unzipped pack folder on the template pack dialog
- **THEN** it is checked and remembered as if chosen with Choose the pack folder…

#### Scenario: Pack moved
- **WHEN** the remembered ETS2 pack folder was deleted and the user picks the Krone Cool Liner
- **THEN** the template pack dialog opens and says the pack folder was moved or deleted

#### Scenario: Accented folder in decomposed form
- **WHEN** the pack was unzipped with its `Kögel Cargo` folder name in decomposed Unicode and the user picks the Kögel Cargo
- **THEN** the Kögel Cargo is installed from that folder

### Requirement: Keeping the catalog up to date
The `tpv` tool SHALL offer `tpv catalog sync <paintjob-packer folder> --commit <commit>`, which reads the official vehicles (`mod: false`) of a local copy of Paintjob Packer and, for each catalog entry:
- replaces its game data with Paintjob Packer's: cabins and internal names, accessory ids by group, alternate UV set, colour picker;
- records the commit;
- prints what changed: cabins added or removed, accessory ids added or removed, and "unchanged" otherwise.

It SHALL also print the official vehicles of Paintjob Packer that the catalog doesn't have yet. It SHALL leave the mapping unchanged: a new id stays uncovered until someone maps it or leaves it out. When a catalog entry's vehicle is missing from Paintjob Packer, or a file can't be read, it SHALL fail with a message and change no entry.

#### Scenario: Accessory id added upstream
- **WHEN** Paintjob Packer adds `sunshield.paint_xd` to the DAF NGD's Sun Visor group and `tpv catalog sync` runs
- **THEN** the entry's game data gains the id, the commit is recorded, the output says the DAF NGD gained `sunshield.paint_xd`, and the mapping is unchanged

#### Scenario: Vehicles not in the catalog
- **WHEN** `tpv catalog sync` runs on a copy of Paintjob Packer that has a new official ETS2 trailer and the official ATS vehicles
- **THEN** it lists that trailer and the ATS vehicles as not in the catalog, and no other ETS2 vehicle

### Requirement: Checking the catalog against a pack
The `tpv` tool SHALL offer `tpv catalog check <pack folder> [--game ets2|ats]` (ETS2 by default). For each catalog entry of that game it SHALL report:
- a missing vehicle folder or file;
- a file whose size in pixels differs from the entry's;
- a cabin or accessory id that no texture covers and that isn't left out on purpose;
- an id covered twice, or covered but absent from the entry's game data.

It SHALL also list, as information, the PNG files in the checked vehicles' folders that no entry using that folder uses, and the pack's version (from its `Info and links.txt`) when it differs from the one the entry was validated with. macOS resource files (names starting with `._`) SHALL be ignored. It SHALL exit with an error when any problem is reported, and successfully otherwise.

#### Scenario: Pack matches the catalog
- **WHEN** `tpv catalog check` runs on the ETS2 pack for game 1.59
- **THEN** it reports no problem, lists `Krone Cool Liner/Accessories/Chassis Mid - Simple Side Skirt.png` among the unused files, and exits successfully

#### Scenario: Uncovered id
- **WHEN** the DAF NGD's game data has an accessory id that no texture covers or leaves out
- **THEN** `tpv catalog check` names the DAF NGD and that id, and exits with an error

#### Scenario: Shared folder
- **WHEN** `tpv catalog check` runs on the ETS2 pack and `Scania R & S/R Cabin A (High Roof).png` is used only by the Scania R entry
- **THEN** that file is not listed as unused for the Scania S

#### Scenario: macOS resource file
- **WHEN** the pack holds `DAF XD/Accessories/._Sun Visor.png`
- **THEN** `tpv catalog check` neither reads it nor lists it as unused

## ADDED Requirements

### Requirement: Proposing a vehicle's mapping
The `tpv` tool SHALL offer `tpv catalog match <pack folder> <paintjob-packer folder> --commit <commit> [--game ets2|ats] [--vehicle <path>]…`. For each official vehicle of Paintjob Packer of that game that the catalog doesn't have (or only those named with `--vehicle`), it SHALL write a draft entry under `catalog/<game>/<path>.json`, at version 1.0.0, with Paintjob Packer's game data, the commit and the pack's version. It SHALL propose:
- **the vehicle's folder:** the pack folder named as the vehicle; otherwise a folder shared with other vehicles (`Brand A & B`) or whose name starts with the vehicle's name, or matches its game path (`Scania S 2024e` for `scania.s_2024e`);
- **the main textures of a truck:** each cabin file (`Cabin A (XG+).png`, `Cabin A Aero (Aero Globetrotter XXL).png`, `R Cabin A (High Roof).png` in a shared folder) matched to the cabin of that name, `Cabin 8x4 (…)` matched to the cabin with code `8`; a single file named after the vehicle covers every cabin;
- **the main texture of a trailer and its other bodies:** following the rule of the Official vehicle catalog requirement;
- **the accessory textures:** each file of the `Accessories` folder matched to the Paintjob Packer group whose name shares its words, a group's ids split between several files when their names differ by a variant ("(4x)", "(6x Long)", "Standard", "Camera") that also appears in the ids;
- **the ids left out:** the groups no file matches, with the reason "no template in the pack".

Texture sizes SHALL be read from the files. The command SHALL print, for each vehicle:
- the folder and every proposed texture;
- each **doubtful match**, with its reason: a folder found other than by its exact name, a group split between files, a file matched by only some of its words;
- the ids it could not place, which are left uncovered so the entry stays incomplete;
- the pack's PNG files no texture uses.

It SHALL NOT change an existing entry, and SHALL write nothing for a vehicle whose folder it can't find, listing it instead. It SHALL also list the pack's folders that match no official vehicle of Paintjob Packer.

#### Scenario: Truck matched by cabin names
- **WHEN** `tpv catalog match` runs on the ETS2 pack for the DAF XD
- **THEN** it writes `catalog/ets2/daf.xd.json` with the main textures Sleeper High, Sleeper and Day Cab from `Cabin A (Sleeper High).png`, `Cabin B (Sleeper).png` and `Cabin C (Day Cab).png`, each covering its cabin, at the files' sizes

#### Scenario: Shared folder
- **WHEN** `tpv catalog match` runs for the Scania R and the Scania S
- **THEN** both entries use the folder `Scania R & S`, the Scania R's main textures come from the `R Cabin …` files and the Scania S's from the `S Cabin …` files, and the shared folder is reported as doubtful

#### Scenario: Trailer with several bodies
- **WHEN** `tpv catalog match` runs for the Kögel Cargo
- **THEN** its main texture comes from `Body.png` and also paints the ids of the group Body, and `Body (Long).png` is an accessory texture covering the ids of the group Body (Long)

#### Scenario: Group without a template
- **WHEN** a vehicle's group Rear Bumper matches no file of its folder
- **THEN** its ids are left out with the reason "no template in the pack", and the report says so

#### Scenario: Id that can't be placed
- **WHEN** a group is split between two files and one of its ids names neither variant
- **THEN** that id is left uncovered, the report names it, and the entry is not listed in the app until it is placed by hand

#### Scenario: Existing entry kept
- **WHEN** `tpv catalog match` runs and `catalog/ets2/daf.2021.json` exists
- **THEN** that file is unchanged and the DAF NGD isn't matched again

#### Scenario: Folder not found
- **WHEN** a vehicle of Paintjob Packer has no folder in the pack under any of the accepted names
- **THEN** no entry is written for it, and the report lists it
