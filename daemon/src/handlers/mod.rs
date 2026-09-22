pub mod pkg;
pub mod system;

use crate::protocol::{Request, Response};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn dispatch(req: Request, rebuild_lock: Arc<Mutex<()>>) -> Response {
    match req {
        Request::Ping => system::ping().await,
        Request::Status => system::status().await,
        Request::PkgAdd { name } => pkg::add(&name, rebuild_lock).await,
        Request::PkgRemove { name } => pkg::remove(&name, rebuild_lock).await,
        Request::PkgList => pkg::list().await,
        Request::PkgPresent { name } => pkg::present(&name).await,
    }
}
