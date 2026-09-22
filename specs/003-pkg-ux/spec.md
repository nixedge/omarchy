# Feature Specification: Package Management UX Polish

**Feature Branch**: `003-pkg-ux`
**Created**: 2026-09-22
**Status**: Draft

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Readable rebuild progress (P1)

When adding or removing a package, the terminal output is concise and readable — not a wall of raw `nixos-rebuild` build logs. The user sees a handful of status lines in the omarchy visual style, and the command returns cleanly when the rebuild is done.

**Why this priority**: Every pkg-add and pkg-drop triggers a nixos-rebuild that generates 50–300 lines of nix output (store path copies, derivation build steps, git cache unpacking). This noise makes the command feel like a raw system tool instead of an omarchy command. Filtering it down is the single highest-impact UX change for the port.

**Independent Test**: Run `omarchy pkg add hello` on a live VM with a cold nix cache. Count the output lines printed to the terminal. Should be ≤ 15.

**Acceptance Scenarios**:

1. **Given** a working omarchy session, **When** `omarchy pkg add bat`, **Then** the output fits in a 24-line terminal without scrolling, even for a cold build that fetches hundreds of paths.
2. **Given** a rebuild in progress, **When** individual nix store paths are being copied or built, **Then** those individual lines are not printed to the terminal in default mode.
3. **Given** a rebuild that fails, **When** the failure is a nix evaluation or build error, **Then** the relevant error message is printed and the command exits non-zero.
4. **Given** the `--verbose` flag, **When** `omarchy pkg add bat --verbose`, **Then** the full raw nixos-rebuild output streams to the terminal.
5. **Given** a successful rebuild, **When** the command completes, **Then** the final output line indicates success, e.g. `✓ bat installed`.
6. **Given** a failed rebuild, **When** the command exits non-zero, **Then** the final output line indicates failure, e.g. `✗ rebuild failed`.

---

### User Story 2 — User-installed package list (P1)

`omarchy pkg list` shows only the packages the user has explicitly installed, in a clean two-column format with versions.

**Why this priority**: Currently `omarchy pkg list` dumps the full system manifest (`packages.json`), which contains thousands of entries from the NixOS closure. A user asking "what have I installed?" gets an unusable flood. This is a confusing, broken-feeling experience that must be fixed before the port is ready for real use.

**Independent Test**: Install exactly 3 packages with `omarchy pkg add`; run `omarchy pkg list`. Output contains exactly 3 package data lines, not thousands.

**Acceptance Scenarios**:

1. **Given** 3 user-installed packages, **When** `omarchy pkg list`, **Then** exactly 3 package entries appear.
2. **Given** each entry, **When** the list is displayed, **Then** the package name and its version appear side-by-side in a consistent format.
3. **Given** no user-installed packages, **When** `omarchy pkg list`, **Then** a "No packages installed." message is shown and the command exits 0.
4. **Given** multiple packages installed, **When** the list is displayed, **Then** entries are sorted alphabetically by name.

---

### User Story 3 — Smart name resolution and helpful errors (P2)

When a user types a package name the way they know it — from Arch, Ubuntu, or common convention — the command either resolves it transparently or returns an actionable error that tells them what to do next.

**Why this priority**: The current alias table covers ~30 Arch-to-nixpkgs renames. But common names like `python`, `node`, `code` (VS Code), and `ruby` are absent or pass through unchanged and hit the nixos-rebuild evaluator, producing an opaque failure. Good name resolution is what distinguishes a polished tool from a prototype.

**Independent Test**: Run `omarchy pkg add python` and `omarchy pkg add node`. Both should succeed, installing `python3` and `nodejs` respectively, without the user knowing the nix attribute names.

**Acceptance Scenarios**:

1. **Given** an aliased Arch package name, **When** `omarchy pkg add <arch-name>`, **Then** the correct nixpkgs attribute is installed without user intervention.
2. **Given** a common non-nix name like `python`, **When** `omarchy pkg add python`, **Then** `python3` is installed.
3. **Given** a package name that resolves to nothing in nixpkgs, **When** `omarchy pkg add <unknown>`, **Then** the error message includes a suggestion: `Search with: nix search nixpkgs <name>`.
4. **Given** a service-managed package like `docker`, **When** `omarchy pkg add docker`, **Then** the error clearly names the controlling NixOS option (e.g. `virtualisation.docker.enable`) and exits non-zero without triggering a rebuild.
5. **Given** an Arch-only package like `yay`, **When** `omarchy pkg add yay`, **Then** the error explains it does not exist on NixOS and why, and exits non-zero.
6. **Given** the existing alias `nvim` → `neovim`, **When** `omarchy pkg add nvim`, **Then** neovim is installed (regression check: new aliases must not break existing ones).

---

### User Story 4 — Themed output (P2)

All pkg-* command output uses colors from the active omarchy theme and nerd font glyphs consistent with the omarchy visual language.

