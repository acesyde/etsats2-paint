#!/usr/bin/env bash
# Decides whether a change touches code, so CI can skip the Rust jobs for
# documentation-only changes.
#
# Usage: ci-changes.sh BASE HEAD
#   BASE: the base branch commit (pull requests) or the previous commit
#         (pushes); empty or all zeros when unknown.
#   HEAD: the commit being checked.
#
# Prints `code=true` or `code=false` (also appended to $GITHUB_OUTPUT when
# set). Documentation files are `*.md` anywhere and everything under
# `openspec/` and `.claude/`; any other file counts as code. Fails safe:
# whenever the changed files cannot be determined, it reports code=true.

set -uo pipefail

base="${1:-}"
head="${2:-HEAD}"

report() {
    echo "code=$1"
    if [ -n "${GITHUB_OUTPUT:-}" ]; then
        echo "code=$1" >> "$GITHUB_OUTPUT"
    fi
}

summary() {
    if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
        printf '%s\n' "$@" >> "$GITHUB_STEP_SUMMARY"
    fi
}

run_everything() {
    echo "Running the full CI: $1" >&2
    summary "## Changed files" "" "Full CI: $1."
    report true
    exit 0
}

# Unknown base: new branch, force push, first commit, or missing history.
if [ -z "$base" ] || [[ "$base" =~ ^0+$ ]]; then
    run_everything "the base commit is unknown"
fi
if ! git cat-file -e "${base}^{commit}" 2>/dev/null; then
    run_everything "the base commit $base is not available"
fi
if ! git cat-file -e "${head}^{commit}" 2>/dev/null; then
    run_everything "the commit $head is not available"
fi

# Merge-base diff: changes on the base branch since the fork do not count.
if ! files=$(git diff --name-only --no-renames "${base}...${head}"); then
    run_everything "the changed files could not be listed"
fi
if [ -z "$files" ]; then
    run_everything "no changed files were found"
fi

code=false
while IFS= read -r file; do
    case "$file" in
        *.md | openspec/* | .claude/*) ;;
        *)
            code=true
            break
            ;;
    esac
done <<< "$files"

if [ "$code" = true ]; then
    summary "## Changed files" "" "Code changed: running format, lint, tests and builds."
else
    echo "Documentation-only change: skipping build, lint and tests." >&2
    summary "## Changed files" "" \
        "Documentation-only change: build, lint and tests were skipped." "" \
        '```' "$files" '```'
fi
report "$code"
