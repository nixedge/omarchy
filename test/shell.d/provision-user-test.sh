#!/bin/bash

set -euo pipefail

source "$(dirname "$0")/base-test.sh"

test_tmp=$(mktemp -d)
trap 'rm -rf "$test_tmp"' EXIT

mock_bin="$test_tmp/bin"
mkdir -p "$mock_bin" "$test_tmp/home" "$test_tmp/home/.hermes/profiles/james"

for command in xdg-user-dirs-update xdg-settings xdg-mime; do
  printf '#!/bin/sh\nexit 0\n' >"$mock_bin/$command"
done

# omarchy-done is a Rust CLI symlink not present in the source tree; stub it so
# the test can run without a built binary.  Uses /bin/sh for NixOS compatibility.
cat >"$mock_bin/omarchy-done" <<'STUB'
#!/bin/sh
action=$1 name=$2
done_dir="${HOME}/.local/state/omarchy/done"
marker="${done_dir}/${name}"
case "$action" in
  check)  [ -f "$marker" ] ;;
  mark)   mkdir -p "$done_dir" && touch "$marker" ;;
  ensure) [ -f "$marker" ] || { mkdir -p "$done_dir"; touch "$marker"; } ;;
esac
STUB

chmod +x "$mock_bin"/*

# Provisioning prepends $OMARCHY_PATH/bin, which shadows $mock_bin for anything
# Omarchy ships. Stub out commands at the OMARCHY_PATH level so they take
# precedence over both the source bin/ and the live system.
#
# A minimal OMARCHY_PATH tree: bin/ with stubs, plus symlinks to the real
# default/agents/skills and migrations directories so skill linking and
# migration marking use the actual source files.
omarchy_path="$test_tmp/omarchy-path"
mkdir -p "$omarchy_path/bin"
ln -s "$ROOT/default" "$omarchy_path/default"
ln -s "$ROOT/migrations" "$omarchy_path/migrations"

for cmd in omarchy-refresh-applications xdg-settings xdg-mime; do
  printf '#!/bin/sh\nexit 0\n' >"$omarchy_path/bin/$cmd"
  chmod +x "$omarchy_path/bin/$cmd"
done

# The install suite is stubbed so provision-user doesn't retheme the live session.
mkdir -p "$test_tmp/install/user"
: >"$test_tmp/install/user/all.sh"

HOME="$test_tmp/home" PATH="$mock_bin:$ROOT/bin:$PATH" OMARCHY_PATH="$omarchy_path" \
  OMARCHY_INSTALL="$test_tmp/install" bash "$ROOT/bin/omarchy-provision-user" >/dev/null ||
  fail "omarchy-provision-user finishes"

for skill in omarchy diagnose-crash; do
  link="$test_tmp/home/.gemini/config/skills/$skill"
  [[ -L $link && $(readlink "$link") == "$omarchy_path/default/agents/skills/$skill" ]] ||
    fail "omarchy-provision-user provisions the $skill skill for Antigravity"

  link="$test_tmp/home/.hermes/skills/$skill"
  [[ -L $link && $(readlink "$link") == "$omarchy_path/default/agents/skills/$skill" ]] ||
    fail "omarchy-provision-user provisions the $skill skill for Hermes"

  link="$test_tmp/home/.hermes/profiles/james/skills/$skill"
  [[ -L $link && $(readlink "$link") == "$omarchy_path/default/agents/skills/$skill" ]] ||
    fail "omarchy-provision-user provisions the $skill skill for a Hermes profile"
done

pass "omarchy-provision-user provisions Antigravity and Hermes skills"
