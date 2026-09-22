use crate::alias::{self, ResolveError};
use crate::protocol::Response;
use crate::rebuild;
use crate::state::State;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

const MANIFEST_PATH: &str = "/run/omarchy/packages.json";

pub async fn add(
    name: &str,
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: mpsc::UnboundedSender<String>,
) -> Response {
    let nix_attr = match alias::resolve(name) {
        Ok(a) => a,
        Err(ResolveError::ServiceManaged(opt)) => {
            return Response::err(format!(
                "'{name}' is managed by NixOS option `{opt}`; \
                 use omarchy-setup to toggle it"
            ));
        }
        Err(ResolveError::Eliminated(reason)) => {
            return Response::err(format!("'{name}' is not available on NixOS: {reason}"));
        }
    };

    let _ = progress_tx.send(format!("queuing rebuild for '{name}'…"));

    let _guard = rebuild_lock.lock().await;
    let _ = progress_tx.send("rebuild lock acquired, loading state…".into());

    let mut state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    if state.packages.contains(&nix_attr.to_owned()) {
        return Response::err(format!("'{name}' is already installed"));
    }

    if let Err(e) = State::save_backup().await {
        tracing::warn!("backup failed (non-fatal): {e}");
    }

    state.packages.push(nix_attr.to_owned());
    if let Err(e) = state.save().await {
        return Response::err(format!("state save failed: {e}"));
    }

    let _ = progress_tx.send(format!("running nixos-rebuild switch for '{name}'…"));

    if let Err(e) = rebuild::run(&state, progress_tx).await {
        return Response::err(format!("rebuild failed: {e}"));
    }

    tracing::info!("pkg-add: {name} → {nix_attr} installed");
    Response::ok(None)
}

pub async fn remove(
    name: &str,
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: mpsc::UnboundedSender<String>,
) -> Response {
    let nix_attr = match alias::resolve(name) {
        Ok(a) => a,
        Err(ResolveError::ServiceManaged(opt)) => {
            return Response::err(format!(
                "'{name}' is managed by NixOS option `{opt}`; \
                 use omarchy-setup to toggle it"
            ));
        }
        Err(ResolveError::Eliminated(reason)) => {
            return Response::err(format!("'{name}' is not available on NixOS: {reason}"));
        }
    };

    let _ = progress_tx.send(format!("queuing rebuild to remove '{name}'…"));

    let _guard = rebuild_lock.lock().await;

    let mut state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    let before = state.packages.len();
    state.packages.retain(|p| p != nix_attr);

    if state.packages.len() == before {
        return Response::err(format!("'{name}' is not installed"));
    }

    if let Err(e) = State::save_backup().await {
        tracing::warn!("backup failed (non-fatal): {e}");
    }

    if let Err(e) = state.save().await {
        return Response::err(format!("state save failed: {e}"));
    }

    let _ = progress_tx.send(format!("running nixos-rebuild switch to remove '{name}'…"));

    if let Err(e) = rebuild::run(&state, progress_tx).await {
        return Response::err(format!("rebuild failed: {e}"));
    }

    tracing::info!("pkg-remove: {name} → {nix_attr} removed");
    Response::ok(None)
}

pub async fn list() -> Response {
    match tokio::fs::read(MANIFEST_PATH).await {
        Ok(data) => match serde_json::from_slice(&data) {
            Ok(v) => Response::ok(Some(v)),
            Err(e) => Response::err(format!("manifest parse error: {e}")),
        },
        Err(e) => Response::err(format!("manifest unavailable: {e}")),
    }
}

pub async fn present(name: &str) -> Response {
    match tokio::fs::read(MANIFEST_PATH).await {
        Ok(data) => {
            let pkgs: Vec<serde_json::Value> = serde_json::from_slice(&data).unwrap_or_default();
            let found = pkgs
                .iter()
                .any(|p| p.get("name").and_then(|n| n.as_str()) == Some(name));
            Response::ok(Some(serde_json::json!({ "present": found })))
        }
        Err(_) => Response::ok(Some(serde_json::json!({ "present": false }))),
    }
}
