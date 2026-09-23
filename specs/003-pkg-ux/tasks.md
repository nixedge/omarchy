# Tasks: Package Management UX Polish

**Input**: `specs/003-pkg-ux/plan.md`, `specs/003-pkg-ux/spec.md`
**Branch**: `003-pkg-ux` (off `002-daemon-pkg-mgmt`)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Parallelizable (different files, no blocking dependency on an incomplete task)
- **[US#]**: User story this task belongs to
- All file paths are relative to repo root

---

## Phase 1: Setup

**Purpose**: Create the branch and shared infrastructure that all user stories depend on.

- [ ] T001 Create branch `003-pkg-ux` off `002-daemon-pkg-mgmt` and verify worktree path
- [ ] T002 Add `omarchy pkg sync` to `GROUP_DESCRIPTIONS` in `bin/omarchy` so the command group is advertised

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Changes shared across multiple user stories. Must be complete before US-specific work begins.

- [ ] T003 [P] Write `bin/omarchy-pkg-colors` — sourced color helper that reads `~/.local/state/omarchy/current/theme/colors.toml` and sets `USE_COLOR`, `C_ACCENT`, `C_GREEN`, `C_RED`, `C_MUTED`, `C_FG`, `RESET`, `GLYPH_PKG`, `GLYPH_OK`, `GLYPH_FAIL` per plan.md Phase 4
- [ ] T004 [P] Extend `daemon/src/alias.rs` with new mappings: `python`→`python3`, `node`→`nodejs`, `code`→`vscode`, `chromium-browser`→`chromium`, `rust`→Eliminated, `java`→Eliminated; add unit tests for each new mapping and regression tests for existing aliases per plan.md Phase 1
- [ ] T005 [P] Extend `daemon/src/protocol.rs` with `PkgAddAsync`, `PkgDropAsync`, `PkgSync` request variants and `pending: Option<bool>` field on `Response` per plan.md Phase 2

**Checkpoint**: Foundation ready — all user story tasks can begin

---

## Phase 3: User Story 0 — Fast Package Install (P1) 🎯 MVP

**Goal**: `omarchy pkg add bat` returns within 10 seconds with bat immediately usable; daemon handles async rebuild and sends desktop notification on completion.

**Independent Test**: On VM with bat in binary cache — `time omarchy pkg add bat` must complete in ≤10s. Immediately after: `bat --version` succeeds. Within a few minutes a desktop notification appears.

- [ ] T006 [US0] Extend `daemon/src/handlers/pkg.rs` with `add_async` handler: resolve alias, update state.json optimistically, spawn background tokio task that acquires rebuild lock, runs `rebuild::run`, calls cleanup and notification helpers on completion per plan.md Phase 3
- [ ] T007 [US0] Implement `cleanup_profile(attr)` async helper in `daemon/src/handlers/pkg.rs`: runs `nix profile remove nixpkgs#<attr>` as the login user via `sudo -u <user>` per plan.md Phase 3
- [ ] T008 [US0] Implement `notify_success(name)` and `notify_failure(name)` async helpers in `daemon/src/handlers/pkg.rs`: call `omarchy-notification-send` as the login user per plan.md Phase 3
- [ ] T009 [US0] Implement `get_login_user()` in `daemon/src/handlers/pkg.rs`: reads `/run/omarchy/login-user`; falls back to `"omarchy"` per plan.md Phase 3
- [ ] T010 [US0] Wire `PkgAddAsync` and `PkgDropAsync` into `daemon/src/handlers/mod.rs` dispatch table
- [ ] T011 [P] [US0] Extend activation script in `flake/nixosModules/omarchy.nix` to write `/run/omarchy/login-user` with the primary user's login name per plan.md Phase 5
- [ ] T012 [US0] Rewrite `bin/omarchy-pkg-add`: fast path runs `nix profile install nixpkgs#<pkg>`, then sends `pkg-add-async` to daemon; verbose path sends `pkg-add` and blocks; source `omarchy-pkg-colors` for themed output; handle alias resolution error from Phase 1 with search hint per plan.md Phase 4
- [ ] T013 [US0] Rewrite `bin/omarchy-pkg-drop`: fast path runs `nix profile remove nixpkgs#<pkg>`, then sends `pkg-drop-async` to daemon; verbose path sends `pkg-remove` and blocks; source `omarchy-pkg-colors` per plan.md Phase 4
- [ ] T014 [P] [US0] Implement `add_async` failure path in daemon: on rebuild error, call `State::restore_backup()`, call `cleanup_profile`, call `notify_failure` — verify state.json is correctly reverted
- [ ] T015 [P] [US0] Extend `daemon/src/handlers/pkg.rs` with `sync` handler: reads state.json, triggers a fresh rebuild with all current packages, sends success/failure notification on completion

**Checkpoint**: US0 complete — fast install works end-to-end in VM

---

## Phase 4: User Story 1 — Readable Rebuild Notifications (P1)

**Goal**: `--verbose` mode streams filtered rebuild output (≤15 lines); notification text is clean and actionable.

**Independent Test**: `omarchy pkg add hello --verbose` on VM with cold cache. Count terminal output lines — must be ≤15.

- [ ] T016 [US1] Define `_filter_rebuild_line` in `bin/omarchy-pkg-colors` (sourced helper): suppresses `unpacking`, `copying path`, indented `/nix/store/` paths, derivation-list headers; translates `building the system configuration`, `activating the configuration`, first `building '/nix/store/...drv'` to themed summary lines per FR-001/FR-002
- [ ] T017 [US1] Update `bin/omarchy-pkg-add` verbose path to call `_filter_rebuild_line` for each progress frame instead of printing raw lines; verify ≤15 lines output on cold cache install
- [ ] T018 [US1] Update `bin/omarchy-pkg-drop` verbose path to call `_filter_rebuild_line` similarly
- [ ] T019 [P] [US1] Verify notification text in daemon helpers matches spec: success: `✓ <name> installed`, failure: `✗ <name>: system sync failed — run omarchy pkg sync to retry`

**Checkpoint**: US1 complete — verbose mode shows clean filtered output

---

## Phase 5: User Story 2 — User-Installed Package List (P1)

**Goal**: `omarchy pkg list` shows exactly the packages the user installed (from state.json), two columns with versions, sorted alphabetically.

**Independent Test**: After `omarchy pkg add bat && omarchy pkg add ripgrep && omarchy pkg add jq`: run `omarchy pkg list` — output contains exactly 3 data lines, sorted: bat, jq, ripgrep.

- [ ] T020 [US2] Rewrite `bin/omarchy-pkg-list`: read packages from `/var/lib/omarchy/state.json`; look up version from `/run/omarchy/packages.json` by matching on `name` field; display two-column aligned output sorted alphabetically; print "No packages installed." when state is empty; source `omarchy-pkg-colors` for themed output per plan.md Phase 4 and FR-004 through FR-007
- [ ] T021 [P] [US2] Add shell test in `test/shell.d/pkg-list-test.sh`: mock state.json with 3 packages, mock packages.json with matching versions, assert exactly 3 output lines and correct sort order

**Checkpoint**: US2 complete — `omarchy pkg list` shows correct user-installed packages

---

## Phase 6: User Story 3 — Smart Name Resolution (P2)

**Goal**: `python`, `node`, `code` resolve transparently; `docker`, `yay` give actionable errors; unknown names suggest `nix search nixpkgs <name>`.

**Independent Test**: `omarchy pkg add python` and `omarchy pkg add node` succeed, installing python3 and nodejs. `omarchy pkg add docker` exits non-zero within 1s without triggering rebuild.

- [ ] T022 [US3] Verify that `bin/omarchy-pkg-add` fast path surfaces alias resolution errors correctly: ServiceManaged packages exit immediately with the option name (no Phase 1 `nix profile` call); Eliminated packages exit immediately with the elimination reason — these must be caught before calling `nix profile install`
- [ ] T023 [P] [US3] Add alias resolution pre-check to `bin/omarchy-pkg-add` fast path: query daemon with `pkg-resolve <name>` (or resolve client-side by duplicating the alias table as a bash case statement) before running `nix profile install`; ensure docker, rust, java give correct errors within 1 second with no rebuild

Note: The alias table is in Rust and not directly callable from bash. Two options: (a) add a `PkgResolve { name }` daemon command that returns the resolved attr or error, or (b) duplicate the service-managed/eliminated entries as a bash case statement in `omarchy-pkg-colors` for fast pre-check. Option (b) is simpler; option (a) is DRY. Choose (a) if daemon changes are acceptable; choose (b) if shim-only is preferred.

- [ ] T024 [US3] Implement whichever option is chosen from T023 and update both `omarchy-pkg-add` and `omarchy-pkg-drop` to use it
- [ ] T025 [P] [US3] Verify `omarchy pkg add <unknown>` output contains the string `nix search nixpkgs` (FR-009 / SC-007) — this comes from the Phase 1 failure path in `bin/omarchy-pkg-add`

**Checkpoint**: US3 complete — alias table expanded, error messages actionable

---

## Phase 7: User Story 4 — Themed Output (P2)

**Goal**: All pkg output uses colors from the active theme; nerd font glyphs match the omarchy visual language; non-TTY output is plain text.

**Independent Test**: Switch to gruvbox (`omarchy theme set gruvbox`), run `omarchy pkg add hello`. The success line color visually matches gruvbox green, not the previous theme.

- [ ] T026 [P] [US4] Verify `bin/omarchy-pkg-colors` TTY detection works correctly: run `omarchy-pkg-add bat 2>&1 | cat` and confirm no ANSI escape codes appear in the output (FR-010)
- [ ] T027 [P] [US4] Switch theme to gruvbox in VM, run `omarchy pkg add hello`, confirm success line uses gruvbox green (manual visual test per `agents/skills/visual-verification.md`)
- [ ] T028 [P] [US4] Confirm `omarchy pkg list` uses `C_FG` for package names and `C_MUTED` for versions (FR-005 / US4 scenario 5)
- [ ] T029 [P] [US4] Confirm `omarchy pkg list` empty-state message uses `C_MUTED` (US4 scenario 6)

**Checkpoint**: US4 complete — all pkg output is themed

---

## Phase 8: User Story 5 — Package Search (P3)

**Goal**: `omarchy pkg search <term>` shows name, version, description capped at 20 results; installed packages are marked.

**Independent Test**: `omarchy pkg search json` returns results including at least `jq` within 10 seconds; no more than 20 entries appear; if jq is installed it is visually marked.

- [ ] T030 [US5] Create `bin/omarchy-pkg-search`: invoke `nix search nixpkgs <term> --json`, parse JSON output via jq, display name/version/description rows capped at 20, mark installed packages from state.json with `GLYPH_OK` in `C_GREEN`, print "No packages found." on empty results, source `omarchy-pkg-colors` per plan.md Phase 4 and FR-013 through FR-016
- [ ] T031 [P] [US5] Add `omarchy:summary`, `omarchy:args`, and `omarchy:examples` metadata headers to `bin/omarchy-pkg-search` so it appears in `omarchy pkg help`

**Checkpoint**: US5 complete — search works with themed output and installed markers

---

## Phase 9: Polish & Cross-Cutting Concerns

- [ ] T032 [P] Cargo build: run `cargo build` in `daemon/` and confirm zero errors after all Rust changes (T004, T005, T006, T007, T008, T009, T010)
- [ ] T033 [P] Run `./test/cli` and confirm all routing, metadata, and hidden-flag tests pass; fix any regressions from new/changed commands
- [ ] T034 [P] Run `./test/shell` and confirm all shell tests pass including new T021 pkg-list tests
- [ ] T035 Run end-to-end VM smoke test: `omarchy pkg add bat`, confirm prompt returns in ≤10s, bat works immediately, notification arrives within 3 minutes, `omarchy pkg list` shows bat, `omarchy pkg drop bat` succeeds

---

## Dependencies

```
T001, T002 (setup) → must complete before all else
T003, T004, T005 (foundational) → must complete before US story tasks
T003 (colors helper) → T006, T007, T008, T009, T012, T013, T016–T019, T020, T026–T029, T030
T004 (alias.rs) → T022, T023, T024
T005 (protocol.rs) → T006, T010
T006 (add_async handler) → T007, T008, T009, T010, T014
T011 (activation script) → T009 (login-user file)
T012 (pkg-add rewrite) → T016, T017
T013 (pkg-drop rewrite) → T018
T015 (sync handler) → T035
T020 (pkg-list rewrite) → T021, T028, T029
T030 (pkg-search) → T031
T032, T033, T034, T035 (validation) → all prior tasks
```

## Parallel Opportunities

**Within US0**: T006+T007+T008+T009 can be written in parallel (different functions). T011 and T012 and T013 can be written in parallel.

**Across stories**: US2 (T020–T021), US4 (T026–T029), and US5 (T030–T031) are all independent of US0 Phase 2 daemon work — they only need T003 (colors helper) to be done first.

## MVP Scope

US0 + US1 + US2 = the minimum that makes the NixOS pkg experience feel correct:
- packages install fast (US0)
- verbose mode shows clean output (US1)
- `omarchy pkg list` shows the right packages (US2)

US3, US4, US5 add polish and can follow in a subsequent commit.
