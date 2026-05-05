use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token: String,
    pub nickname: String,
    pub email: String,
    pub currency: String,
    #[serde(default)]
    pub api_key: Option<String>,
}

static CONFIG_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

fn config_path() -> Option<PathBuf> {
    if let Ok(guard) = CONFIG_OVERRIDE.lock()
        && let Some(ref dir) = *guard
    {
        return Some(dir.join("ds-check").join("auth.json"));
    }
    dirs::config_dir().map(|p| p.join("ds-check").join("auth.json"))
}

pub fn load() -> anyhow::Result<Option<AuthConfig>> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(None),
    };
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(anyhow::anyhow!("Failed to read {}: {}", path.display(), e)),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn with_config_override(dir: &std::path::Path, f: impl FnOnce()) {
        *CONFIG_OVERRIDE.lock().unwrap() = Some(dir.to_path_buf());
        f();
        *CONFIG_OVERRIDE.lock().unwrap() = None;
    }

    fn sample_config() -> AuthConfig {
        AuthConfig {
            token: "test-token-123".into(),
            nickname: "TestUser".into(),
            email: "test@example.com".into(),
            currency: "CNY".into(),
            api_key: None,
        }
    }

    #[test]
    fn test_json_roundtrip() {
        let config = sample_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let restored: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.token, config.token);
        assert_eq!(restored.nickname, config.nickname);
        assert_eq!(restored.email, config.email);
        assert_eq!(restored.currency, config.currency);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        with_config_override(temp_dir.path(), || {
            let config = sample_config();
            save(&config).unwrap();

            let loaded = load().unwrap();
            assert!(loaded.is_some());
            let loaded = loaded.unwrap();
            assert_eq!(loaded.token, config.token);
            assert_eq!(loaded.nickname, config.nickname);
            assert_eq!(loaded.api_key, config.api_key);
        });
    }

    #[test]
    fn test_load_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        with_config_override(temp_dir.path(), || {
            let loaded = load().unwrap();
            assert!(loaded.is_none());
        });
    }

    #[test]
    fn test_load_corrupted_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_dir = temp_dir.path().join("ds-check");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("auth.json"), "not valid json").unwrap();

        with_config_override(temp_dir.path(), || {
            let result = load();
            assert!(result.is_err());
        });
    }
}
