use crate::manifest;
use crate::state::State;
use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

/// Path the daemon writes; imported by the NixOS module with builtins.pathExists.
const MODULE_PATH: &str = "/var/lib/omarchy/omarchy-managed.nix";
/// Store path of the omarchy flake written by the NixOS module activation script.
const FLAKE_URI_PATH: &str = "/etc/omarchy/flake-uri";
const FLAKE_CONFIG: &str = "omarchy-cinque";
const MANIFEST_PATH: &str = "/run/omarchy/packages.json";

/// Generate a NixOS module from the current State.
pub fn generate_module(state: &State) -> String {
    let attrs: Vec<String> = state
        .packages
        .iter()
        .map(|p| format!("    pkgs.{p}"))
        .collect();
    let list = attrs.join("\n");
    format!(
        "# Managed by omarchy-nix-daemon — do not edit manually.\n\
         {{ pkgs, ... }}:\n\
         {{\n\
           environment.systemPackages = [\n\
         {list}\n\
           ];\n\
         }}\n"
    )
}

pub async fn write_module(state: &State) -> Result<()> {
    let content = generate_module(state);
    let tmp = format!("{MODULE_PATH}.tmp");
    tokio::fs::write(&tmp, content).await?;
    tokio::fs::rename(&tmp, MODULE_PATH).await?;
    Ok(())
}

/// Read the flake URI written by the NixOS module activation script.
async fn flake_uri() -> Result<String> {
    let uri = tokio::fs::read_to_string(FLAKE_URI_PATH).await?;
    Ok(uri.trim().to_owned())
}

/// Write the managed module, run nixos-rebuild switch, and update packages.json.
/// Restores state.json from backup on rebuild failure.
pub async fn run(state: &State) -> Result<()> {
    write_module(state).await?;

    let uri = flake_uri().await?;
    let flake_arg = format!("{uri}#{FLAKE_CONFIG}");

    let status = Command::new("sudo")
        .args([
            "--",
            "nixos-rebuild",
            "switch",
            "--impure",
            "--flake",
            &flake_arg,
        ])
        .status()
        .await?;

    if !status.success() {
        State::restore_backup().await?;
        anyhow::bail!("nixos-rebuild switch failed (exit {})", status);
    }

    if let Err(e) = manifest::write_manifest(Path::new(MANIFEST_PATH)).await {
        tracing::warn!("manifest update failed (non-fatal): {e}");
    }

    Ok(())
}
