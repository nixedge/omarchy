use crate::protocol::Response;

pub async fn ping() -> Response {
    Response::ok(Some(serde_json::json!({ "pong": true })))
}

pub async fn status() -> Response {
    let manifest_ok = tokio::fs::metadata("/run/omarchy/packages.json")
        .await
        .is_ok();
    Response::ok(Some(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "manifest_present": manifest_ok,
    })))
}
