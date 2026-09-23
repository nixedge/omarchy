use crate::alias::{self, ResolveError};
use crate::protocol::Response;
use crate::rebuild;
use crate::state::State;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};

const MANIFEST_PATH: &str = "/run/omarchy/packages.json";
const LOGIN_USER_PATH: &str = "/run/omarchy/login-user";

// ── Notification and profile helpers ──────────────────────────────────────────

fn get_login_user() -> String {
    std::fs::read_to_string(LOGIN_USER_PATH)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "omarchy".to_owned())
}

async fn get_login_uid(user: &str) -> u32 {
    Command::new("id")
        .args(["-u", user])
        .output()
        .await
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(1000)
}

async fn send_desktop_notification(msg: &str, urgency: &str) {
    let user = get_login_user();
    let uid = get_login_uid(&user).await;
    let dbus = format!("unix:path=/run/user/{uid}/bus");
    let xdg = format!("/run/user/{uid}");
    // systemd-run --user delivers into the user's session even from a system service
    let result = Command::new("systemd-run")
        .args([
            "--uid", &uid.to_string(),
            "--user",
            "--no-ask-password",
            "--pipe",
            "--quiet",
            "--",
            "omarchy-notification-send",
            msg,
            "-u", urgency,
        ])
        .env("DBUS_SESSION_BUS_ADDRESS", &dbus)
        .env("XDG_RUNTIME_DIR", &xdg)
        .output()
        .await;
    if let Err(e) = result {
        tracing::warn!("notification delivery failed: {e}");
    }
}

async fn cleanup_profile(attr: &str) {
    let user = get_login_user();
    let uid = get_login_uid(&user).await;
    let xdg = format!("/run/user/{uid}");
    let installable = format!("nixpkgs#{attr}");
    let result = Command::new("systemd-run")
        .args([
            "--uid", &uid.to_string(),
            "--user",
            "--no-ask-password",
            "--pipe",
            "--quiet",
            "--",
            "nix",
            "--extra-experimental-features", "nix-command flakes",
            "profile", "remove", &installable,
        ])
        .env("XDG_RUNTIME_DIR", &xdg)
        .output()
        .await;
    if let Err(e) = result {
        tracing::warn!("profile cleanup failed for {attr}: {e}");
    }
}

async fn run_background_rebuild(
    rebuild_lock: Arc<Mutex<()>>,
    name: String,
    attr: String,
    is_removal: bool,
) {
    let _guard = rebuild_lock.lock().await;
    let state = match State::load().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("background rebuild: state load failed: {e}");
            return;
        }
    };
    let (tx, _rx) = mpsc::unbounded_channel::<String>();
    match rebuild::run(&state, tx).await {
        Ok(_) => {
            if !is_removal {
                cleanup_profile(&attr).await;
            }
            send_desktop_notification(&format!("\u{f00c} {name} installed"), "low").await;
        }
        Err(e) => {
            tracing::error!("background rebuild failed for {name}: {e}");
            let _ = State::restore_backup().await;
            if !is_removal {
                cleanup_profile(&attr).await;
            }
            send_desktop_notification(
                &format!(
                    "\u{f00d} {name}: system sync failed \u{2014} run omarchy pkg sync to retry"
                ),
                "critical",
            )
            .await;
        }
    }
}

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

// ── Fast-path async handlers ──────────────────────────────────────────────────

pub async fn add_async(name: &str, rebuild_lock: Arc<Mutex<()>>) -> Response {
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

    let attr_owned = nix_attr.to_owned();
    let name_owned = name.to_owned();
    tokio::spawn(run_background_rebuild(rebuild_lock, name_owned, attr_owned, false));

    tracing::info!("pkg-add-async: {name} → {nix_attr} queued");
    Response::ok_pending()
}

pub async fn drop_async(name: &str, rebuild_lock: Arc<Mutex<()>>) -> Response {
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

    let name_owned = name.to_owned();
    let attr_owned = nix_attr.to_owned();
    tokio::spawn(run_background_rebuild(rebuild_lock, name_owned, attr_owned, true));

    tracing::info!("pkg-drop-async: {name} → {nix_attr} queued");
    Response::ok_pending()
}

pub async fn sync(rebuild_lock: Arc<Mutex<()>>) -> Response {
    let state = match State::load().await {
        Ok(s) => s,
        Err(e) => return Response::err(format!("state load failed: {e}")),
    };
    tokio::spawn(async move {
        let _guard = rebuild_lock.lock().await;
        let (tx, _rx) = mpsc::unbounded_channel::<String>();
        match rebuild::run(&state, tx).await {
            Ok(_) => {
                send_desktop_notification("\u{f00c} System sync complete", "low").await;
            }
            Err(e) => {
                tracing::error!("pkg-sync rebuild failed: {e}");
                send_desktop_notification(
                    "\u{f00d} System sync failed \u{2014} check `omarchy pkg sync` output",
                    "critical",
                )
                .await;
            }
        }
    });
    Response::ok_pending()
}

pub async fn resolve_name(name: &str) -> Response {
    match alias::resolve(name) {
        Ok(attr) => Response::ok(Some(serde_json::Value::String(attr.to_owned()))),
        Err(ResolveError::ServiceManaged(opt)) => Response::err(format!(
            "'{name}' is managed by NixOS option `{opt}`; \
             use omarchy-setup to toggle it"
        )),
        Err(ResolveError::Eliminated(reason)) => {
            Response::err(format!("'{name}' is not available on NixOS: {reason}"))
        }
    }
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
