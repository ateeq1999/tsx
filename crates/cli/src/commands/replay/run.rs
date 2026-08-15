use std::path::PathBuf;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

use super::types::ReplaySession;
use super::utils::execute_step;

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
