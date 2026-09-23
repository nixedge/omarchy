# Feature Specification: Rust pkg CLI

**Feature Branch**: `004-rust-pkg-cli`
**Created**: 2026-09-23
**Status**: Draft

## Overview

Replace the bash `omarchy-pkg-*` scripts with a single compiled Rust binary (`omarchy-pkg`) that handles all package management subcommands. Users see identical behaviour; developers get typed code, generated completions, and unit-testable logic.

## User Scenarios & Testing

### User Story 1 — Install a package (P1)

A user runs `omarchy pkg add neovim`. They see a "Fetching…" status line, then "✓ neovim installed" within seconds (nix profile fast path), followed by filtered rebuild progress ending with "✓ System sync complete". The package is usable immediately after the first line.

**Independent Test**: `omarchy pkg add bat` on a live VM — bat is on PATH before the rebuild finishes; sync completes without error.

**Acceptance Scenarios**:

1. **Given** a valid package name, **When** `omarchy pkg add <name>`, **Then** the binary exits 0 and the package is usable immediately.
2. **Given** `--async` flag, **When** `omarchy pkg add --async <name>`, **Then** the command exits 0 within 10 seconds with "System sync queued" message; rebuild continues in background.
3. **Given** an Arch alias like `node`, **When** `omarchy pkg add node`, **Then** `nodejs` is installed (alias resolved before nix profile call).
4. **Given** a service-managed name like `docker`, **When** `omarchy pkg add docker`, **Then** the command exits non-zero immediately with an actionable message ("managed by NixOS option `virtualisation.docker.enable`").
5. **Given** an unknown package, **When** `omarchy pkg add nonexistent-xyz`, **Then** exits non-zero with a "Search with: omarchy pkg search" hint.
6. **Given** `omarchy pkg add --help`, **Then** clap-generated help text is shown with args and examples.

---

### User Story 2 — Remove a package (P1)

A user runs `omarchy pkg drop neovim`. The package vanishes from the nix profile immediately; filtered rebuild progress follows.

**Independent Test**: Add then drop a package; confirm it leaves PATH after the rebuild.

**Acceptance Scenarios**:

1. **Given** an installed package, **When** `omarchy pkg drop <name>`, **Then** exits 0; package removed from profile immediately.
2. **Given** `--async` flag, **When** `omarchy pkg drop --async <name>`, **Then** exits 0 quickly; rebuild continues in background.
3. **Given** a package not installed, **When** `omarchy pkg drop <name>`, **Then** exits non-zero with "not installed" message.

---

### User Story 3 — List installed packages (P1)

A user runs `omarchy pkg list`. They see an aligned two-column table of package names and versions, sorted alphabetically, in theme colours.

**Independent Test**: After adding bat and ripgrep, `omarchy pkg list` shows exactly those two entries, sorted, with versions.

**Acceptance Scenarios**:

1. **Given** packages in state.json, **When** `omarchy pkg list`, **Then** one line per package, sorted.
2. **Given** no packages installed, **When** `omarchy pkg list`, **Then** prints "No packages installed." and exits 0.
3. **Given** a TTY-less context (piped output), **When** `omarchy pkg list | cat`, **Then** no ANSI escape codes appear.

---

### User Story 4 — Search packages (P1)

A user runs `omarchy pkg search json`. They see up to 20 results from nixpkgs with name, version, and description. Installed packages are visually marked.

**Independent Test**: `omarchy pkg search json` returns jq in results within 10 seconds.

**Acceptance Scenarios**:

1. **Given** a search term with results, **When** `omarchy pkg search <term>`, **Then** up to 20 results shown with name, version, description.
2. **Given** a package in state.json appears in results, **Then** it is marked with a check glyph.
3. **Given** no results, **When** `omarchy pkg search <term>`, **Then** "No packages found." and exit 0.

---

### User Story 5 — Retry system rebuild (P1)

A user runs `omarchy pkg sync` after a previous rebuild failure. The daemon re-runs `nixos-rebuild switch` for all packages in state.json.

**Acceptance Scenarios**:

1. **When** `omarchy pkg sync`, **Then** exits 0 with "System rebuild queued" within 2 seconds.
2. **Given** daemon unreachable, **Then** exits non-zero with a clear error.

---

### User Story 6 — Tab completion (P1)

After installing Omarchy, pressing `<Tab>` after `omarchy pkg ` in bash, zsh, or fish offers the subcommand names (`add`, `drop`, `list`, `search`, `sync`).

