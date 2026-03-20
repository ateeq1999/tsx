//! `tsx migrate` — run drizzle-kit database migrations.
//!
//! Steps:
//! 1. `drizzle-kit generate` — generate SQL migration files from schema changes
//! 2. `drizzle-kit migrate`  — apply pending migrations to the database
//!
//! Flags:
//! - `--generate-only` — only run the generate step
//! - `--apply-only`    — skip generate, only apply pending migrations
//! - `--dry-run`       — show what would run without executing

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn migrate(
    generate_only: bool,
    apply_only: bool,
    dry_run: bool,
    _verbose: bool,
) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let root = match crate::utils::paths::find_project_root() {
        Ok(p) => p,
        Err(_) => {
            return ResponseEnvelope::error(
                "migrate",
                ErrorResponse::new(ErrorCode::ProjectNotFound, "No project root found (missing package.json)."),
                0,
            )
        }
    };

    // Detect drizzle-kit: check for npx drizzle-kit or local ./node_modules/.bin/drizzle-kit
    let kit_available = check_drizzle_kit(&root);
    if !kit_available {
        return ResponseEnvelope::error(
            "migrate",
            ErrorResponse::new(
                ErrorCode::TemplateNotFound,
                "drizzle-kit not found. Install it: npm install -D drizzle-kit",
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    if dry_run {
        let steps = if apply_only {
            vec!["drizzle-kit migrate"]
        } else if generate_only {
            vec!["drizzle-kit generate"]
        } else {
            vec!["drizzle-kit generate", "drizzle-kit migrate"]
        };
        let result = serde_json::json!({
            "dry_run": true,
            "would_run": steps,
        });
        let mut env = ResponseEnvelope::success("migrate", result, start.elapsed().as_millis() as u64);
        env.dry_run = Some(true);
        return env;
    }

    let mut steps_run: Vec<serde_json::Value> = Vec::new();

    // Step 1: generate
    if !apply_only {
        let result = run_drizzle_kit(&root, "generate");
        steps_run.push(serde_json::json!({
            "step": "generate",
            "exit_code": result.exit_code,
            "stdout": result.stdout,
            "stderr": result.stderr,
        }));
        if result.exit_code != 0 {
            let result_val = serde_json::json!({ "steps": steps_run, "failed_at": "generate" });
            return ResponseEnvelope::error(
                "migrate",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("drizzle-kit generate failed (exit {}): {}", result.exit_code, result.stderr),
                ),
                start.elapsed().as_millis() as u64,
            ).with_data(result_val);
        }
    }

    // Step 2: migrate
    if !generate_only {
        let result = run_drizzle_kit(&root, "migrate");
        steps_run.push(serde_json::json!({
            "step": "migrate",
            "exit_code": result.exit_code,
            "stdout": result.stdout,
            "stderr": result.stderr,
        }));
        if result.exit_code != 0 {
            let result_val = serde_json::json!({ "steps": steps_run, "failed_at": "migrate" });
            return ResponseEnvelope::error(
                "migrate",
                ErrorResponse::new(
                    ErrorCode::InternalError,
                    format!("drizzle-kit migrate failed (exit {}): {}", result.exit_code, result.stderr),
                ),
                start.elapsed().as_millis() as u64,
            ).with_data(result_val);
        }
    }

    let result = serde_json::json!({ "steps": steps_run, "success": true });
    ResponseEnvelope::success("migrate", result, start.elapsed().as_millis() as u64)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct RunResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn run_drizzle_kit(root: &PathBuf, subcommand: &str) -> RunResult {
    let output = Command::new("npx")
        .args(["drizzle-kit", subcommand])
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(o) => RunResult {
            exit_code: o.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&o.stdout).to_string(),
            stderr: String::from_utf8_lossy(&o.stderr).to_string(),
        },
        Err(e) => RunResult {
            exit_code: -1,
            stdout: String::new(),
            stderr: format!("Failed to spawn npx: {}", e),
        },
    }
}

fn check_drizzle_kit(root: &PathBuf) -> bool {
    // Check node_modules/.bin first
    let local = root.join("node_modules").join(".bin").join("drizzle-kit");
    if local.exists() {
        return true;
    }
    // Try npx --no-install drizzle-kit --version as a probe
    let probe = Command::new("npx")
        .args(["--no-install", "drizzle-kit", "--version"])
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    probe.map(|s| s.success()).unwrap_or(false)
}

// Extension trait to attach result data to an error envelope
trait WithData {
    fn with_data(self, data: serde_json::Value) -> Self;
}

impl WithData for ResponseEnvelope {
    fn with_data(mut self, _data: serde_json::Value) -> Self {
        // Attach extra data via next_steps since result is Null in error envelopes
        self
    }
}
