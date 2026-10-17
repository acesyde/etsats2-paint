## MODIFIED Requirements

### Requirement: Isolated batch outputs
Each invocation that starts a batch SHALL create a unique batch subdirectory under the output parent, defaulting to `dist/`. Intermediates SHALL remain under `target/`, separate from published packages, in a work folder unique to the batch. When the batch succeeds (every requested or discovered vehicle built or skipped as not paintable), its work folder SHALL be deleted after the report is written. When any vehicle fails or the report cannot be written, the work folder SHALL be kept, so converter logs named in failure messages remain available. Work folders of other batches SHALL NOT be touched. Existing batch contents SHALL NOT be overwritten. Vehicle failures SHALL NOT stop other vehicles. Only fully built and validated packages SHALL be published; failed vehicles and interrupted package writes SHALL leave no partial `.tpv`. The command SHALL return zero only when every requested/discovered vehicle succeeds or is skipped as not paintable and the final report is written; any incomplete batch or output failure SHALL return nonzero.

#### Scenario: Converter failure for one truck
- **WHEN** the converter fails for one vehicle and succeeds for another
- **THEN** the successful package is published, the failed vehicle has no package, both results are reported and the command returns nonzero

#### Scenario: Repeated invocation
- **WHEN** two batches use the same output parent
- **THEN** they use distinct directories and neither overwrites the other's outputs

#### Scenario: Package write failure
- **WHEN** publishing a package fails partway through writing
- **THEN** no partial `.tpv` is left as a published package and the command returns nonzero

#### Scenario: Successful batch cleans its work folder
- **WHEN** every vehicle of a batch is built or skipped as not paintable
- **THEN** the packages and report are published and the batch's work folder under `target/` no longer exists

#### Scenario: Failed batch keeps its work folder
- **WHEN** one vehicle of a batch fails
- **THEN** the batch's work folder, with its converter logs, is kept and the report names it

### Requirement: Traceable technical report
Every completed or recoverably failed batch SHALL publish `report.json` beside its successful packages. The report SHALL identify the selected game installation, mounted archive sources and precedence, game version, converter identity/version when available, tool and package versions, requested and discovered vehicles, successes, failures and limits, the batch work folder with whether it was kept, and the model cache folder with the number of models converted and reused. Per-vehicle evidence SHALL identify source definitions and model/look/variant/UV selections contributing to each texture, its dimensions and coverage, or the stage and reason for refusal. Unavailable source/converter version information SHALL be marked explicitly. Native package validation SHALL be described as technical validation, never as proof of in-game correctness. Newly generated vehicles SHALL be identified as needing in-game checks unless explicit matching in-game evidence is available.

#### Scenario: Technical success without game evidence
- **WHEN** a new truck passes extraction and native package validation
- **THEN** the report records technical success and separately states that in-game verification remains required

#### Scenario: Explain a failed vehicle
- **WHEN** a vehicle fails, for example because a required model file is missing
- **THEN** its report identifies the concerned definitions or files and the failure stage

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
- **THEN** extraction refuses the vehicle rather than treating the definition as an orphan reference

#### Scenario: Accessory without livery material
- **WHEN** a paint override references an installed accessory whose selected model has no truckpaint material
- **THEN** the vehicle package is generated without a template for it, and `missing_dependencies` records it as `surface_without_livery_material` with an `excluded` disposition

## ADDED Requirements

### Requirement: Model cache between batches
Converted models SHALL be kept in a cache under `target/` shared by batches, keyed by the identity of every mounted archive (name, size, modification time) and of the converter executable (path, size, modification time). A batch whose key matches the cache's SHALL reuse a cached model instead of converting it again. A batch whose key differs SHALL clear the cache before converting, so no model converted from other game files or by another converter is used. A model SHALL enter the cache only once its conversion has succeeded completely; an interrupted or failed conversion SHALL leave no cached model. Packages and report evidence SHALL be identical whether models were reused or converted. The cache SHALL survive the cleanup of a batch work folder.

#### Scenario: Second batch on the same game
- **WHEN** a batch runs after another on the same archives with the same converter
- **THEN** it converts no model already converted, reports them as reused, and publishes byte-identical packages

#### Scenario: Game update
- **WHEN** a mounted archive's size or modification time differs from the cache key, or an archive is added or removed
- **THEN** the cache is cleared and every model the batch needs is converted again

#### Scenario: Interrupted conversion
- **WHEN** a model conversion fails or the batch stops during it
- **THEN** that model is not in the cache and the next batch converts it
