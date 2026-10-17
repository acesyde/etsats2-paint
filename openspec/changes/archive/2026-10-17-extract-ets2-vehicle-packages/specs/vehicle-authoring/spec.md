## ADDED Requirements

### Requirement: Maintained game extraction command
The Rust `tpv` command SHALL provide the ETS2 extraction interface and behavior defined by the ets2-game-extraction capability alongside its existing pack and check commands. The old `tpv catalog check` and `tpv catalog sync` commands SHALL be removed from the accepted interface and help.

#### Scenario: Authoring command help
- **WHEN** an author asks for `tpv --help`
- **THEN** it documents extract, pack and check, with no Carsmaniac catalog commands

#### Scenario: Removed command
- **WHEN** an author invokes `tpv catalog sync`
- **THEN** the tool rejects the command with a nonzero exit status
