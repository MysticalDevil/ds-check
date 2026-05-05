use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_TTL_SECS: u64 = 60;

#[derive(Serialize, Deserialize)]
struct CacheEntry<T> {
    timestamp: u64,
    data: T,
}

pub fn base_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join("ds-check"))
}

fn cache_dir() -> Option<PathBuf> {
    base_dir().map(|p| p.join("api_cache"))
}

fn sha256(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(input.as_bytes());
    let mut hex = String::with_capacity(64);
    for b in hash {
        hex.push_str(&format!("{:02x}", b));
    }
    hex
}

pub fn cache_path(token: &str, path: &str) -> Option<PathBuf> {
    let key = format!("{}:{}", token, path);
    let hash = sha256(&key);
    cache_dir().map(|p| p.join(format!("{}.json", &hash[..16])))
}

pub fn read_cache<T>(path: &Path) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let data = std::fs::read_to_string(path).ok()?;
    let entry: CacheEntry<T> = serde_json::from_str(&data).ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    if now.saturating_sub(entry.timestamp) > CACHE_TTL_SECS {
        return None;
    }
    Some(entry.data)
}

pub fn write_cache<T>(path: &Path, data: &T) -> anyhow::Result<()>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let entry = CacheEntry {
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        data,
    };
    let json = serde_json::to_string_pretty(&entry)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_cache_dir(f: impl FnOnce(PathBuf)) {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp = std::env::temp_dir().join(format!("ds-check-cache-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();
        unsafe { std::env::set_var("XDG_CACHE_HOME", &temp) };
        f(temp.clone());
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_roundtrip() {
        with_temp_cache_dir(|_| {
            let dir = cache_dir().unwrap();
            let path = dir.join("test.json");
            let data = serde_json::json!({"key": "value", "num": 42});

            write_cache(&path, &data).unwrap();
            let restored: serde_json::Value = read_cache(&path).unwrap();
            assert_eq!(restored, data);
        });
    }

    #[test]
    fn test_read_missing_file() {
        with_temp_cache_dir(|_| {
            let dir = cache_dir().unwrap();
            let path = dir.join("nonexistent.json");
            let result: Option<serde_json::Value> = read_cache(&path);
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_read_corrupted_file() {
        with_temp_cache_dir(|_| {
            let dir = cache_dir().unwrap();
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("corrupt.json");
            std::fs::write(&path, "not valid json").unwrap();
            let result: Option<serde_json::Value> = read_cache(&path);
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_cache_path_hash_is_stable() {
        let path1 = cache_path("token-a", "/api/v0/test");
        let path2 = cache_path("token-a", "/api/v0/test");
        assert_eq!(path1, path2);

        // Different tokens produce different paths
        let path3 = cache_path("token-b", "/api/v0/test");
        assert_ne!(path1, path3);
    }

    #[test]
    fn test_base_dir_returns_expected() {
        with_temp_cache_dir(|temp| {
            let base = base_dir().unwrap();
            assert_eq!(base, temp.join("ds-check"));
        });
    }
}
