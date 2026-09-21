# Implementation Plan: Omarchy Cinque — Bootable VM ISO

**Branch**: `001-vm-iso` | **Date**: 2026-09-21 | **Spec**: `spec.md`

## Summary

Stand up a flake-parts flake (following the daedalus pattern in `linux-packaging/`) that
produces `nix build .#iso` and `nix build .#vm` — a bootable Hyprland + Quickshell ISO with
all `omarchy-*` commands in PATH and `omarchy-nix-daemon` (Rust) running as a systemd service.
The daemon is a stub in phase 1: it listens, acks requests, and writes `packages.json`.
Full `nixos-rebuild` integration is phase 2.

## Technical Context

**Language/Version**: Rust 1.80+ (daemon via crane+fenix), Nix (modules/packages), Bash (shims)
**Primary Dependencies**: nixpkgs-unstable, flake-parts, crane, fenix, treefmt-nix, hyprland (hyprwm)
**Storage**: `/var/lib/omarchy/state.json`, `/run/omarchy/packages.json`
**Testing**: `nix build .#vm`, `checks.vm-smoke` NixOS VM test, `checks.shellcheck`
**Target Platform**: x86_64-linux only (phase 1)
**Constraints**: No ekapkgs. No flake-compat needed (all inputs are proper flakes). Daemon
must tolerate missing `state.json` on first boot.

## Constitution Check

- **I. Identity Preservation** ✓ — bin/ copied verbatim; Arch commands shimmed cleanly
- **II. Daemon-Mediated Configuration** ✓ — shims → socket → daemon; no direct nix writes
- **III. Native NixOS Implementation** ✓ — nixpkgs base; no pacman/paru/mise
- **IV. Language Choices** ✓ — Rust daemon (crane+fenix); Bash shims only
- **V. Mutable Config Ownership** ✓ — ~/.config/ untouched; OMARCHY_PATH from session env
- **VIII. Code Quality** ✓ — shellcheck check; cargo clippy; nix flake check passes

## Repository Layout

```
flake.nix
flake.lock
flake/
  lib/
    recursive-imports.nix         # copied verbatim from daedalus
  nixosModules/
    omarchy.nix                   # { flake.nixosModules.omarchy = {...}; }
perSystem/
  common.nix                      # _module.args.common, crane/fenix setup
  packages.nix                    # omarchy package, omarchy-nix-daemon package
  iso.nix                         # packages.iso, packages.vm, apps.run-vm
  checks.nix                      # checks.shellcheck, checks.vm-smoke
  devshells.nix                   # devShells.default
  formatter.nix                   # treefmt-nix formatter
daemon/                           # Rust crate root
  Cargo.toml
  Cargo.lock
  src/
    main.rs
    protocol.rs
    manifest.rs
    state.rs
    handlers/
      mod.rs
      pkg.rs
      system.rs
pkgs/
  omarchy/
    default.nix
    cinque-not-implemented.sh
```

## Phase 1 — `flake/lib/recursive-imports.nix`

Copy verbatim from daedalus. This provides the `recursiveImports` helper used in `flake.nix`
to auto-import every `.nix` file found under `./flake` and `./perSystem`.

## Phase 2 — `flake.nix`

```nix
{
  description = "Omarchy Cinque — Nix-native desktop environment";

  inputs = {
    nixpkgs.url         = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url     = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    crane.url           = "github:ipetkov/crane";
    fenix.url           = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url     = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    hyprland.url        = "github:hyprwm/Hyprland";
    hyprland.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, flake-parts, nixpkgs, ... } @ inputs:
  let
    inherit ((import ./flake/lib/recursive-imports.nix { inherit inputs; }).flake.lib)
      recursiveImports;
  in
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports =
        recursiveImports [ ./flake ./perSystem ]
        ++ [ inputs.treefmt-nix.flakeModule ];

      systems = [ "x86_64-linux" ];

      flake = {
        # Convenience for nix flake check on older Nix
        defaultPackage = builtins.mapAttrs (_: a: a.default) self.outputs.packages;
      };
    };

  nixConfig = {
    extra-substituters      = [ "https://hyprland.cachix.org" ];
    extra-trusted-public-keys = [
      "hyprland.cachix.org-1:a7pgxzMz7+chwVL3/pzj6jIITemDosxrE9/Kb+PfYvE="
    ];
  };
}
```

Key decisions:
- `hyprland.inputs.nixpkgs.follows = "nixpkgs"` — safe since we are building the ISO ourselves
  (not relying on the hyprwm binary cache). Revisit if we add a binary cache.
