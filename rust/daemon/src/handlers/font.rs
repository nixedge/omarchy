use crate::rebuild;
use crate::state::State;
use omarchy_lib::alias;
use omarchy_lib::protocol::Response;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub async fn add(
    name: &str,
    enable_family: Option<&str>,
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: mpsc::UnboundedSender<String>,
) -> Response {
    let nix_attr = alias::resolve_font(name);

    let _ = progress_tx.send(format!("queuing font rebuild for '{name}'…"));
    let _guard = rebuild_lock.lock().await;

    let mut state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    if state.fonts.contains(&nix_attr.to_owned()) {
        return Response::err(format!("'{name}' is already installed"));
    }

    if let Err(e) = State::save_backup().await {
        tracing::warn!("backup failed (non-fatal): {e}");
    }

    state.fonts.push(nix_attr.to_owned());
    if let Some(family) = enable_family {
        state.font_default = Some(family.to_owned());
    }
    if let Err(e) = state.save().await {
        return Response::err(format!("state save failed: {e}"));
    }

    let _ = progress_tx.send(format!("running nixos-rebuild switch for font '{name}'…"));

    if let Err(e) = rebuild::run(&state, rebuild::Mode::Switch, progress_tx).await {
        return Response::err(format!("rebuild failed: {e}"));
    }

    tracing::info!("font-add: {name} → {nix_attr} installed");
    Response::ok(None)
}

pub async fn enable(
    family: &str,
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: mpsc::UnboundedSender<String>,
) -> Response {
    let _ = progress_tx.send(format!("queuing rebuild to enable font '{family}'…"));
    let _guard = rebuild_lock.lock().await;

    let mut state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    state.font_default = Some(family.to_owned());
    if let Err(e) = state.save().await {
        return Response::err(format!("state save failed: {e}"));
    }

    let _ = progress_tx.send(format!("running nixos-rebuild switch to enable font '{family}'…"));

    if let Err(e) = rebuild::run(&state, rebuild::Mode::Switch, progress_tx).await {
        return Response::err(format!("rebuild failed: {e}"));
    }

    tracing::info!("font-enable: '{family}' set as NixOS default monospace font");
    Response::ok(None)
}

pub async fn remove(
    name: &str,
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: mpsc::UnboundedSender<String>,
) -> Response {
    let nix_attr = alias::resolve_font(name);

    let _ = progress_tx.send(format!("queuing rebuild to remove font '{name}'…"));
    let _guard = rebuild_lock.lock().await;

    let mut state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    let before = state.fonts.len();
    state.fonts.retain(|f| f != nix_attr);

    if state.fonts.len() == before {
        return Response::err(format!("'{name}' is not installed"));
    }

    if let Err(e) = State::save_backup().await {
        tracing::warn!("backup failed (non-fatal): {e}");
    }

    if let Err(e) = state.save().await {
        return Response::err(format!("state save failed: {e}"));
    }

    let _ = progress_tx.send(format!("running nixos-rebuild switch to remove font '{name}'…"));

    if let Err(e) = rebuild::run(&state, rebuild::Mode::Switch, progress_tx).await {
        return Response::err(format!("rebuild failed: {e}"));
    }

    tracing::info!("font-remove: {name} → {nix_attr} removed");
    Response::ok(None)
}

pub async fn list() -> Response {
    let state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };
    let fonts: Vec<serde_json::Value> = state
        .fonts
        .iter()
        .map(|name| serde_json::json!({ "name": name }))
        .collect();
    Response::ok(Some(serde_json::json!(fonts)))
}
