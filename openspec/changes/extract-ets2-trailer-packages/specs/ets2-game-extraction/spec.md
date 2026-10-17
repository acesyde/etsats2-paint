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


### Requirement: Definition-driven selection
The extractor SHALL derive vehicle metadata, cabins, accessory game ids, model references and compatibility from game definitions rather than per-vehicle profiles. It SHALL resolve referenced inclusions and archive overrides with recorded provenance. It SHALL select each model's requested look and variant, retain only visible painted faces, and choose the UV stream by its semantic alias. Missing required files, inclusion cycles, conflicting unresolved definitions, unsupported paint shaders or UV mappings, and ambiguous looks, variants or UV streams SHALL refuse the affected vehicle. A look or variant repeated under the same name with identical content (a game data duplicate) is not ambiguous: it SHALL be used once. Unsupported data SHALL NOT be silently omitted. Paint overrides referencing accessory ids with no definition in the installed source view SHALL be excluded from texture assignments and reported as ignored references, without refusing the vehicle. A defined accessory whose painted geometry cannot be extracted (missing files, invalid geometry, unsupported paint shaders) SHALL still refuse the vehicle. A defined accessory whose definition has no exterior model field at all (for example a container body whose model is commented out) has nothing to paint: it SHALL be excluded from all templates and reported as `accessory_without_model` with an `excluded` disposition, without refusing the vehicle. A defined accessory whose selected looks/variants contain no truckpaint material (for example a base-colour-only `paint` flavor) cannot show a livery: it SHALL be excluded from all templates and reported with an `excluded` disposition, without refusing the vehicle.

#### Scenario: Included shared model
- **WHEN** an accessory definition includes a file that references a shared model outside the truck's model directory
- **THEN** the referenced definition and model contribute to the vehicle and their source is recorded

#### Scenario: Visible variant and semantic UV alias
- **WHEN** a look contains painted materials and a variant hides one model part, with the paint UV alias on a stream whose numeric tag differs
- **THEN** the template contains only visible painted geometry using the aliased stream, without an additional vertical flip

#### Scenario: Ambiguous or unsupported selection
- **WHEN** a definition has an unresolved include or its painted model has ambiguous look, variant or UV data, or unsupported UV data
- **THEN** the vehicle fails with the concerned definition or model identified

#### Scenario: Look repeated with identical content
- **WHEN** a painted model defines the requested look twice with identical content
- **THEN** the extractor uses that look once and does not refuse the vehicle
#### Scenario: Body without any model
- **WHEN** a paint override targets a trailer body whose definition has no model field
- **THEN** the trailer package is generated without it and `missing_dependencies` records it as `accessory_without_model` with an `excluded` disposition

### Requirement: Isolated batch outputs
Each invocation that starts a batch SHALL create a unique batch subdirectory under the output parent, defaulting to `dist/`. Intermediates SHALL remain under `target/`, separate from published packages. Existing batch contents SHALL NOT be overwritten. Vehicle failures SHALL NOT stop other vehicles. Only fully built and validated packages SHALL be published; failed vehicles and interrupted package writes SHALL leave no partial `.tpv`. The command SHALL return zero only when every requested/discovered vehicle succeeds or is skipped as not paintable and the final report is written; any incomplete batch or output failure SHALL return nonzero.

#### Scenario: Converter failure for one truck
- **WHEN** the converter fails for one vehicle and succeeds for another
- **THEN** the successful package is published, the failed vehicle has no package, both results are reported and the command returns nonzero

#### Scenario: Repeated invocation
- **WHEN** two batches use the same output parent
- **THEN** they use distinct directories and neither overwrites the other's outputs

#### Scenario: Package write failure
- **WHEN** publishing a package fails partway through writing
- **THEN** no partial `.tpv` is left as a published package and the command returns nonzero

## ADDED Requirements

### Requirement: Trailer packages
Each successful owned trailer SHALL produce one `.tpv` with id `official.ets2.<trailer>`, `kind: "trailer"`, its game path, and the same validation, size rules, UV rules, exclusions, layout versions and report entries as trucks. A trailer package SHALL have exactly one main texture with no cabin ids. When at least one painted body, chassis or accessory has no independent texture, the main texture SHALL be 4096 × 4096 and its template SHALL superpose all those painted options. When every painted part has its own texture, the main texture SHALL be 64 × 64 with a blank template: it only carries the trailer's base colour, as the game's fill mask does. Accessory textures SHALL be shared by the whole trailer; trailers SHALL have no variants. Accessory ids that use the same sized mask in every game paint job that overrides any of them SHALL form one accessory texture listing all those game ids, whose size follows the size rules applied to that mask and whose template superposes their layouts; ids whose masks differ in at least one paint job SHALL keep separate textures. A trailer whose paint jobs define no main mask and no override (only plain, possibly locked, colours) cannot carry a livery: it SHALL be reported as `skipped` with a `not_paintable` reason, SHALL produce no package and SHALL NOT count as a failure. The report SHALL record each vehicle's kind and, for every grouped texture, its game ids and the shared mask references.

#### Scenario: Bodies sharing one livery region
- **WHEN** the dry van, insulated and reefer 13.6 m bodies of `scs.box` use the same 4096×2048 mask in every paint job that overrides them
- **THEN** the package has one accessory texture of 4096 × 2048 listing the three body ids, whose template shows the three layouts superposed

#### Scenario: Ids that differ in one paint job
- **WHEN** two body ids share a mask in most paint jobs but one paint job gives them different masks
- **THEN** they keep separate accessory textures

#### Scenario: Trailer main texture
- **WHEN** a trailer's chassis has no paint override with crop dimensions
- **THEN** the chassis contributes to the single 4096 × 4096 main texture, which has no cabin ids

#### Scenario: Every part has its own region
- **WHEN** every painted body, chassis and accessory of a trailer has an independent texture
- **THEN** the package has a 64 × 64 main texture with a blank template and no cabin ids

#### Scenario: Colour-only trailer
- **WHEN** a trailer's paint jobs only set base colours, with no mask and no override
- **THEN** it is reported as skipped and not paintable, no package is written, and the batch can still succeed

#### Scenario: Install and export an extracted trailer
- **WHEN** the player installs `official.ets2.krone.coolliner` and exports a mod painting its main texture and body
- **THEN** the mod writes the trailer paint job under `trailer_owned` with one override for the body texture listing its game ids
