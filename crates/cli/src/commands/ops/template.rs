//! `tsx template` — manage forge template bundles.
//!
//! Subcommands:
//! - `tsx template list [--source <global|project|framework>]`
//! - `tsx template info <name>`
//! - `tsx template init <name> [--dest <dir>]`
//! - `tsx template install <source>`
//! - `tsx template uninstall <name>`
//! - `tsx template schema <name> <command>`

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
