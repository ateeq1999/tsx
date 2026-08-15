use serde::Serialize;
use std::time::Instant;

use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Serialize)]
struct ValidateResult {
    framework: String,
    path: String,
    valid: bool,
    issues: Vec<String>,
    warnings: Vec<String>,
}

/// Lint a framework package directory: check manifest.json, knowledge files, starter schemas.
pub fn framework_validate(path: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let pkg_dir = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir().unwrap_or_default(),
    };

    let mut issues: Vec<String> = vec![];
    let mut warnings: Vec<String> = vec![];

    // 1. Check manifest.json exists and parses
    let manifest_path = pkg_dir.join("manifest.json");
    let framework_id = if manifest_path.exists() {
        match std::fs::read_to_string(&manifest_path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(m) => {
                        // Required fields
                        for field in ["id", "name", "version"] {
                            if m.get(field).is_none() {
                                issues.push(format!("manifest.json: missing required field '{}'", field));
                            }
                        }
                        m.get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string()
                    }
                    Err(e) => {
                        issues.push(format!("manifest.json: invalid JSON — {}", e));
                        "unknown".to_string()
                    }
                }
            }
            Err(e) => {
                issues.push(format!("manifest.json: cannot read — {}", e));
                "unknown".to_string()
            }
        }
    } else {
        issues.push("manifest.json not found".to_string());
        "unknown".to_string()
    };

    // 2. Check knowledge directory
    let knowledge_dir = pkg_dir.join("knowledge");
    if knowledge_dir.exists() {
        let sections = ["overview", "concepts", "patterns", "faq", "decisions"];
        let has_any = sections.iter().any(|s| {
            knowledge_dir.join(format!("{}.md", s)).exists()
        });
        if !has_any {
            warnings.push("knowledge/: no standard sections found (overview, concepts, patterns, faq, decisions)".to_string());
        }

        // Check each .md file has frontmatter token_estimate
        if let Ok(entries) = std::fs::read_dir(&knowledge_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map_or(false, |e| e == "md") {
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        if !content.starts_with("---") {
                            warnings.push(format!(
                                "knowledge/{}: missing frontmatter (expected token_estimate)",
                                p.file_name().unwrap_or_default().to_string_lossy()
                            ));
                        }
                    }
                }
            }
        }
    } else {
        warnings.push("knowledge/ directory not found".to_string());
    }

    // 3. Check starters
    let starters_dir = pkg_dir.join("starters");
    if starters_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&starters_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map_or(false, |e| e == "json") {
                    match std::fs::read_to_string(&p) {
                        Ok(content) => {
                            if let Err(e) = serde_json::from_str::<serde_json::Value>(&content) {
                                issues.push(format!(
                                    "starters/{}: invalid JSON — {}",
                                    p.file_name().unwrap_or_default().to_string_lossy(),
                                    e
                                ));
                            }
                        }
                        Err(e) => {
                            issues.push(format!(
                                "starters/{}: cannot read — {}",
                                p.file_name().unwrap_or_default().to_string_lossy(),
                                e
                            ));
                        }
                    }
                }
            }
        }
    } else {
        warnings.push("starters/ directory not found".to_string());
    }

    let valid = issues.is_empty();
    let duration_ms = start.elapsed().as_millis() as u64;

    let result = ValidateResult {
        framework: framework_id,
        path: pkg_dir.to_string_lossy().to_string(),
        valid,
        issues,
        warnings,
    };

    let response = ResponseEnvelope::success(
        "framework:validate",
        serde_json::to_value(result).unwrap(),
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

    CommandResult::ok("framework:validate", vec![])
}
