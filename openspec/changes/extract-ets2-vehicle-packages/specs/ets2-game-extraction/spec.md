## Purpose

Generate autonomous official ETS2 truck packages from an installed game's definitions and models, with explicit coverage checks and traceable batch results.

## ADDED Requirements

### Requirement: Extraction command and source boundary
The tool SHALL expose `tpv extract <installation> --converter <executable> [--vehicle <identifier>] [--output <directory>] --package-version <version>`. The converter and a valid SemVer package version SHALL be required. Without a vehicle filter it SHALL discover official trucks from the installed ETS2 archives, including installed vehicle and tuning DLC. It SHALL exclude user mods, ATS vehicles and trailers, and SHALL NOT modify the installation. An unknown or excluded requested vehicle, invalid installation or invalid arguments SHALL return failure rather than an empty successful batch.

#### Scenario: Installed DLC trucks
- **WHEN** extraction runs without a vehicle filter on ETS2 with vehicle and tuning DLC installed
- **THEN** its truck discovery and painted option coverage include those DLC and exclude user mods and trailers

#### Scenario: Explicit version
- **WHEN** the package version is missing or is not SemVer
- **THEN** extraction fails before producing packages

#### Scenario: Filtered vehicle
- **WHEN** `--vehicle daf.2021` names a discovered truck
- **THEN** only that vehicle is processed, with its available cabins and accessories

#### Scenario: Unsupported installation or vehicle
- **WHEN** the installation is ATS or the requested vehicle is missing or a trailer
- **THEN** the command fails and produces no package for that request

### Requirement: Definition-driven selection
The extractor SHALL derive vehicle metadata, cabins, accessory game ids, model references and compatibility from game definitions rather than per-vehicle profiles. It SHALL resolve referenced inclusions and archive overrides with recorded provenance. It SHALL select each model's requested look and variant, retain only visible painted faces, and choose the UV stream by its semantic alias. Missing required files, inclusion cycles, conflicting unresolved definitions, unsupported paint shaders or UV mappings, and ambiguous looks, variants or UV streams SHALL refuse the affected vehicle. Unsupported data SHALL NOT be silently omitted. Paint overrides referencing accessory ids with no definition in the installed source view SHALL be excluded from texture assignments and reported as ignored references, without refusing the vehicle. A defined accessory with missing or unusable painted coverage SHALL still refuse the vehicle.

#### Scenario: Included shared model
- **WHEN** an accessory definition includes a file that references a shared model outside the truck's model directory
- **THEN** the referenced definition and model contribute to the vehicle and their source is recorded

#### Scenario: Visible variant and semantic UV alias
- **WHEN** a look contains painted materials and a variant hides one model part, with the paint UV alias on a stream whose numeric tag differs
- **THEN** the template contains only visible painted geometry using the aliased stream, without an additional vertical flip

#### Scenario: Ambiguous or unsupported selection
- **WHEN** a definition has an unresolved include or its painted model has ambiguous look, variant or UV data, or an unsupported alternate UV mapping
- **THEN** the vehicle fails with the concerned definition or model identified

### Requirement: Shared shader and UV rules
The extractor SHALL recognize the `eut2.truckpaint`, `eut2.truckpaint.tsnmap` and `eut2.truckpaint.tsnmapuv` families with optional `airbrush`, `flipflake` and `altuv` flavors and optional `.a` or `.asafew.a` alpha-test suffixes, without vehicle-specific profiles. Alpha-tested material geometry SHALL contribute its UV wireframe without attempting pixel-level opacity-mask clipping. Other flavors and conflicting normal-map flavors SHALL remain unsupported. Only visible painted materials of the selected cabin looks/variants SHALL establish the package UV mode: all such cabin materials SHALL agree on ordinary or alternate mapping. Ordinary mapping SHALL use `_TEXCOORD1`, alternate mapping SHALL use `_TEXCOORD2`. Every painted accessory SHALL use the package mode, independently of its source material's paint-job flavor. The package's existing `game.alt_uv` flag SHALL match that mode so mod export selects the same layout. Missing or ambiguous selected UV aliases, invalid selected geometry and conflicting cabin modes SHALL refuse the vehicle. Reports SHALL identify selected shaders, UV aliases, coordinate bounds and out-of-bounds vertex counts. Finite coordinates outside [0, 1] SHALL be preserved without wrapping, coordinate clamping or automatic UV-channel changes; the SVG viewport SHALL bound the visible wireframe. Non-finite coordinates and coordinates that cannot be represented by SVG edge quantization SHALL refuse the vehicle. Definition-driven exterior model resolution SHALL include `detail_model`, including UK variants reusing that model.

#### Scenario: Ordinary family without normal maps
- **WHEN** a visible painted face uses `eut2.truckpaint` and its cabin establishes ordinary UV mapping
- **THEN** extraction selects `_TEXCOORD1` by alias and writes `game.alt_uv: false`

#### Scenario: Alternate cabin and shared accessory mapping
- **WHEN** cabin paint materials consistently use an `altuv` flavor and a painted accessory has a different source flavor
- **THEN** all painted contributions use `_TEXCOORD2`, the package records `game.alt_uv: true` and native mod export can emit `alternate_uvset: true`

#### Scenario: Conflicting cabin modes
- **WHEN** visible painted cabin materials disagree about ordinary versus alternate UV mapping
- **THEN** the vehicle is refused with the relevant cabin definition identified

#### Scenario: Missing selected mapping
- **WHEN** the chosen package mode requires an absent or ambiguous UV alias on a painted accessory
- **THEN** extraction refuses the vehicle rather than falling back to another UV stream

