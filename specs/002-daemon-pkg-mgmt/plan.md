# Implementation Plan: Daemon Phase 2 — Live Package Management

**Branch**: `002-daemon-pkg-mgmt` | **Date**: 2026-09-22 | **Spec**: `spec.md`

## Summary

Wire the daemon's stub `pkg-add` / `pkg-remove` handlers into a real pipeline: resolve the
caller-supplied Arch name → nixpkgs attribute, write state.json, generate
`/etc/nixos/omarchy-managed.nix`, and run `nixos-rebuild switch` as a background task.
Stream progress to the caller. Atomically update `packages.json` on success; roll back
state.json on failure. Port the five shell-side pkg commands to use the daemon.

## Technical Context

**Language/Version**: Rust 1.80+ (daemon), Bash (shims)
**Primary Dependencies**: tokio (async runtime, already present), serde_json (already present)
**Storage**: `/var/lib/omarchy/state.json`, `/etc/nixos/omarchy-managed.nix`, `/run/omarchy/packages.json`
**Testing**: NixOS VM test (`checks.vm-smoke`), shell tests (`test/shell.d/`)
**Target Platform**: x86_64-linux (NixOS Cinque)
**Constraints**: No concurrent rebuilds; serialize via tokio Mutex. Rebuild must not block the daemon's accept loop.

## Constitution Check

- **I. Identity Preservation** ✓ — Arch names accepted via alias table; user-visible commands unchanged
- **II. Daemon-Mediated Configuration** ✓ — All nix writes go through daemon; shims never write directly
- **III. Native NixOS Implementation** ✓ — generates valid nixpkgs module; calls `nixos-rebuild switch`
- **IV. Language Choices** ✓ — Rust daemon; Bash shims only
- **VIII. Code Quality** ✓ — unit tests for alias resolution and module generation; shellcheck on shims

## Repository Layout

```
daemon/src/
  alias.rs          # NEW — Arch→nixpkgs name table + resolve()
  rebuild.rs        # NEW — async nixos-rebuild runner with progress streaming
  handlers/pkg.rs   # REWRITE — real add/remove logic (was stub)
  state.rs          # EXTEND — add backup/restore for rollback
  main.rs           # EXTEND — add rebuild queue (Mutex<()>)
bin/
  omarchy-pkg-drop      # REWRITE — daemon IPC (was direct pacman)
  omarchy-pkg-present   # REWRITE — reads packages.json directly
  omarchy-pkg-missing   # REWRITE — reads packages.json directly
  omarchy-pkg-install   # REWRITE — alias for omarchy-pkg-add
flake/nixosModules/
  omarchy.nix           # EXTEND — sudoers rule for nixos-rebuild, /etc/nixos provision
specs/002-daemon-pkg-mgmt/
  plan.md               # this file
  spec.md
```

## Phase 1 — Provision `/etc/nixos/` on first boot

The daemon needs a mutable local flake that `nixos-rebuild switch` can read and that imports
the daemon's generated module. Extend the activation script in `omarchy.nix` to set up
`/etc/nixos/` if it does not already exist.

