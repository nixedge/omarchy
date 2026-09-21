use crate::protocol::Response;

const MANIFEST_PATH: &str = "/run/omarchy/packages.json";

pub async fn add(name: &str) -> Response {
    // Phase 1 stub: acknowledge the request; phase 2 writes state.json and rebuilds.
    tracing::info!("pkg-add: {name} (stub — nixos-rebuild deferred to phase 2)");
    Response::ok(None)
}

pub async fn remove(name: &str) -> Response {
    tracing::info!("pkg-remove: {name} (stub — nixos-rebuild deferred to phase 2)");
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
            let pkgs: Vec<serde_json::Value> =
                serde_json::from_slice(&data).unwrap_or_default();
            let found = pkgs.iter().any(|p| {
                p.get("name").and_then(|n| n.as_str()) == Some(name)
            });
            Response::ok(Some(serde_json::json!({ "present": found })))
        }
        Err(e) => Response::err(format!("manifest unavailable: {e}")),
    }
}
