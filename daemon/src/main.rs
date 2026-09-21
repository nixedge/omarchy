mod handlers;
mod manifest;
mod protocol;
mod state;

use anyhow::Result;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};

const SOCKET_PATH: &str = "/run/omarchy/daemon.sock";
const MANIFEST_PATH: &str = "/run/omarchy/packages.json";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    if let Err(e) = manifest::write_manifest(Path::new(MANIFEST_PATH)).await {
        error!("manifest write failed (non-fatal): {e}");
    } else {
        info!("manifest written to {MANIFEST_PATH}");
    }

    // Remove a stale socket from a previous run.
    let _ = tokio::fs::remove_file(SOCKET_PATH).await;
    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("listening on {SOCKET_PATH}");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(mut stream: UnixStream) {
    let (reader, mut writer) = stream.split();
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let resp = match serde_json::from_str::<protocol::Request>(&line) {
            Ok(req) => handlers::dispatch(req).await,
            Err(e) => protocol::Response::err(format!("parse error: {e}")),
        };
        let mut buf = serde_json::to_vec(&resp).unwrap_or_default();
        buf.push(b'\n');
        if writer.write_all(&buf).await.is_err() {
            break;
        }
    }
}