The initial `/etc/nixos/flake.nix` is a minimal wrapper that re-imports the Omarchy module
(pinned to the current system generation's store path) and includes `./omarchy-managed.nix`:

```nix
# /etc/nixos/flake.nix  (provisioned by activation script on first boot)
{
  description = "Omarchy Cinque system — managed by omarchy-nix-daemon";

  inputs.nixpkgs.url = "path:/run/current-system/sw/share/omarchy/nixpkgs";
  # ^ Reuse the pinned nixpkgs from the active generation so no network is needed.

  outputs = { self, nixpkgs }: {
    nixosConfigurations.default = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        /run/current-system/etc/omarchy-base-module.nix  # the built omarchy module
        ./omarchy-managed.nix                            # daemon writes this
      ];
    };
  };
}
```

In practice, the activation script writes a simpler version: a
`/etc/nixos/configuration.nix` (non-flake) that the daemon uses with
`nixos-rebuild switch -I nixos-config=/etc/nixos/configuration.nix`. This sidesteps
flake hermeticity constraints entirely:

```nix
# /etc/nixos/configuration.nix
{ config, pkgs, lib, ... }: {
  imports = [
    # The omarchy base module is already active — this config only adds
    # daemon-managed user packages on top.
    ./omarchy-managed.nix
  ];
  system.stateVersion = "25.11";
}
```

The activation script (in `omarchy.nix`) also writes the initial `omarchy-managed.nix`
(empty package list) if it does not exist.

**Rebuild invocation**: `nixos-rebuild switch -I nixos-config=/etc/nixos/configuration.nix`

This avoids flake evaluation for the rebuild path and lets the daemon generate a plain
NixOS module that gets picked up at rebuild time without git staging or impure flags.

## Phase 2 — `daemon/src/alias.rs`

A static `phf::Map` (compile-time perfect hash) from Arch package name → nixpkgs attribute
path. Covers all `install/omarchy-base.packages` entries where the names differ.

```rust
// Key excerpts from the alias table
static ALIAS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // Arch name           → nixpkgs attribute
    "dua-cli"            => "dua",
    "gvfs-mtp"           => "gvfs",
    "gvfs-nfs"           => "gvfs",
    "gvfs-smb"           => "gvfs",
    "libvips"            => "vips",
    "libreoffice-fresh"  => "libreoffice",
    "lua51"              => "lua5_1",
    "mariadb-libs"       => "mariadb",
    "mise-bin"           => "mise",
    "noto-fonts-cjk"     => "noto-fonts-cjk-sans",
    "noto-fonts-emoji"   => "noto-fonts-color-emoji",
    "postgresql-libs"    => "libpq",
    "qt6-imageformats"   => "qt6.qtimageformats",
    "tree-sitter-cli"    => "tree-sitter",
    "ttf-jetbrains-mono-nerd-basic" => "nerd-fonts.jetbrains-mono",
    "vi"                 => "vim",
    "woff2-font-awesome" => "font-awesome",
    "xorg-xwayland"      => "xwayland",
    "yaru-icon-theme"    => "yaru-theme",
    // Service-managed packages — redirect to a helpful message instead of installing:
    // (handled separately in resolve(); these are controlled by NixOS module options)
    "bluez"           => "_service:hardware.bluetooth",
    "cups"            => "_service:services.printing",
    "docker"          => "_service:virtualisation.docker",
    "networkmanager"  => "_service:networking.networkmanager",
    "pipewire"        => "_service:services.pipewire",
    "sddm"            => "_service:services.displayManager.sddm",
    "wireplumber"     => "_service:services.pipewire",
};

pub fn resolve(arch_name: &str) -> Result<&'static str, ResolveError> {
    match ALIAS.get(arch_name) {
        Some(attr) if attr.starts_with("_service:") =>
            Err(ResolveError::ServiceManaged { option: &attr[9..] }),
        Some(attr) => Ok(attr),
        None => Ok(arch_name),   // assume 1:1 match if not in table
    }
}
```

Add `phf` to `Cargo.toml`:
```toml
phf = { version = "0.11", features = ["macros"] }
```

## Phase 3 — `daemon/src/state.rs` — add backup/restore

Extend `State` with snapshot helpers used by the rebuild pipeline for rollback:

```rust
impl State {
    pub async fn save_with_backup(&self) -> Result<PathBuf> {
        let backup = PathBuf::from(format!("{STATE_PATH}.bak"));
        if Path::new(STATE_PATH).exists() {
            tokio::fs::copy(STATE_PATH, &backup).await?;
        }
        self.save().await?;
        Ok(backup)
    }

    pub async fn restore_backup(backup: &Path) -> Result<()> {
        tokio::fs::rename(backup, STATE_PATH).await?;
        Ok(())
    }
}
```

Also add a `channel` field for future use:

```rust
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub packages: Vec<String>,  // nixpkgs attribute names
    #[serde(default)]
    pub channel: String,        // reserved for omarchy-channel-set (future spec)
}
```

## Phase 4 — `daemon/src/rebuild.rs`

Async function that generates the managed module, runs `nixos-rebuild switch`, streams
progress to the caller, and updates `packages.json` on success.

```rust
pub struct RebuildResult {
    pub success: bool,
    pub output: String,   // concatenated stdout+stderr for the caller
}

/// Generate /etc/nixos/omarchy-managed.nix from the current state.
pub fn generate_module(state: &State) -> String {
    let pkgs: String = state.packages
        .iter()
        .map(|p| format!("    {p}\n"))
        .collect();

    format!(
        "# Generated by omarchy-nix-daemon — do not edit manually\n\
         {{ pkgs, lib, ... }}: {{\n\
           environment.systemPackages = with pkgs; [\n\
         {pkgs}\
           ];\n\
         }}\n"
    )
}

pub async fn write_module(state: &State) -> Result<()> {
    let content = generate_module(state);
    tokio::fs::write("/etc/nixos/omarchy-managed.nix", content).await?;
    Ok(())
}

pub async fn run(state: &State, progress_tx: tokio::sync::mpsc::Sender<String>) -> RebuildResult {
    write_module(state).await.unwrap_or_else(|e| {
        tracing::error!("module write failed: {e}");
    });

    let mut cmd = tokio::process::Command::new("sudo");
    cmd.args(["nixos-rebuild", "switch",
              "-I", "nixos-config=/etc/nixos/configuration.nix"])
       .stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("nixos-rebuild spawn failed: {e}");
            let _ = progress_tx.send(msg.clone()).await;
            return RebuildResult { success: false, output: msg };
        }
    };

    // Stream stderr (nixos-rebuild writes progress there) to caller
    let mut output = String::new();
    if let Some(stderr) = child.stderr.take() {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            output.push_str(&line);
            output.push('\n');
            let _ = progress_tx.send(line).await;
        }
    }

    let status = child.wait().await.map(|s| s.success()).unwrap_or(false);

    if status {
        // Update packages.json to reflect the new generation
        if let Err(e) = manifest::write_manifest(Path::new(MANIFEST_PATH)).await {
            tracing::warn!("manifest refresh failed: {e}");
        }
    }

    RebuildResult { success: status, output }
}
```

## Phase 5 — `daemon/src/handlers/pkg.rs` — real implementation

Replace the phase-1 stubs with the full pipeline. The rebuild lock (`Arc<Mutex<()>>`) is
passed from `main.rs` so only one rebuild runs at a time; callers queue behind it.

```rust
pub async fn add(
    name: &str,
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: tokio::sync::mpsc::Sender<String>,
) -> Response {
    let attr = match alias::resolve(name) {
        Ok(a) => a,
        Err(alias::ResolveError::ServiceManaged { option }) => {
            return Response::err(format!(
                "'{name}' is managed by the NixOS option `{option}`; \
                 use omarchy-setup or the Omarchy settings panel to toggle it."
            ));
        }
    };

    let _lock = rebuild_lock.lock().await;

    let mut state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    if state.packages.contains(&attr.to_string()) {
        return Response::err(format!("'{name}' is already installed"));
    }

    state.packages.push(attr.to_string());

    let backup = match state.save_with_backup().await {
        Ok(b) => b,
        Err(e) => return Response::err(format!("state save failed: {e}")),
    };

    let result = rebuild::run(&state, progress_tx).await;

    if !result.success {
        let _ = State::restore_backup(&backup).await;
        return Response::err(format!("rebuild failed:\n{}", result.output));
    }

    Response::ok(None)
}

pub async fn remove(
    name: &str,
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: tokio::sync::mpsc::Sender<String>,
) -> Response {
    let attr = match alias::resolve(name) {
        Ok(a) => a,
        Err(alias::ResolveError::ServiceManaged { option }) => {
            return Response::err(format!(
                "'{name}' is managed by `{option}`"
            ));
        }
    };

    let _lock = rebuild_lock.lock().await;
    let mut state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    let before = state.packages.len();
    state.packages.retain(|p| p != attr);

    if state.packages.len() == before {
        return Response::err(format!("'{name}' is not installed"));
    }

    let backup = match state.save_with_backup().await {
        Ok(b) => b,
        Err(e) => return Response::err(format!("state save failed: {e}")),
    };

    let result = rebuild::run(&state, progress_tx).await;

    if !result.success {
        let _ = State::restore_backup(&backup).await;
        return Response::err(format!("rebuild failed:\n{}", result.output));
    }

    Response::ok(None)
}
```

The `progress_tx` side: the socket protocol gains a new response variant for streaming:

```rust
// protocol.rs — add streaming progress lines
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Frame {
    Progress { line: String },
    Done(Response),
}
```

The connection handler streams `Frame::Progress` lines until the rebuild finishes, then
sends `Frame::Done`.

## Phase 6 — `daemon/src/main.rs` — rebuild queue

Add the shared rebuild lock and thread it into handlers:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    // ... existing init ...
    let rebuild_lock: Arc<Mutex<()>> = Arc::new(Mutex::new(()));

    loop {
        let (stream, _) = listener.accept().await?;
        let lock = Arc::clone(&rebuild_lock);
        tokio::spawn(handle_connection(stream, lock));
    }
}
```

## Phase 7 — NixOS module additions (`flake/nixosModules/omarchy.nix`)

### 7.1 sudoers rule for nixos-rebuild

```nix
security.sudo.extraRules = [
  {
    users = [ "omarchy-daemon" ];
    commands = [
      {
        command = "/run/current-system/sw/bin/nixos-rebuild";
        options = [ "NOPASSWD" ];
      }
    ];
  }
];
```

### 7.2 Daemon service — writable paths

```nix
systemd.services.omarchy-nix-daemon.serviceConfig = {
  # ... existing ...
  ReadWritePaths = [ "/etc/nixos" "/var/lib/omarchy" "/run/omarchy" ];
};
```

### 7.3 Activation script extension

Extend the existing `omarchyUserConfig` activation script to also provision `/etc/nixos/`:

```bash
# Provision /etc/nixos/ for nixos-rebuild on first boot
if [[ ! -f /etc/nixos/configuration.nix ]]; then
  mkdir -p /etc/nixos
  cat > /etc/nixos/configuration.nix << 'EOF'
