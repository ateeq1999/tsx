//! `tsx template` — manage forge template bundles.
//!
//! Subcommands:
//! - `tsx template list [--source <global|project|framework>]`
//! - `tsx template info <name>`
//! - `tsx template init <name> [--dest <dir>]`
//! - `tsx template install <source>`
//! - `tsx template uninstall <name>`
//! - `tsx template schema <name> <command>`
//! - `tsx template lint [<path>]`
//! - `tsx template config show`
//! - `tsx template config set <key> <value>`
//! - `tsx template config init`

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

pub fn template_list(source: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let templates = match source.as_deref() {
        Some("global") => forge::discover_from_source(forge::TemplateSource::Global),
        Some("project") => forge::discover_from_source(forge::TemplateSource::Project),
        Some("framework") => forge::discover_from_source(forge::TemplateSource::Framework),
        Some(other) => {
            return ResponseEnvelope::error(
                "template:list",
                ErrorResponse::new(
                    ErrorCode::ValidationError,
                    format!("Unknown source '{}'. Use: global, project, or framework", other),
                ),
                0,
            );
        }
        None => forge::discover_templates(),
    };

    let items: Vec<serde_json::Value> = templates
        .iter()
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "name": t.name,
                "version": t.version,
                "description": t.description,
                "source": t.source.to_string(),
                "path": t.path.to_string_lossy(),
            })
        })
        .collect();

    let data = serde_json::json!({
        "count": items.len(),
        "templates": items,
    });

    ResponseEnvelope::success("template:list", data, start.elapsed().as_millis() as u64)
}

pub fn template_info(name: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    match forge::find_template(&name) {
        None => ResponseEnvelope::error(
            "template:info",
            ErrorResponse::new(
                ErrorCode::TemplateNotFound,
                format!("Template '{}' not found. Run `tsx template list` to see available templates.", name),
            ),
            start.elapsed().as_millis() as u64,
        ),
        Some(info) => {
            let data = serde_json::json!({
                "id": info.id,
                "name": info.name,
                "version": info.version,
                "description": info.description,
                "source": info.source.to_string(),
                "path": info.path.to_string_lossy(),
                "manifest": info.manifest,
            });
            ResponseEnvelope::success("template:info", data, start.elapsed().as_millis() as u64)
        }
    }
}

pub fn template_init(name: String, dest: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let dest_path = match dest {
        Some(d) => std::path::PathBuf::from(d),
        None => std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(&name),
    };

    match forge::init_template(&name, &dest_path) {
        Ok(()) => {
            let data = serde_json::json!({
                "id": name,
                "path": dest_path.to_string_lossy(),
                "files_created": ["manifest.json", "README.md"],
            });
            ResponseEnvelope::success("template:init", data, start.elapsed().as_millis() as u64)
        }
        Err(e) => ResponseEnvelope::error(
            "template:init",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            start.elapsed().as_millis() as u64,
        ),
    }
}

pub fn template_install(source: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    let src_path = std::path::Path::new(&source);

    if !src_path.exists() {
        return ResponseEnvelope::error(
            "template:install",
            ErrorResponse::new(
                ErrorCode::ProjectNotFound,
                format!(
                    "Source '{}' not found. Only local directory installation is currently supported.",
                    source
                ),
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    match forge::install_from_dir(src_path) {
        Ok(info) => {
            let data = serde_json::json!({
                "installed": {
                    "id": info.id,
                    "name": info.name,
                    "version": info.version,
                    "path": info.path.to_string_lossy(),
                }
            });
            ResponseEnvelope::success("template:install", data, start.elapsed().as_millis() as u64)
        }
        Err(e) => ResponseEnvelope::error(
            "template:install",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            start.elapsed().as_millis() as u64,
        ),
    }
}

pub fn template_uninstall(name: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    match forge::uninstall(&name) {
        Ok(()) => {
            let data = serde_json::json!({ "uninstalled": name });
            ResponseEnvelope::success("template:uninstall", data, start.elapsed().as_millis() as u64)
        }
        Err(e) => ResponseEnvelope::error(
            "template:uninstall",
            ErrorResponse::new(ErrorCode::InternalError, e.to_string()),
            start.elapsed().as_millis() as u64,
        ),
    }
}

pub fn template_schema(name: String, command: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();

    match forge::template_schema(&name, &command) {
        None => ResponseEnvelope::error(
            "template:schema",
            ErrorResponse::new(
                ErrorCode::TemplateNotFound,
                format!("No schema found for template '{}' command '{}'", name, command),
            ),
            start.elapsed().as_millis() as u64,
        ),
        Some(schema) => {
            ResponseEnvelope::success("template:schema", schema, start.elapsed().as_millis() as u64)
        }
    }
}

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

pub fn template_config_show(_verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let global = forge::load_global_config();
    let project = forge::load_project_config();
    let resolved = forge::resolve_config(None, None);

    let data = serde_json::json!({
        "global_config_path": forge::global_config_path().to_string_lossy(),
        "project_config_path": forge::project_config_path().to_string_lossy(),
        "global": serde_json::to_value(&global).unwrap_or_default(),
        "project": serde_json::to_value(&project).unwrap_or_default(),
        "resolved": {
            "registry_url": resolved.registry_url,
            "template_for": resolved.template_for,
        },
    });
    ResponseEnvelope::success("template:config:show", data, start.elapsed().as_millis() as u64)
}

pub fn template_config_set(key: String, value: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let mut cfg = forge::load_global_config();

    match key.as_str() {
        "registry_url" => cfg.registry_url = Some(value.clone()),
        other => {
            cfg.preferred_templates.insert(other.to_string(), value.clone());
        }
    }

    match forge::save_global_config(&cfg) {
        Ok(()) => {
            let data = serde_json::json!({ "key": key, "value": value });
            ResponseEnvelope::success("template:config:set", data, start.elapsed().as_millis() as u64)
        }
        Err(e) => ResponseEnvelope::error(
            "template:config:set",
            ErrorResponse::new(ErrorCode::InternalError, e),
            start.elapsed().as_millis() as u64,
        ),
    }
}

pub fn template_config_init(overwrite: bool, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let global_path = forge::global_config_path();
    let project_path = forge::project_config_path();

    if global_path.exists() && !overwrite {
        return ResponseEnvelope::error(
            "template:config:init",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                format!("{} already exists. Pass --overwrite to replace.", global_path.display()),
            ),
            start.elapsed().as_millis() as u64,
        );
    }

    let created_global = forge::save_global_config(&forge::GlobalConfig::default()).is_ok();
    let created_project = forge::save_project_config(&forge::ProjectConfig::default()).is_ok();

    let data = serde_json::json!({
        "created": {
            "global": if created_global { global_path.to_string_lossy().to_string() } else { String::new() },
            "project": if created_project { project_path.to_string_lossy().to_string() } else { String::new() },
        }
    });
    ResponseEnvelope::success("template:config:init", data, start.elapsed().as_millis() as u64)
}
