# official-vehicles Specification

## Purpose
Lets players paint the games' official trucks and trailers without typing any game data: TruckPaint embeds a catalog of official vehicles and builds each vehicle's package from the free template pack the player downloads once per game.

## Requirements

### Requirement: Official vehicle catalog
TruckPaint SHALL embed a catalog of official vehicles. It is data only and needs no network access. Each entry SHALL hold:
- the vehicle's game (ETS2 or ATS), name, brand, kind (truck or trailer) and game path (`daf.2021`);
- its game data, from Paintjob Packer: the cabins with their internal names, the accessory ids by group, whether it uses the alternate UV set and the colour picker;
- its mapping to the template pack: the vehicle's folder in the pack and, for each main texture and accessory, the pack file it comes from, its size, its name, and the cabins or accessory ids it covers. A main texture MAY also list accessory ids it is applied to (see the vehicle-packages capability);
- the cabins and accessory ids left out on purpose, each with a reason ("no template in the pack");
- the entry's version and each texture's layout version;
- its source: the Paintjob Packer commit its game data comes from, and the template pack version its mapping was validated with.

An entry SHALL be listed only when it is complete: every cabin is covered by exactly one main texture, and every accessory id of its game data is covered by exactly one texture or left out on purpose. The first catalog SHALL hold, for ETS2:
- the **DAF NGD** (`daf.2021`, truck): main textures XG+, XG and XF, one per cabin layout; accessories Sun Visor, Sideskirts, Mirrors (standard) and Mirrors (camera);
- the **Krone Cool Liner** (`krone.coolliner`, trailer): the main texture Body (8192 × 1024), also applied to the body accessories; accessories for the chassis front pallet box, front side skirt, front simple side skirt and rear pallet box. The rear bumper is left out: the pack has no template for it.

About SHALL credit Paintjob Packer by Carsmaniac for the vehicle data, with its MIT license notice, and Carsmaniac for the template packs.

#### Scenario: Complete entries only
- **WHEN** the catalog is built into the app
- **THEN** every entry covers each of its cabins with one main texture and each of its accessory ids with one texture or a reason, and an entry that doesn't is refused by the tests

#### Scenario: No network needed
- **WHEN** the computer is offline and the user opens New Project with the ETS2 game
- **THEN** the DAF NGD and the Krone Cool Liner are listed

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

### Requirement: Installing an official vehicle
Picking an official vehicle in New Project or Add Vehicle… SHALL install it when its pack folder is known and the newest installed version of the vehicle is older than the catalog's entry, or the vehicle isn't installed. TruckPaint SHALL build the vehicle's package from the catalog entry and the pack's files, with the same packing and validation as Custom vehicle and `tpv pack`, and install it like any package (see the vehicle-packages capability):
- its id is `official.<game>.<path>` (`official.ets2.daf.2021`), its version the entry's version, and its name, brand and kind the entry's;
- its game data and texture game ids come from the entry;
- its templates are copies of the pack's files.

While it installs, the dialog SHALL show "Installing <vehicle>…" with a progress indicator, and SHALL offer **Cancel**, which stops it and installs nothing. When it is installed, the vehicle SHALL be chosen as any installed vehicle is: its textures listed, the first main texture and every accessory checked.

Installation SHALL be refused, with nothing installed and a message, when:
- a file the entry needs is missing from the pack: the message names the vehicle and the file;
- a file's size in pixels differs from the entry's: the message names the file and says the pack may be for another game version.

An installed official vehicle SHALL NOT need the pack: it keeps working, in projects and in the vehicle lists, after the pack folder is moved or deleted. Picking an official vehicle that is already installed at the catalog's version SHALL choose it without reading the pack.

#### Scenario: Pick and install
- **WHEN** the ETS2 pack folder is known and the user picks the DAF NGD in New Project
- **THEN** "Installing DAF NGD…" is shown, then the DAF NGD is chosen with XG+ checked and its four accessories checked

#### Scenario: Installed without the pack
- **WHEN** the DAF NGD was installed, the pack folder is deleted, and the user picks the DAF NGD again
- **THEN** it is chosen at once, and no pack dialog opens

#### Scenario: Cancel the installation
- **WHEN** the user clicks Cancel while the Krone Cool Liner installs
- **THEN** no vehicle is chosen and the Krone Cool Liner is not installed

#### Scenario: Missing file
- **WHEN** the pack's `DAF NGD/Cabin B (XG).png` is missing and the user picks the DAF NGD
- **THEN** a message names the DAF NGD and `Cabin B (XG).png`, and nothing is installed

#### Scenario: Pack of another version
- **WHEN** the pack's `Krone Cool Liner/Krone Cool Liner.png` is 4096 × 1024 instead of 8192 × 1024
- **THEN** a message names that file and says the pack may be for another game version, and nothing is installed

#### Scenario: A mod for the Krone
- **WHEN** the user creates a project from the Krone Cool Liner, paints its body, and exports the mod
- **THEN** the mod paints `body.reefer` and `body.reef_13ct` with the body texture, and the chassis accessories with their own textures

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
- **WHEN** `tpv catalog sync` runs on the current Paintjob Packer
- **THEN** it lists the DAF XF, the Krone Dry Liner and the other official vehicles the catalog doesn't have yet

### Requirement: Checking the catalog against a pack
The `tpv` tool SHALL offer `tpv catalog check <pack folder> [--game ets2|ats]` (ETS2 by default). For each catalog entry of that game it SHALL report:
- a missing vehicle folder or file;
- a file whose size in pixels differs from the entry's;
- a cabin or accessory id that no texture covers and that isn't left out on purpose;
- an id covered twice, or covered but absent from the entry's game data.

It SHALL also list, as information, the PNG files in the checked vehicles' folders that their entry doesn't use, and the pack's version (from its `Info and links.txt`) when it differs from the one the entry was validated with. It SHALL exit with an error when any problem is reported, and successfully otherwise.

#### Scenario: Pack matches the catalog
- **WHEN** `tpv catalog check` runs on the ETS2 pack for game 1.59
- **THEN** it reports no problem, lists `Krone Cool Liner/Accessories/Chassis Mid - Simple Side Skirt.png` as unused, and exits successfully

#### Scenario: Uncovered id
- **WHEN** the DAF NGD's game data has an accessory id that no texture covers or leaves out
- **THEN** `tpv catalog check` names the DAF NGD and that id, and exits with an error
