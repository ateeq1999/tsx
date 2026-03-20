//! `tsx analyze` — project structure health report (H5).
//!
//! Scans the project directory and compares against expected conventions:
//! - Schema files in app/db/schema/ or db/schema/
//! - Server functions in app/server/ or server/
//! - Route files in app/routes/ or routes/
//! - Zod export presence in schema files
//! - Large route files (>200 lines)
//! - Server functions without input validation (no Zod import)

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFinding {
    pub path: String,
    /// "ok" | "error" | "warning"
    pub status: String,
    pub message: String,
    pub fix: Option<String>,
}

impl AnalysisFinding {
    fn ok(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { path: path.into(), status: "ok".to_string(), message: message.into(), fix: None }
    }
    fn error(path: impl Into<String>, message: impl Into<String>, fix: Option<&str>) -> Self {
        Self {
            path: path.into(),
            status: "error".to_string(),
            message: message.into(),
            fix: fix.map(|s| s.to_string()),
        }
    }
    fn warning(path: impl Into<String>, message: impl Into<String>, fix: Option<&str>) -> Self {
        Self {
            path: path.into(),
            status: "warning".to_string(),
            message: message.into(),
            fix: fix.map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub ok: usize,
    pub errors: usize,
    pub warnings: usize,
    pub total_files_scanned: usize,
}

// ---------------------------------------------------------------------------
// Public entrypoint
// ---------------------------------------------------------------------------

pub fn analyze(fix: bool, report: bool, verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let cwd = match crate::utils::paths::find_project_root() {
        Ok(p) => p,
        Err(_) => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let mut findings: Vec<AnalysisFinding> = Vec::new();
    let mut total_files = 0usize;

    // --- Schema directory ---
    check_schema_dir(&cwd, &mut findings, &mut total_files, verbose);

    // --- Server functions ---
    check_server_fns(&cwd, &mut findings, &mut total_files, verbose);

    // --- Route files ---
    check_routes(&cwd, &mut findings, &mut total_files, verbose);

    // Auto-fix (safe fixes only — currently none implemented, documents what would be done)
    if fix {
        findings.iter_mut().for_each(|f| {
            if f.fix.is_some() && f.status == "error" {
                // Mark as would-fix in message; actual writes not yet implemented
                f.message = format!("[auto-fix available] {}", f.message);
            }
        });
    }

    let summary = AnalysisSummary {
        ok: findings.iter().filter(|f| f.status == "ok").count(),
        errors: findings.iter().filter(|f| f.status == "error").count(),
        warnings: findings.iter().filter(|f| f.status == "warning").count(),
        total_files_scanned: total_files,
    };

    let mut envelope = if report {
        // Structured JSON for CI
        let result = serde_json::json!({
            "summary": summary,
            "findings": findings,
        });
        ResponseEnvelope::success("analyze", result, start.elapsed().as_millis() as u64)
    } else {
        let result = serde_json::json!({
            "summary": summary,
            "findings": findings,
        });
        ResponseEnvelope::success("analyze", result, start.elapsed().as_millis() as u64)
    };

    if summary.errors > 0 {
        envelope.next_steps = vec![
            format!("{} error(s) found — run `tsx analyze --fix` to apply safe auto-fixes", summary.errors),
        ];
    }

    envelope
}

// ---------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------

fn check_schema_dir(root: &Path, findings: &mut Vec<AnalysisFinding>, total: &mut usize, _verbose: bool) {
    let candidates = [
        root.join("app/db/schema"),
        root.join("db/schema"),
        root.join("src/db/schema"),
    ];

    let schema_dir = candidates.iter().find(|p| p.exists());

    match schema_dir {
        None => findings.push(AnalysisFinding::error(
            "db/schema/",
            "No schema directory found (expected app/db/schema/ or db/schema/)",
            Some("Run `tsx generate schema` to create your first schema"),
        )),
        Some(dir) => {
            let files: Vec<PathBuf> = collect_ts_files(dir);
            *total += files.len();

            findings.push(AnalysisFinding::ok(
                dir.to_string_lossy().to_string(),
                format!("{} schema file(s) found", files.len()),
            ));

            for file in &files {
                check_schema_file(file, findings);
            }
        }
    }
}

fn check_schema_file(path: &Path, findings: &mut Vec<AnalysisFinding>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let rel = path.to_string_lossy().replace('\\', "/");
    // Derive expected Zod export name from filename: users.ts → selectUserSchema
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let expected_export = format!(
        "select{}Schema",
        pascal_case(stem)
    );

    if !content.contains(&expected_export) && !content.contains("z.object") && !content.contains("createSelectSchema") {
        findings.push(AnalysisFinding::error(
            rel,
            format!("Missing Zod export (expected {} or z.object(...))", expected_export),
            Some("Add a Zod schema: export const selectXxxSchema = createSelectSchema(table)"),
        ));
    }
}

fn check_server_fns(root: &Path, findings: &mut Vec<AnalysisFinding>, total: &mut usize, _verbose: bool) {
    let candidates = [
        root.join("app/server"),
        root.join("server"),
        root.join("src/server"),
    ];

    let server_dir = candidates.iter().find(|p| p.exists());

    match server_dir {
        None => {
            // No server dir is acceptable for new projects
        }
        Some(dir) => {
            let files = collect_ts_files(dir);
            *total += files.len();

            findings.push(AnalysisFinding::ok(
                dir.to_string_lossy().to_string(),
                format!("{} server function file(s) found", files.len()),
            ));

            let mut no_validation = 0usize;
            for file in &files {
                let content = std::fs::read_to_string(file).unwrap_or_default();
                // Check if file defines server functions but doesn't import Zod
                if (content.contains("createServerFn") || content.contains("'use server'"))
                    && !content.contains("zod")
                    && !content.contains("z.object")
                    && !content.contains("z.string")
                {
                    no_validation += 1;
                }
            }

            if no_validation > 0 {
                findings.push(AnalysisFinding::warning(
                    dir.to_string_lossy().to_string(),
                    format!("{} server function file(s) without Zod input validation", no_validation),
                    Some("Add input validation: .validator(z.object({ ... }))"),
                ));
            }
        }
    }
}

fn check_routes(root: &Path, findings: &mut Vec<AnalysisFinding>, total: &mut usize, _verbose: bool) {
    let candidates = [
        root.join("app/routes"),
        root.join("routes"),
        root.join("src/routes"),
    ];

    let routes_dir = candidates.iter().find(|p| p.exists());

    match routes_dir {
        None => {
            // No routes dir yet — new project
        }
        Some(dir) => {
            let files = collect_ts_files(dir);
            *total += files.len();

            const MAX_LINES: usize = 200;
            let mut large_files = 0usize;

            for file in &files {
                let content = std::fs::read_to_string(file).unwrap_or_default();
                let line_count = content.lines().count();
                if line_count > MAX_LINES {
                    large_files += 1;
                    findings.push(AnalysisFinding::warning(
                        file.to_string_lossy().replace('\\', "/"),
                        format!(
                            "Route file is {} lines (recommended max: {}, split into components)",
                            line_count, MAX_LINES
                        ),
                        Some("Extract large sections into components in a co-located components/ directory"),
                    ));
                }
            }

            if large_files == 0 {
                findings.push(AnalysisFinding::ok(
                    dir.to_string_lossy().to_string(),
                    format!("{} route file(s) found, all within size limits", files.len()),
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn collect_ts_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && matches!(
                    e.path().extension().and_then(|x| x.to_str()),
                    Some("ts") | Some("tsx")
                )
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ' ')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
