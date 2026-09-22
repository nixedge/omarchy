# Feature Specification: Daemon Phase 2 — Live Package Management

**Feature Branch**: `002-daemon-pkg-mgmt`
**Created**: 2026-09-22
**Status**: Draft

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Install a package (P1)

A user runs `omarchy pkg add ripgrep` in a terminal. The command sends a request to the daemon, which adds the package to its state, triggers a `nixos-rebuild switch` in the background, and reports when the package is available. The user can immediately verify the package is present without rebooting.

**Why this priority**: This is the core operation the port is built around. Without it every other pkg command is academic and the daemon serves no purpose beyond the stub it already is.

**Independent Test**: Run `omarchy pkg add bat` on a live NixOS VM and confirm `bat --version` works after the rebuild completes.

**Acceptance Scenarios**:

1. **Given** a running Omarchy Cinque session, **When** `omarchy pkg add bat`, **Then** the command contacts the daemon, receives an acknowledgement, and exits 0.
2. **Given** the daemon has accepted a pkg-add request, **When** the background rebuild completes, **Then** `bat` is present on PATH in the same session without relogging.
3. **Given** a successful add, **When** `/run/omarchy/packages.json` is read, **Then** it contains an entry for `bat`.
4. **Given** an unknown package name, **When** `omarchy pkg add nonexistent-package-xyz`, **Then** the command exits non-zero with a message identifying the package as not found.

---

### User Story 2 — Remove a package (P1)

A user runs `omarchy pkg drop chromium`. The daemon removes it from state and triggers a rebuild. After the rebuild, `chromium` is gone.

**Why this priority**: Add and drop are symmetric; a system where you can add but not remove is unusable long-term.

**Independent Test**: Add a package, then drop it; confirm it vanishes from PATH and from `packages.json`.

**Acceptance Scenarios**:

1. **Given** a package in the current state, **When** `omarchy pkg drop <name>`, **Then** the daemon removes it from state.json and begins a rebuild.
2. **Given** the rebuild completes, **When** the previously installed package is invoked, **Then** the command is not found.
3. **Given** a package not in state, **When** `omarchy pkg drop <name>`, **Then** the command exits non-zero with a clear "not installed" message.

---

### User Story 3 — Query package presence (P2)

Scripts and users can check whether a package is installed without invoking Nix directly.

**Why this priority**: `omarchy-pkg-present` and `omarchy-pkg-missing` are used as guards inside many other `omarchy-*` commands; they must work correctly before higher-level commands can be ported.

**Independent Test**: After adding a package, `omarchy-pkg-present <name>` exits 0; after removing it, `omarchy-pkg-missing <name>` exits 0.

**Acceptance Scenarios**:

1. **Given** a package in `packages.json`, **When** `omarchy-pkg-present <name>`, **Then** exits 0.
2. **Given** a package absent from `packages.json`, **When** `omarchy-pkg-present <name>`, **Then** exits 1.
3. **Given** a package absent from `packages.json`, **When** `omarchy-pkg-missing <name>`, **Then** exits 0.
4. **Given** a package in `packages.json`, **When** `omarchy-pkg-missing <name>`, **Then** exits 1.

---

### User Story 4 — Arch→nixpkgs name aliasing (P2)

A user types `omarchy pkg add chromium` using the Arch package name. The daemon resolves it to the correct nixpkgs attribute without the user needing to know the difference.

**Why this priority**: Name aliasing is what makes the port transparent. Without it, users must learn a second namespace — a direct violation of Identity Preservation (Principle I).

**Independent Test**: Run `omarchy pkg add chromium` (Arch name) on a system where chromium is not installed; confirm it resolves and installs via nixpkgs.

**Acceptance Scenarios**:

1. **Given** a package whose Arch name differs from the nixpkgs attr, **When** `omarchy pkg add <arch-name>`, **Then** the daemon resolves and uses the correct nixpkgs attribute.
2. **Given** a name that exists identically in both Arch and nixpkgs, **When** `omarchy pkg add <name>`, **Then** it installs without error (no spurious alias lookup failure).
3. **Given** a name with no alias and no matching nixpkgs attribute, **When** `omarchy pkg add <name>`, **Then** the daemon returns a "not found" error and exits non-zero.

---