#### Scenario: Metallic paint flavor
- **WHEN** a selected painted material uses `flipflake`, optionally with airbrush, a normal map and alternate UVs
- **THEN** its geometry contributes using the same semantic UV rules, while duplicate or unknown flavors remain invalid

#### Scenario: Finite out-of-bounds coordinates
- **WHEN** selected painted geometry has finite representable UV coordinates outside [0, 1]
- **THEN** its original coordinates and selected alias are retained, its bounds and overflow count are reported, and no wrapping or alternate-channel fallback is applied

#### Scenario: Detailed cabin model
- **WHEN** a cabin references `detail_model` and a UK variant reuses it
- **THEN** both selected variants contribute to its template with recorded provenance

### Requirement: Complete autonomous vehicle package
Each successful vehicle SHALL produce one `.tpv` containing its SVG templates and manifest, with id `official.ets2.<vehicle>` and the supplied package version. It SHALL record the game path, supported game version range and paint/export metadata. Every cabin SHALL be covered exactly once by a main texture, and every supported painted accessory assignment SHALL be covered by its independent texture or the appropriate main texture. Main templates SHALL superpose compatible painted options. Cabin textures SHALL be 4096 × 4096. Accessory dimensions SHALL come from game paint job definitions using the validated crop-name method; missing or conflicting dimensions SHALL refuse the vehicle. All such accessory failures SHALL appear in `dimension_diagnostics`, with candidate sizes and mask references tied to their source definitions/archives. Shared fill masks without usable crop dimensions SHALL NOT supply an inferred template size. Empty templates, invalid geometry and incomplete coverage SHALL refuse the vehicle. Successful packages SHALL pass the same validation as manual application installation and SHALL require neither the game nor ConverterPIX for installation and use.

#### Scenario: Complete MAN or DAF
- **WHEN** extraction succeeds for MAN TGX or DAF NGD
- **THEN** its package contains every discovered cabin and painted accessory, valid export identifiers and all referenced templates

#### Scenario: Conflicting accessory dimensions
- **WHEN** an accessory has two incompatible declared crop dimensions or no usable dimension
- **THEN** the vehicle fails and no package for it is published

#### Scenario: Missing painted coverage
- **WHEN** a cabin or painted accessory cannot be assigned a valid nonempty template
- **THEN** the entire vehicle is refused rather than producing a reduced package

### Requirement: Isolated batch outputs
Each invocation that starts a batch SHALL create a unique batch subdirectory under the output parent, defaulting to `dist/`. Intermediates SHALL remain under `target/`, separate from published packages. Existing batch contents SHALL NOT be overwritten. Vehicle failures SHALL NOT stop other vehicles. Only fully built and validated packages SHALL be published; failed vehicles and interrupted package writes SHALL leave no partial `.tpv`. The command SHALL return zero only when all requested/discovered vehicles succeed and the final report is written; any incomplete batch or output failure SHALL return nonzero.

#### Scenario: Converter failure for one truck
- **WHEN** the converter fails for one vehicle and succeeds for another
- **THEN** the successful package is published, the failed vehicle has no package, both results are reported and the command returns nonzero

#### Scenario: Repeated invocation
- **WHEN** two batches use the same output parent
- **THEN** they use distinct directories and neither overwrites the other's outputs

#### Scenario: Package write failure
- **WHEN** publishing a package fails partway through writing
- **THEN** no partial `.tpv` is left as a published package and the command returns nonzero

### Requirement: Traceable technical report
Every completed or recoverably failed batch SHALL publish `report.json` beside its successful packages. The report SHALL identify the selected game installation, mounted archive sources and precedence, game version, converter identity/version when available, tool and package versions, requested and discovered vehicles, successes, failures and limits. Per-vehicle evidence SHALL identify source definitions and model/look/variant/UV selections contributing to each texture, its dimensions and coverage, or the stage and reason for refusal. Unavailable source/converter version information SHALL be marked explicitly. Native package validation SHALL be described as technical validation, never as proof of in-game correctness. Newly generated vehicles SHALL be identified as needing in-game checks unless explicit matching in-game evidence is available.

#### Scenario: Technical success without game evidence
- **WHEN** a new truck passes extraction and native package validation
- **THEN** the report records technical success and separately states that in-game verification remains required

#### Scenario: Explain a failed vehicle
- **WHEN** a vehicle fails because an accessory size is ambiguous
- **THEN** its report identifies the accessory, relevant definitions, conflicting dimensions and failure stage

#### Scenario: Missing reference with unknown DLC ownership
- **WHEN** a required game file is missing
- **THEN** the vehicle is refused and its `missing_dependencies` entries identify the missing reference, its kind and a `blocking` disposition
- **AND** `expected_dlc` is null and `dlc_origin` is `unknown` when the installed definitions do not establish which DLC supplies the missing data; the archive containing a reference is not presented as the missing DLC

#### Scenario: Orphan paint override does not prevent extraction
- **WHEN** a paint override references accessory ids with no corresponding installed definition and all defined painted content can be covered
- **THEN** the vehicle package is generated with the same templates as without those references
- **AND** `missing_dependencies` records each excluded id, referencing definitions and archives, an `ignored_reference` disposition and unknown DLC ownership rather than an asserted missing DLC

#### Scenario: Defined accessory is still required
- **WHEN** a paint override references an installed accessory definition whose painted geometry cannot be extracted
- **THEN** extraction refuses the vehicle and records a `blocking` disposition rather than treating the definition as an orphan reference
