use crate::rebuild;
use crate::state::State;
use omarchy_lib::protocol::Response;
use omarchy_lib::services::KNOWN as KNOWN_SERVICES;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub async fn enable(
    name: &str,
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: mpsc::UnboundedSender<String>,
) -> Response {
    if !KNOWN_SERVICES.contains(&name) {
        return Response::err(format!(
            "unknown service '{name}'; run 'omarchy service list' to see available services"
        ));
    }

    let _guard = rebuild_lock.lock().await;

    let mut state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    if state.services.contains(&name.to_owned()) {
        return Response::err(format!("service '{name}' is already enabled"));
    }

    if let Err(e) = State::save_backup().await {
        tracing::warn!("backup failed (non-fatal): {e}");
    }

    state.services.push(name.to_owned());
    if let Err(e) = state.save().await {
        return Response::err(format!("state save failed: {e}"));
    }

    let _ = progress_tx.send(format!("enabling service '{name}' via nixos-rebuild…"));

    if let Err(e) = rebuild::run(&state, rebuild::Mode::Switch, progress_tx).await {
        return Response::err(format!("rebuild failed: {e}"));
    }

    tracing::info!("service-enable: '{name}' enabled");
    Response::ok(None)
}

pub async fn disable(
    name: &str,
    rebuild_lock: Arc<Mutex<()>>,
    progress_tx: mpsc::UnboundedSender<String>,
) -> Response {
    let _guard = rebuild_lock.lock().await;

    let mut state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    let before = state.services.len();
    state.services.retain(|s| s != name);

    if state.services.len() == before {
        return Response::err(format!("service '{name}' is not enabled"));
    }

    if let Err(e) = State::save_backup().await {
        tracing::warn!("backup failed (non-fatal): {e}");
    }

    if let Err(e) = state.save().await {
        return Response::err(format!("state save failed: {e}"));
    }

    let _ = progress_tx.send(format!("disabling service '{name}' via nixos-rebuild…"));

    if let Err(e) = rebuild::run(&state, rebuild::Mode::Switch, progress_tx).await {
        return Response::err(format!("rebuild failed: {e}"));
    }

    tracing::info!("service-disable: '{name}' disabled");
    Response::ok(None)
}

pub async fn list() -> Response {
    let state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };

    let enabled: Vec<serde_json::Value> =
        state.services.iter().map(|s| serde_json::Value::String(s.clone())).collect();

    let available: Vec<serde_json::Value> =
        KNOWN_SERVICES.iter().map(|s| serde_json::Value::String(s.to_string())).collect();

    Response::ok(Some(serde_json::json!({
        "enabled": enabled,
        "available": available,
    })))
}