**Why this priority**: Omarchy's UI has a strong visual identity — themed colors, nerd font icons, consistent iconography in the bar, notifications, and shell. Pkg commands that print plain unstyled text feel foreign next to everything else. Color and glyph alignment is what makes the tool feel native to omarchy rather than bolted on.

**Independent Test**: Switch the active theme to `gruvbox` with `omarchy theme set gruvbox`, then run `omarchy pkg add hello`. The success line color should visually match gruvbox's green, not catppuccin blue.

**Acceptance Scenarios**:

1. **Given** any pkg-add or pkg-drop, **When** progress lines print, **Then** they are styled using the active theme's `accent` color.
2. **Given** a successful operation, **When** the final status line prints, **Then** it uses the theme's `green` color and a ✓ or nerd font checkmark glyph (U+F00C or equivalent).
3. **Given** a failed operation, **When** the final status line prints, **Then** it uses the theme's `red` color and an ✗ or nerd font error glyph (U+F00D or equivalent).
4. **Given** non-TTY output (piped or redirected), **When** any pkg command runs, **Then** ANSI escape codes are omitted and output is plain text.
5. **Given** `omarchy pkg list` with packages, **When** the list prints, **Then** package names use `foreground` color and versions use `muted` color.
6. **Given** `omarchy pkg list` empty state, **When** the "No packages installed." message prints, **Then** it uses `muted` color.
7. **Given** the theme is changed between two pkg operations, **When** the second operation runs, **Then** its output reflects the new theme's colors, not the previous one's.

---

### User Story 5 — Search for packages (P3)

A user can search for available packages by name or description without leaving the omarchy CLI and without knowing the nix command syntax.

**Why this priority**: Without a search command, users who don't know the exact nixpkgs attribute name must context-switch to a browser or run `nix search nixpkgs <term>` manually. This is the last step to making pkg management self-contained. It's P3 because add/drop/list/aliases are more pressing, and `nix search` already works as a fallback.

**Independent Test**: Run `omarchy pkg search json`. Output contains at least `jq` and `yq` with brief descriptions, formatted consistently with other pkg output.

**Acceptance Scenarios**:

1. **Given** a search term, **When** `omarchy pkg search <term>`, **Then** results from nixpkgs matching that term are displayed with package name, version, and a one-line description.
2. **Given** a term with many results, **When** `omarchy pkg search <term>`, **Then** results are paginated or capped at a reasonable limit (e.g. 20) to avoid flooding the terminal.
3. **Given** a term with no results, **When** `omarchy pkg search <term>`, **Then** a "No packages found." message is shown and the command exits 0.
4. **Given** a result that matches a user-installed package, **When** the list is shown, **Then** that entry is visually marked as already installed (e.g. a different glyph or color).
5. **Given** the search results, **When** displayed, **Then** package names and versions use the same themed color scheme as `omarchy pkg list` (`foreground` for names, `muted` for versions, `accent` for the installed marker).

---

## Edge Cases

- `--verbose` flag on a non-rebuild command (e.g., `omarchy pkg list --verbose`): flag is silently ignored; list output is unchanged.
- `colors.toml` missing or malformed: all color output falls back to no ANSI codes; the command succeeds functionally without color.
- Terminal does not support nerd fonts: glyphs render as boxes; this is acceptable — the spec does not require font detection or ASCII fallback substitution.
- Very long package name: column alignment adjusts; names are not truncated.
- A package in user state whose name does not appear in `packages.json` (e.g., before the manifest is refreshed after install): version column shows `—` or is omitted for that entry.
- `omarchy pkg add <name> --verbose` when the rebuild fails: the full raw output should still be visible so the user can diagnose the failure.

---

## Requirements

### Functional Requirements

**FR-001** — Default progress filtering: in default mode, the following nixos-rebuild output line types are suppressed: `unpacking '...'`, `copying path '/nix/store/...'`, `these N derivations will be built:`, `these N paths will be fetched`, individual derivation build lines, and nix evaluation trace lines.

**FR-002** — Summary progress lines: the following semantic events produce a single visible line each during a rebuild: dependency resolution start, fetch summary (N packages, N MB), build start, activation start, and completion.

**FR-003** — Verbose passthrough: `--verbose` on `omarchy pkg add` and `omarchy pkg drop` disables all filtering and streams raw nixos-rebuild output to the terminal.

**FR-004** — List source: `omarchy pkg list` reads user-installed packages from `/var/lib/omarchy/state.json`, not from `/run/omarchy/packages.json`.

**FR-005** — List version lookup: for each package name in user state, the list command looks up the version from `/run/omarchy/packages.json` by matching on the `name` field; if not found, the version is shown as `—`.

**FR-006** — List format: output is two-column — package name left-aligned, version right-aligned or tab-separated — sorted alphabetically by name.

**FR-007** — Empty list: when user state is empty, `omarchy pkg list` prints "No packages installed." and exits 0.

