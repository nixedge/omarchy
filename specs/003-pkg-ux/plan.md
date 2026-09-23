# Implementation Plan: Package Management UX Polish

**Branch**: `003-pkg-ux` | **Date**: 2026-09-22 | **Spec**: `spec.md`

## Summary

Polish the spec 002 pkg pipeline into a first-class UX. The headline change is a two-phase install model: Phase 1 (fast, sync) uses `nix profile install` to make packages immediately usable; Phase 2 (slow, async) runs `nixos-rebuild switch` in the background and notifies the user on completion. Additional polish: `pkg list` reads from state.json, themed terminal output, extended alias table, and a `pkg search` command.

## Technical Context

**Language/Version**: Rust 1.80+ (daemon), Bash (shims)
**Primary Dependencies**: tokio (already present), serde_json (already present), `nix` binary at `/run/current-system/sw/bin/nix` (already present)
**Storage**: `/var/lib/omarchy/state.json` (source of truth for user packages), `/run/omarchy/packages.json` (version lookup), `~/.nix-profile` (Phase 1 fast path), `~/.local/state/omarchy/current/theme/colors.toml` (themed output)
**Testing**: Existing shell test suite (`test/shell.d/`), manual VM testing
**Target Platform**: x86_64-linux (NixOS Cinque, omarchy-nix-daemon branch)
**Constraints**: `nix profile` requires experimental `nix-command` feature (already enabled in VM nix.conf). Desktop notifications from the daemon (system service) require delivering to the user's D-Bus session — use `sudo -u <user> omarchy-notification-send` or env-var passthrough.

## Constitution Check

- **I. Identity Preservation** ✓ — User-visible commands unchanged; Arch names accepted via alias table
- **II. Daemon-Mediated Configuration** ✓ — state.json and nixos-rebuild still owned by daemon; Phase 1 profile write is user-space, not system config
- **III. Native NixOS Implementation** ✓ — uses `nix profile` and `nixos-rebuild switch`; no pacman
- **IV. Language Choices** ✓ — Rust daemon extensions; Bash shims only; new shell helper for colors
- **VIII. Code Quality** ✓ — alias unit tests extended; shell tests for pkg-list and color helper

## Architecture: Two-Phase Install

```
User types: omarchy pkg add bat
                │
                ▼
         ┌──────────────────────────────────┐
         │  PHASE 1 (shim, sync, ~2–10s)   │
         │  nix profile install nixpkgs#bat  │
         │  → bat available NOW in PATH     │
         └──────────────┬───────────────────┘
                        │ success
                        ▼
         ┌──────────────────────────────────┐
         │  daemon: pkg-add-async {bat}     │
         │  - add bat to state.json         │
         │  - spawn background rebuild task │
         │  - return Done frame immediately │
         └──────────────┬───────────────────┘
                        │ Done received
                        ▼
         "✓ bat installed (system sync in progress)"
         shim exits

         ════════ background, in daemon ════════
                        │
                        ▼
         ┌──────────────────────────────────────┐
         │  PHASE 2 (daemon, async, 1–3 min)   │
         │  acquire rebuild lock                │
         │  nixos-rebuild switch                │
         │                                      │
         │  on success:                         │
         │    nix profile remove nixpkgs#bat    │
         │    notify: "✓ bat installed"         │
         │                                      │
         │  on failure:                         │
         │    revert state.json                 │
         │    nix profile remove nixpkgs#bat    │
         │    notify: "✗ bat: system sync failed│
         │             run omarchy pkg sync"    │
         └──────────────────────────────────────┘
```

**Verbose mode** skips Phase 1 entirely and uses the original blocking daemon flow from spec 002, with filtered output. This gives users a way to watch the rebuild live.

## Repository Layout

