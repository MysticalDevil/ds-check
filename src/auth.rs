use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token: String,
    pub nickname: String,
    pub email: String,
    pub currency: String,
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("ds-check").join("auth.json"))
}

pub fn load() -> anyhow::Result<Option<AuthConfig>> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(None),
    };
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return Ok(None),
    };
    match serde_json::from_str(&data) {
        Ok(config) => Ok(Some(config)),
        Err(e) => Err(anyhow::anyhow!("Failed to parse auth config: {}", e)),
    }
}

pub fn save(config: &AuthConfig) -> anyhow::Result<()> {
    let path = config_path().ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, data)?;
    Ok(())
}

pub fn config_path_str() -> String {
    config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
