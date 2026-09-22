#!/bin/bash

set -euo pipefail

source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/base-test.sh"

# Test the cinque-shim versions of pkg-present and pkg-missing against a
# synthetic packages.json. These scripts read /run/omarchy/packages.json, so
# we override the path via a temp file and a PATH-prepended wrapper.

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

manifest="$tmpdir/packages.json"
cat > "$manifest" << 'JSON'
[
  {"name": "bat", "version": "0.25.0"},
  {"name": "ripgrep", "version": "14.1.1"}
]
JSON

shim_dir="$tmpdir/shims"
mkdir -p "$shim_dir"

# Wrap the shim scripts so they read our tmp manifest instead of the live one.
# Rewrite the shebang to use the current bash so tests run outside NixOS too.
bash_bin=$(command -v bash)
for cmd in omarchy-pkg-present omarchy-pkg-missing; do
  script="$ROOT/pkgs/omarchy/cinque-shims/$cmd"
  wrapper="$shim_dir/$cmd"
  # Replace manifest path and shebang in a copy.
  sed \
    -e "s|/run/omarchy/packages.json|$manifest|g" \
    -e "1s|#!/bin/bash|#!$bash_bin|" \
    "$script" > "$wrapper"
  chmod +x "$wrapper"
done

export PATH="$shim_dir:$ROOT/bin:$PATH"

# ── omarchy-pkg-present ────────────────────────────────────────────────────────

omarchy-pkg-present bat \
  || fail "pkg-present exits 0 for a package that is in packages.json"
pass "pkg-present exits 0 for an installed package"

omarchy-pkg-present bat ripgrep \
  || fail "pkg-present exits 0 when all named packages are in packages.json"
pass "pkg-present exits 0 when all packages are installed"

omarchy-pkg-present nonexistent-package-xyz && fail "pkg-present exits 1 for a missing package" || true
pass "pkg-present exits 1 for a package not in packages.json"

omarchy-pkg-present bat nonexistent-package-xyz && fail "pkg-present exits 1 when any package is missing" || true
pass "pkg-present exits 1 when one of the named packages is absent"

# ── omarchy-pkg-missing ────────────────────────────────────────────────────────

omarchy-pkg-missing nonexistent-package-xyz \
  || fail "pkg-missing exits 0 for a package not in packages.json"
pass "pkg-missing exits 0 for a package that is not installed"

omarchy-pkg-missing bat && fail "pkg-missing exits 1 when the package is installed" || true
pass "pkg-missing exits 1 for a package that is installed"

omarchy-pkg-missing bat nonexistent-package-xyz \
  || fail "pkg-missing exits 0 when any package is missing"
pass "pkg-missing exits 0 when at least one package is absent"

omarchy-pkg-missing bat ripgrep && fail "pkg-missing exits 1 when all packages are installed" || true
pass "pkg-missing exits 1 when all named packages are present"

# ── Missing manifest is treated as not-present, not a crash ───────────────────

no_manifest="$tmpdir/absent.json"
for cmd in omarchy-pkg-present omarchy-pkg-missing; do
  wrapper="$shim_dir/$cmd"
  sed \
    -e "s|/run/omarchy/packages.json|$no_manifest|g" \
    -e "1s|#!/bin/bash|#!$bash_bin|" \
    "$ROOT/pkgs/omarchy/cinque-shims/$cmd" > "$wrapper"
  chmod +x "$wrapper"
done

# pkg-present: absent manifest → not present → exit 1
omarchy-pkg-present bat && fail "pkg-present exits 1 when manifest is absent" || true
pass "pkg-present exits 1 when packages.json does not exist"

# pkg-missing: absent manifest → not present → exit 0 (the package IS missing)
omarchy-pkg-missing bat \
  || fail "pkg-missing exits 0 when manifest is absent"
pass "pkg-missing exits 0 when packages.json does not exist"
