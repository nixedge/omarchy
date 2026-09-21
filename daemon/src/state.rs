use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

const STATE_PATH: &str = "/var/lib/omarchy/state.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub packages: Vec<String>,
}

impl State {
    pub async fn load() -> Result<Self> {
        let path = Path::new(STATE_PATH);
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = tokio::fs::read(path).await?;
        Ok(serde_json::from_slice(&data)?)
    }

    pub async fn save(&self) -> Result<()> {
        let data = serde_json::to_vec_pretty(self)?;
        tokio::fs::write(STATE_PATH, data).await?;
        Ok(())
    }
}
