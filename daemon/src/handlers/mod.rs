pub mod pkg;
pub mod system;

use crate::protocol::{Request, Response};

pub async fn dispatch(req: Request) -> Response {
    match req {
        Request::Ping => system::ping().await,
        Request::Status => system::status().await,
        Request::PkgAdd { name } => pkg::add(&name).await,
        Request::PkgRemove { name } => pkg::remove(&name).await,
        Request::PkgList => pkg::list().await,
        Request::PkgPresent { name } => pkg::present(&name).await,
    }
}
