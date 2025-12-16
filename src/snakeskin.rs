use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use crate::daemon::{ModuleError, DaemonConfig};
use snakegg::native::dirs;
use snakegg::native::style::{dim, green};

#[derive(Debug, Serialize, Deserialize)]
pub struct SnakeskinState {
    pub timestamp: u64,
    pub daemon_id: String,
    pub active_errors: Vec<ModuleError>,
    pub config: DaemonConfig,
    pub installed_packages: Vec<String>,
}

#[derive(Debug)]
pub struct Snakeskin {
    path: PathBuf,
}

impl Snakeskin {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?
            .join("snakepit");
            
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        Ok(Self {
            path: data_dir.join("snakeskin.json"),
        })
    }

    pub async fn shed(&self, state: &SnakeskinState) -> Result<()> {
        let json = serde_json::to_string_pretty(state)?;
        fs::write(&self.path, json).await?;
        println!("{}", dim("🐍 Snakeskin shed (state saved)"));
        Ok(())
    }

    pub async fn regrow(&self) -> Result<Option<SnakeskinState>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&self.path).await?;
        let state: SnakeskinState = serde_json::from_str(&json)?;
        println!("{}", green("🐍 Snakeskin regrown (state restored)"));
        Ok(Some(state))
    }
}