```
daemon/src/
  alias.rs            EXTEND — add python→python3, node→nodejs, code→vscode, etc.
  handlers/pkg.rs     EXTEND — add pkg-add-async and pkg-drop-async handlers
  protocol.rs         EXTEND — add PkgAddAsync, PkgDropAsync, PkgSync request types

bin/
  omarchy-pkg-add     REWRITE — Phase 1 fast path + daemon async signal; --verbose falls back to sync
  omarchy-pkg-drop    REWRITE — Phase 1 profile remove + daemon async signal; was pacman
  omarchy-pkg-list    REWRITE — reads state.json, looks up versions from packages.json
  omarchy-pkg-colors  NEW     — sourced helper: reads colors.toml, sets C_ACCENT/GREEN/RED/MUTED/FG vars
  omarchy-pkg-search  NEW     — wraps `nix search nixpkgs`, formats results, marks installed

specs/003-pkg-ux/
  plan.md             this file
  tasks.md            task list
```

## Phase 1: Extended Alias Table (`daemon/src/alias.rs`)

Add to the existing match arms:

```rust
// Common ecosystem names → nixpkgs attributes
"python"            => Ok("python3"),
"node"              => Ok("nodejs"),
"code"              => Ok("vscode"),
"chromium-browser"  => Ok("chromium"),

// Eliminated with helpful reasons
"rust" => Err(ResolveError::Eliminated(
    "use `rustup` or install `rustup` via `omarchy pkg add rustup`"
)),
"java" => Err(ResolveError::Eliminated(
    "use a specific JDK: jdk21, jdk17, etc. (e.g. omarchy pkg add jdk21)"
)),
```

Pass-through names (`python3`, `nodejs`, `php`, `ruby`, `go`, `google-chrome`) already
fall through to `Ok(name)` in the catch-all; no explicit arms needed.

New unit tests:
```rust
assert_eq!(resolve("python").unwrap(), "python3");
assert_eq!(resolve("node").unwrap(), "nodejs");
assert_eq!(resolve("code").unwrap(), "vscode");
assert!(matches!(resolve("rust"), Err(ResolveError::Eliminated(_))));
assert!(matches!(resolve("java"), Err(ResolveError::Eliminated(_))));
// regression: existing aliases still work
assert_eq!(resolve("nvim").unwrap(), "neovim");
```

## Phase 2: Daemon Protocol Extension (`daemon/src/protocol.rs`)

Add two new request types:

```rust
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "kebab-case")]
pub enum Request {
    // ... existing variants ...
    PkgAddAsync { name: String },   // fast path: update state + background rebuild
    PkgDropAsync { name: String },  // fast path: update state + background rebuild
    PkgSync,                        // retry background rebuild for all state.json packages
}
```

The `Done` frame carries an extra `pending: bool` field to tell the shim that a background
rebuild is running:

```rust
#[derive(Serialize)]
pub struct Response {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending: Option<bool>,   // NEW: true when background rebuild is queued
    // ... existing fields ...
}
```

## Phase 3: Daemon Handler Extension (`daemon/src/handlers/pkg.rs`)

### `pkg-add-async` handler

```rust
pub async fn add_async(
    name: &str,
    rebuild_lock: Arc<Mutex<()>>,
) -> Response {
    // 1. Resolve alias
    let nix_attr = match alias::resolve(name) { ... };

    // 2. Update state.json optimistically
    let mut state = State::load().await?;
    if state.packages.contains(&nix_attr.to_owned()) {
        return Response::err(format!("'{name}' is already installed"));
    }
    State::save_backup().await?;
    state.packages.push(nix_attr.to_owned());
    state.save().await?;

    // 3. Spawn background rebuild — does not block this handler
    let attr_owned = nix_attr.to_owned();
    let name_owned = name.to_owned();
    tokio::spawn(async move {
        let _guard = rebuild_lock.lock().await;
        let state = State::load().await.unwrap_or_default();
        match rebuild::run(&state, mpsc::unbounded_channel().0).await {
            Ok(_) => {
                cleanup_profile(&attr_owned).await;
                notify_success(&name_owned).await;
            }
            Err(e) => {
                tracing::error!("background rebuild failed for {name_owned}: {e}");
                let _ = State::restore_backup().await;
                cleanup_profile(&attr_owned).await;
                notify_failure(&name_owned).await;
            }
        }
    });

    // 4. Return immediately with pending=true
    Response { ok: true, pending: Some(true), data: None, error: None }
}
```

