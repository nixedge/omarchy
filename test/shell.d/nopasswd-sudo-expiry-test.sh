#!/bin/bash

set -euo pipefail
source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/base-test.sh"
source "$SHELL_TEST_DIR/fixtures/passwordless-sudo-test.sh"

(
  source "$library"
  for minutes in 1 15 1440 00015; do valid_minutes "$minutes" || exit 1; done
  for minutes in 0 1441 -1 1m '' 18446744073709551617; do ! valid_minutes "$minutes" || exit 1; done
  for name in audituser 'buildbot$'; do valid_account_name "$name" || exit 1; done
  # Upper-case words are sudoers alias references, so ALICE must never publish.
  for name in 'a$b' '$' 'a b' 'a#b' Alice ALICE ALL aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; do ! valid_account_name "$name" || exit 1; done
  ! valid_uid 18446744073709551617
)
pass "duration and account validation retains bounded inputs and trailing-dollar usernames"

(
  source "$library"
  assert_status 2 root_dispatch __status 1000
  assert_status 2 env TEST_EUID=0 SUDO_UID=1001 /usr/bin/bash -p "$test_tmp/omarchy-sudo-passwordless" __status 1000
  assert_status 3 env TEST_EUID=0 SUDO_UID=1000 /usr/bin/bash -p "$test_tmp/omarchy-sudo-passwordless" __status 1000
)
pass "internal actions reject missing root and mismatched sudo identity"

mock_bin="$test_tmp/bin"
grant="$test_tmp/grant"
calls="$test_tmp/calls"
mkdir -p "$mock_bin"

cat >"$mock_bin/gum" <<'SH'
#!/usr/bin/env bash
exit 0
SH

cat >"$mock_bin/systemctl" <<'SH'
#!/usr/bin/env bash

printf 'systemctl %s\n' "$*" >>"$TEST_CALLS"
[[ ${1:-} == "is-active" && ${TEST_TIMER_ACTIVE:-false} == "true" ]]
SH

cat >"$mock_bin/sudo" <<'SH'
#!/usr/bin/env bash

printf 'sudo %s\n' "$*" >>"$TEST_CALLS"

case ${1:-} in
test)
  [[ ${2:-} == "-f" && -f $TEST_GRANT ]]
  ;;
tee)
  tee "$TEST_GRANT"
  ;;
chmod)
  chmod "$2" "$TEST_GRANT"
  ;;
systemd-run)
  [[ ${TEST_FAIL_SYSTEMD_RUN:-false} != "true" ]]
  ;;
rm)
  rm -f -- "$TEST_GRANT"
  ;;
systemctl)
  exit 0
  ;;
*)
  echo "unexpected sudo command: $*" >&2
  exit 90
  ;;
esac
SH

chmod +x "$mock_bin/gum" "$mock_bin/sudo" "$mock_bin/systemctl"

run_command() {
  TEST_CALLS="$calls" TEST_GRANT="$grant" PATH="$mock_bin:$PATH" USER=alice \
    "$script" "$@"
}

: >"$calls"
enable_output=$(run_command 15)
[[ -f $grant ]] || fail "successful timer setup leaves the passwordless sudo grant enabled"
[[ $(cat "$grant") == "alice ALL=(ALL) NOPASSWD: ALL" ]] ||
  fail "the enabled grant belongs to the current user" "$(cat "$grant")"
grep -q '^sudo systemd-run --on-active=15m .* rm -f -- /etc/sudoers.d/99-omarchy-nopasswd-alice$' "$calls" ||
  fail "enabling arms the expiry timer" "$(cat "$calls")"
[[ $enable_output == *"automatically disable in 15 minutes"* ]] ||
  fail "success is reported after the timer is armed" "$enable_output"
pass "enabling arms expiry before reporting success"

: >"$calls"
rm -f "$grant"
if failure_output=$(TEST_FAIL_SYSTEMD_RUN=true run_command 15 2>&1); then
  fail "enabling fails when the expiry timer cannot be armed"
fi
[[ ! -e $grant ]] || fail "timer setup failure revokes the new passwordless sudo grant"
[[ $failure_output == *"Revoking access now"* ]] ||
  fail "timer setup failure explains the fail-closed revocation" "$failure_output"
[[ $failure_output != *"Passwordless sudo has been ENABLED"* ]] ||
  fail "timer setup failure does not report that passwordless sudo was enabled" "$failure_output"
