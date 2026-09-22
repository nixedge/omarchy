use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

/// Query the active NixOS generation and write a JSON array of {name, version}
/// objects to `dest`.  Failures are non-fatal — the daemon starts even if nix
/// is unavailable in early boot or a sandbox.
pub async fn write_manifest(dest: &Path) -> Result<()> {
    let out = Command::new("/run/current-system/sw/bin/nix")
        .args(["path-info", "--json", "--recursive", "/run/current-system"])
        .output()
        .await?;

    if !out.status.success() {
        anyhow::bail!(
            "nix path-info exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let raw: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let pkgs: Vec<serde_json::Value> = raw
        .as_object()
        .map(|m| {
            m.values()
                .filter_map(|v| {
                    let name = v
                        .get("pname")
                        .or_else(|| v.get("name"))?
                        .as_str()?
                        .to_owned();
                    let version = v
                        .get("version")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_owned();
                    Some(serde_json::json!({ "name": name, "version": version }))
                })
                .collect()
        })
        .unwrap_or_default();

    tokio::fs::write(dest, serde_json::to_vec_pretty(&pkgs)?).await?;
    Ok(())
}
