use crate::manifest;
use crate::state::State;
use anyhow::Result;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

const MODULE_PATH: &str = "/var/lib/omarchy/omarchy-managed.nix";
const FLAKE_URI_PATH: &str = "/etc/omarchy/flake-uri";
const FLAKE_CONFIG: &str = "omarchy-cinque";
const MANIFEST_PATH: &str = "/run/omarchy/packages.json";

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

async fn flake_uri() -> Result<String> {
    let uri = tokio::fs::read_to_string(FLAKE_URI_PATH).await.map_err(|e| {
        anyhow::anyhow!(
            "{FLAKE_URI_PATH} not found ({e}); \
             the system must be rebuilt from the latest omarchy flake \
             so the activation script can write this file"
        )
    })?;
    Ok(uri.trim().to_owned())
}

/// Write the managed module, run nixos-rebuild switch (streaming stderr to
/// `progress_tx`), and update packages.json on success.
/// Restores state.json from backup on rebuild failure.
pub async fn run(state: &State, progress_tx: mpsc::UnboundedSender<String>) -> Result<()> {
    write_module(state).await?;

    let uri = flake_uri().await?;
    let flake_arg = format!("{uri}#{FLAKE_CONFIG}");

    // Use the absolute path so sudo's restricted PATH doesn't matter.
    let mut child = Command::new("sudo")
        .args([
            "--",
            "/run/current-system/sw/bin/nixos-rebuild",
            "switch",
            "--impure",
            "--flake",
            &flake_arg,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    // Stream nixos-rebuild stderr to the caller line by line.
    if let Some(stderr) = child.stderr.take() {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = progress_tx.send(line);
        }
    }

    let status = child.wait().await?;

    if !status.success() {
        State::restore_backup().await?;
        anyhow::bail!("nixos-rebuild switch failed (exit {})", status);
    }

    if let Err(e) = manifest::write_manifest(Path::new(MANIFEST_PATH)).await {
        tracing::warn!("manifest update failed (non-fatal): {e}");
    }

    Ok(())
}