- `crane.url` uses the default nixpkgs toolchain via `crane.mkLib pkgs`; fenix overrides it
  to a musl-targeting toolchain for the final daemon binary (static, no glibc dependency).
- `treefmt-nix.flakeModule` wired at the top level; `perSystem/formatter.nix` configures it.

## Phase 3 — `perSystem/common.nix`

Injects `_module.args.common` into every `perSystem` module. For phase 1, `common` is small:

```nix
{ inputs, ... }: {
  perSystem = { pkgs, system, ... }: let
    fenixPkgs  = inputs.fenix.packages.${system};
    muslToolchain = fenixPkgs.combine [
      fenixPkgs.stable.cargo
      fenixPkgs.stable.rustc
      fenixPkgs.targets.x86_64-unknown-linux-musl.stable.rust-std
    ];
    craneLib = (inputs.crane.mkLib pkgs).overrideToolchain muslToolchain;

    common = {
      inherit craneLib muslToolchain;
      # Source for the daemon crate, isolated from other repo files so a
      # non-Rust change doesn't invalidate the Cargo build cache.
      daemonSrc = pkgs.lib.fileset.toSource {
        root    = ../daemon;
        fileset = pkgs.lib.fileset.unions [
          ../daemon/Cargo.toml
          ../daemon/Cargo.lock
          ../daemon/src
        ];
      };
    };
  in {
    _module.args.common = common;
  };
}
```

## Phase 4 — `perSystem/packages.nix`

### 4.1 `omarchy-nix-daemon` (Rust, static musl)

```nix
{ common, pkgs, ... }: {
  perSystem = { common, pkgs, ... }: {
    packages.omarchy-nix-daemon =
      common.craneLib.buildPackage {
        src           = common.daemonSrc;
        pname         = "omarchy-nix-daemon";
        version       = "0.1.0";
        strictDeps    = true;
        CARGO_BUILD_TARGET =
          "x86_64-unknown-linux-musl";
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER =
          "${pkgs.pkgsStatic.stdenv.cc}/bin/${pkgs.pkgsStatic.stdenv.cc.targetPrefix}cc";
      };
  };
}
```

Static musl binary means the daemon has no glibc dependency and survives `nixos-rebuild
switch` without needing to restart because its interpreter path doesn't change.

### 4.2 `omarchy` package

```nix
packages.omarchy = pkgs.stdenv.mkDerivation {
  pname   = "omarchy";
  version = inputs.self.shortRev or "dev";
  src     = inputs.self;

  installPhase = ''
    # Copy the whole source tree as OMARCHY_PATH.
    # bin/ files must be unmodified real files so the header-grep in
    # bin/omarchy (which scans for # omarchy:summary= lines) still works.
    mkdir -p $out
    cp -r bin config default shell themes migrations manual $out/

    # Not-implemented shims for eliminated Arch commands. Placed in
    # $out/cinque-shims/ so they can be prepended to PATH in the session
    # environment without modifying the real bin/ tree.
    mkdir -p $out/cinque-shims
    for cmd in \
      omarchy-pkg-add-aur omarchy-update-aur-pkgs omarchy-update-keyring \
      omarchy-update-pacman-guard omarchy-update-mise omarchy-dev-pkg-test \
      omarchy-upgrade-to-quattro omarchy-refresh-pacman; do
      ln -s ${./cinque-not-implemented.sh} $out/cinque-shims/$cmd
    done
  '';

  # patchShebangs would rewrite #!/bin/bash to the nix-store bash path,
  # which breaks nothing at runtime (NixOS provides /bin/bash) but
  # invalidates the header-grep if the grep reads from $out/bin directly.
  # Leave shebangs alone; NixOS provides /bin/bash via environment.binsh.
  dontPatchShebangs = true;
};
```

`pkgs/omarchy/cinque-not-implemented.sh`:
```bash
#!/bin/bash
cmd=$(basename "$0")
echo "omarchy: '$cmd' is not available in Omarchy Cinque." >&2
echo "Run 'omarchy help' for available commands." >&2
exit 1
```

## Phase 5 — `flake/nixosModules/omarchy.nix`

Defines `flake.nixosModules.omarchy`. Auto-imported by `recursiveImports ./flake`.

