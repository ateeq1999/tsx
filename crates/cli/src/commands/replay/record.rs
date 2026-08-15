use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::types::{RecordingLock, ReplaySession, LOCK_FILE};
use super::utils::{current_timestamp_iso, current_timestamp_slug, detect_framework, load_history_steps};

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
