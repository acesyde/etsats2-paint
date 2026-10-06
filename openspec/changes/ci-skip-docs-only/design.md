## Context

`.github/workflows/ci.yml` runs on every `pull_request` and on pushes to `main`:
- **`checks`:** format and lint on Ubuntu;
- **`build`:** depends on `checks`; tests and a release build in an OS matrix (Ubuntu, Windows, macOS).

CI steps run mise tasks (`mise run fmt:check`, `lint`, `test:release`, `build`), with `MISE_TASK_RUN_AUTO_INSTALL=false` and only Rust installed by `jdx/mise-action`. Actions are pinned by commit SHA.

The repository's branch protection settings can't be read from this environment (no API access to the repo). The design therefore has to work whether or not required status checks are configured.

## Goals / Non-Goals

**Goals:**
- Documentation-only PRs and pushes finish in seconds and stay mergeable.
- The detection fails safe: it runs the full CI whenever in doubt.
- The detection logic is testable locally and stays a mise task, like the other CI steps.

**Non-Goals:**
- Skipping parts of the matrix depending on which crate changed.
- Caching changes.
- Skipping CI for other "safe" files, such as images in `assets/`, which may be embedded in the binary.

## Decisions

### D1. Job-level gating, not workflow `paths-ignore`

A `changes` job outputs `code=true|false`, and `checks` and `build` run with `if: needs.changes.outputs.code == 'true'`.

Alternative considered: `on.pull_request.paths-ignore`. Rejected for two reasons:
- a workflow that doesn't start reports no checks at all, so any required check would wait forever and block the merge;
- it can't express "skip only if every file is docs" for mixed changes. `paths-ignore` does handle this, but its semantics are subtle and it can't produce a summary.

### D2. Aggregate "CI result" job

`result` runs with `needs: [changes, checks, build]` and `if: always()`:
- it fails when `needs.*.result` contains `failure` or `cancelled`, or when the `changes` job itself didn't succeed;
- otherwise it succeeds, including when the Rust jobs were `skipped` for a docs-only change.

It's needed because GitHub reports a skipped matrix job under its unexpanded name ("Test & build (${{ matrix.os }})"), so required checks named per OS would never resolve. Requiring only "CI result" avoids that. The README documents the setting.

### D3. Detection as a mise task with a Bash script

The new `ci:changes` task in `mise.toml` runs `scripts/ci-changes.sh BASE HEAD`. The script:
1. Exits with "code changed" when `BASE` is empty, all zeros (new branch or force push) or not a commit present locally.
2. Runs `git diff --name-only BASE...HEAD`: a merge-base diff for PRs, so files changed on the base branch don't count.
3. Matches each path with a `case` statement against `*.md`, `openspec/*` and `.claude/*` (in `case`, `*` also matches `/`).
4. Prints `code=true|false`, appending it to `$GITHUB_OUTPUT` when that's set, and writes the list of changed files to `$GITHUB_STEP_SUMMARY` when skipping.

The commits compared:
- **PRs:** `github.event.pull_request.base.sha` and `github.sha`; the PR merge commit and base are both fetched with `fetch-depth: 0`.
- **Pushes:** `github.event.before` and `github.sha`.

The repository is small, so a full history fetch is cheap.

The `changes` job installs mise without tools (the `mise-action` `install: false` input), since the script only needs Git and Bash.

Alternative considered: `dorny/paths-filter`. Rejected: a third-party action to pin and audit, for logic that fits in a short script testable locally.

### D4. Fail-safe defaults

The job falls back to `code=true` in every one of these cases:
- the script errors (the step's output defaults to `true`, and the gating condition only skips on the exact string `false`);
- the base is unknown;
- the diff is empty, which happens for a re-run or a merge with no file changes.

## Risks / Trade-offs

- [A Markdown file that the build uses, e.g. `include_str!("../README.md")`] → It would be skipped. Nothing does this today (checked: no `include_str!` of `.md` files). The rule is documented in the spec and in the CI file.
- [The required checks need updating in the repository settings] → This is a manual step, called out in the PR description. Until it's done, existing per-job required checks keep working for code PRs, and docs-only PRs would wait on them.
- [A full-history fetch on each run] → The repo is small. Only the `changes` job fetches the full history; the other jobs keep their shallow checkout.

## Migration Plan

1. Merge.
2. In GitHub › Settings › Branches (or Rulesets), replace the required checks with "CI result".
3. Rollback: revert the workflow commit.