```nix
{ inputs, ... }: {
  flake.nixosModules.omarchy = { config, lib, pkgs, ... }: let
    cfg     = config.programs.omarchy;
    omarchy = inputs.self.packages.${pkgs.stdenv.hostPlatform.system}.omarchy;
    daemon  = inputs.self.packages.${pkgs.stdenv.hostPlatform.system}.omarchy-nix-daemon;
  in {
    options.programs.omarchy = {
      enable = lib.mkEnableOption "Omarchy Cinque";
      user   = lib.mkOption { type = lib.types.str; default = "omarchy"; };
    };

    config = lib.mkIf cfg.enable {
      programs.hyprland = {
        enable  = true;
        package = inputs.hyprland.packages.${pkgs.stdenv.hostPlatform.system}.hyprland;
      };

      # OMARCHY_PATH in the uwsm session environment, not hardcoded in the package.
      environment.sessionVariables.OMARCHY_PATH = "${omarchy}";
      # Cinque shims shadow Arch-only commands (not-implemented stubs).
      environment.sessionVariables.PATH = [ "${omarchy}/cinque-shims" ];

      environment.systemPackages = with pkgs; [
        omarchy daemon
        # Omarchy runtime invariants (commands the scripts call without guards)
        foot hyprpaper hyprlock hypridle hyprshot
        socat jq git curl wget rsync
        inputs.hyprland.packages.${pkgs.stdenv.hostPlatform.system}.xdg-desktop-portal-hyprland
      ];

      # omarchy-nix-daemon systemd service
      systemd.services.omarchy-nix-daemon = {
        description = "Omarchy Nix configuration daemon";
        wantedBy    = [ "multi-user.target" ];
        after       = [ "network.target" ];
        serviceConfig = {
          ExecStart        = "${daemon}/bin/omarchy-nix-daemon";
          Restart          = "on-failure";
          RestartSec       = "2s";
          # Creates /run/omarchy (mode 0755) and /var/lib/omarchy (mode 0750)
          RuntimeDirectory = "omarchy";
          StateDirectory   = "omarchy";
          # Socket is 0660; group omarchy; users in group omarchy can connect.
          UMask            = "0117";
          User             = "omarchy-daemon";
          Group            = "omarchy";
        };
      };

      users.users.omarchy-daemon = { isSystemUser = true; group = "omarchy"; };
      users.groups.omarchy = {};

      # Add the autologin user to the omarchy group so their scripts can reach
      # the daemon socket.
      users.users.${cfg.user}.extraGroups = [ "omarchy" "video" "audio" "networkmanager" "wheel" ];
    };
  };
}
```

## Phase 6 — `perSystem/iso.nix`

```nix
{ inputs, ... }: {
  perSystem = { system, pkgs, lib, config, ... }:
    lib.mkIf (system == "x86_64-linux") {

      packages.iso = inputs.self.nixosConfigurations.iso.config.system.build.isoImage;
      packages.vm  = inputs.self.nixosConfigurations.vm.config.system.build.vm;

      apps.run-vm = {
        type    = "app";
        program = lib.getExe config.packages.vm;
      };
    };

  flake.nixosConfigurations = let
    nixpkgs = inputs.nixpkgs;
    mkSystem = extraModules: nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      specialArgs = { inherit inputs; };
      modules = [
        inputs.self.nixosModules.omarchy
        { programs.omarchy.enable = true; }
        { networking.hostName = "omarchy-cinque"; }
        {
          users.users.omarchy = {
            isNormalUser    = true;
            initialPassword = "omarchy";
            extraGroups     = [ "omarchy" "wheel" "video" "audio" "networkmanager" ];
          };
        }
      ] ++ extraModules;
    };
  in {
    iso = mkSystem [{
      imports = [ "${nixpkgs}/nixos/modules/installer/cd-dvd/installation-cd-graphical-base.nix" ];
      isoImage.isoName             = "omarchy-cinque.iso";
      isoImage.squashfsCompression = "zstd -Xcompression-level 6";
      isoImage.appendToMenuLabel   = " Omarchy Cinque";
      services.displayManager.autoLogin = {
        enable = true;
        user   = "omarchy";
      };
    }];

    vm = mkSystem [{
      virtualisation.vmVariant = true;
      virtualisation.memorySize = 4096;
      virtualisation.diskSize   = 8192;
    }];
  };
}
```

Note: `flake.nixosConfigurations` is defined here (not in `perSystem`) because NixOS
configurations are not per-system outputs in flake-parts. The `iso` and `vm` packages in
`perSystem` are thin derivation-extracting wrappers over these configurations.

## Phase 7 — `daemon/` Rust crate

### 7.1 `daemon/Cargo.toml`

```toml
[package]
name    = "omarchy-nix-daemon"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "omarchy-nix-daemon"
path = "src/main.rs"

[dependencies]
tokio        = { version = "1", features = ["full"] }
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
tracing      = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
anyhow       = "1"
```

