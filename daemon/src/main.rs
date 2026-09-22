mod alias;
mod handlers;
mod manifest;
mod protocol;
mod rebuild;
mod state;

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, Mutex};
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

    let rebuild_lock: Arc<Mutex<()>> = Arc::new(Mutex::new(()));

    if let Err(e) = manifest::write_manifest(Path::new(MANIFEST_PATH)).await {
        error!("manifest write failed (non-fatal): {e}");
    } else {
        info!("manifest written to {MANIFEST_PATH}");
    }

    let _ = tokio::fs::remove_file(SOCKET_PATH).await;
    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("listening on {SOCKET_PATH}");

    loop {
        let (stream, _) = listener.accept().await?;
        let lock = rebuild_lock.clone();
        tokio::spawn(handle_connection(stream, lock));
    }
}

async fn handle_connection(mut stream: UnixStream, rebuild_lock: Arc<Mutex<()>>) {
    let (reader, mut writer) = stream.split();
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let req = match serde_json::from_str::<protocol::Request>(&line) {
            Ok(r) => r,
            Err(e) => {
                let frame = protocol::Frame::Done(protocol::Response::err(format!("parse error: {e}")));
                write_frame(&mut writer, &frame).await;
                continue;
            }
        };

        let (frame_tx, mut frame_rx) = mpsc::unbounded_channel::<protocol::Frame>();
        let lock = rebuild_lock.clone();

        // Run the handler concurrently so we can forward frames as they arrive.
        let handler = tokio::spawn(handlers::dispatch(req, lock, frame_tx));

        while let Some(frame) = frame_rx.recv().await {
            write_frame(&mut writer, &frame).await;
        }

        let _ = handler.await;
    }
}

async fn write_frame(writer: &mut (impl AsyncWriteExt + Unpin), frame: &protocol::Frame) {
    let mut buf = serde_json::to_vec(frame).unwrap_or_default();
    buf.push(b'\n');
    let _ = writer.write_all(&buf).await;
}
