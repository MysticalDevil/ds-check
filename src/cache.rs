use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_TTL_SECS: u64 = 60;

#[derive(Serialize, Deserialize)]
struct CacheEntry<T> {
    timestamp: u64,
    data: T,
}

fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join("ds-check").join("api_cache"))
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
