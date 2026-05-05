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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn sample_config() -> AuthConfig {
        AuthConfig {
            token: "test-token-123".into(),
            nickname: "TestUser".into(),
            email: "test@example.com".into(),
            currency: "CNY".into(),
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
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = std::env::temp_dir().join(format!("ds-check-test-{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        unsafe { std::env::set_var("XDG_CONFIG_HOME", &temp_dir) };

        // Clean up any existing file
        let _ = std::fs::remove_dir_all(&temp_dir.join("ds-check"));

        let config = sample_config();
        save(&config).unwrap();

        let loaded = load().unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.token, config.token);
        assert_eq!(loaded.nickname, config.nickname);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_missing_file() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir =
            std::env::temp_dir().join(format!("ds-check-test-missing-{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        unsafe { std::env::set_var("XDG_CONFIG_HOME", &temp_dir) };

        let loaded = load().unwrap();
        assert!(loaded.is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_corrupted_json() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir =
            std::env::temp_dir().join(format!("ds-check-test-bad-{}", std::process::id()));
        let config_dir = temp_dir.join("ds-check");
        std::fs::create_dir_all(&config_dir).unwrap();
        unsafe { std::env::set_var("XDG_CONFIG_HOME", &temp_dir) };

        std::fs::write(config_dir.join("auth.json"), "not valid json").unwrap();

        let result = load();
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
