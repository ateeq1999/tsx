use std::path::Path;
use std::time::Instant;

use crate::framework::registry::FrameworkRegistry;
use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

/// Validation errors for a registry file.
#[derive(Debug, serde::Serialize)]
pub struct RegistryValidationError {
    pub field: String,
    pub message: String,
}

/// Validate a `registry.json` file and return all errors found.
pub fn validate_registry(
    registry: &FrameworkRegistry,
) -> Vec<RegistryValidationError> {
    let mut errors = Vec::new();

    if registry.framework.trim().is_empty() {
        errors.push(RegistryValidationError {
            field: "framework".into(),
            message: "framework name is required".into(),
        });
    }

    if registry.slug.trim().is_empty() {
        errors.push(RegistryValidationError {
            field: "slug".into(),
            message: "slug is required".into(),
        });
    } else {
        // slug must be lowercase alphanumeric with hyphens only
        let ok = registry
            .slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !ok {
            errors.push(RegistryValidationError {
                field: "slug".into(),
                message: format!(
                    "slug '{}' must be lowercase alphanumeric with hyphens only",
                    registry.slug
                ),
            });
        }
    }

    if registry.version.trim().is_empty() {
        errors.push(RegistryValidationError {
            field: "version".into(),
            message: "version is required".into(),
        });
    } else {
        // Basic semver check: major.minor.patch
        let parts: Vec<&str> = registry.version.split('.').collect();
        let valid = parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok());
        if !valid {
            errors.push(RegistryValidationError {
                field: "version".into(),
                message: format!(
                    "version '{}' must follow semver (major.minor.patch)",
                    registry.version
                ),
            });
        }
    }

    if registry.docs.trim().is_empty() {
        errors.push(RegistryValidationError {
            field: "docs".into(),
            message: "docs URL is required".into(),
        });
    }

    if registry.conventions.naming.files.is_none()
        && registry.conventions.naming.components.is_none()
    {
        errors.push(RegistryValidationError {
            field: "conventions.naming".into(),
            message: "conventions.naming must specify at least files or components naming convention".into(),
        });
    }

    errors
}

/// `tsx publish --registry <path> [--output <path>]`
///
/// Validates and "publishes" a framework registry JSON file:
/// - validates required fields and format
/// - stamps a `published_at` timestamp
/// - writes the validated registry to `--output` (default: stdout)
pub fn publish(registry_path: String, output: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();
    let path = Path::new(&registry_path);

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                ErrorCode::DirectoryNotFound,
                format!("Cannot read registry file '{}': {}", registry_path, e),
            );
            ResponseEnvelope::error("publish", error, duration_ms).print();
            return CommandResult::err("publish", e.to_string());
        }
    };

    let registry: FrameworkRegistry = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                ErrorCode::InvalidPayload,
                format!("Invalid registry JSON: {}", e),
            );
            ResponseEnvelope::error("publish", error, duration_ms).print();
            return CommandResult::err("publish", e.to_string());
        }
    };

    // Validate
    let errors = validate_registry(&registry);
    if !errors.is_empty() {
        let duration_ms = start.elapsed().as_millis() as u64;
        let msgs: Vec<String> = errors
            .iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect();
        let error = ErrorResponse::new(
            ErrorCode::ValidationError,
            format!("Registry validation failed:\n{}", msgs.join("\n")),
        );
        ResponseEnvelope::error("publish", error, duration_ms).print();
        return CommandResult::err("publish", "validation failed".to_string());
    }

    // Stamp published_at into the slug field comment (stored externally as metadata)
    let published_at = {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let s = secs % 60;
        let m = (secs / 60) % 60;
        let h = (secs / 3600) % 24;
        let days = secs / 86400;
        let year = 1970 + days / 365;
        let day_of_year = days % 365 + 1;
        let month = (day_of_year / 30).min(11) + 1;
        let day = day_of_year % 30 + 1;
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, h, m, s)
    };

    // Build published package JSON (wraps registry with publication metadata).
    let package = serde_json::json!({
        "slug": registry.slug,
        "framework": registry.framework,
        "version": registry.version,
        "category": registry.category,
        "docs": registry.docs,
        "published_at": published_at,
        "registry": serde_json::to_value(&registry).unwrap_or_default(),
    });

    let output_json = serde_json::to_string_pretty(&package)
        .unwrap_or_else(|_| "{}".to_string());

    let duration_ms = start.elapsed().as_millis() as u64;

    match output {
        Some(ref out_path) => {
            if let Err(e) = std::fs::write(out_path, &output_json) {
                let error = ErrorResponse::new(
                    ErrorCode::PermissionDenied,
                    format!("Cannot write to '{}': {}", out_path, e),
                );
                ResponseEnvelope::error("publish", error, duration_ms).print();
                return CommandResult::err("publish", e.to_string());
            }

            let response = ResponseEnvelope::success(
                "publish",
                serde_json::json!({
                    "slug": registry.slug,
                    "version": registry.version,
                    "output": out_path,
                    "published_at": published_at,
                }),
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
            CommandResult::ok("publish", vec![out_path.clone()])
        }
        None => {
            // Print validated registry JSON to stdout
            println!("{}", output_json);
            CommandResult::ok("publish", vec![])
        }
    }
}

/// `tsx publish --list`  — list all registries installed in `.tsx/frameworks/`
pub fn publish_list(verbose: bool) -> CommandResult {
    let start = Instant::now();

    let root = match crate::utils::paths::find_project_root() {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(ErrorCode::ProjectNotFound, e.to_string());
            ResponseEnvelope::error("publish:list", error, duration_ms).print();
            return CommandResult::err("publish:list", e.to_string());
        }
    };

    let frameworks_dir = root.join(".tsx").join("frameworks");
    let registries: Vec<serde_json::Value> = std::fs::read_dir(&frameworks_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let reg_path = e.path().join("registry.json");
            let content = std::fs::read_to_string(&reg_path).ok()?;
            let reg: FrameworkRegistry = serde_json::from_str(&content).ok()?;
            Some(serde_json::json!({
                "slug": reg.slug,
                "framework": reg.framework,
                "version": reg.version,
                "category": reg.category,
            }))
        })
        .collect();

    let duration_ms = start.elapsed().as_millis() as u64;
    let response = ResponseEnvelope::success(
        "publish:list",
        serde_json::json!({ "registries": registries }),
        duration_ms,
    );

    if verbose {
        let context = crate::json::response::Context {
            project_root: root.to_string_lossy().to_string(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        response.with_context(context).print();
    } else {
        response.print();
    }

    CommandResult::ok("publish:list", vec![])
}