### Notification helpers

```rust
async fn notify_success(name: &str) {
    run_as_user(&["omarchy-notification-send",
        &format!("✓ {name} installed"), "-g", ""]).await;
}

async fn notify_failure(name: &str) {
    run_as_user(&["omarchy-notification-send",
        &format!("✗ {name}: system sync failed — run omarchy pkg sync to retry"),
        "-u", "critical"]).await;
}

async fn cleanup_profile(attr: &str) {
    // Remove from user's nix profile after rebuild bakes it into the system
    let _ = Command::new("sudo")
        .args(["-u", &get_login_user(), "nix", "--extra-experimental-features",
               "nix-command flakes", "profile", "remove", &format!("nixpkgs#{attr}")])
        .output().await;
}
```

### User session lookup for notifications

The daemon needs the logged-in user's name to run notification commands in their session.
Read from `/run/omarchy/state.json` (add a `user` field written by the activation script)
or fall back to parsing the owner of `/run/user/*/` directories:

```rust
fn get_login_user() -> String {
    // Read /run/omarchy/login-user written by activation script
    std::fs::read_to_string("/run/omarchy/login-user")
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|_| "omarchy".to_owned())
}
```

The activation script writes `/run/omarchy/login-user` with the primary user's login name.

## Phase 4: Shell Shim Rewrites

### `bin/omarchy-pkg-colors` (sourced helper)

```bash
# No shebang — intended to be sourced, not executed directly.
# Usage: source omarchy-pkg-colors
# After sourcing: USE_COLOR, C_ACCENT, C_GREEN, C_RED, C_MUTED, C_FG, RESET,
#                 GLYPH_PKG, GLYPH_OK, GLYPH_FAIL are set.

_COLORS_TOML="${HOME}/.local/state/omarchy/current/theme/colors.toml"
USE_COLOR=false
[[ -t 1 && -f $_COLORS_TOML ]] && USE_COLOR=true

_read_color() {
  local key="$1"
  awk -F= -v key="$key" '
    function clean(raw) {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", raw)
      if (raw ~ /^"/) { sub(/^"/, "", raw); sub(/".*$/, "", raw) }
      return raw
    }
    { field = $1; gsub(/^[[:space:]]+|[[:space:]]+$/, "", field)
      if (field == key) { print clean($2); exit } }
  ' "$_COLORS_TOML"
}

_ansi_fg() {
  local hex="${1#\#}"
  printf '\033[38;2;%d;%d;%dm' \
    "$((16#${hex:0:2}))" "$((16#${hex:2:2}))" "$((16#${hex:4:2}))"
}

if [[ $USE_COLOR == "true" ]]; then
  C_ACCENT=$(_ansi_fg "$(_read_color accent)")
  C_GREEN=$(_ansi_fg "$(_read_color green)")
  C_RED=$(_ansi_fg "$(_read_color red)")
  C_MUTED=$(_ansi_fg "$(_read_color muted)")
  C_FG=$(_ansi_fg "$(_read_color foreground)")
  RESET=$'\033[0m'
else
  C_ACCENT="" C_GREEN="" C_RED="" C_MUTED="" C_FG="" RESET=""
fi

GLYPH_PKG="󰏖"
GLYPH_OK=""
GLYPH_FAIL=""
```

### `bin/omarchy-pkg-add` (rewrite)

