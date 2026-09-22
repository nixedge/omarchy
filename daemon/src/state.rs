use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

const STATE_PATH: &str = "/var/lib/omarchy/state.json";
const BACKUP_PATH: &str = "/var/lib/omarchy/state.json.bak";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

    /// Write state and snapshot a backup for rollback.
    pub async fn save(&self) -> Result<()> {
        let data = serde_json::to_vec_pretty(self)?;
        // Write to a temp file then rename for atomicity.
        let tmp = format!("{STATE_PATH}.tmp");
        tokio::fs::write(&tmp, &data).await?;
        tokio::fs::rename(&tmp, STATE_PATH).await?;
        Ok(())
    }

    /// Copy current state.json → state.json.bak before a risky operation.
    pub async fn save_backup() -> Result<()> {
        if Path::new(STATE_PATH).exists() {
            tokio::fs::copy(STATE_PATH, BACKUP_PATH).await?;
        }
        Ok(())
    }

    /// Restore state.json from state.json.bak after a failed rebuild.
    pub async fn restore_backup() -> Result<()> {
        if Path::new(BACKUP_PATH).exists() {
            tokio::fs::copy(BACKUP_PATH, STATE_PATH).await?;
        }
        Ok(())
    }
}
