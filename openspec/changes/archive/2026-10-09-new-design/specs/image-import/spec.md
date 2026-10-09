## MODIFIED Requirements

### Requirement: Assets are deduplicated
Importing a file whose content is identical to an existing asset SHALL reuse that asset instead of storing a second copy.

#### Scenario: Same logo twice
- **WHEN** the user places "logo.png" twice
- **THEN** two image objects exist and the images list of the Resources tab lists one asset used 2 times