### Edge Cases

- What happens if `nixos-rebuild switch` fails? The daemon rolls back state.json to its pre-request snapshot and reports the build error to the caller.
- What happens if two concurrent `pkg add` requests arrive while a rebuild is running? The daemon queues them and applies them together in the next rebuild (no parallel rebuilds).
- What if the daemon is restarted mid-rebuild? On startup it detects a stale in-progress marker and either resumes or cleans up, leaving state.json consistent.
- What if `packages.json` is missing or corrupt when `omarchy-pkg-present` is called? Returns a safe "not present" answer rather than crashing.
- What if a nixpkgs attribute exists but the package fails to build (broken derivation)? The rebuild fails; the error surfaces to the user and state.json is not updated.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The daemon MUST persist the desired package set to `/var/lib/omarchy/state.json` before triggering any rebuild.
- **FR-002**: The daemon MUST generate a valid NixOS module from state.json that is imported by the system configuration, so `nixos-rebuild switch` picks it up without manual flake edits.
- **FR-003**: The daemon MUST trigger `nixos-rebuild switch` asynchronously after each state change and stream progress back to connected callers via the socket.
- **FR-004**: The daemon MUST update `/run/omarchy/packages.json` after every successful rebuild to reflect the new installed set.
- **FR-005**: The daemon MUST roll back state.json to its last known-good snapshot if `nixos-rebuild switch` fails.
- **FR-006**: The daemon MUST process pkg-add and pkg-remove requests serially; concurrent requests MUST be queued, not parallelised.
- **FR-007**: `omarchy-pkg-add` MUST accept an Arch package name and resolve it to a nixpkgs attribute via the alias table before writing state.
- **FR-008**: `omarchy-pkg-drop` MUST remove a package from state and trigger a rebuild; it MUST refuse and exit non-zero if the package is not currently in state.
- **FR-009**: `omarchy-pkg-present` and `omarchy-pkg-missing` MUST read `/run/omarchy/packages.json` directly (no socket IPC) and return within 100 ms.
- **FR-010**: The alias table MUST cover at minimum all packages in `install/omarchy-base.packages` that have different Arch vs nixpkgs attribute names.
- **FR-011**: `omarchy-pkg-install` MUST be an alias for `omarchy-pkg-add` (backward compatibility).

### Key Entities

- **state.json**: Persistent JSON at `/var/lib/omarchy/state.json`. Contains the canonical desired package set (list of nixpkgs attribute names) plus channel/generation metadata. Daemon is the sole writer.
- **omarchy-managed.nix**: Generated NixOS module at a path imported by the system flake. Derived deterministically from state.json on every write.
- **packages.json**: Runtime manifest at `/run/omarchy/packages.json`. Flat list of `{name, version}` objects reflecting the active NixOS generation. Updated after each successful rebuild.
- **alias table**: Embedded Arch→nixpkgs name map inside the daemon binary. Curated from the quattro base package list.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `omarchy pkg add <name>` acknowledgement is returned in under 2 seconds; the rebuilt package is available on PATH within 3 minutes on a warm cache.
- **SC-002**: `omarchy-pkg-present` and `omarchy-pkg-missing` return a correct result in under 100 ms on any post-boot system.
- **SC-003**: All packages in `install/omarchy-base.packages` that exist in nixpkgs can be installed by their Arch name via `omarchy pkg add` without error.
- **SC-004**: A failed rebuild leaves the previous generation active and leaves state.json unchanged; no manual intervention is required.
- **SC-005**: Two simultaneous `pkg add` requests result in both packages installed after exactly one rebuild cycle, not two.

## Assumptions

- The NixOS system flake imports the daemon-generated module at a stable, pre-agreed path; the daemon writes only to that path and never modifies the flake itself.
- `nixos-rebuild switch` is available and authorized for the daemon's service account (via a NixOS sudoers rule); no interactive password prompt occurs.
- The nixpkgs channel is pinned in the flake; the daemon does not update the channel as part of pkg-add (channel updates belong to a future `omarchy-update` spec).
- The alias table is curated manually and shipped with the daemon binary; no automatic resolution is attempted for unknown names.
- Packages that require `allowUnfree = true` are enabled globally by a separate module option, not per-package; this spec does not change that behavior.
