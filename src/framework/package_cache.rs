use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct PackageEntry {
    pub version: String,
    /// "local", "npm", "github"
    pub source: String,
    /// Unix timestamp (seconds since epoch)
    pub installed_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PackageCache {
    pub packages: HashMap<String, PackageEntry>,
}

/// Path of the package cache file: `<frameworks_dir>/packages.json`
fn cache_path() -> PathBuf {
    crate::utils::paths::get_frameworks_dir().join("packages.json")
}

fn now_unix() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl PackageCache {
    /// Load the cache from disk, or return an empty cache if not present.
    pub fn load() -> Self {
        let path = cache_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Persist the cache to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = cache_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Record (or update) an installed package.
    pub fn record(&mut self, id: &str, version: &str, source: &str) {
        self.packages.insert(
            id.to_string(),
            PackageEntry {
                version: version.to_string(),
                source: source.to_string(),
                installed_at: now_unix(),
            },
        );
    }

    pub fn get(&self, id: &str) -> Option<&PackageEntry> {
        self.packages.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut cache = PackageCache::default();
        cache.record("tanstack-start", "1.0.0", "local");
        assert_eq!(cache.get("tanstack-start").unwrap().source, "local");
        assert_eq!(cache.get("tanstack-start").unwrap().version, "1.0.0");
        assert!(cache.get("missing").is_none());
    }

    #[test]
    fn serializes_to_json() {
        let mut cache = PackageCache::default();
        cache.record("my-fw", "0.2.0", "npm");
        let json = serde_json::to_string(&cache).unwrap();
        assert!(json.contains("my-fw"));
        assert!(json.contains("0.2.0"));
        assert!(json.contains("npm"));
    }
}
