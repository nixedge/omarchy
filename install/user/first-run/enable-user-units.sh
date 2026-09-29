#!/bin/bash

# Enable AND start the user systemd units we ship. Runs at first-run rather
# than at finalize-user time because the user manager isn't live during the
# ISO chroot — by first-run, the Hyprland/uwsm session is up and
# `systemctl --user enable --now` both writes the correct .wants symlinks
# (based on each unit's [Install]/WantedBy) and starts the services so the
# first session has bluetooth pairing, sleep lock, etc. live immediately
# instead of waiting for the next login. ConditionPath* in the unit files
# keep the enabled units inert on hardware they don't apply to.

set -euo pipefail

systemctl --user daemon-reload

# Enable each unit individually so an absent unit (e.g. on NixOS where units
# are declared in the module rather than dropped as loose files) does not abort
# the whole step and block first-run from completing.
for unit in \
  bt-agent.service \
  owed.service \
  omarchy-recover-internal-monitor.service \
  omarchy-sleep-lock.service \
  omarchy-migrate-notify.service \
  omarchy-fcitx5.service \
  omarchy-crash-watch.service; do
  systemctl --user enable --now "$unit" 2>/dev/null || true
done

# Only ship this hook if the OWE path is present (Arch-only package).
if [[ -f /usr/share/owe/10-owe-sync ]]; then
  omarchy-hook-install theme-set /usr/share/owe/10-owe-sync
fi