pass "timer setup failure revokes a new grant"

: >"$calls"
printf 'alice ALL=(ALL) NOPASSWD: ALL\n' >"$grant"
if update_output=$(TEST_TIMER_ACTIVE=true TEST_FAIL_SYSTEMD_RUN=true run_command 30 2>&1); then
  fail "updating fails when the replacement expiry timer cannot be armed"
fi
[[ ! -e $grant ]] || fail "timer update failure revokes the existing passwordless sudo grant"
[[ $update_output != *"timer updated"* ]] ||
  fail "timer update failure does not report success" "$update_output"
pass "timer update failure revokes the existing grant"

mapfile -t tmpfiles_rules < <(grep -vE '^[[:space:]]*(#|$)' "$tmpfiles_file")
(( ${#tmpfiles_rules[@]} == 1 )) ||
  fail "passwordless sudo ships one tmpfiles rule" "${tmpfiles_rules[*]}"

fake_root="$test_tmp/root"
sudoers_dir="$fake_root/etc/sudoers.d"
mkdir -p "$sudoers_dir"
grant_names=(alice buildbot-2 user.123 'service$')
for grant_name in "${grant_names[@]}"; do
  touch "$sudoers_dir/99-omarchy-nopasswd-$grant_name"
done
pass "public status distinguishes inactive from errors and revokes authorization on exit"

printf ': >"$TEST_STARTUP_MARKER"\nset -o privileged\nunset BASH_ENV\n' >"$test_tmp/startup"
: >"$test_tmp/commands"
if TEST_STARTUP_MARKER="$test_tmp/startup-ran" BASH_ENV="$test_tmp/startup" bash "$test_tmp/omarchy-sudo-passwordless" -p >/dev/null 2>&1; then
  fail "ordinary Bash with a decoy -p was accepted"
fi
[[ -f $test_tmp/startup-ran && ! -s $test_tmp/commands ]] || fail "startup rejection must precede sudo"
pass "startup validation rejects ordinary Bash before authorization"

(
  source "$library"
  enable_locked 1000 15
  read_grant 1000
  [[ $GRANT_NAME == audituser && $(stat -c '%a' "$(rule_file 1000)") == 440 ]]
  /usr/sbin/visudo -cf "$(rule_file 1000)" >/dev/null
  expiry=$(sed -n 's/^systemd-run .*--on-calendar=@\([0-9]*\).*$/\1/p' "$test_tmp/commands" | tail -1)
  [[ $GRANT_DEADLINE == "$(/usr/bin/date -u -d "@$expiry" +%Y%m%d%H%M%SZ)" ]]
  if [[ -n ${OMARCHY_TEST_SUDOERS:-} ]]; then
    [[ -x $OMARCHY_TEST_SUDOERS ]] || fail "OMARCHY_TEST_SUDOERS must name an executable"
    printf 'root:x:0:0:root:/root:/bin/bash\naudituser:x:1000:1000:Test:/nonexistent:/bin/bash\n' >"$test_tmp/passwd"
    printf 'root:x:0:\naudituser:x:1000:\n' >"$test_tmp/group"
    { printf 'audituser ALL=(ALL) ALL\n'; cat "$(rule_file 1000)"; } >"$test_tmp/policy"
    for offset in -1 1; do
      when=$(/usr/bin/date -u -d "@$((expiry + offset))" +%Y%m%d%H%M%SZ)
      "$OMARCHY_TEST_SUDOERS" -p "$test_tmp/passwd" -P "$test_tmp/group" -T "$when" audituser /usr/bin/true <"$test_tmp/policy" >"$test_tmp/policy-result"
      if (( offset < 0 )); then
        ! grep -q 'Password required' "$test_tmp/policy-result" || fail "native policy requires a password before expiry"
      else
        grep -q 'Password required' "$test_tmp/policy-result" || fail "native policy remains passwordless after expiry"
      fi
    done
    pass "native sudoers evaluation requires authentication after the generated deadline"
  fi
  assert_status 0 status_locked 1000
  TEST_INACTIVE_TIMER=1 assert_status 0 status_locked 1000
  [[ ! -e $test_tmp/var/lib/omarchy/sudo-passwordless ]]
  ! compgen -G "$test_tmp/etc/sudoers.d/.omarchy-nopasswd.*"
)
pass "one complete mode-0440 sudoers rule holds the deadline with no separate grant state"

(
  source "$library"
  before=$(cat "$(rule_file 1000)")
  expire_locked 1000 omarchy-nopasswd-expire-1000-ffffffffffffffffffffffffffffffff
  [[ $(cat "$(rule_file 1000)") == "$before" ]]
  enable_locked 1000 30
  renewed=$(cat "$(rule_file 1000)")
  [[ $renewed != "$before" ]]
  expire_locked 1000
  [[ $(cat "$(rule_file 1000)") == "$renewed" ]]
  TEST_EXPIRED=1 expire_locked 1000
  [[ ! -e $(rule_file 1000) ]]
)
pass "legacy and current callbacks preserve renewed grants and remove expired ones"

(
  source "$library"
  enable_locked 1000 1
  assert_status 2 env TEST_EXPIRED=1 TEST_DELETE_FAIL=1 TEST_EUID=0 SUDO_UID=1000 /usr/bin/bash -p "$test_tmp/omarchy-sudo-passwordless" __status 1000
  [[ -e $(rule_file 1000) ]]
  TEST_EXPIRED=1 assert_status 3 status_locked 1000
  [[ ! -e $(rule_file 1000) ]]
)
pass "expired status reports cleanup failure separately from confirmed inactivity"

for failure in TEST_TIMER_FAIL TEST_INACTIVE_TIMER TEST_PUBLISH_FAIL TEST_POST_PUBLISH_FAIL TEST_CANCEL_ENABLE; do
  reset_grant
  expected=1
  if [[ $failure == "TEST_CANCEL_ENABLE" ]]; then expected=143; fi
  (
    source "$library"
    enable_locked 1000 15
    assert_status "$expected" env "$failure=1" TEST_EUID=0 SUDO_UID=1000 /usr/bin/bash -p "$test_tmp/omarchy-sudo-passwordless" __enable 1000 30
    [[ ! -e $(rule_file 1000) ]]
  )
done
pass "timer, publication, post-publication and cancellation failures revoke renewed access"

reset_grant
(
  source "$library"
  TEST_POST_PUBLISH_FAIL=1 TEST_DELETE_FAIL=1 assert_status 1 enable_locked 1000 15
  [[ -e $(rule_file 1000) ]]
  ! grep -q '^systemctl stop ' "$test_tmp/commands"
)
pass "failed policy deletion retains the timer and reports failure"

reset_grant
(
  source "$library"
  printf 'audituser ALL=(ALL) NOPASSWD: /usr/bin/true\n' >"$(rule_file 1000)"
  cp "$(rule_file 1000)" "$test_tmp/admin-rule"
  assert_status 2 status_locked 1000
  assert_status 1 enable_locked 1000 15
  assert_status 1 cleanup_uid_locked 1000
  cmp "$(rule_file 1000)" "$test_tmp/admin-rule"
  rm "$(rule_file 1000)"
  ln -s "$test_tmp/admin-rule" "$(rule_file 1000)"
  assert_status 2 status_locked 1000
  assert_status 1 cleanup_uid_locked 1000
  [[ -L $(rule_file 1000) ]]
)
pass "grant operations preserve administrator policies and reject symlinks"

reset_grant
(
  source "$library"
  TEST_BAD_PATH="$test_tmp/etc/sudoers.d" assert_status 1 enable_locked 1000 15
  [[ ! -e $(rule_file 1000) ]]
  rm "$PACKAGE_HOOK"
  assert_status 1 enable_locked 1000 15
)
pass "publication requires trusted paths and the packaged cleanup hook"

reset_grant
cp "$ROOT/default/libalpm/hooks/05-omarchy-passwordless-revoke.hook" "$test_tmp/hooks/"
(
  source "$library"
  TEST_ACCOUNT='buildbot$' enable_locked 1000 15
  read_grant 1000
  [[ $GRANT_NAME == 'buildbot$' ]]
  /usr/sbin/visudo -cf "$(rule_file 1000)" >/dev/null
  cleanup_uid_locked 1000
  [[ ! -e $(rule_file 1000) ]]
  TEST_ACCOUNT=ALICE assert_status 1 enable_locked 1000 15
  [[ ! -e $(rule_file 1000) ]]
)
pass "trailing-dollar accounts publish valid native policy and alias-shaped names never publish"
