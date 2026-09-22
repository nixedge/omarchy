pub mod pkg;
pub mod system;

use crate::protocol::{Frame, Request, Response};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Dispatch a request, returning a stream of frames.
/// Simple (fast) requests produce a single Done frame immediately.
/// Rebuild operations produce Progress frames followed by Done.
pub async fn dispatch(
    req: Request,
    rebuild_lock: Arc<Mutex<()>>,
    frame_tx: mpsc::UnboundedSender<Frame>,
) {
    // Wrap a progress-line sender that maps strings to Frame::Progress.
    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<String>();

    // Forward progress lines to the frame channel while the handler runs.
    let frame_tx2 = frame_tx.clone();
    let forwarder = tokio::spawn(async move {
        while let Some(line) = progress_rx.recv().await {
            let _ = frame_tx2.send(Frame::Progress { line });
        }
    });

    let resp = match req {
        Request::Ping => system::ping().await,
        Request::Status => system::status().await,
        Request::PkgAdd { name } => pkg::add(&name, rebuild_lock, progress_tx).await,
        Request::PkgRemove { name } => pkg::remove(&name, rebuild_lock, progress_tx).await,
        Request::PkgList => pkg::list().await,
        Request::PkgPresent { name } => pkg::present(&name).await,
    };

    let _ = forwarder.await;
    let _ = frame_tx.send(Frame::Done(resp));
}

/// Helper for commands that need no rebuild — returns a single Response directly.
#[allow(dead_code)]
pub async fn dispatch_simple(req: Request, rebuild_lock: Arc<Mutex<()>>) -> Response {
    match req {
        Request::Ping => system::ping().await,
        Request::Status => system::status().await,
        Request::PkgList => pkg::list().await,
        Request::PkgPresent { name } => pkg::present(&name).await,
        _ => {
            let (tx, _rx) = mpsc::unbounded_channel();
            match req {
                Request::PkgAdd { name } => pkg::add(&name, rebuild_lock, tx).await,
                Request::PkgRemove { name } => pkg::remove(&name, rebuild_lock, tx).await,
                _ => Response::err("unreachable"),
            }
        }
    }
}