```bash
#!/bin/bash
# omarchy:summary=Add a package
# omarchy:args=[--verbose] <package>
# omarchy:examples=omarchy pkg add git

verbose=false; pkg=""
for arg in "$@"; do
  case "$arg" in --verbose|-v) verbose=true ;; *) pkg="$arg" ;; esac
done
[[ -z $pkg ]] && { echo "Usage: omarchy-pkg-add [--verbose] <package>" >&2; exit 1; }

source omarchy-pkg-colors
NI="nix --extra-experimental-features 'nix-command flakes'"

if [[ $verbose == "false" ]]; then
  # FAST PATH — Phase 1: install to nix profile immediately
  if ! $NI profile install "nixpkgs#$pkg" 2>&1; then
    printf '%b%s package not found: %s%b\n' "$C_RED" "$GLYPH_FAIL" "$pkg" "$RESET" >&2
    printf '%bSearch with: nix search nixpkgs %s%b\n' "$C_MUTED" "$pkg" "$RESET" >&2
    exit 1
  fi

  # Phase 2: signal daemon for async rebuild
  coproc DAEMON { socat - UNIX-CONNECT:/run/omarchy/daemon.sock 2>/dev/null; }
  printf '{"cmd":"pkg-add-async","name":"%s"}\n' "$pkg" >&"${DAEMON[1]}"
  while IFS= read -r frame <&"${DAEMON[0]}"; do
    ok=$(printf '%s' "$frame" | jq -r '.ok // false')
    break
  done
  exec {DAEMON[1]}>&-; wait "$DAEMON_PID" 2>/dev/null

  if [[ $ok == "true" ]]; then
    printf '%b%s %s installed (system sync in progress)%b\n' \
      "$C_GREEN" "$GLYPH_OK" "$pkg" "$RESET"
  else
    err=$(printf '%s' "$frame" | jq -r '.error // "daemon error"')
    printf '%b%s %s%b\n' "$C_RED" "$GLYPH_FAIL" "$err" "$RESET" >&2
    exit 1
  fi

else
  # VERBOSE/SYNC PATH — blocks until rebuild completes
  coproc DAEMON { socat - UNIX-CONNECT:/run/omarchy/daemon.sock 2>/dev/null; }
  printf '{"cmd":"pkg-add","name":"%s"}\n' "$pkg" >&"${DAEMON[1]}"
  building_shown=false
  exit_code=0
  while IFS= read -r frame <&"${DAEMON[0]}"; do
    type=$(printf '%s' "$frame" | jq -r '.type // empty')
    if [[ $type == "progress" ]]; then
      _filter_rebuild_line "$(printf '%s' "$frame" | jq -r '.line')"
    else
      ok=$(printf '%s' "$frame" | jq -r '.ok // false')
      [[ $ok != "true" ]] && exit_code=1
      break
    fi
  done
  exec {DAEMON[1]}>&-; wait "$DAEMON_PID" 2>/dev/null
  if (( exit_code == 0 )); then
    printf '%b%s %s installed%b\n' "$C_GREEN" "$GLYPH_OK" "$pkg" "$RESET"
  else
    err=$(printf '%s' "$frame" | jq -r '.error // "unknown error"')
    printf '%b%s rebuild failed: %s%b\n' "$C_RED" "$GLYPH_FAIL" "$err" "$RESET" >&2
    [[ $err =~ "attribute"|"undefined"|"does not exist" ]] && \
      printf '%bSearch with: nix search nixpkgs %s%b\n' "$C_MUTED" "$pkg" "$RESET" >&2
    exit 1
  fi
fi
```

The `_filter_rebuild_line` function (defined in `omarchy-pkg-colors` or inlined) suppresses
store-path noise and translates semantic events to summary lines (see FR-001/FR-002).

### `bin/omarchy-pkg-drop` (rewrite)

Mirror of `pkg-add`: Phase 1 runs `nix profile remove nixpkgs#<attr>`, Phase 2 sends
`pkg-drop-async` to daemon. Verbose mode sends `pkg-remove` and blocks.

### `bin/omarchy-pkg-list` (rewrite)

```bash
#!/bin/bash
# omarchy:summary=List installed packages
# omarchy:group=pkg

source omarchy-pkg-colors
STATE_JSON="/var/lib/omarchy/state.json"
PACKAGES_JSON="/run/omarchy/packages.json"

if [[ ! -f $STATE_JSON ]]; then
  printf '%bNo packages installed.%b\n' "$C_MUTED" "$RESET"
  exit 0
fi

mapfile -t packages < <(jq -r '.packages[]' "$STATE_JSON" 2>/dev/null | sort)

if (( ${#packages[@]} == 0 )); then
  printf '%bNo packages installed.%b\n' "$C_MUTED" "$RESET"
  exit 0
fi

lookup_version() {
  [[ ! -f $PACKAGES_JSON ]] && { echo "—"; return; }
  jq -r --arg n "$1" '
    first(.[] | select(.name == $n) | .version) // "—"
  ' "$PACKAGES_JSON" 2>/dev/null
}

max_len=0
for p in "${packages[@]}"; do (( ${#p} > max_len )) && max_len=${#p}; done

for p in "${packages[@]}"; do
  ver=$(lookup_version "$p")
  printf '%b%-*s%b  %b%s%b\n' "$C_FG" "$max_len" "$p" "$RESET" "$C_MUTED" "$ver" "$RESET"
done
```

