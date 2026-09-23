use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

/// Query the active NixOS generation and write a JSON array of {name, version}
/// objects to `dest`.  Failures are non-fatal — the daemon starts even if nix
/// is unavailable in early boot or a sandbox.
pub async fn write_manifest(dest: &Path) -> Result<()> {
    let out = Command::new("/run/current-system/sw/bin/nix")
        .args([
            "--extra-experimental-features",
            "nix-command",
            "path-info",
            "--json",
            "--recursive",
            "/run/current-system",
        ])
        .output()
        .await?;

    if !out.status.success() {
        anyhow::bail!(
            "nix path-info exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    // nix path-info --json outputs an object keyed by store path.
    // Extract the package name by stripping the /nix/store/<hash>- prefix.
    let raw: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let pkgs: Vec<serde_json::Value> = raw
        .as_object()
        .map(|m| {
            m.keys()
                .filter_map(|path| {
                    // Store paths look like /nix/store/HASH-name[-version]
                    let after_hash = path
                        .strip_prefix("/nix/store/")?
                        .splitn(2, '-')
                        .nth(1)?;
                    Some(serde_json::json!({ "name": after_hash }))
                })
                .collect()
        })
        .unwrap_or_default();

    tokio::fs::write(dest, serde_json::to_vec_pretty(&pkgs)?).await?;
    Ok(())
}
