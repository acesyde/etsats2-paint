## MODIFIED Requirements

### Requirement: Extraction command and source boundary
The tool SHALL expose `tpv extract <installation> --converter <executable> [--vehicle <identifier>] [--output <directory>] [--previous <batch directory>] --package-version <version>`. The converter and a valid SemVer package version SHALL be required. Without a vehicle filter it SHALL discover official trucks (`def/vehicle/truck/`) and official owned trailers (`def/vehicle/trailer_owned/`) from the installed ETS2 archives, including installed vehicle, trailer and tuning DLC. It SHALL exclude user mods and ATS vehicles, and SHALL NOT modify the installation. A game id defined both as a truck and as an owned trailer SHALL refuse that id rather than pick one. An unknown or excluded requested vehicle, invalid installation or invalid arguments SHALL return failure rather than an empty successful batch.

#### Scenario: Installed DLC trucks
- **WHEN** extraction runs without a vehicle filter on ETS2 with vehicle and tuning DLC installed
- **THEN** its truck and trailer discovery and painted option coverage include those DLC and exclude user mods

#### Scenario: Explicit version
- **WHEN** the package version is missing or is not SemVer
- **THEN** extraction fails before producing packages

#### Scenario: Filtered vehicle
- **WHEN** `--vehicle daf.2021` names a discovered truck
- **THEN** only that vehicle is processed, with its available cabins and accessories

#### Scenario: Filtered trailer
- **WHEN** `--vehicle krone.coolliner` names a discovered owned trailer
- **THEN** only that trailer is processed, with its available bodies, chassis and accessories

#### Scenario: Unsupported installation or vehicle
- **WHEN** the installation is ATS or the requested vehicle is missing
- **THEN** the command fails and produces no package for that request

### Requirement: Game names
Truck, trailer, brand and cabin names SHALL come from the installed game's English localization (`/locale/common.sui` for brands and series, `/locale/en_us/accessories.sui` for cabins), looked up from the `@@key@@` values of the truck, trailer and cabin definitions. A trailer whose brand is empty (SCS's own trailers) SHALL be named "SCS" followed by its game series id in readable form, words capitalized and underscores as spaces (`scs.box` → "SCS Box", `scs.fueltank` → "SCS Fueltank"), since the game's series strings for them are abbreviations. A truck's optional short series (fifth `info[]` value) SHALL be appended when the series does not already contain it, so trucks sharing a series stay distinct. Comments after a translated value SHALL be ignored. Cabins of one truck sharing a translated name (such as 8x4 versions) SHALL have their game cabin id appended in parentheses. Plain values are used as written; a key without a translation SHALL fall back to a readable form of the key. The report SHALL identify the localization files and their archives.

#### Scenario: Trucks sharing a series
- **WHEN** `iveco.hiway` declares the Stralis series with the Hi-Way short series
- **THEN** its package is named "Iveco Stralis Hi-Way", distinct from `iveco.stralis` ("Iveco Stralis")

#### Scenario: SCS trailer without a brand
- **WHEN** `scs.box` declares an empty brand and a series translated as "STD"
- **THEN** its package is named "SCS Box"

#### Scenario: Branded trailer
- **WHEN** `krone.coolliner` declares the Krone brand and the Cool Liner series
- **THEN** its package is named "Krone Cool Liner"

## ADDED Requirements

### Requirement: Trailer packages
Each successful owned trailer SHALL produce one `.tpv` with id `official.ets2.<trailer>`, `kind: "trailer"`, its game path, and the same validation, size rules, UV rules, exclusions, layout versions and report entries as trucks. A trailer package SHALL have exactly one main texture, 4096 × 4096, with no cabin ids, covering every painted body, chassis and accessory that has no independent texture; its template SHALL superpose all those painted options. Accessory textures SHALL be shared by the whole trailer; trailers SHALL have no variants. Accessory ids that use the same sized mask in every game paint job that overrides any of them SHALL form one accessory texture listing all those game ids, whose size follows the size rules applied to that mask and whose template superposes their layouts; ids whose masks differ in at least one paint job SHALL keep separate textures. A trailer whose paint jobs define no main mask and no override SHALL be refused with that reason. The report SHALL record each vehicle's kind and, for every grouped texture, its game ids and the shared mask references.

#### Scenario: Bodies sharing one livery region
- **WHEN** the dry van, insulated and reefer 13.6 m bodies of `scs.box` use the same 4096×2048 mask in every paint job that overrides them
- **THEN** the package has one accessory texture of 4096 × 2048 listing the three body ids, whose template shows the three layouts superposed

#### Scenario: Ids that differ in one paint job
- **WHEN** two body ids share a mask in most paint jobs but one paint job gives them different masks
- **THEN** they keep separate accessory textures

#### Scenario: Trailer main texture
- **WHEN** a trailer's chassis has no paint override with crop dimensions
- **THEN** the chassis contributes to the single 4096 × 4096 main texture, which has no cabin ids

#### Scenario: Install and export an extracted trailer
- **WHEN** the player installs `official.ets2.krone.coolliner` and exports a mod painting its main texture and body
- **THEN** the mod writes the trailer paint job under `trailer_owned` with one override for the body texture listing its game ids
