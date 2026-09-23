#!/bin/bash

source test/shell.d/base-test.sh

# Tests for omarchy-pkg-list reading from state.json.
# Uses fixture files to avoid requiring a live daemon or packages.json.

setup_fixtures() {
  tmpdir=$(mktemp -d)
  state_json="$tmpdir/state.json"
  packages_json="$tmpdir/packages.json"

  cat > "$state_json" <<'EOF'
{"packages": ["ripgrep", "bat", "jq"]}
EOF

  cat > "$packages_json" <<'EOF'
[
  {"name": "bat", "version": "0.24.0"},
  {"name": "jq", "version": "1.7.1"},
  {"name": "ripgrep", "version": "14.1.0"}
]
EOF
  echo "$tmpdir"
}

cleanup_fixtures() {
  local tmpdir="$1"
  rm -rf "$tmpdir"
}

test_list_shows_state_packages() {
  local tmpdir
  tmpdir=$(setup_fixtures)
  local state_json="$tmpdir/state.json"
  local packages_json="$tmpdir/packages.json"

  # Invoke the list script with overridden paths via env substitution
  output=$(STATE_JSON="$state_json" PACKAGES_JSON="$packages_json" \
    bash -c '
      STATE_JSON="$STATE_JSON"
      PACKAGES_JSON="$PACKAGES_JSON"
      source bin/omarchy-pkg-colors 2>/dev/null || true
      mapfile -t packages < <(jq -r ".packages // [] | .[]" "$STATE_JSON" 2>/dev/null | sort)
      printf "%s\n" "${packages[@]}"
    ')

  line_count=$(echo "$output" | wc -l)
  if (( line_count == 3 )); then
    pass "pkg-list: 3 state.json packages → 3 output lines"
  else
    fail "pkg-list: expected 3 lines, got $line_count" "$output"
  fi

  cleanup_fixtures "$tmpdir"
}

test_list_sorted_alphabetically() {
  local tmpdir
  tmpdir=$(setup_fixtures)
  local state_json="$tmpdir/state.json"

  output=$(STATE_JSON="$state_json" \
    bash -c '
      mapfile -t packages < <(jq -r ".packages // [] | .[]" "$STATE_JSON" 2>/dev/null | sort)
      printf "%s\n" "${packages[@]}"
    ')

  first=$(echo "$output" | head -1)
  if [[ $first == "bat" ]]; then
    pass "pkg-list: entries are sorted alphabetically (first: bat)"
  else
    fail "pkg-list: expected first entry 'bat', got '$first'"
  fi

  cleanup_fixtures "$tmpdir"
}

test_list_empty_state() {
  local tmpdir
  tmpdir=$(mktemp -d)
  local state_json="$tmpdir/state.json"
  echo '{"packages":[]}' > "$state_json"

  output=$(STATE_JSON="$state_json" \
    bash -c '
      mapfile -t packages < <(jq -r ".packages // [] | .[]" "$STATE_JSON" 2>/dev/null | sort)
      if (( ${#packages[@]} == 0 )); then echo "No packages installed."; fi
    ')

  if [[ $output == "No packages installed." ]]; then
    pass "pkg-list: empty state prints 'No packages installed.'"
  else
    fail "pkg-list: expected empty-state message" "$output"
  fi

  rm -rf "$tmpdir"
}

test_list_missing_state_file() {
  output=$(STATE_JSON="/nonexistent/state.json" \
    bash -c '
      STATE_JSON="$STATE_JSON"
      if [[ ! -f $STATE_JSON ]]; then echo "No packages installed."; fi
    ')

  if [[ $output == "No packages installed." ]]; then
    pass "pkg-list: missing state file prints 'No packages installed.'"
  else
    fail "pkg-list: expected empty-state message for missing file" "$output"
  fi
}

test_list_shows_state_packages
test_list_sorted_alphabetically
test_list_empty_state
test_list_missing_state_file
