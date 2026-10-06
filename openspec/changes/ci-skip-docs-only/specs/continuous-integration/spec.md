## Purpose

Defines what the continuous integration checks on pull requests and pushes to `main`, when the Rust checks are skipped because only documentation changed, and the single result check that branch protection relies on.

## ADDED Requirements

### Requirement: Full checks for code changes
For every pull request and every push to `main` that changes at least one file that is not documentation, CI SHALL run, in order:
1. format and lint checks once;
2. tests and a release build on Linux, Windows and macOS.

Documentation files are Markdown files (`*.md`) anywhere in the repository, and every file under `openspec/` or `.claude/`. Any other file, including files of a kind CI does not know, SHALL be treated as code.

#### Scenario: Code change
- **WHEN** a pull request changes `crates/tp-app/src/state.rs` and `README.md`
- **THEN** format, lint, tests and builds run on all three platforms

#### Scenario: Unknown file kind
- **WHEN** a pull request only adds `docs/diagram.svg`
- **THEN** the full checks run

### Requirement: Documentation-only changes skip the Rust checks
When every file changed by a pull request (compared with its base branch) or by a push to `main` (compared with the previous commit of `main`) is a documentation file, CI SHALL skip format, lint, tests and builds. When the changed files cannot be determined (for example the previous commit is unknown), CI SHALL run the full checks.

#### Scenario: Archive pull request
- **WHEN** a pull request only moves files under `openspec/changes/` and edits files under `openspec/specs/`
- **THEN** format, lint, tests and builds are skipped

#### Scenario: Unknown base
- **WHEN** CI cannot determine which files a push changed
- **THEN** the full checks run

### Requirement: Single result check
CI SHALL always end with a check named "CI result". It SHALL succeed when the Rust checks all passed, or were all skipped for a documentation-only change, and SHALL fail when any of them failed or was cancelled. Branch protection can require this one check whatever the change.

#### Scenario: Docs-only pull request can merge
- **WHEN** "CI result" is the required check and a pull request only changes `README.md`
- **THEN** "CI result" succeeds and the pull request is mergeable without running builds

#### Scenario: A failing platform fails the result
- **WHEN** tests fail on Windows only
- **THEN** "CI result" fails

### Requirement: Explained skips
When the Rust checks are skipped, the CI run summary SHALL say that the change is documentation-only and list the changed files. The detection SHALL be available as a mise task so it can be run locally with the same result as in CI.

#### Scenario: Summary of a skipped run
- **WHEN** a documentation-only pull request is checked
- **THEN** the run summary states that build, lint and tests were skipped because only documentation changed, followed by the changed files