### 7.2 `daemon/src/protocol.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "kebab-case")]
pub enum Request {
    Ping,
    PkgAdd     { name: String },
    PkgRemove  { name: String },
    PkgList,
    PkgPresent { name: String },
    Status,
}

#[derive(Serialize)]
pub struct Response {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data:  Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn ok(data: impl Into<Option<serde_json::Value>>) -> Self {
        Self { ok: true, data: data.into(), error: None }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(msg.into()) }
    }
}
```

### 7.3 `daemon/src/manifest.rs`

Builds `/run/omarchy/packages.json` from the active NixOS generation:

```rust
use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

/// Query nix path-info for the current system and write a JSON array of
/// {name, version} objects to `dest`.  Failures are logged but non-fatal —
/// the daemon starts even if this step fails (e.g. nix not in PATH in early
/// boot).
pub async fn write_manifest(dest: &Path) -> Result<()> {
    let out = Command::new("nix")
        .args(["path-info", "--json", "--recursive", "/run/current-system"])
        .output()
        .await?;

    let raw: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let pkgs: Vec<serde_json::Value> = raw
        .as_object()
        .map(|m| m.values().filter_map(|v| {
            let name    = v.get("pname").or_else(|| v.get("name"))?.as_str()?;
            let version = v.get("version").and_then(|v| v.as_str()).unwrap_or("");
            Some(serde_json::json!({ "name": name, "version": version }))
        }).collect())
        .unwrap_or_default();

    tokio::fs::write(dest, serde_json::to_vec_pretty(&pkgs)?).await?;
    Ok(())
}
```

### 7.4 `daemon/src/main.rs`

```rust
mod handlers;
mod manifest;
mod protocol;
mod state;

use anyhow::Result;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};

const SOCKET_PATH:   &str = "/run/omarchy/daemon.sock";
const MANIFEST_PATH: &str = "/run/omarchy/packages.json";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    // Write initial packages manifest (best-effort).
    if let Err(e) = manifest::write_manifest(Path::new(MANIFEST_PATH)).await {
        error!("manifest write failed (non-fatal): {e}");
    } else {
        info!("manifest written to {MANIFEST_PATH}");
    }

    // Remove stale socket from a previous run.
    let _ = tokio::fs::remove_file(SOCKET_PATH).await;
    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("listening on {SOCKET_PATH}");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(mut stream: UnixStream) {
    let (reader, mut writer) = stream.split();
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let resp = match serde_json::from_str::<protocol::Request>(&line) {
            Ok(req)  => handlers::dispatch(req).await,
            Err(e)   => protocol::Response::err(format!("parse error: {e}")),
        };
        let mut buf = serde_json::to_vec(&resp).unwrap_or_default();
        buf.push(b'\n');
        if writer.write_all(&buf).await.is_err() { break; }
    }
}
```

### 7.5 `daemon/src/handlers/pkg.rs` (phase 1 stub)

```rust
use crate::protocol::Response;

pub async fn add(name: &str) -> Response {
    // Phase 1: acknowledge only. Phase 2 will write state.json and rebuild.
    tracing::info!("pkg-add request: {name} (stub — rebuild deferred to phase 2)");
    Response::ok(None)
}

pub async fn list() -> Response {
    match tokio::fs::read("/run/omarchy/packages.json").await {
        Ok(data) => match serde_json::from_slice(&data) {
            Ok(v)  => Response::ok(Some(v)),
            Err(e) => Response::err(format!("manifest parse error: {e}")),
        },
        Err(e) => Response::err(format!("manifest unavailable: {e}")),
    }
}

