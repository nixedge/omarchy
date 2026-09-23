use crate::manifest;
use crate::state::State;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

const MODULE_PATH: &str = "/var/lib/omarchy/omarchy-managed.nix";
const FLAKE_URI_PATH: &str = "/etc/omarchy/flake-uri";
const REBUILD_OPTS_PATH: &str = "/etc/omarchy/rebuild-opts";
const MANIFEST_PATH: &str = "/run/omarchy/packages.json";

const SUDO: &str = "/run/wrappers/bin/sudo";
const NIXOS_REBUILD: &str = "/run/current-system/sw/bin/nixos-rebuild";

pub fn generate_module(state: &State) -> String {
    let attrs: Vec<String> = state.packages.iter().map(|p| format!("    pkgs.{p}")).collect();
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
    tokio::fs::write(&tmp, &content).await.with_context(|| format!("writing {tmp}"))?;
    tokio::fs::rename(&tmp, MODULE_PATH).await.with_context(|| format!("renaming {tmp} → {MODULE_PATH}"))?;
    Ok(())
}

async fn flake_uri() -> Result<String> {
    let uri = tokio::fs::read_to_string(FLAKE_URI_PATH).await.with_context(|| {
        format!(
            "{FLAKE_URI_PATH} not found; rebuild the VM from the latest omarchy flake \
             so the activation script can write this file"
        )
    })?;
    Ok(uri.trim().to_owned())
}

pub enum Mode {
    Switch,
    DryActivate,
}

pub async fn run(state: &State, mode: Mode, progress_tx: mpsc::UnboundedSender<String>) -> Result<()> {
    write_module(state).await?;

    let flake_arg = flake_uri().await?;

    let extra_opts: Vec<String> = tokio::fs::read_to_string(REBUILD_OPTS_PATH)
        .await
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_owned)
        .collect();

    let subcommand = match mode {
        Mode::Switch => "switch",
        Mode::DryActivate => "dry-activate",
    };

    let mut args = vec![
        "--".to_owned(),
        NIXOS_REBUILD.to_owned(),
        subcommand.to_owned(),
        "--impure".to_owned(),
        "--accept-flake-config".to_owned(),
        "--flake".to_owned(),
        flake_arg,
    ];
    if matches!(mode, Mode::Switch) {
        args.extend(extra_opts);
    }

    let mut child = Command::new(SUDO)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawning {SUDO} {NIXOS_REBUILD}"))?;

    use tokio::io::AsyncReadExt;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let forward = |mut reader: tokio::process::ChildStdout, tx: mpsc::UnboundedSender<String>| async move {
        let mut buf = String::new();
        let _ = reader.read_to_string(&mut buf).await;
        for line in buf.lines() {
            let _ = tx.send(line.to_owned());
        }
    };

    let forward_err = |reader: tokio::process::ChildStderr, tx: mpsc::UnboundedSender<String>| async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = tx.send(line);
        }
    };

    let tx1 = progress_tx.clone();
    let tx2 = progress_tx.clone();
    tokio::join!(
        async { if let Some(s) = stdout { forward(s, tx1).await } },
        async { if let Some(s) = stderr { forward_err(s, tx2).await } },
    );

    let status = child.wait().await?;

    if !status.success() {
        if matches!(mode, Mode::Switch) {
            State::restore_backup().await?;
        }
        anyhow::bail!("nixos-rebuild {} failed (exit {})", subcommand, status);
    }

    if matches!(mode, Mode::Switch) {
        if let Err(e) = manifest::write_manifest(Path::new(MANIFEST_PATH)).await {
            tracing::warn!("manifest update failed (non-fatal): {e}");
        }
    }

    Ok(())
}
