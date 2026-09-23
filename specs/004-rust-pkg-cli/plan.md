# Plan: Rust pkg CLI

**Spec**: `specs/004-rust-pkg-cli/spec.md`
**Branch**: `004-rust-pkg-cli` (off `003-pkg-ux`)

---

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Identity Preservation | ✅ | Same commands, same args, same output format |
| II. Daemon-Mediated Configuration | ✅ | CLI talks to daemon; no direct NixOS config writes |
| III. Native NixOS Implementation | ✅ | No pacman/AUR; nix profile + daemon |
| IV. Language Choices | ✅ | Rust CLI with clap; bash only for metadata shims |
| V. Mutable Config Ownership | ✅ | Reads colours from ~/.local/state; no write |
| VIII. Testing | ✅ | Unit tests + shell tests; no mocks for NixOS |
| IX. Code Quality | ✅ | clippy -D warnings, cargo fmt, shellcheck on shims |

---

## Architecture

### Crate Structure

The `omarchy-pkg` binary lives as a new workspace member inside `daemon/`:

```
daemon/
  Cargo.toml          # workspace [members = [".", "omarchy-pkg"]]
  src/                # existing daemon code
  omarchy-pkg/
    Cargo.toml
    src/
      main.rs         # clap App, subcommand dispatch
      socket.rs       # sync UnixStream client (no tokio needed)
      theme.rs        # read colors.toml, build ANSI palette
      output.rs       # print helpers (pkg list table, progress lines)
      filter.rs       # rebuild line filter (port of _pkg_filter_rebuild_line)
      cmds/
        add.rs
        drop.rs
        list.rs
        search.rs
        sync.rs
        present.rs
        missing.rs
        resolve.rs
    build.rs          # clap_complete: generate completions into $OUT_DIR
```

The CLI does **not** import daemon crate internals. Protocol types are redefined locally as simple structs — they're small enough that duplication is preferable to a shared crate at this stage.

### Dispatch Flow

```
omarchy pkg add neovim
  → bash router: tries omarchy-pkg-add → finds shim
  → shim: exec omarchy-pkg add neovim
  → omarchy-pkg: clap parses "add neovim"
  → cmds/add.rs:
      1. socket.send({"cmd":"pkg-resolve","name":"neovim"}) → attr="neovim"
      2. nix profile install nixpkgs#neovim  (suppressed, "Fetching…" shown)
      3. print "✓ neovim installed"
      4. socket.stream({"cmd":"pkg-add","name":"neovim"}) → filter+print progress
      5. nix profile remove nixpkgs#neovim  (cleanup on success)
      6. print "✓ System sync complete"
```

### Socket Client Design

A thin synchronous client — no tokio, just `std::os::unix::net::UnixStream`:

```rust
pub struct DaemonClient { stream: UnixStream }

impl DaemonClient {
    pub fn connect() -> Result<Self>
    pub fn send_recv(&mut self, req: &Value) -> Result<Value>       // single Done frame
    pub fn stream(&mut self, req: &Value, cb: impl Fn(Frame)) -> Result<Response>  // progress + Done
}
```

`Frame` is an enum: `Progress { line: String }` or `Done(Response)`. The client reads newline-delimited JSON, yielding frames until `Done`.

### Themed Output

`theme.rs` reads `~/.local/state/omarchy/current/theme/colors.toml` synchronously. On any error (file missing, parse failure, not a TTY), it returns a `Palette` with all colours as empty strings — so every `format!("{}{}{}", c.green, text, c.reset)` call degrades gracefully to plain text.

TOML parsing via the `toml` crate. ANSI 24-bit colour from `accent`, `green`, `red` fields in the theme file — same fields the bash `omarchy-pkg-colors` reads.

### Completion Generation

`build.rs` uses `clap_complete` to write completion files for bash, zsh, and fish to `$OUT_DIR/completions/`. The `installPhase` in `perSystem/packages.nix` copies them into `$out/share/`:

```bash
install -Dm644 $CARGO_TARGET_DIR/../omarchy-pkg/completions/omarchy-pkg.bash \
  $out/share/bash-completion/completions/omarchy-pkg
# etc.
```

These replace the hand-written dynamic completions in `completions/bash/omarchy` for the `pkg` subgroup.

### Bash Shims

Each existing bash script is replaced by a ≤5-line shim:

```bash
#!/bin/bash
# omarchy:summary=Add a package
# omarchy:args=[--async] <package>
# omarchy:examples=omarchy pkg add git
exec omarchy-pkg add "$@"
```

The shims serve one purpose: providing `# omarchy:*` metadata to the router. All logic is in Rust.

`omarchy-pkg-colors` (the sourced colour helper) is deleted entirely — its logic moves into `theme.rs`.

### Rebuild Progress Filter

Port `_pkg_filter_rebuild_line` from bash to Rust in `filter.rs`. Same suppression rules and semantic translations. The filter is a pure function `fn filter_line(line: &str, state: &mut FilterState) -> Option<String>` — straightforward to unit test.

---

## Complexity Tracking

| Risk | Mitigation |
|------|-----------|
| Router metadata gap | Bash shims carry metadata; router unchanged |
| Socket framing bugs | Existing daemon tests + CLI unit tests for frame parsing |
| Colour degradation in non-TTY | `isatty()` check before palette construction |
| nix profile attr path | Resolved via daemon `pkg-resolve` before any nix call |
| Cargo workspace restructure | Additive — existing daemon crate unchanged |

---

## Open Questions

None — all resolved by the spec and existing spec-003 work.

---

## Phase Summary

| Phase | Content | Blocking? |
|-------|---------|-----------|
| 1 | Cargo workspace + crate skeleton, socket client, protocol types | Yes |
| 2 | Themed output, rebuild filter | After Phase 1 |
| 3 | Subcommand implementations (add, drop, list, search, sync, present, missing, resolve) | After Phase 2 |
| 4 | Build.rs completions, installPhase wiring, delete dynamic completion scripts | After Phase 3 |
| 5 | Bash shims, delete old bash scripts, update test/cli assertions | After Phase 3 |
| 6 | Unit tests, cargo clippy/fmt, test/shell green | After all |
