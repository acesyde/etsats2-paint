## MODIFIED Requirements

### Requirement: Full checks for code changes
For every pull request and every push to `main` that changes at least one file that is not documentation, CI SHALL run, in order:
1. format and lint checks once;
2. tests and a release build on Linux, Windows and macOS.

Documentation files are Markdown files (`*.md`) anywhere in the repository, and every file under `openspec/`, `.claude/` or `website/`. Any other file, including files of a kind CI does not know, SHALL be treated as code.

#### Scenario: Code change
- **WHEN** a pull request changes `crates/tp-app/src/state.rs` and `README.md`
- **THEN** format, lint, tests and builds run on all three platforms

#### Scenario: Unknown file kind
- **WHEN** a pull request only adds `docs/diagram.svg`
- **THEN** the full checks run

#### Scenario: Documentation site change
- **WHEN** a pull request only changes files under `website/` (pages, media, styles, the site's package files)
- **THEN** format, lint, tests and builds are skipped

## ADDED Requirements

### Requirement: Documentation site build and deployment
For every pull request that changes a file under `website/`, CI SHALL build the documentation site and fail when the build fails. For every push to `main` that changes a file under `website/`, CI SHALL build the site and deploy it to GitHub Pages. These jobs SHALL need no Rust toolchain and no GPU, and SHALL NOT be part of the "CI result" check.

#### Scenario: Broken page in a pull request
- **WHEN** a pull request references a screenshot missing in German
- **THEN** the documentation build job fails on that pull request

#### Scenario: Deploy on merge
- **WHEN** a push to `main` changes `website/src/content/docs/fr/export/mod.mdx`
- **THEN** the site is built and deployed to GitHub Pages

#### Scenario: No site change
- **WHEN** a push to `main` only changes Rust files
- **THEN** the documentation site is neither built nor deployed
