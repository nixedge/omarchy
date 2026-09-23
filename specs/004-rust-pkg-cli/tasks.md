# Tasks: Rust pkg CLI

**Input**: `specs/004-rust-pkg-cli/plan.md`, `specs/004-rust-pkg-cli/spec.md`
**Branch**: `004-rust-pkg-cli` (off `003-pkg-ux`)

## Format

- **[P]**: Parallelisable (no dependency on an incomplete sibling in this phase)
- **[US#]**: User story
- File paths relative to repo root

---

## Phase 1: Cargo Workspace & Foundation

**Goal**: Compilable crate with socket client and protocol types. No logic yet — just the skeleton everything else builds on.

- [ ] T001 — Convert `daemon/Cargo.toml` to a workspace manifest with `members = [".", "omarchy-pkg"]`; create `daemon/omarchy-pkg/Cargo.toml` with deps: `clap` (features: derive), `clap_complete`, `serde`, `serde_json`, `toml`, `anyhow`
- [ ] T002 — Write `daemon/omarchy-pkg/src/main.rs`: clap `App` with subcommands `add`, `drop`, `list`, `search`, `sync`, `present`, `missing`, `resolve`; each subcommand is a stub that prints "not implemented" and exits 0
- [ ] T003 — Write `daemon/omarchy-pkg/src/socket.rs`: `DaemonClient::connect()`, `send_recv()`, `stream()`; parse newline-delimited JSON frames into `Frame` enum; return `anyhow::Error` on socket or parse failure
- [ ] T004 — Write `daemon/omarchy-pkg/src/proto.rs`: local protocol types — `Frame { Progress { line }, Done(Response) }`, `Response { ok, error, data, pending }`; derive `serde::Deserialize`
- [ ] T005 — Confirm `cargo build -p omarchy-pkg` produces a binary with zero errors and zero clippy warnings

**Checkpoint**: `cargo build -p omarchy-pkg` clean; binary prints "not implemented" for any subcommand

---

## Phase 2: Themed Output & Rebuild Filter

**Goal**: All visual output infrastructure ready before subcommand logic is written.

- [ ] T006 [P] — Write `daemon/omarchy-pkg/src/theme.rs`: read `~/.local/state/omarchy/current/theme/colors.toml` via `toml` crate; expose `Palette { accent, green, red, muted, fg, reset }` as 24-bit ANSI strings; fall back to empty strings on any error or when `stdout` is not a TTY (`libc::isatty` or `std::io::IsTerminal`)
- [ ] T007 [P] — Write `daemon/omarchy-pkg/src/filter.rs`: port `_pkg_filter_rebuild_line` from bash; `struct FilterState { building_shown: bool }`; `fn filter(line: &str, state: &mut FilterState, palette: &Palette) -> Option<String>` — returns `None` for suppressed lines, `Some(translated)` for semantic summaries, `Some(line.to_owned())` for pass-through
- [ ] T008 [P] — Write `daemon/omarchy-pkg/src/output.rs`: `fn print_status(glyph, colour, msg, palette)`, `fn print_error(glyph, msg, palette)`, `fn print_muted(msg, palette)`; glyphs as constants (`GLYPH_PKG`, `GLYPH_OK`, `GLYPH_FAIL`)
- [ ] T009 — Unit tests for `filter.rs`: suppressed lines return `None`; "building the system configuration" → `Some("󰏖 Resolving dependencies…")`; `building_shown` flips after first building line; plain lines pass through
- [ ] T010 — Unit tests for `theme.rs`: missing file → empty palette; non-TTY → empty palette; valid TOML → correct hex-to-ANSI conversion

**Checkpoint**: `cargo test -p omarchy-pkg` green for filter and theme tests

---

## Phase 3: Subcommand Implementations

**Goal**: All subcommands functional against the running daemon.

- [ ] T011 [US1] — Implement `cmds/resolve.rs`: `send_recv({"cmd":"pkg-resolve","name":name})`; print resolved attr or error; exit code reflects ok/err — used internally by add and drop
- [ ] T012 [US1] — Implement `cmds/add.rs`:
  1. Call resolve (exit on error)
  2. Print "Fetching…" status
  3. `nix profile install nixpkgs#<attr>` with output suppressed; on failure print error + search hint, exit 1
  4. Print "✓ <name> installed"
  5. If `--async`: `send_recv(pkg-add-async)`, print "System sync queued…", exit
  6. Default: `stream(pkg-add)` through filter, print progress, on success `nix profile remove`, print "✓ System sync complete"
- [ ] T013 [US2] — Implement `cmds/drop.rs`:
  1. Call resolve (exit on error)
  2. `nix profile remove nixpkgs#<attr>` suppressed (ignore errors)
  3. Print "✓ <name> removed"
  4. If `--async`: `send_recv(pkg-drop-async)`, print "System sync queued…", exit
  5. Default: `stream(pkg-remove)` through filter, print progress, print "✓ System sync complete"
- [ ] T014 [US3] [P] — Implement `cmds/list.rs`: read `/var/lib/omarchy/state.json`; look up versions from `/run/omarchy/packages.json`; print aligned two-column table sorted alphabetically; "No packages installed." on empty
- [ ] T015 [US4] [P] — Implement `cmds/search.rs`: run `nix search nixpkgs <term> --json`, parse output; display up to 20 results (name, version, description capped at 60 chars); mark installed packages (from state.json) with GLYPH_OK; "No packages found." on empty
- [ ] T016 [US5] [P] — Implement `cmds/sync.rs`: `send_recv({"cmd":"pkg-sync"})`; print "System rebuild queued…" on ok, error on fail
- [ ] T017 [US7] [P] — Implement `cmds/present.rs` and `cmds/missing.rs`: read `/run/omarchy/packages.json`; exit 0/1 based on presence; no output (these are used as guards in scripts)
- [ ] T018 — Manual smoke test on VM: `omarchy pkg add bat`, `omarchy pkg list`, `omarchy pkg drop bat`

**Checkpoint**: All subcommands functional on VM; existing bash equivalents produce identical output

---

## Phase 4: Completions

**Goal**: clap_complete generates completion files at build time; derivation installs them.

- [ ] T019 [P] — Write `daemon/omarchy-pkg/build.rs`: use `clap_complete::generate_to` to write bash, zsh, fish completions for the binary name `omarchy-pkg` into `$OUT_DIR/completions/`
- [ ] T020 [P] — Update `perSystem/packages.nix` `installPhase` to copy the generated completions from `$CARGO_TARGET_DIR` into `$out/share/bash-completion/completions/omarchy-pkg`, `$out/share/zsh/site-functions/_omarchy-pkg`, `$out/share/fish/vendor_completions.d/omarchy-pkg.fish`
- [ ] T021 — Delete `completions/bash/omarchy`, `completions/zsh/_omarchy`, `completions/fish/omarchy.fish` and the corresponding `installPhase` lines — the hand-written dynamic completions are superseded
- [ ] T022 — Verify: `nix build` → `result/share/bash-completion/completions/omarchy-pkg` exists; source it and confirm `omarchy-pkg add --help` completes flags

**Checkpoint**: `nix build` clean; completion files present in result/share/

---

## Phase 5: Shims & Cleanup

**Goal**: Router metadata preserved; old bash logic deleted.

- [ ] T023 [P] — Replace `bin/omarchy-pkg-add` with a 5-line bash shim: metadata comments + `exec omarchy-pkg add "$@"`
- [ ] T024 [P] — Replace `bin/omarchy-pkg-drop` with shim: metadata comments + `exec omarchy-pkg drop "$@"`
- [ ] T025 [P] — Replace `bin/omarchy-pkg-list` with shim: metadata + `exec omarchy-pkg list "$@"`
- [ ] T026 [P] — Replace `bin/omarchy-pkg-search` with shim: metadata + `exec omarchy-pkg search "$@"`
- [ ] T027 [P] — Replace `bin/omarchy-pkg-sync` with shim: metadata + `exec omarchy-pkg sync "$@"`
- [ ] T028 [P] — Replace `bin/omarchy-pkg-present` with shim: metadata + `exec omarchy-pkg present "$@"`
- [ ] T029 [P] — Add `bin/omarchy-pkg-missing` shim: metadata + `exec omarchy-pkg missing "$@"` (currently a shim in cinque-shims; move to bin/)
- [ ] T030 — Delete `bin/omarchy-pkg-colors` (sourced helper; logic now in theme.rs and filter.rs)
- [ ] T031 — Update `perSystem/packages.nix` cinque-shims list: remove `omarchy-pkg-drop`, `omarchy-pkg-present`, `omarchy-pkg-missing` from the shim list (they now come from bin/); add `omarchy-pkg` binary to installPhase
- [ ] T032 — Run `shellcheck` on all new shims

**Checkpoint**: `omarchy pkg add/drop/list/search/sync` all route through Rust binary; shims pass shellcheck

---

## Phase 6: Tests & Quality

**Goal**: Constitution compliance, all test suites green.

- [ ] T033 [P] — Unit tests for `cmds/add.rs`: mock socket client returns canned frames; verify error exit on resolve failure; verify profile-remove called on sync success
- [ ] T034 [P] — Unit tests for `cmds/list.rs`: state.json with 3 packages → 3 lines sorted; empty state → "No packages installed."
- [ ] T035 [P] — Unit tests for `socket.rs`: frame parser handles multi-line input, partial reads, malformed JSON
- [ ] T036 [P] — `cargo fmt --check` and `cargo clippy -- -D warnings` pass for `omarchy-pkg` crate
- [ ] T037 — Update `test/cli` assertions: `pkg add help shows direct route` checks for `[--async] <package>` from shim metadata; all 56 tests green
- [ ] T038 — Update `test/shell.d/pkg-list-test.sh` if needed to exercise the Rust binary path
- [ ] T039 — `./test/cli` green; `./test/shell` green

**Checkpoint**: All tests pass; cargo clean bill of health

---

## Dependencies

```
T001 → T002 → T003 → T004 → T005 (workspace + skeleton)
T005 → T006, T007, T008 (output infrastructure)
T006 + T007 + T008 → T011–T017 (subcommands need theme + filter)
T011 → T012, T013 (resolve used internally by add/drop)
T012–T017 → T018 (all subcommands before smoke test)
T019 + T020 → T021 → T022 (completions)
T012–T017 → T023–T031 (shims after implementations done)
T005–T031 → T033–T039 (tests last)
```

## Parallel Opportunities

- Phase 2 (T006, T007, T008, T009, T010) — all independent, write in parallel
- Phase 3 (T014, T015, T016, T017) — independent of each other, after T011
- Phase 5 (T023–T030) — all shims are independent of each other
- Phase 6 (T033, T034, T035, T036) — independent unit test files
