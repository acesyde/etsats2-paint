#!/usr/bin/env bash
# Self-test of ci-changes.sh on a throwaway Git repository.

set -euo pipefail

script="$(cd "$(dirname "$0")" && pwd)/ci-changes.sh"
repo="$(mktemp -d)"
trap 'rm -rf "$repo"' EXIT
cd "$repo"
git init -q -b main
git config user.email "test@example.invalid"
git config user.name "ci-changes test"
git config commit.gpgsign false

commit() {
    git add -A
    git commit -q --allow-empty -m "$1"
    git rev-parse HEAD
}

failures=0
expect() {
    local name="$1" want="$2" base="$3" head="$4"
    local got
    got=$(bash "$script" "$base" "$head" 2>/dev/null | tail -n 1)
    if [ "$got" = "code=$want" ]; then
        echo "ok   $name"
    else
        echo "FAIL $name: expected code=$want, got $got"
        failures=$((failures + 1))
    fi
}

mkdir -p crates/app/src openspec/specs .claude docs
echo 'fn main() {}' > crates/app/src/main.rs
echo '# Readme' > README.md
root=$(commit "initial")

echo 'more' >> README.md
echo 'spec' > openspec/specs/spec.md
echo 'cfg' > .claude/settings.json
echo 'nested' > docs/guide.md
docs=$(commit "docs only")
expect "docs only (md, openspec, .claude)" false "$root" "$docs"

echo '// change' >> crates/app/src/main.rs
code=$(commit "code only")
expect "code only" true "$docs" "$code"

echo 'again' >> README.md
echo '// again' >> crates/app/src/main.rs
mixed=$(commit "docs and code")
expect "mixed docs and code" true "$code" "$mixed"

echo '<svg/>' > docs/diagram.svg
unknown=$(commit "unknown file kind")
expect "unknown file kind" true "$mixed" "$unknown"

git mv crates/app/src/main.rs crates/app/src/app.rs
renamed=$(commit "rename code")
expect "renamed code file" true "$unknown" "$renamed"

git mv docs/guide.md openspec/guide.md
moved=$(commit "move docs")
expect "moved docs" false "$renamed" "$moved"

empty=$(commit "empty")
expect "empty diff" true "$moved" "$empty"

expect "unknown base (empty)" true "" "$empty"
expect "unknown base (zeros)" true "0000000000000000000000000000000000000000" "$empty"
expect "missing base commit" true "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef" "$empty"

# A pull request: changes on the base branch after the fork do not count.
git checkout -q -b feature "$moved"
echo 'pr docs' >> README.md
pr=$(commit "pr docs")
git checkout -q main
echo '// main moved on' >> crates/app/src/app.rs
main_tip=$(commit "main code")
expect "pull request vs moved base" false "$main_tip" "$pr"

# $GITHUB_OUTPUT and $GITHUB_STEP_SUMMARY are written.
out="$repo/.out"
sum="$repo/.summary"
GITHUB_OUTPUT="$out" GITHUB_STEP_SUMMARY="$sum" bash "$script" "$root" "$docs" > /dev/null 2>&1
if grep -qx 'code=false' "$out" && grep -q 'Documentation-only change' "$sum" &&
    grep -q 'openspec/specs/spec.md' "$sum"; then
    echo "ok   GitHub output and summary"
else
    echo "FAIL GitHub output and summary"
    failures=$((failures + 1))
fi

if [ "$failures" -gt 0 ]; then
    echo "$failures failure(s)"
    exit 1
fi
echo "all passed"