# Omarchy Cinque system configuration — managed by omarchy-nix-daemon
{ config, pkgs, lib, ... }: {
  imports = [ ./omarchy-managed.nix ];
  system.stateVersion = "25.11";
}
EOF
  cat > /etc/nixos/omarchy-managed.nix << 'EOF'
# Generated by omarchy-nix-daemon — do not edit manually
{ pkgs, lib, ... }: {
  environment.systemPackages = with pkgs; [];
}
EOF
fi
```

## Phase 8 — Shell command rewrites

### `bin/omarchy-pkg-drop`

```bash
#!/bin/bash
# omarchy:summary=Remove a package
# omarchy:group=pkg
pkg="$1"
[[ -z $pkg ]] && { echo "Usage: omarchy-pkg-drop <package>" >&2; exit 1; }
response=$(printf '{"cmd":"pkg-remove","name":"%s"}\n' "$pkg" \
  | socat - UNIX-CONNECT:/run/omarchy/daemon.sock)
ok=$(printf '%s' "$response" | jq -r '.ok // false')
if [[ $ok != "true" ]]; then
  printf '%s\n' "$response" | jq -r '.error // "unknown error"' >&2
  exit 1
fi
```

### `bin/omarchy-pkg-present`

```bash
#!/bin/bash
# omarchy:summary=Exit 0 if a package is installed
# omarchy:group=pkg
# omarchy:hidden=true
pkg="$1"
[[ -z $pkg ]] && { echo "Usage: omarchy-pkg-present <package>" >&2; exit 1; }
jq -e --arg p "$pkg" 'any(.[]; .name == $p)' /run/omarchy/packages.json >/dev/null 2>&1
```

### `bin/omarchy-pkg-missing`

```bash
#!/bin/bash
# omarchy:summary=Exit 0 if a package is not installed
# omarchy:group=pkg
# omarchy:hidden=true
pkg="$1"
[[ -z $pkg ]] && { echo "Usage: omarchy-pkg-missing <package>" >&2; exit 1; }
! jq -e --arg p "$pkg" 'any(.[]; .name == $p)' /run/omarchy/packages.json >/dev/null 2>&1
```

### `bin/omarchy-pkg-install`

```bash
#!/bin/bash
# omarchy:summary=Install a package (alias for omarchy-pkg-add)
# omarchy:group=pkg
# omarchy:hidden=true
exec omarchy-pkg-add "$@"
```

Also update `bin/omarchy-pkg-add` to stream progress lines from the daemon:

```bash
#!/bin/bash
# omarchy:summary=Add a package
# omarchy:group=pkg
pkg="$1"
[[ -z $pkg ]] && { echo "Usage: omarchy-pkg-add <package>" >&2; exit 1; }
printf '{"cmd":"pkg-add","name":"%s"}\n' "$pkg" \
  | socat - UNIX-CONNECT:/run/omarchy/daemon.sock \
  | while IFS= read -r frame; do
      type=$(printf '%s' "$frame" | jq -r '.type // "done"')
      if [[ $type == "progress" ]]; then
        printf '%s' "$frame" | jq -r '.line'
      else
        ok=$(printf '%s' "$frame" | jq -r '.ok // false')
        if [[ $ok != "true" ]]; then
          printf '%s\n' "$frame" | jq -r '.error // "unknown error"' >&2
          exit 1
        fi
        break
      fi
    done
