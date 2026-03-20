//! `tsx replay` — record and replay generation sessions (H4).
//!
//! Subcommands:
//! - `tsx replay record --out <file>` — start recording a session
//! - `tsx replay record --stop`       — stop recording and write the session file
//! - `tsx replay run <file>`           — replay a recorded session
//! - `tsx replay list`                 — list recorded session files in .tsx/sessions/

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

// ---------------------------------------------------------------------------
// Session format
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySession {
    pub tsx_version: String,
    pub framework: String,
    pub recorded_at: String,
    pub steps: Vec<ReplayStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayStep {
    pub command: String,
    pub args: serde_json::Value,
    pub outputs: Vec<String>,
}

// ---------------------------------------------------------------------------
// Lock file written while recording
// ---------------------------------------------------------------------------

const LOCK_FILE: &str = ".tsx/replay-recording.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordingLock {
    pub out: String,
    pub started_at: String,
}

// ---------------------------------------------------------------------------
// Public entrypoints
// ---------------------------------------------------------------------------

/// Start recording a session.
pub fn replay_record_start(out: Option<String>, verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let cwd = match crate::utils::paths::find_project_root() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let lock_path = cwd.join(LOCK_FILE);
    if lock_path.exists() {
        return ResponseEnvelope::error(
            "replay record",
            ErrorResponse::new(
                ErrorCode::FileExists,
                "A recording is already in progress. Run `tsx replay record --stop` to finish it.",
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    // Determine output path
    let out_file = out.unwrap_or_else(|| {
        format!(
            ".tsx/sessions/session-{}.json",
            current_timestamp_slug()
        )
    });

    // Ensure sessions directory exists
    if let Some(parent) = PathBuf::from(&out_file).parent() {
        let parent_abs = if parent.is_relative() { cwd.join(parent) } else { parent.to_path_buf() };
        let _ = std::fs::create_dir_all(&parent_abs);
    }

    // Write lock
    let lock = RecordingLock {
        out: out_file.clone(),
        started_at: current_timestamp_iso(),
    };
    let lock_str = serde_json::to_string_pretty(&lock).unwrap_or_default();
    if let Some(parent) = lock_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&lock_path, &lock_str) {
        return ResponseEnvelope::error(
            "replay record",
            ErrorResponse::new(
                ErrorCode::InternalError,
                format!("Could not write recording lock: {}", e),
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    if verbose {
        eprintln!("Recording started → {}", out_file);
    }

    let result = serde_json::json!({
        "status": "recording",
        "out": out_file,
    });
    let mut env = ResponseEnvelope::success("replay record", result, start.elapsed().as_millis() as u64);
    env.next_steps = vec![
        "Run your tsx generate/add commands now.".to_string(),
        format!("When done, run: tsx replay record --stop"),
    ];
    env
}

/// Stop recording and write the session file.
pub fn replay_record_stop(verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let cwd = match crate::utils::paths::find_project_root() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let lock_path = cwd.join(LOCK_FILE);
    if !lock_path.exists() {
        return ResponseEnvelope::error(
            "replay record --stop",
            ErrorResponse::new(
                ErrorCode::TemplateNotFound,
                "No recording in progress. Start one with `tsx replay record --out <file>`.",
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    let lock_str = std::fs::read_to_string(&lock_path).unwrap_or_default();
    let lock: RecordingLock = match serde_json::from_str(&lock_str) {
        Ok(l) => l,
        Err(_) => {
            let _ = std::fs::remove_file(&lock_path);
            return ResponseEnvelope::error(
                "replay record --stop",
                ErrorResponse::new(ErrorCode::InternalError, "Recording lock file is corrupt."),
                start.elapsed().as_millis() as u64,
            );
        }
    };

    // Determine framework from package.json if possible
    let framework = detect_framework(&cwd);

    // Build session from the command history log (if exists)
    let history_path = cwd.join(".tsx/replay-history.jsonl");
    let steps = load_history_steps(&history_path);

    let session = ReplaySession {
        tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        framework,
        recorded_at: lock.started_at.clone(),
        steps,
    };

    let session_str = match serde_json::to_string_pretty(&session) {
        Ok(s) => s,
        Err(e) => {
            return ResponseEnvelope::error(
                "replay record --stop",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Could not serialize session: {}", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    };

    let out_path = if PathBuf::from(&lock.out).is_relative() {
        cwd.join(&lock.out)
    } else {
        PathBuf::from(&lock.out)
    };

    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&out_path, &session_str) {
        return ResponseEnvelope::error(
            "replay record --stop",
            ErrorResponse::new(
                ErrorCode::InternalError,
                format!("Could not write session file {}: {}", out_path.display(), e),
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    // Clean up lock + history
    let _ = std::fs::remove_file(&lock_path);
    let _ = std::fs::remove_file(&history_path);

    if verbose {
        eprintln!("Session saved → {}", out_path.display());
    }

    let result = serde_json::json!({
        "out": lock.out,
        "steps": session.steps.len(),
        "framework": session.framework,
    });
    let mut env = ResponseEnvelope::success("replay record --stop", result, start.elapsed().as_millis() as u64);
    env.next_steps = vec![
        format!("Replay with: tsx replay run {}", lock.out),
        format!("Dry-run first: tsx replay run {} --dry-run", lock.out),
    ];
    env
}

/// Replay a session file.
pub fn replay_run(file: String, dry_run: bool, verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let cwd = match crate::utils::paths::find_project_root() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let session_path = if PathBuf::from(&file).is_relative() {
        cwd.join(&file)
    } else {
        PathBuf::from(&file)
    };

    if !session_path.exists() {
        return ResponseEnvelope::error(
            "replay run",
            ErrorResponse::new(
                ErrorCode::TemplateNotFound,
                format!("Session file not found: {}", session_path.display()),
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    let session_str = match std::fs::read_to_string(&session_path) {
        Ok(s) => s,
        Err(e) => {
            return ResponseEnvelope::error(
                "replay run",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("Could not read session file: {}", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    };

    let session: ReplaySession = match serde_json::from_str(&session_str) {
        Ok(s) => s,
        Err(e) => {
            return ResponseEnvelope::error(
                "replay run",
                ErrorResponse::new(
                    ErrorCode::InvalidPayload,
                    format!("Invalid session file format: {}", e),
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    };

    let mut replayed: Vec<serde_json::Value> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for step in &session.steps {
        if verbose {
            eprintln!(
                "[replay] {} {}",
                if dry_run { "(dry-run)" } else { "" },
                step.command
            );
        }

        let result = execute_step(step, dry_run, &cwd);
        match result {
            Ok(files) => replayed.push(serde_json::json!({
                "command": step.command,
                "status": "ok",
                "files": files,
            })),
            Err(e) => {
                errors.push(format!("{}: {}", step.command, e));
                replayed.push(serde_json::json!({
                    "command": step.command,
                    "status": "error",
                    "error": e,
                }));
            }
        }
    }

    let result = serde_json::json!({
        "dry_run": dry_run,
        "session": file,
        "framework": session.framework,
        "steps_total": session.steps.len(),
        "steps_ok": replayed.iter().filter(|s| s["status"] == "ok").count(),
        "steps_failed": errors.len(),
        "steps": replayed,
    });

    let mut env = ResponseEnvelope::success("replay run", result, start.elapsed().as_millis() as u64);
    if !errors.is_empty() {
        env.metadata.warnings = errors;
    }
    env.dry_run = Some(dry_run);
    env
}

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

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn execute_step(
    step: &ReplayStep,
    dry_run: bool,
    _root: &Path,
) -> Result<Vec<String>, String> {
    use crate::commands::ops::batch;

    let result = batch::execute_command_pub(
        &step.command,
        &step.args,
        false, // overwrite: false by default on replay
        dry_run,
    );
    result.map_err(|(_, msg)| msg)
}

fn load_history_steps(history_path: &Path) -> Vec<ReplayStep> {
    if !history_path.exists() {
        return Vec::new();
    }
    let content = std::fs::read_to_string(history_path).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<ReplayStep>(line).ok())
        .collect()
}

fn detect_framework(root: &Path) -> String {
    let pkg = root.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&pkg) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                if deps.contains_key("@tanstack/start") || deps.contains_key("@tanstack/react-start") {
                    return "tanstack-start".to_string();
                }
                if deps.contains_key("next") {
                    return "next".to_string();
                }
                if deps.contains_key("remix") || deps.contains_key("@remix-run/react") {
                    return "remix".to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

fn current_timestamp_slug() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // YYYYMMDD-HHMMSS approximation
    let days = secs / 86400;
    let time = secs % 86400;
    let h = time / 3600;
    let m = (time % 3600) / 60;
    let s = time % 60;
    format!("{}-{:02}{:02}{:02}", days, h, m, s)
}

fn current_timestamp_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;
    // Approximate: count from 1970-01-01
    format!("1970-01-{:02}T{:02}:{:02}:{:02}Z", (days_since_epoch % 365) + 1, h, m, s)
}
