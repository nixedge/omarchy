use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "kebab-case")]
pub enum Request {
    Ping,
    PkgAdd { name: String },
    PkgRemove { name: String },
    PkgList,
    PkgPresent { name: String },
    Status,
}

#[derive(Serialize)]
pub struct Response {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn ok(data: impl Into<Option<serde_json::Value>>) -> Self {
        Self {
            ok: true,
            data: data.into(),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// Newline-delimited frames written to the socket for long-running operations.
/// The connection stays open until a `Done` frame is sent.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Frame {
    /// One line of output from nixos-rebuild stderr.
    Progress { line: String },
    /// Final result — client should check `ok` and exit accordingly.
    Done(Response),
}
