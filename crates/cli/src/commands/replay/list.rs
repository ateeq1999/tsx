use std::path::PathBuf;

use crate::json::response::ResponseEnvelope;

use super::types::ReplaySession;

/// List session files stored in `.tsx/sessions/`.
pub fn replay_list(_verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let cwd = match crate::utils::paths::find_project_root() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let sessions_dir = cwd.join(".tsx/sessions");
    if !sessions_dir.exists() {
        let empty: Vec<serde_json::Value> = Vec::new();
        let result = serde_json::json!({ "sessions": empty });
        return ResponseEnvelope::success("replay list", result, start.elapsed().as_millis() as u64);
    }

    let mut sessions: Vec<serde_json::Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            // Quick parse for metadata
            let meta = std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<ReplaySession>(&s).ok())
                .map(|s| serde_json::json!({
                    "file": path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                    "framework": s.framework,
                    "recorded_at": s.recorded_at,
                    "steps": s.steps.len(),
                }))
                .unwrap_or_else(|| serde_json::json!({
                    "file": path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                    "framework": "unknown",
                    "recorded_at": "unknown",
                    "steps": 0,
                }));
            sessions.push(meta);
        }
    }

    sessions.sort_by(|a, b| {
        a["file"].as_str().unwrap_or("").cmp(b["file"].as_str().unwrap_or(""))
    });

    let result = serde_json::json!({
        "count": sessions.len(),
        "sessions": sessions,
    });
    ResponseEnvelope::success("replay list", result, start.elapsed().as_millis() as u64)
}
