use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::get_frameworks_dir;

#[derive(Debug, Deserialize)]
struct GeneratorSpec {
    id: String,
    command: String,
    #[allow(dead_code)]
    description: String,
    #[allow(dead_code)]
    token_estimate: Option<u32>,
    #[allow(dead_code)]
    schema: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct GenerateResult {
    generator: String,
    framework: String,
    command: String,
    files_created: Vec<String>,
}

pub fn generate(
    id: String,
    framework: Option<String>,
    json: Option<String>,
    overwrite: bool,
    dry_run: bool,
    verbose: bool,
) -> CommandResult {
    use crate::framework::detect::detect_framework;

    let start = Instant::now();

    // Resolve framework: explicit > auto-detect > error
    let fw_slug = match framework {
        Some(f) => f,
        None => {
            let root = std::env::current_dir().unwrap_or_default();
            match detect_framework(&root) {
                Some(f) => f,
                None => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    let error = ErrorResponse::validation(
                        "Could not detect framework. Use --fw <framework> to specify one.",
                    );
                    ResponseEnvelope::error("generate", error, duration_ms).print();
                    return CommandResult::err("generate", "Framework not detected");
                }
            }
        }
    };

    // Resolve generator spec
    let frameworks_dir = get_frameworks_dir();
    let spec_path = frameworks_dir
        .join(&fw_slug)
        .join("generators")
        .join(format!("{}.json", id));

    if !spec_path.exists() {
        let duration_ms = start.elapsed().as_millis() as u64;

        // List available generators to help the user
        let available = list_generators(&frameworks_dir.join(&fw_slug));
        let error = ErrorResponse::validation(&format!(
            "Generator '{}' not found for framework '{}'. Available: {}",
            id,
            fw_slug,
            if available.is_empty() {
                "none".to_string()
            } else {
                available.join(", ")
            }
        ));
        ResponseEnvelope::error("generate", error, duration_ms).print();
        return CommandResult::err("generate", "Generator not found");
    }

    let spec_content = match std::fs::read_to_string(&spec_path) {
        Ok(c) => c,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                crate::json::error::ErrorCode::InternalError,
                &format!("Failed to read generator spec: {}", e),
            );
            ResponseEnvelope::error("generate", error, duration_ms).print();
            return CommandResult::err("generate", "Failed to read generator spec");
        }
    };

    let spec: GeneratorSpec = match serde_json::from_str(&spec_content) {
        Ok(s) => s,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(
                crate::json::error::ErrorCode::InternalError,
                &format!("Invalid generator spec JSON: {}", e),
            );
            ResponseEnvelope::error("generate", error, duration_ms).print();
            return CommandResult::err("generate", "Invalid generator spec");
        }
    };

    // Parse args JSON (required by most generators)
    let options: serde_json::Value = match json.as_deref() {
        Some(j) => match serde_json::from_str(j) {
            Ok(v) => v,
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ErrorResponse::validation(&format!("Invalid --json: {}", e));
                ResponseEnvelope::error("generate", error, duration_ms).print();
                return CommandResult::err("generate", "Invalid JSON args");
            }
        },
        None => serde_json::json!({}),
    };

    // Dispatch through the batch execute_command machinery
    use crate::commands::ops::batch::execute_command_pub;
    match execute_command_pub(&spec.command, &options, overwrite, dry_run) {
        Ok(files_created) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let result = GenerateResult {
                generator: spec.id,
                framework: fw_slug,
                command: spec.command,
                files_created: files_created.clone(),
            };
            let response = ResponseEnvelope::success(
                "generate",
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
            CommandResult::ok("generate", files_created)
        }
        Err((code, msg)) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(code, &msg);
            ResponseEnvelope::error("generate", error, duration_ms).print();
            CommandResult::err("generate", msg)
        }
    }
}

/// List available generator IDs from the generators/ directory.
fn list_generators(fw_dir: &std::path::Path) -> Vec<String> {
    let gen_dir = fw_dir.join("generators");
    let Ok(entries) = std::fs::read_dir(gen_dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().map_or(false, |ext| ext == "json") {
                p.file_stem().map(|s| s.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect()
}

/// List all generators for a framework (for tsx list --kind generators).
pub fn list_framework_generators(fw_slug: &str) -> Vec<serde_json::Value> {
    let frameworks_dir = get_frameworks_dir();
    let gen_dir = frameworks_dir.join(fw_slug).join("generators");
    let Ok(entries) = std::fs::read_dir(gen_dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().map_or(false, |ext| ext == "json") {
                let content = std::fs::read_to_string(&p).ok()?;
                let spec: serde_json::Value = serde_json::from_str(&content).ok()?;
                Some(serde_json::json!({
                    "id": spec.get("id"),
                    "description": spec.get("description"),
                    "token_estimate": spec.get("token_estimate"),
                }))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_spec_deserializes() {
        let json = r#"{
            "id": "add-schema",
            "command": "add:schema",
            "description": "Generate schema",
            "token_estimate": 30
        }"#;
        let spec: GeneratorSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.id, "add-schema");
        assert_eq!(spec.command, "add:schema");
    }

    #[test]
    fn generator_spec_without_schema_field() {
        let json = r#"{
            "id": "add-page",
            "command": "add:page",
            "description": "Generate page"
        }"#;
        let spec: GeneratorSpec = serde_json::from_str(json).unwrap();
        assert!(spec.schema.is_none());
    }
}