**Independent Test**: Source the bash completion and verify `complete -p omarchy-pkg` is registered; `COMP_WORDS` test shows correct candidates.

**Acceptance Scenarios**:

1. **Given** bash, **When** `omarchy pkg <Tab>`, **Then** subcommands are offered.
2. **Given** zsh with compinit, **When** `omarchy pkg <Tab>`, **Then** subcommands are offered with descriptions.
3. **Given** fish, **When** `omarchy pkg <Tab>`, **Then** subcommands are offered with descriptions.
4. **Given** the omarchy derivation is built, **Then** completion files exist at the standard paths in `$out/share/`.

---

### User Story 7 — Script-level presence checks (P2)

Other omarchy scripts call `omarchy-pkg-present <name>` and `omarchy-pkg-missing <name>` as guards. These continue to work with the Rust binary under the hood.

**Acceptance Scenarios**:

1. **Given** an installed package, **When** `omarchy pkg present <name>`, **Then** exits 0.
2. **Given** an absent package, **When** `omarchy pkg missing <name>`, **Then** exits 0.
3. Both commands exit non-zero for the opposite case.

---

## Functional Requirements

- FR-001: A single compiled binary `omarchy-pkg` handles all `omarchy pkg *` subcommands via clap.
- FR-002: Subcommands: `add`, `drop`, `list`, `search`, `sync`, `present`, `missing`, `resolve`.
- FR-003: `add` and `drop` accept `--async` flag; default mode streams filtered rebuild progress.
- FR-004: All socket I/O uses the existing daemon wire protocol (newline-delimited JSON). No socat dependency.
- FR-005: Alias resolution happens via the daemon's `pkg-resolve` command before any nix profile operation.
- FR-006: Coloured output reads `~/.local/state/omarchy/current/theme/colors.toml`; falls back to no colour when absent or when stdout is not a TTY.
- FR-007: Shell completions (bash, zsh, fish) are generated at build time via clap_complete and installed in `$out/share/`.
- FR-008: Thin bash shims (`omarchy-pkg-add`, `omarchy-pkg-drop`, etc.) carry the `# omarchy:*` metadata comments and exec into `omarchy-pkg <subcommand> "$@"`. They are the only bash remaining in this group.
- FR-009: The existing bash scripts (`omarchy-pkg-colors`, `omarchy-pkg-add`, `omarchy-pkg-drop`, `omarchy-pkg-list`, `omarchy-pkg-search`, `omarchy-pkg-sync`) are removed or replaced by shims once the Rust binary is in place.
- FR-010: `./test/cli` and `./test/shell` remain green.
- FR-011: The binary passes `cargo clippy -- -D warnings` and `cargo fmt --check`.
- FR-012: Unit tests cover subcommand dispatch, alias resolution client-side logic, output formatting, and TTY detection.

---

## Success Criteria

- SC-001: `omarchy pkg add <cached-pkg>` returns the "✓ installed" line within 10 seconds on a warm cache.
- SC-002: `omarchy pkg add --help` shows clap-generated help with subcommand descriptions within 100ms.
- SC-003: `omarchy pkg list` produces correctly formatted output for 0, 1, and 20+ packages.
- SC-004: Completion files are present in `result/share/bash-completion/`, `result/share/zsh/site-functions/`, and `result/share/fish/vendor_completions.d/` after `nix build`.
- SC-005: The binary has no runtime dependency on socat, jq, or awk.
- SC-006: All existing `test/cli` and `test/shell` tests pass unchanged.
- SC-007: `cargo test` passes for the `omarchy-pkg` crate.

---

## Assumptions

- The daemon socket at `/run/omarchy/daemon.sock` is available when the binary runs; connection failure is an error, not a fallback.
- The `pkg-resolve` daemon command (introduced in spec 003) handles alias resolution; the CLI does not duplicate the alias table.
- `nix` is on PATH (NixOS runtime invariant).
- The omarchy bash router's fallback behaviour (tries `omarchy-pkg-add`, falls back to `omarchy-pkg add`) handles dispatch without router changes.
- Colour theme file path: `~/.local/state/omarchy/current/theme/colors.toml`.

---

## Out of Scope

- Rewriting any other `omarchy-*` command groups (theme, system, etc.) in Rust — this spec covers pkg only.
- GUI or Quickshell integration changes.
- Changes to the daemon itself.
- Package group expansion (new subcommands beyond the current set).
