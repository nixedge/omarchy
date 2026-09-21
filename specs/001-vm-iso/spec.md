# Feature Specification: Omarchy Cinque — Bootable VM ISO

**Feature Branch**: `001-vm-iso`
**Created**: 2026-09-21
**Status**: Draft

## User Scenarios

### User Story 1 — Developer boots the ISO in a VM (P1)

A developer on the nix-port branch wants to see Omarchy Cinque running. They run
`nix build .#iso`, write the result to a VM disk image, and boot it. The Omarchy desktop
appears: Hyprland starts, the Quickshell bar and panels render, and the `omarchy` CLI is
available in the terminal.

**Acceptance Scenarios:**

1. **Given** the nix-port repo, **When** `nix build .#iso`, **Then** it produces a bootable
   ISO at `result/iso/omarchy-cinque.iso` in under 20 minutes on a cold build.
2. **Given** the ISO booted in QEMU/virt-manager, **When** the autologin user's session
   starts, **Then** Hyprland + Quickshell launch without manual intervention.
3. **Given** a running session, **When** `omarchy help`, **Then** the command router lists
   all available command groups with summaries.
4. **Given** a running session, **When** `omarchy pkg list`, **Then** the installed package
   list is returned from `/run/omarchy/packages.json` without invoking nix.

### User Story 2 — The daemon is running and accepting connections (P1)

The `omarchy-nix-daemon` is a systemd service that starts at boot. Shim scripts can reach it.

**Acceptance Scenarios:**

1. **Given** the booted ISO, **When** `systemctl status omarchy-nix-daemon`, **Then** the
   service is active (running).
2. **Given** the running daemon, **When** `omarchy pkg add git`, **Then** the shim sends a
   JSON request to the daemon socket and the daemon acknowledges it (even if the rebuild is
   deferred/stubbed in phase 1).
3. **Given** the running daemon, **When** `/run/omarchy/packages.json` is read, **Then** it
   contains the packages installed in the active NixOS generation.

### User Story 3 — All existing omarchy-* commands are present (P2)

The full `bin/` tree from the omarchy source is available on the ISO. Commands that still
reference pacman print a clear "not available in Cinque" message rather than failing silently.

**Acceptance Scenarios:**

1. **Given** the booted ISO, **When** `which omarchy-pkg-add`, **Then** the command is found.
2. **Given** a command not yet ported, **When** it is run, **Then** it exits non-zero with a
   message identifying it as not yet implemented in Cinque, not a raw pacman error.

### Edge Cases

- ISO must boot with the default kernel; no custom kernel patches required for phase 1.
- QEMU `-enable-kvm` is optional; the ISO must also work without KVM (for CI).
- The daemon must start even if `/var/lib/omarchy/state.json` does not yet exist (first boot).

## Requirements

### Functional Requirements

- **FR-001**: `nix build .#iso` MUST produce a bootable ISO.
- **FR-002**: The ISO MUST boot into a graphical Hyprland + Quickshell session with autologin.
- **FR-003**: `omarchy-nix-daemon` MUST run as a systemd service and listen on
  `/run/omarchy/daemon.sock`.
- **FR-004**: The daemon MUST write `/run/omarchy/packages.json` on startup from the active
  NixOS generation's closure.
- **FR-005**: All `omarchy-*` commands from `bin/` MUST be in PATH.
- **FR-006**: Commands that invoke pacman/paru/yay/mise MUST be shimmed to exit with a clear
  "not available in Cinque" message.
- **FR-007**: `omarchy pkg add <name>` MUST send a request to the daemon socket and exit 0
  on daemon acknowledgment (actual rebuild deferred to a later phase).
- **FR-008**: `omarchy pkg list` MUST read from `/run/omarchy/packages.json`.
- **FR-009**: The flake MUST expose `packages.x86_64-linux.iso`, `packages.x86_64-linux.vm`
  (QEMU-runnable), and `nixosConfigurations.omarchy-cinque`.

### Key Entities

- **omarchy-nix-daemon**: Rust binary; systemd service; manages daemon.sock; writes
  packages.json; owns state.json.
- **omarchy-cinque NixOS module**: NixOS module that wires the daemon service, sets
  `OMARCHY_PATH`, installs the omarchy package, enables Hyprland, configures autologin.
- **omarchy package**: Nix derivation wrapping `bin/` from the omarchy source tree, with
  runtime dependencies from nixpkgs.

## Success Criteria

- **SC-001**: `nix build .#iso` completes without error on a clean checkout.
- **SC-002**: The ISO boots to a graphical Omarchy session in QEMU within 60 seconds.
- **SC-003**: `omarchy-nix-daemon` is active in `systemctl status` after boot.
- **SC-004**: `/run/omarchy/packages.json` is present and non-empty after boot.
- **SC-005**: `omarchy help` lists command groups without errors.
- **SC-006**: `omarchy pkg add git` sends a request to the daemon and exits 0.

## Assumptions

- Target architecture is x86_64-linux only for phase 1.
- Autologin user is `omarchy` (consistent with existing install scripts).
- Quickshell and the existing `shell/` QML are used as-is from the omarchy source.
- The daemon stub for phase 1 acknowledges pkg-add requests but does not trigger a rebuild.
  Full rebuild integration is phase 2.