```

## Phase 9 — Shell tests

New test file `test/shell.d/pkg-commands-test.sh`:

- Verify `omarchy-pkg-present` exits 0 when a known package from `packages.json` is queried.
- Verify `omarchy-pkg-missing` exits 0 when an absent package is queried.
- Verify `omarchy-pkg-present` exits 1 when a package is absent.
- Verify `omarchy-pkg-missing` exits 1 when a package is present.
- Mock `packages.json` with a fixture file to avoid daemon dependency in unit tests.

## Phase 10 — Cargo.toml update

```toml
[dependencies]
# ... existing ...
phf = { version = "0.11", features = ["macros"] }

[build-dependencies]
phf_codegen = "0.11"
```

## Constitution Check (Implementation)

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Identity | ✓ | Arch names accepted; user commands unchanged |
| II. Daemon-mediated | ✓ | All nix writes via daemon; shims are pure IPC callers |
| III. Native NixOS | ✓ | Generates valid nixpkgs module; uses nixos-rebuild |
| IV. Language | ✓ | Rust daemon; Bash shims only |
| VII. Backward compat | ✓ | pkg-install aliased to pkg-add; Arch names resolved |
| VIII. Testing | ✓ | Shell tests for pkg-present/missing; VM test for rebuild |

## Open Questions

1. **nixos-rebuild path under sudo**: The sudoers rule references `/run/current-system/sw/bin/nixos-rebuild`. This symlink changes with every generation. Consider using `Cmnd_Alias` in sudoers pointing to the wrapper or using a NixOS setuid wrapper via `security.wrappers`.

2. **Progress streaming and socat**: `socat` closes the connection when the first `\n` terminates. To keep the connection open for streaming, switch to `socat - UNIX-CONNECT:... 2>&1` with a loop, or use a dedicated long-lived connection. Alternatively, make the daemon write all progress to a logfile and let the shim tail it.

3. **Concurrent callers during rebuild**: If two clients both call `pkg-add` concurrently, only one holds the rebuild lock; the other blocks until the first finishes. This is correct per the spec, but the blocked client will appear to hang. Consider sending a `{type: "queued"}` frame immediately so the user sees feedback.

4. **State bootstrap on a fresh VM**: The first time the daemon writes `managed.nix`, nixos-rebuild must succeed even with no additional packages. Ensure the generated module with an empty `systemPackages` list is syntactically valid and doesn't conflict with the base module.
