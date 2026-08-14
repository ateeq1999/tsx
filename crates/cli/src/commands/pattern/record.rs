use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::types::{PatternArg, PatternDefinition, PatternOutput};
use super::utils::chrono_now;

// ---------------------------------------------------------------------------
// Record session state — stored at `.tsx/patterns/.record`
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct RecordSession {
    name: String,
    started_at: String,
    /// Snapshot of files at record start: path → content-hash
    baseline: HashMap<String, String>,
}

pub fn pattern_record_start(name: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let session_file = cwd.join(".tsx").join("patterns").join(".record");

    if session_file.exists() {
        return ResponseEnvelope::error(
            "pattern record",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                "A recording session is already active. Run `tsx pattern record --stop` first.",
            ),
            0,
        );
    }

    // Snapshot the current working directory (top-level files only for speed)
    let baseline = snapshot_dir(&cwd);
    let session = RecordSession {
        name: name.clone(),
        started_at: chrono_now(),
        baseline,
    };

    if let Some(parent) = session_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match std::fs::write(&session_file, serde_json::to_string_pretty(&session).unwrap_or_default()) {
        Ok(_) => ResponseEnvelope::success(
            "pattern record",
            serde_json::json!({
                "status": "recording",
                "name": name,
                "message": "Recording started. Create or edit files, then run `tsx pattern record --stop`.",
            }),
            0,
        ),
        Err(e) => ResponseEnvelope::error(
            "pattern record",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

pub fn pattern_record_stop(_verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let session_file = cwd.join(".tsx").join("patterns").join(".record");

    let session_content = match std::fs::read_to_string(&session_file) {
        Ok(s) => s,
        Err(_) => {
            return ResponseEnvelope::error(
                "pattern record",
                ErrorResponse::new(
                    ErrorCode::ProjectNotFound,
                    "No active recording session. Run `tsx pattern record --name <name>` first.",
                ),
                0,
            )
        }
    };

    let session: RecordSession = match serde_json::from_str(&session_content) {
        Ok(s) => s,
        Err(e) => {
            return ResponseEnvelope::error(
                "pattern record",
                ErrorResponse::new(ErrorCode::InternalError, format!("Corrupt session file: {}", e)),
                0,
            )
        }
    };

    // Diff the current state against the baseline
    let current = snapshot_dir(&cwd);
    let mut new_files: Vec<String> = Vec::new();
    let mut modified_files: Vec<String> = Vec::new();

    for (path, hash) in &current {
        if let Some(old_hash) = session.baseline.get(path) {
            if old_hash != hash {
                modified_files.push(path.clone());
            }
        } else {
            new_files.push(path.clone());
        }
    }

    let _ = std::fs::remove_file(&session_file);

    // If new files were created, create a pattern from the first one
    let all_changed: Vec<String> = new_files.iter().chain(modified_files.iter()).cloned().collect();

    if all_changed.is_empty() {
        return ResponseEnvelope::success(
            "pattern record",
            serde_json::json!({
                "status": "stopped",
                "name": session.name,
                "changed_files": 0,
                "message": "No file changes detected. Pattern not created.",
            }),
            0,
        );
    }

    // Create a pattern definition from the recorded changes
    let pattern = PatternDefinition {
        id: session.name.clone(),
        description: format!("Recorded pattern: {}", session.name),
        args: vec![PatternArg {
            name: "name".to_string(),
            arg_type: "string".to_string(),
            description: Some("Feature name".to_string()),
        }],
        outputs: all_changed
            .iter()
            .map(|f| PatternOutput {
                path: templatize_path(f),
                template: PathBuf::from(f)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("template.forge")
                    .to_string()
                    + ".forge",
            })
            .collect(),
        slots: Vec::new(),
        post_hooks: Vec::new(),
        version: "1.0.0".to_string(),
    };

    // Copy changed files into pattern directory as template stubs
    let pattern_dir = PatternDefinition::dir(&cwd, &session.name);
    let _ = std::fs::create_dir_all(&pattern_dir);
    for file in &all_changed {
        let src = cwd.join(file);
        if src.exists() {
            let dest_name = format!("{}.forge", src.file_name().and_then(|n| n.to_str()).unwrap_or("template"));
            let _ = std::fs::copy(&src, pattern_dir.join(&dest_name));
        }
    }

    match pattern.save(&cwd) {
        Ok(_) => ResponseEnvelope::success(
            "pattern record",
            serde_json::json!({
                "status": "captured",
                "name": session.name,
                "changed_files": all_changed.len(),
                "new_files": new_files,
                "modified_files": modified_files,
                "pattern": serde_json::to_value(&pattern).unwrap_or_default(),
            }),
            0,
        )
        .with_next_steps(vec![
            format!(
                "Edit templates in {}",
                pattern_dir.display()
            ),
            format!("Add {{{{name}}}} and other placeholders to the templates"),
            format!("Run with: tsx run {}", session.name),
        ]),
        Err(e) => ResponseEnvelope::error(
            "pattern record",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            0,
        ),
    }
}

/// Create a lightweight snapshot of a directory: relative path → simple content hash.
fn snapshot_dir(dir: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return map; };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Ok(rel) = path.strip_prefix(dir) {
                let key = rel.to_string_lossy().replace('\\', "/");
                // Simple hash: file size + first 64 bytes
                if let Ok(content) = std::fs::read(&path) {
                    let hash = format!("{}-{}", content.len(), &hex_first64(&content));
                    map.insert(key, hash);
                }
            }
        }
    }
    map
}

fn hex_first64(data: &[u8]) -> String {
    data.iter()
        .take(64)
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Templatize a file path: replace common name-like segments with {{name}}.
fn templatize_path(path: &str) -> String {
    // Simple heuristic: replace the filename stem with {{kebab(name)}}
    let p = PathBuf::from(path);
    if let Some(parent) = p.parent() {
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("ts");
        let parent_str = parent.to_string_lossy();
        if parent_str.is_empty() || parent_str == "." {
            return format!("{{{{kebab(name)}}}}.{}", ext);
        }
        return format!("{}/{{{{kebab(name)}}}}.{}", parent_str, ext);
    }
    path.to_string()
}
