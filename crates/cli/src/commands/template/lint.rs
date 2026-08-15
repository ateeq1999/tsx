use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_lint(path: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let target = match &path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let candidates = [
                cwd.join(".tsx").join("templates"),
                cwd.join("templates"),
            ];
            match candidates.into_iter().find(|p| p.exists()) {
                Some(p) => p,
                None => {
                    return ResponseEnvelope::error(
                        "template:lint",
                        ErrorResponse::new(
                            ErrorCode::ProjectNotFound,
                            "No template directory found. Pass a path: tsx template lint ./templates/",
                        ),
                        0,
                    );
                }
            }
        }
    };

    if !target.exists() {
        return ResponseEnvelope::error(
            "template:lint",
            ErrorResponse::new(
                ErrorCode::ProjectNotFound,
                format!("Path does not exist: {}", target.display()),
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    let files: Vec<std::path::PathBuf> = if target.is_file() {
        vec![target.clone()]
    } else {
        walkdir::WalkDir::new(&target)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| {
                let ext = e.path().extension().and_then(|x| x.to_str()).unwrap_or("");
                ext == "forge" || ext == "jinja"
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    };

    let mut all_errors = 0usize;
    let mut all_warnings = 0usize;
    let mut diagnostics: Vec<serde_json::Value> = Vec::new();

    for file in &files {
        match forge::lint_file(file) {
            Ok(result) => {
                all_errors += result.errors.len();
                all_warnings += result.warnings.len();
                let rel = file.to_string_lossy();
                for e in &result.errors {
                    diagnostics.push(serde_json::json!({
                        "file": rel, "line": e.line,
                        "severity": "error", "code": e.code, "message": e.message
                    }));
                }
                for w in &result.warnings {
                    diagnostics.push(serde_json::json!({
                        "file": rel, "line": w.line,
                        "severity": "warning", "code": "W000", "message": w.message
                    }));
                }
                for s in &result.suggestions {
                    diagnostics.push(serde_json::json!({
                        "file": rel, "line": s.line,
                        "severity": "suggestion", "code": "S000", "message": s.message
                    }));
                }
            }
            Err(e) => {
                diagnostics.push(serde_json::json!({
                    "file": file.to_string_lossy(), "line": 0,
                    "severity": "error", "code": "E001", "message": e.to_string()
                }));
                all_errors += 1;
            }
        }
    }

    let data = serde_json::json!({
        "files_checked": files.len(),
        "errors": all_errors,
        "warnings": all_warnings,
        "diagnostics": diagnostics,
    });

    if all_errors > 0 {
        ResponseEnvelope::error(
            "template:lint",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                format!("{} error(s), {} warning(s) in {} file(s)", all_errors, all_warnings, files.len()),
            ),
            start.elapsed().as_millis() as u64,
        )
    } else {
        ResponseEnvelope::success("template:lint", data, start.elapsed().as_millis() as u64)
    }
}
