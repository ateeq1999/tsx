use std::time::Instant;

use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

/// Render a forge template with test data and print to stdout.
pub fn framework_preview(template: String, data: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let template_path = std::path::Path::new(&template);
    if !template_path.exists() {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::validation(&format!(
            "Template not found: {}",
            template
        ));
        ResponseEnvelope::error("framework:preview", error, duration_ms).print();
        return CommandResult::err("framework:preview", "Template not found");
    }

    // Parse context from --data JSON or use empty context
    let ctx_value: serde_json::Value = match data.as_deref() {
        Some(d) => match serde_json::from_str(d) {
            Ok(v) => v,
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let error = ErrorResponse::validation(&format!("Invalid --data JSON: {}", e));
                ResponseEnvelope::error("framework:preview", error, duration_ms).print();
                return CommandResult::err("framework:preview", "Invalid data JSON");
            }
        },
        None => serde_json::json!({}),
    };

    let mut engine = forge::Engine::default();

    // Load the single template file
    let file_name = template_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let content = match std::fs::read_to_string(template_path) {
        Ok(c) => c,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Failed to read template: {}", e));
            ResponseEnvelope::error("framework:preview", error, duration_ms).print();
            return CommandResult::err("framework:preview", "Failed to read template");
        }
    };

    if let Err(e) = engine.add_raw(&file_name, &content) {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Failed to load template: {}", e));
        ResponseEnvelope::error("framework:preview", error, duration_ms).print();
        return CommandResult::err("framework:preview", "Failed to load template");
    }

    let forge_ctx = match forge::ForgeContext::from_serialize(&ctx_value) {
        Ok(c) => c,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Context error: {}", e));
            ResponseEnvelope::error("framework:preview", error, duration_ms).print();
            return CommandResult::err("framework:preview", "Context error");
        }
    };

    match engine.render(&file_name, &forge_ctx) {
        Ok(rendered) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let result = serde_json::json!({
                "template": file_name,
                "output": rendered,
                "tier": engine.tier_of(&file_name).to_string(),
            });
            let response = ResponseEnvelope::success("framework:preview", result, duration_ms);
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
            CommandResult::ok("framework:preview", vec![])
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Render error: {}", e));
            ResponseEnvelope::error("framework:preview", error, duration_ms).print();
            CommandResult::err("framework:preview", "Render failed")
        }
    }
}
