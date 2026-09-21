# Omarchy Cinque Constitution

Omarchy Cinque replaces Arch Linux + pacman with NixOS as the system foundation. Users see the
same commands, the same mutable `~/.config/`, the same plugins and themes. Nix is plumbing;
Omarchy is identity. ekapkgs may replace nixpkgs behind the daemon abstraction in a future
phase; no current implementation depends on it.

## Core Principles

### I. Identity Preservation

The user-visible surface — the `omarchy-*` CLI router, `~/.config/` ownership, plugin
marketplace, theme runtime switching, Quickshell desktop shell, menu system, and hook pipeline
— is NON-NEGOTIABLE. No user-facing command may be renamed or removed without a migration entry
in `migrations/`. The Quattro plugin and theme contract MUST be honored: a plugin or theme that
works on Quattro (Arch) MUST work on Cinque (NixOS) without modification. Any deviation MUST
be justified in the Complexity Tracking table of the relevant plan before implementation.

### II. Daemon-Mediated Configuration

A persistent Rust daemon (`omarchy-nix-daemon`) is the single writer of all NixOS
configuration managed by Omarchy. The `omarchy-*` shim scripts communicate with the daemon
via a Unix socket and MUST NOT write NixOS configuration files directly. The daemon owns a
deterministic state file (JSON) from which it regenerates the managed NixOS module on every
write; the same JSON input MUST always produce the same Nix output. This indirection is the
abstraction boundary that allows a different package backend (e.g., ekapkgs) to be swapped in
by changing the daemon's resolver, not any user-facing code.

### III. Native NixOS Implementation

Package management, system configuration, and update orchestration MUST be implemented using
nixpkgs primitives. Pacman, AUR helpers (paru, yay), makepkg, expac, and mise MUST NOT appear
in the post-Cinque runtime code path. Every deliverable MUST have a corresponding Nix package
or module attribute. The command-migration catalog in
`omarchy-nix-pkgs/plan/omarchy-command-migration.md` is the authoritative substitution table;
consult it when replacing or eliminating any `bin/` script.

### IV. Language Choices

All daemon and backend code MUST be written in Rust. All frontend and UI code (including any
Quickshell panels backed by a web view) MUST be written in Elm. Bash is permitted only for
thin `omarchy-*` shim scripts (IPC callers) and migration/install scripts. No new Python, Go,
JavaScript, or TypeScript may be introduced.

### V. Mutable Config Ownership

`~/.config/` MUST remain user-owned plain text files — never Home Manager symlinks. Theme
switching MUST remain runtime via Quickshell IPC; build-time-only theming is a regression.
User mutable state (`~/.local/state/omarchy`, `~/.config/omarchy`) MUST remain outside the
Nix store. Nix owns packages, services, and `$OMARCHY_PATH`; users own their configs.

### VI. Atomic Updates with Rollback

Every system update MUST be atomic: either the new NixOS generation activates or the previous
generation remains active with no manual intervention. The boot menu MUST always show at least
the two most recent generations. `omarchy rollback` MUST switch to the previous generation in
under 60 seconds. The Quattro→Cinque in-place upgrade MUST preserve all running services, user
configs, NetworkManager connections, and hardware-specific configuration. No service may have
more than 30 seconds of downtime during upgrade.

### VII. Backward Compatibility

The command-migration catalog defines Tier 1–7 substitutions. Tier 1 (core package primitives)
MUST be implemented before any Tier 2–7 work begins, because most higher-tier scripts route
through them. Commands marked "eliminated" in the catalog MUST NOT be reimplemented as thin
wrappers; they must be genuinely removed with migration entries where user-visible.

### VIII. Testing

Every new or modified `omarchy-*` command MUST have shell test coverage in `test/shell.d/`
before merging. Rust code MUST have unit tests per crate and integration tests against a real
NixOS test environment (no mocks for NixOS configuration). The Quattro→Cinque upgrade path
MUST be covered by a NixOS VM test. `./test/cli` and `./test/shell` MUST remain green on every
commit. Graphical acceptance tests run in a disposable VM, never in the development session.

### IX. Code Quality

All scripts MUST pass `shellcheck`. Nix code MUST pass `nix flake check`. Rust code MUST pass
`cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test --workspace`. Command
metadata headers (`# omarchy:summary=`, `# omarchy:group=`) MUST be present on every public
`bin/` command. Commits MUST be atomic — one coherent change per commit.

## Architecture Constraints

**Daemon IPC:** The Unix socket is at `/run/omarchy/daemon.sock`. The wire protocol is
newline-delimited JSON (request/response). The daemon MUST be started by systemd and MUST be
available before any `omarchy-*` script that delegates to it runs.

**Daemon state:** The daemon maintains a JSON state file at
`/var/lib/omarchy/state.json` (system packages, enabled services, channel, etc.). On each
write, it regenerates `/etc/nixos/omarchy-managed.nix` from the state and triggers
`nixos-rebuild switch` in a background job with progress reported via the socket.

**Manifest:** After each successful rebuild, the daemon materializes
`/run/omarchy/packages.json` from the active NixOS generation so `omarchy-pkg-present`,
`omarchy-capabilities check`, and Quickshell panels can query the installed set without
invoking Nix.

**Package namespace:** Use `nixpkgs` attribute names directly. The daemon maintains a curated
alias table (Arch name → nixpkgs attr) compiled from the command-migration catalog. Unknown
package names are passed to `nix search nixpkgs` and the daemon presents candidates.

**No ekapkgs dependency:** The initial implementation MUST NOT require ekapkgs. The daemon
abstraction boundary is the future integration point; ekapkgs is a swap-in, not a prerequisite.

## Development Workflow

- Conventional Commits format. One concern per commit. Every commit MUST build and pass tests.
- Feature specs live in `specs/NNN-feature-name/` with spec.md, plan.md, tasks.md.
- Plans MUST include a Constitution Check gate. Violations unjustified in the Complexity
  Tracking table are blockers.
- `omarchy-nix-pkgs/plan/` documents are design references. Consult them when touching
  package management, upgrade path, capability manifest, or distro operations.

## Governance

This constitution supersedes all prior guidelines for the `nix-port` branch. All specs, plans,
and implementations MUST demonstrate compliance. Amendment: open a PR against `nix-port`, bump
the version, get one review from a core maintainer.

**Version**: 1.1.0 | **Ratified**: 2026-09-21 | **Last Amended**: 2026-09-21
