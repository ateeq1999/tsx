//! `tsx build` — detect and run the project's build command.
//!
//! Detection order:
//! 1. `scripts.build` in package.json
//! 2. `scripts["build:prod"]`
//! 3. Fallback: `npm run build`
//!
//! Flags:
//! - `--json-events` — emit structured JSON events to stdout (agent mode)

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn build(json_events: bool, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let root = match crate::utils::paths::find_project_root() {
        Ok(p) => p,
        Err(_) => {
            return ResponseEnvelope::error(
                "build",
                ErrorResponse::new(ErrorCode::ProjectNotFound, "No project root found (missing package.json)."),
                0,
            )
        }
    };

    let script = detect_build_script(&root);
    let script_name = match &script {
        Some(s) => s.clone(),
        None => {
            return ResponseEnvelope::error(
                "build",
                ErrorResponse::new(
                    ErrorCode::TemplateNotFound,
                    "No build script found in package.json. Add a 'build' script.",
                ),
                start.elapsed().as_millis() as u64,
            )
        }
    };

    if json_events {
        let event = serde_json::json!({ "event": "build:start", "script": script_name });
        println!("{}", event);
    }

    let output = Command::new("npm")
        .args(["run", &script_name])
        .current_dir(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(o) => {
            let exit_code = o.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let success = exit_code == 0;

            if json_events {
                let event = serde_json::json!({
                    "event": if success { "build:success" } else { "build:error" },
                    "exit_code": exit_code,
                });
                println!("{}", event);
            }

            let result = serde_json::json!({
                "script": script_name,
                "exit_code": exit_code,
                "stdout": stdout,
                "stderr": stderr,
            });

            if success {
                ResponseEnvelope::success("build", result, start.elapsed().as_millis() as u64)
            } else {
                ResponseEnvelope::error(
                    "build",
                    ErrorResponse::new(
                        ErrorCode::InternalError,
                        format!("Build failed (exit {}): {}", exit_code, stderr.lines().last().unwrap_or("")),
                    ),
                    start.elapsed().as_millis() as u64,
                )
            }
        }
        Err(e) => ResponseEnvelope::error(
            "build",
            ErrorResponse::new(
                ErrorCode::InternalError,
                format!("Failed to run npm: {}", e),
            ),
            start.elapsed().as_millis() as u64,
        ),
    }
}

fn detect_build_script(root: &PathBuf) -> Option<String> {
    let pkg_path = root.join("package.json");
    let content = std::fs::read_to_string(&pkg_path).ok()?;
    let pkg: serde_json::Value = serde_json::from_str(&content).ok()?;
    let scripts = pkg.get("scripts")?.as_object()?;

    for candidate in ["build", "build:prod", "build:production", "compile"] {
        if scripts.contains_key(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}