pub async fn present(name: &str) -> Response {
    match tokio::fs::read("/run/omarchy/packages.json").await {
        Ok(data) => {
            let pkgs: Vec<serde_json::Value> = serde_json::from_slice(&data).unwrap_or_default();
            let found = pkgs.iter().any(|p| {
                p.get("name").and_then(|n| n.as_str()) == Some(name)
            });
            Response::ok(Some(serde_json::json!({ "present": found })))
        }
        Err(e) => Response::err(format!("manifest unavailable: {e}")),
    }
}
```

## Phase 8 — `perSystem/checks.nix`

```nix
{ inputs, ... }: {
  perSystem = { system, pkgs, lib, config, ... }:
    lib.mkIf (system == "x86_64-linux") {
      checks = {

        # shellcheck on all bin/ scripts with #!/bin/bash
        shellcheck = pkgs.runCommand "shellcheck" {
          nativeBuildInputs = [ pkgs.shellcheck ];
        } ''
          shellcheck ${config.packages.omarchy}/bin/omarchy-*
          touch $out
        '';

        # Minimal NixOS VM smoke test
        vm-smoke = import ./tests/vm-smoke.nix { inherit inputs pkgs lib; };
      };
    };
}
```

`perSystem/tests/vm-smoke.nix`:
```nix
{ inputs, pkgs, lib }: pkgs.testers.nixosTest {
  name = "omarchy-cinque-vm-smoke";
  nodes.machine = { ... }: {
    imports = [ inputs.self.nixosModules.omarchy ];
    programs.omarchy.enable = true;
    virtualisation.memorySize = 2048;
  };
  testScript = ''
    machine.wait_for_unit("omarchy-nix-daemon.service")
    machine.wait_for_file("/run/omarchy/packages.json")
    machine.succeed("omarchy help")
    out = machine.succeed("omarchy pkg list")
    assert len(out.strip()) > 0, "pkg list output was empty"
  '';
}
```

## Phase 9 — `perSystem/devshells.nix` and `perSystem/formatter.nix`

Dev shell with Rust, nix, shellcheck, and the omarchy scripts on PATH:

```nix
{ inputs, ... }: {
  perSystem = { pkgs, common, config, ... }: {
    devShells.default = pkgs.mkShell {
      packages = with pkgs; [
        common.muslToolchain.cargo
        common.muslToolchain.rustc
        rust-analyzer
        shellcheck
        nix
        jq
        socat
      ];
      OMARCHY_PATH = config.packages.omarchy;
    };
  };
}
```

`perSystem/formatter.nix` — treefmt-nix wires nixfmt-rfc-style for `.nix` and rustfmt for `.rs`:

```nix
{ ... }: {
  perSystem = { ... }: {
    treefmt.config = {
      projectRootFile = "flake.nix";
      programs.nixfmt.enable  = true;   # nixfmt-rfc-style
      programs.rustfmt.enable = true;
    };
  };
}
```

## Phase 10 — Shim rewrites for `omarchy-pkg-add` and `omarchy-pkg-list`

The two commands in the acceptance criteria. These replace the Arch implementations
in `bin/` with Cinque-native versions that talk to the daemon socket via `socat`.

`bin/omarchy-pkg-add`:
```bash
#!/bin/bash
# omarchy:summary=Add a package
# omarchy:group=pkg
pkg="$1"
[[ -z $pkg ]] && { echo "Usage: omarchy-pkg-add <package>" >&2; exit 1; }
response=$(printf '{"cmd":"pkg-add","name":"%s"}\n' "$pkg" \
  | socat - UNIX-CONNECT:/run/omarchy/daemon.sock)
ok=$(printf '%s' "$response" | jq -r '.ok')
if [[ $ok != "true" ]]; then
  printf '%s' "$response" | jq -r '.error // "unknown error"' >&2
  exit 1
fi
```

`bin/omarchy-pkg-list`:
```bash
#!/bin/bash
# omarchy:summary=List installed packages
# omarchy:group=pkg
jq -r '.[].name' /run/omarchy/packages.json
```

## Complexity Tracking

> No constitution violations.

## Open Questions

1. **`dontPatchShebangs` and `/bin/bash`**: NixOS provides `/bin/sh` (via
   `environment.binsh`) and `/bin/bash` (via the bash compatibility package in the default
   system config). Verify that `installation-cd-graphical-base.nix` includes bash at
   `/bin/bash`; if not, add `environment.etc."bin/bash".source = "${pkgs.bash}/bin/bash"`.

2. **Quickshell version in nixpkgs-unstable**: Check `pkgs.quickshell.version >= "0.3.1"`
   at evaluation time. If not, override with outfoxxed's upstream (same pattern nixarchy uses
   in its overlay at `flake.nix:732-740`).

3. **`installation-cd-graphical-base.nix` vs `installation-cd-graphical-calamares.nix`**:
   The base module gives us the live-ISO image builder without an installer UI, which is what
   we want. It includes SDDM; we override the session to start Hyprland via uwsm.

4. **`flake.nixosConfigurations` placement**: flake-parts' `perSystem` can't export
   `nixosConfigurations` directly (they're not per-system). The approach above puts them in
   `iso.nix` under `flake.nixosConfigurations`. Verify that `recursiveImports ./flake` and
   `recursiveImports ./perSystem` combined don't conflict with a `flake.*` key defined in
   a `perSystem` file. If they do, move `nixosConfigurations` to `flake/nixosConfigurations.nix`.
