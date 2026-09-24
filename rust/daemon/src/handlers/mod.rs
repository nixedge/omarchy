pub mod config;
pub mod font;
pub mod pkg;
pub mod service;
pub mod system;

use omarchy_lib::protocol::{Frame, Request};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub async fn dispatch(
    req: Request,
    rebuild_lock: Arc<Mutex<()>>,
    frame_tx: mpsc::UnboundedSender<Frame>,
) {
    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<String>();

    let frame_tx2 = frame_tx.clone();
    let forwarder = tokio::spawn(async move {
        while let Some(line) = progress_rx.recv().await {
            let _ = frame_tx2.send(Frame::Progress { line });
        }
    });

    let resp = match req {
        // Streaming handlers — keep progress_tx alive through the rebuild.
        Request::PkgAdd { name } => pkg::add(&name, rebuild_lock, progress_tx).await,
        Request::PkgAddMany { names } => pkg::add_many(&names, rebuild_lock, progress_tx).await,
        Request::PkgRemove { name } => pkg::remove(&name, rebuild_lock, progress_tx).await,
        Request::PkgRemoveMany { names } => pkg::remove_many(&names, rebuild_lock, progress_tx).await,
        Request::ConfigApply { content } => config::apply(content, rebuild_lock, progress_tx).await,
        Request::ConfigCheck => config::check(rebuild_lock, progress_tx).await,
        Request::ServiceEnable { name } => service::enable(&name, rebuild_lock, progress_tx).await,
        Request::ServiceDisable { name } => service::disable(&name, rebuild_lock, progress_tx).await,
        Request::FontAdd { name } => font::add(&name, rebuild_lock, progress_tx).await,
        Request::FontRemove { name } => font::remove(&name, rebuild_lock, progress_tx).await,
        req => {
            drop(progress_tx);
            match req {
                Request::Ping => system::ping().await,
                Request::Status => system::status().await,
                Request::PkgAddAsync { name } => pkg::add_async(&name, rebuild_lock).await,
                Request::PkgDropAsync { name } => pkg::drop_async(&name, rebuild_lock).await,
                Request::PkgSync => pkg::sync(rebuild_lock).await,
                Request::PkgList => pkg::list().await,
                Request::PkgPresent { name } => pkg::present(&name).await,
                Request::PkgResolve { name } => pkg::resolve_name(&name).await,
                Request::ConfigGet => config::get().await,
                Request::ServiceList => service::list().await,
                Request::FontList => font::list().await,
                Request::PkgAdd { .. }
                | Request::PkgAddMany { .. }
                | Request::PkgRemove { .. }
                | Request::PkgRemoveMany { .. }
                | Request::ConfigApply { .. }
                | Request::ConfigCheck
                | Request::ServiceEnable { .. }
                | Request::ServiceDisable { .. }
                | Request::FontAdd { .. }
                | Request::FontRemove { .. } => unreachable!(),
            }
        }
    };

    let _ = forwarder.await;
    let _ = frame_tx.send(Frame::Done(resp));
}
