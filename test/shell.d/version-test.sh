#!/usr/bin/env bash

set -euo pipefail

source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/base-test.sh"

# omarchy-version is a Rust CLI binary, not a shell script.
# Prefer the most recently built binary: debug build, then nix build output, then PATH.
if [[ -x $ROOT/rust/target/debug/omarchy ]]; then
  omarchy_version() { local p="${1:-}"; OMARCHY_PATH="$p" "$ROOT/rust/target/debug/omarchy" version; }
elif [[ -x $ROOT/result/bin/omarchy-version ]]; then
  omarchy_version() { local p="${1:-}"; OMARCHY_PATH="$p" "$ROOT/result/bin/omarchy-version"; }
elif command -v omarchy-version >/dev/null 2>&1; then
  omarchy_version() { local p="${1:-}"; OMARCHY_PATH="$p" omarchy-version; }
else
  skip "omarchy-version binary not found; build the package first (nix build .#omarchy-cli)"
  exit 0
fi

test_tmp=$(mktemp -d)
trap 'rm -rf "$test_tmp"' EXIT

# Dev checkout: OMARCHY_PATH pointing to the repo has a .git dir, so git
# rev-parse succeeds and the output matches "dev (<hash>)".
if [[ ! -d $ROOT/.git ]]; then
  skip "dev checkout reports a git hash # SKIP: no .git at ROOT (synced without git history)"
else
  output=$(omarchy_version "$ROOT" 2>&1)
  [[ $output == "dev ("* ]] || fail "dev checkout reports a git hash" "got: $output"
  pass "dev checkout reports a git hash"
fi

# Production install: a plain directory with a version file returns its contents.
prod_dir="$test_tmp/prod"
mkdir -p "$prod_dir"
printf '5.0.0\n' >"$prod_dir/version"
output=$(omarchy_version "$prod_dir" 2>&1)
[[ $output == "5.0.0" ]] || fail "production install reads the version file" "got: $output"
pass "production install reads the version file"

# Missing version file: exits non-zero and prints an error.
empty_dir="$test_tmp/empty"
mkdir -p "$empty_dir"
omarchy_version "$empty_dir" >/dev/null 2>&1 && fail "missing version file exits successfully — should fail"
pass "missing version file exits non-zero"