### `bin/omarchy-pkg-search` (new)

```bash
#!/bin/bash
# omarchy:summary=Search for packages
# omarchy:args=<term>
# omarchy:examples=omarchy pkg search json

source omarchy-pkg-colors
[[ -z $1 ]] && { echo "Usage: omarchy-pkg-search <term>" >&2; exit 1; }

STATE_JSON="/var/lib/omarchy/state.json"
installed=()
[[ -f $STATE_JSON ]] && mapfile -t installed < <(jq -r '.packages[]' "$STATE_JSON" 2>/dev/null)

# nix search nixpkgs outputs JSON when passed --json
results=$(nix --extra-experimental-features 'nix-command flakes' \
  search nixpkgs "$1" --json 2>/dev/null)

if [[ -z $results || $results == "{}" ]]; then
  printf '%bNo packages found.%b\n' "$C_MUTED" "$RESET"
  exit 0
fi

count=0
while IFS= read -r line; do
  (( count >= 20 )) && break
  name=$(printf '%s' "$line" | jq -r '.attrName // empty')
  ver=$(printf '%s' "$line"  | jq -r '.version // "—"')
  desc=$(printf '%s' "$line" | jq -r '.description // ""' | cut -c1-60)

  marker=" "
  for inst in "${installed[@]}"; do
    [[ $inst == "$name" ]] && { marker="${GLYPH_OK}"; break; }
  done

  if [[ $marker != " " ]]; then
    printf '%b%s %-30s%b  %b%-10s%b  %s\n' \
      "$C_GREEN" "$marker" "$name" "$RESET" "$C_MUTED" "$ver" "$RESET" "$desc"
  else
    printf '  %-30s  %b%-10s%b  %s\n' \
      "$name" "$C_MUTED" "$ver" "$RESET" "$desc"
  fi
  (( count++ ))
done < <(printf '%s' "$results" | jq -c 'to_entries[] | {attrName: .key, version: .value.version, description: .value.description}')
```

## Phase 5: Activation Script (`flake/nixosModules/omarchy.nix`)

Extend the activation script to write `/run/omarchy/login-user` with the primary user name, so the daemon can find the session for notifications:

```bash
# In the activation script block
echo "$primaryUser" > /run/omarchy/login-user
chmod 644 /run/omarchy/login-user
```

## Open Questions

1. **`nix profile` attribute path format**: The exact syntax for profile install may need to be `nixpkgs#legacyPackages.x86_64-linux.bat` rather than `nixpkgs#bat`. Verify in VM before finalizing.

2. **Profile cleanup timing**: After Phase 2 rebuild succeeds, running `nix profile remove` from the daemon (as root via `sudo -u user`) may fail if the profile path isn't in the expected location. Test this in the VM and adjust the cleanup command accordingly.

3. **Rebuild lock and background tasks**: If a user runs `pkg-add bat` followed immediately by `pkg-add ripgrep`, both Phase 1s succeed quickly. Both Phase 2 rebuilds queue behind the same lock. The second rebuild naturally includes both packages. State.json will be consistent since both `add_async` handlers write to it before spawning. But the profile cleanup for `bat` happens in the first background task's success handler, while the cleanup for `ripgrep` happens in the second's — this ordering is correct since each task cleans up its own attr.

4. **`pkg sync` command**: Needed as a recovery path when Phase 2 fails. Implementation: read all packages from state.json, run a single nixos-rebuild switch, clean up any profile entries for packages in state.json on success.