**FR-008** — Extended alias table: `alias.rs` covers at minimum these additional mappings beyond the current set:
- `python` → `python3`
- `python3` → `python3` (pass-through, explicit)
- `node` → `nodejs`
- `nodejs` → `nodejs` (pass-through)
- `code` → `vscode`
- `chromium-browser` → `chromium`
- `php` → `php` (pass-through)
- `ruby` → `ruby` (pass-through)
- `go` → `go` (pass-through)
- `rust` → eliminated with reason "use `rustup` or `cargo` via nixpkgs; see `omarchy pkg add rustup`"
- `java` → eliminated with reason "use a specific JDK: `jdk21`, `jdk17`, etc."
- `google-chrome` → `google-chrome` (pass-through; available via nixpkgs unfree)

**FR-009** — Not-found error message: when a nixos-rebuild fails because an attribute does not exist in nixpkgs, the printed error includes: `Search with: nix search nixpkgs <name>`.

**FR-010** — TTY detection: ANSI escape codes are only emitted when `stdout` is a TTY (`[ -t 1 ]` is true).

**FR-011** — Theme color reading: pkg scripts read colors from `~/.local/state/omarchy/current/theme/colors.toml` using the awk-based approach established in `omarchy-plymouth-set-by-theme`. A shared helper (`omarchy-pkg-color` or a sourced function) extracts this logic so all pkg scripts share one implementation.

**FR-013** — Search command: `omarchy pkg search <term>` invokes `nix search nixpkgs <term>` and displays results as a formatted list of name, version, and description.

**FR-014** — Search result limit: results are capped at 20 entries by default to prevent terminal flooding.

**FR-015** — Search installed marker: results that correspond to a package in `/var/lib/omarchy/state.json` are marked with a distinct glyph or color to indicate they are already installed.

**FR-016** — Search empty state: when no results are found, `omarchy pkg search <term>` prints "No packages found." and exits 0.

**FR-012** — Glyph usage: progress lines are prefixed with the nerd font package glyph (󰏖 U+F040E or nearest available); success uses ✓ or  (U+F00C nf-fa-check); failure uses ✗ or  (U+F00D nf-fa-times).

### Key Entities

**`/var/lib/omarchy/state.json`** — User-requested package list. Source of truth for `omarchy pkg list`. Format: `{"packages": ["bat", "ripgrep"]}`. The names are nixpkgs attribute names after alias resolution.

**`/run/omarchy/packages.json`** — Full system closure manifest. Used only as a version lookup by the list command, not as the primary data source.

**`~/.local/state/omarchy/current/theme/colors.toml`** — Active theme palette. Keys consumed by pkg commands: `accent`, `green`, `red`, `foreground`, `muted`.

---

## Success Criteria

**SC-001** — After `omarchy pkg add <package>` completes, the number of lines printed to the terminal in default mode is ≤ 15 for any package, regardless of how many nix store paths are fetched or built.

**SC-002** — `omarchy pkg list` after installing 3 packages returns exactly 3 data lines within 500 ms (reads local files only, no network or nix calls).

**SC-003** — After switching to any of the 22 bundled themes, the next pkg-add or pkg-drop output uses that theme's colors for progress, success, and error lines.

**SC-004** — `omarchy pkg add python` and `omarchy pkg add node` succeed and install `python3` and `nodejs` respectively without the user specifying the nix attribute name.

**SC-005** — `omarchy pkg add docker` exits non-zero within 1 second, prints a message containing `virtualisation.docker.enable`, and triggers no nixos-rebuild.

**SC-006** — All existing unit tests in `alias.rs` pass after the alias table is expanded.

**SC-007** — `omarchy pkg add <unknown-name>` exits non-zero and the output contains the string `nix search nixpkgs`.

**SC-008** — `omarchy pkg search json` returns results including at least `jq` within 10 seconds and displays no more than 20 entries.

**SC-009** — When a package returned by search is already installed, it is visually distinguishable from uninstalled results.

---

## Assumptions

- Spec 002 daemon infrastructure is working: state.json, daemon socket, and nixos-rebuild integration all function correctly.
- The active terminal is alacritty with JetBrains Mono Nerd Font as shipped by omarchy; nerd font glyphs U+F040E, U+F00C, U+F00D are available.
- ANSI 24-bit color (truecolor) is supported in the target terminal.
- Progress filtering is implemented in the shell shims (`omarchy-pkg-add`, `omarchy-pkg-drop`), not in the daemon. The daemon continues to stream all nixos-rebuild output as progress frames; the shim decides what to display.
- The `--verbose` flag is parsed by the shim. The daemon protocol does not change.
- Detecting whether a nixpkgs attribute is valid *before* triggering a rebuild (pre-flight validation) is out of scope. FR-009 covers the error path when nixos-rebuild evaluation fails.
- `packages.json` version lookup uses the `name` field (pname), which may differ from the nixpkgs attribute name; exact matching heuristics are an implementation detail.
- A shared color helper is an acceptable implementation approach. The spec does not prescribe whether it is a sourced bash function, a standalone script, or something else.
