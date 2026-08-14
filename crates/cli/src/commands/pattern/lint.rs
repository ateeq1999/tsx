use std::collections::HashMap;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

/// Validate a pack: check template files exist and render without errors.
pub fn pattern_lint(id: String, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    let Some(pack) = forge::PackManifest::load(&cwd, &id) else {
        return ResponseEnvelope::error(
            "pattern lint",
            ErrorResponse::new(ErrorCode::ProjectNotFound, format!("Pack '{}' not found in .tsx/patterns/", id)),
            0,
        );
    };

    let pack_dir = forge::PackManifest::dir(&cwd, &id);
    let mut errors: Vec<String> = Vec::new();

    // 1. Check all template files exist on disk
    for output in &pack.outputs {
        if !pack_dir.join(&output.template).exists() {
            errors.push(format!("Template '{}' missing for output '{}'", output.template, output.id));
        }
    }

    // 2. Validate outputs[].path as Tera expressions (using dummy args)
    {
        let mut dummy_args = HashMap::new();
        for arg in &pack.args {
            dummy_args.insert(
                arg.name.clone(),
                serde_json::Value::String(format!("dummy_{}", arg.name)),
            );
        }
        for output in &pack.outputs {
            if let Err(e) = forge::interpolate_pack_path(&output.path, &dummy_args) {
                errors.push(format!(
                    "output '{}': invalid path expression '{}': {e}",
                    output.id, output.path
                ));
            }
        }
    }

    // 3. Load engine, validate @schema directives, and attempt render with dummy context
    let mut engine = forge::Engine::new();
    match engine.load_dir(&pack_dir) {
        Err(e) => errors.push(format!("Engine load error: {e}")),
        Ok(_) => {
            let mut ctx = forge::ForgeContext::new();
            for arg in &pack.args {
                ctx.insert_mut(&arg.name, &serde_json::Value::String(format!("dummy_{}", arg.name)));
            }
            for output in &pack.outputs {
                // Validate @schema directive JSON syntax
                let tmpl_path = pack_dir.join(&output.template);
                if let Ok(src) = std::fs::read_to_string(&tmpl_path) {
                    if let Some(schema_errors) = lint_schema_directive(&src, &output.template) {
                        errors.extend(schema_errors);
                    }
                }
                // Render with dummy context
                match engine.render(&output.template, &ctx) {
                    Err(e) => {
                        // Extract line number from tera error if present
                        let msg = e.to_string();
                        errors.push(format!("Render error in '{}': {msg}", output.template));
                    }
                    Ok(_) => {}
                }
            }
        }
    }

    // 4. Check marker files reference valid output paths (warn only)
    let mut warnings: Vec<String> = Vec::new();
    for marker in &pack.markers {
        let marker_path = cwd.join(&marker.file);
        if !marker_path.exists() {
            warnings.push(format!("Marker file '{}' not present in project (may be created later)", marker.file));
        }
        // Validate marker insert expression
        let dummy_args: HashMap<String, serde_json::Value> = pack.args.iter()
            .map(|a| (a.name.clone(), serde_json::Value::String(format!("dummy_{}", a.name))))
            .collect();
        if let Err(e) = forge::interpolate_pack_path(&marker.insert, &dummy_args) {
            errors.push(format!("marker '{}' insert expression '{}' is invalid: {e}", marker.marker, marker.insert));
        }
    }

    if errors.is_empty() {
        ResponseEnvelope::success(
            "pattern lint",
            serde_json::json!({
                "id": id,
                "status": "ok",
                "warnings": warnings,
            }),
            0,
        )
    } else {
        ResponseEnvelope::error(
            "pattern lint",
            ErrorResponse::new(ErrorCode::ValidationError, errors.join("\n")),
            0,
        )
    }
}

/// Scan a forge template source for `@schema({...})` directives and validate
/// that each one contains valid JSON. Returns error strings or `None`.
fn lint_schema_directive(src: &str, template_name: &str) -> Option<Vec<String>> {
    let mut errs = Vec::new();
    for (line_no, line) in src.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("@schema(") {
            // Strip trailing `)`
            let json_str = rest.trim_end_matches(')').trim();
            if let Err(e) = serde_json::from_str::<serde_json::Value>(json_str) {
                errs.push(format!(
                    "{}:{}: invalid @schema JSON: {e}",
                    template_name,
                    line_no + 1
                ));
            }
        }
    }
    if errs.is_empty() { None } else { Some(errs) }
}
