use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "kebab-case")]
pub enum Request {
    Ping,
    PkgAdd { name: String },
    PkgRemove { name: String },
    PkgAddAsync { name: String },
    PkgDropAsync { name: String },
    PkgSync,
    PkgList,
    PkgPresent { name: String },
    PkgResolve { name: String },
    ConfigGet,
    ConfigApply { content: String },
    ConfigCheck,
    Status,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn ok(data: impl Into<Option<serde_json::Value>>) -> Self {
        Self { ok: true, pending: None, data: data.into(), error: None }
    }

    pub fn ok_pending() -> Self {
        Self { ok: true, pending: Some(true), data: None, error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { ok: false, pending: None, data: None, error: Some(msg.into()) }
    }

    pub fn err_msg(&self) -> &str {
        self.error.as_deref().unwrap_or("unknown error")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Frame {
    Progress { line: String },
    Done(Response),
}
