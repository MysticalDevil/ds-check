use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::provider::ProviderId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub token: String,
    pub nickname: String,
    pub email: String,
    pub currency: String,
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderAuth {
    #[serde(default)]
    pub platform_token: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStore {
    pub version: u32,
    #[serde(default)]
    pub providers: BTreeMap<String, ProviderAuth>,
}

impl Default for AuthStore {
    fn default() -> Self {
        Self {
            version: 1,
            providers: BTreeMap::new(),
        }
    }
}

static CONFIG_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

fn config_path() -> Option<PathBuf> {
    if let Ok(guard) = CONFIG_OVERRIDE.lock()
        && let Some(ref dir) = *guard
    {
        return Some(dir.join("metrix").join("auth.json"));
    }
    dirs::config_dir().map(|p| p.join("metrix").join("auth.json"))
}

#[cfg(test)]
pub fn load() -> anyhow::Result<Option<AuthConfig>> {
    load_legacy_deepseek()
}

pub fn load_provider(provider: ProviderId) -> anyhow::Result<Option<ProviderAuth>> {
    let store = match load_store()? {
        Some(store) => store,
        None => return Ok(None),
    };
    Ok(store.providers.get(provider.as_str()).cloned())
}

pub fn save_provider(provider: ProviderId, auth: ProviderAuth) -> anyhow::Result<()> {
    let mut store = load_store()?.unwrap_or_default();
    store.providers.insert(provider.as_str().to_string(), auth);
    save_store(&store)
}

fn load_store() -> anyhow::Result<Option<AuthStore>> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(None),
    };
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(anyhow::anyhow!("Failed to read {}: {}", path.display(), e)),
    };

    parse_store(&data).map(Some)
}

#[cfg(test)]
pub fn save(config: &AuthConfig) -> anyhow::Result<()> {
    let auth = ProviderAuth {
        platform_token: Some(config.token.clone()),
        api_key: config.api_key.clone(),
        nickname: Some(config.nickname.clone()),
        email: Some(config.email.clone()),
        currency: Some(config.currency.clone()),
    };
    save_provider(ProviderId::DeepSeek, auth)
}

fn save_store(store: &AuthStore) -> anyhow::Result<()> {
    let path = config_path().ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, data)?;
    Ok(())
}

pub fn config_path_str() -> String {
    config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn parse_store(data: &str) -> anyhow::Result<AuthStore> {
    if let Ok(store) = serde_json::from_str::<AuthStore>(data)
        && !store.providers.is_empty()
    {
        return Ok(store);
    }

    let legacy: AuthConfig = serde_json::from_str(data)
        .map_err(|e| anyhow::anyhow!("Failed to parse auth config: {}", e))?;
    let mut store = AuthStore::default();
    store.providers.insert(
        ProviderId::DeepSeek.as_str().to_string(),
        ProviderAuth {
            platform_token: Some(legacy.token),
            api_key: legacy.api_key,
            nickname: Some(legacy.nickname),
            email: Some(legacy.email),
            currency: Some(legacy.currency),
        },
    );
    Ok(store)
}

#[cfg(test)]
fn load_legacy_deepseek() -> anyhow::Result<Option<AuthConfig>> {
    let auth = match load_provider(ProviderId::DeepSeek)? {
        Some(auth) => auth,
        None => return Ok(None),
    };

    let token = match auth.platform_token {
        Some(token) => token,
        None => return Ok(None),
    };

    Ok(Some(AuthConfig {
        token,
        nickname: auth.nickname.unwrap_or_default(),
        email: auth.email.unwrap_or_default(),
        currency: auth.currency.unwrap_or_else(|| "CNY".to_string()),
        api_key: auth.api_key,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn with_config_override(dir: &std::path::Path, f: impl FnOnce()) {
        let _guard = TEST_LOCK.lock().unwrap();
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

    fn sample_provider_auth() -> ProviderAuth {
        ProviderAuth {
            platform_token: Some("platform-token".into()),
            api_key: Some("api-key".into()),
            nickname: Some("TestUser".into()),
            email: Some("test@example.com".into()),
            currency: Some("CNY".into()),
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
    fn test_multi_provider_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        with_config_override(temp_dir.path(), || {
            save_provider(ProviderId::Kimi, sample_provider_auth()).unwrap();

            let loaded = load_provider(ProviderId::Kimi).unwrap().unwrap();
            assert_eq!(loaded.api_key.as_deref(), Some("api-key"));
            assert_eq!(loaded.platform_token.as_deref(), Some("platform-token"));
            assert!(load_provider(ProviderId::BigModel).unwrap().is_none());
        });
    }

    #[test]
    fn test_parse_legacy_as_deepseek_provider() {
        let legacy = serde_json::to_string_pretty(&sample_config()).unwrap();
        let store = parse_store(&legacy).unwrap();
        let deepseek = store.providers.get("deepseek").unwrap();
        assert_eq!(deepseek.platform_token.as_deref(), Some("test-token-123"));
        assert_eq!(deepseek.nickname.as_deref(), Some("TestUser"));
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
        let config_dir = temp_dir.path().join("metrix");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("auth.json"), "not valid json").unwrap();

        with_config_override(temp_dir.path(), || {
            let result = load();
            assert!(result.is_err());
        });
    }
}
