## Why

Every pull request runs the full CI: format and lint, then tests and a release build on Linux, Windows and macOS. That takes several minutes of runner time, plus the cold-cache risk on three platforms. Many PRs change only documentation or OpenSpec files, for example every `chore/archive-*` PR. Their files can't affect the build, so running the Rust checks for them only costs time and delays the merge.

## What Changes

- **Changed-files job:** CI first lists the files changed by the pull request, or by the push to `main`.
- **Docs-only changes skip the Rust jobs:** when every changed file is documentation, format, lint, tests and builds are skipped. Documentation means:
  - Markdown files (`*.md`) anywhere;
  - anything under `openspec/`;
  - anything under `.claude/`.
- **Everything else runs the full CI:**
  - code, assets, Cargo files, `mise.toml` and workflows;
  - any new kind of file;
  - a mix of documentation and code.
  When in doubt (e.g. the base commit can't be determined), CI runs.
- **One stable result check:** a final "CI result" job always reports, as success when the Rust jobs passed or were skipped for a docs-only change, and as failure otherwise. This keeps branch protection usable, because GitHub doesn't report skipped matrix jobs under their per-OS names. It's the check to mark as required.
- **Run summary:** the CI summary says when and why the Rust jobs were skipped, and lists the changed files.
- **Local check:** the detection logic is a mise task, so it runs the same way locally and in CI.

## Capabilities

### New Capabilities

- `continuous-integration`: what CI checks on pull requests and pushes to `main`, when the checks are skipped for documentation-only changes, and the required result check.

### Modified Capabilities

(none)

## Impact

- **`.github/workflows/ci.yml`:**
  - a new `changes` job (checkout with history, then a mise task) whose output gates `checks` and `build`;
  - a new `result` job that aggregates the outcomes.
- **`mise.toml`:** a new `ci:changes` task wrapping a small Bash script that compares two commits and prints whether code changed.
- **Repository settings:** after the merge, the branch protection's required checks should be "CI result" instead of the per-job checks, if any are configured. This is a manual step for the repository owner.
- No application code changes.
