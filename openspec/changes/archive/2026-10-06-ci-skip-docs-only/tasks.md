## 1. Detection

- [x] 1.1 Add `scripts/ci-changes.sh` (base/head arguments, fail-safe cases, merge-base diff, docs patterns, `code=` output, `$GITHUB_OUTPUT` and the step summary) and the `ci:changes` mise task. Verify by running it locally:
  - `HEAD~1..HEAD` on an archive commit gives `code=false`;
  - on a feature commit it gives `code=true`;
  - an all-zeros base gives `code=true`;
  - an unknown base gives `code=true`;
  - a mixed `.md` plus `.rs` change gives `code=true`.
- [x] 1.2 Add a self-test (`scripts/ci-changes-test.sh`, run by a `ci:changes:test` mise task) that builds a throwaway Git repository with docs-only, code, mixed, unknown-file, rename and empty commits and checks each output. Verify that the test task passes locally.

## 2. Workflow

- [x] 2.1 Add the `changes` job (full-history checkout, mise without tools, `mise run ci:changes` with the PR or push commits) and gate `checks` and `build` on `needs.changes.outputs.code == 'true'`. Verify with `actionlint`, if available via mise, and by reviewing that both events pass the right commits.
- [x] 2.2 Add the `result` job ("CI result", `if: always()`, fails on any failure or cancellation or a failed `changes` job). Verify that the expression logic covers the success, skipped, failed and cancelled cases, listed in the PR description.
- [x] 2.3 Document the rule and the required "CI result" check in the README's development section. Verify that the README states which files are docs-only and which check to require.

## 3. Validation

- [x] 3.1 Open the PR. It touches `.github/` and `scripts/`, so CI must run in full, and "CI result" must pass. Verify on GitHub; the user reports the result.
- [ ] 3.2 After the merge, check that the next docs-only PR (e.g. the archive PR of this change) skips the Rust jobs and that "CI result" passes. Verify on GitHub; the user reports the result.
