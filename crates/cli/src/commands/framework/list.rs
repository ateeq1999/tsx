use serde::Serialize;
use std::time::Instant;

use crate::framework::package_cache::PackageCache;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::get_frameworks_dir;

#[derive(Serialize)]
struct FrameworkEntry {
    id: String,
    name: String,
    version: String,
    starters: Vec<String>,
    source: Option<String>,
    installed_at: Option<u64>,
    path: String,
}

pub fn framework_list(verbose: bool) -> CommandResult {
    let start = Instant::now();
    let frameworks_dir = get_frameworks_dir();
    let cache = PackageCache::load();
    let mut entries: Vec<FrameworkEntry> = vec![];

    if frameworks_dir.exists() {
        if let Ok(dir_entries) = std::fs::read_dir(&frameworks_dir) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let manifest_path = path.join("manifest.json");
                if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(m) = serde_json::from_str::<serde_json::Value>(&content) {
                        let id = m.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let name = m.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
                        let version = m.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();
                        let starters = m
                            .get("starters")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|s| s.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_default();
                        let cached = cache.get(&id);
                        entries.push(FrameworkEntry {
                            id,
                            name,
                            version,
                            starters,
                            source: cached.map(|c| c.source.clone()),
                            installed_at: cached.map(|c| c.installed_at),
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let response = ResponseEnvelope::success(
        "framework:list",
        serde_json::to_value(&entries).unwrap(),
        duration_ms,
    );

    if verbose {
        let context = crate::json::response::Context {
            project_root: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        response.with_context(context).print();
    } else {
        response.print();
    }

    CommandResult::ok("framework:list", vec![])
}
