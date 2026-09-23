use crate::rebuild;
use crate::state::State;
use omarchy_lib::protocol::Response;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub const USER_CONFIG_PATH: &str = "/var/lib/omarchy/omarchy-user.nix";
const USER_CONFIG_BACKUP: &str = "/var/lib/omarchy/omarchy-user.nix.bak";
const USER_CONFIG_STAGING: &str = "/var/lib/omarchy/omarchy-user.nix.staging";

pub const STUB: &str = "\
# User-managed Omarchy configuration.
# Edit with: omarchy config edit
#
# This is a standard NixOS module. Add any NixOS options here and they
# will be validated and applied on the next rebuild.
#
# Examples:
#   services.tailscale.enable = true;
#   services.nginx.virtualHosts.\"example.com\" = { ... };
{ pkgs, lib, ... }: {
}
";

pub async fn get() -> Response {
    let content = tokio::fs::read_to_string(USER_CONFIG_PATH)
        .await
        .unwrap_or_else(|_| STUB.to_owned());
    Response::ok(Some(serde_json::Value::String(content)))
}

pub async fn apply(
    content: String,
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: mpsc::UnboundedSender<String>,
) -> Response {
    let _guard = rebuild_lock.lock().await;

    // Back up current live config.
    if let Err(e) = tokio::fs::copy(USER_CONFIG_PATH, USER_CONFIG_BACKUP).await {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!("user config backup failed (non-fatal): {e}");
        }
    }

    // Write staging, then atomically rename to live.
    if let Err(e) = tokio::fs::write(USER_CONFIG_STAGING, &content).await {
        return Response::err(format!("failed to write staging config: {e}"));
    }
    if let Err(e) = tokio::fs::rename(USER_CONFIG_STAGING, USER_CONFIG_PATH).await {
        return Response::err(format!("failed to promote staging config: {e}"));
    }

    let state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    let _ = progress_tx.send("running nixos-rebuild switch…".into());
    if let Err(e) = rebuild::run(&state, rebuild::Mode::Switch, progress_tx).await {
        // Rebuild failed — restore backup.
        if let Err(re) = tokio::fs::copy(USER_CONFIG_BACKUP, USER_CONFIG_PATH).await {
            tracing::error!("config rollback failed: {re}");
        }
        return Response::err(format!("rebuild failed, config restored: {e}"));
    }

    tracing::info!("config: user config applied successfully");
    Response::ok(None)
}

pub async fn check(
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: mpsc::UnboundedSender<String>,
) -> Response {
    let _guard = rebuild_lock.lock().await;
    let state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };
    let _ = progress_tx.send("running nixos-rebuild dry-activate…".into());
    match rebuild::run(&state, rebuild::Mode::DryActivate, progress_tx).await {
        Ok(_) => Response::ok(None),
        Err(e) => Response::err(format!("check failed: {e}")),
    }
}
