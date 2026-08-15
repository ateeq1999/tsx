use std::time::Instant;

use crate::framework::command_registry::{apply_defaults, validate_input, CommandRegistry};
use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

use super::types::RunResult;
use super::utils::{expand_path_template, inject_slots};

/// Universal generator dispatcher: resolves any installed generator by id or command name,
/// validates the JSON input against its schema, applies defaults, then executes.
pub fn run(
    id: String,
    framework: Option<String>,
    json_str: Option<String>,
    overwrite: bool,
    dry_run: bool,
    verbose: bool,
) -> CommandResult {
    let start = Instant::now();

    let registry = CommandRegistry::load_all();

    // Resolve spec — honour the framework filter when provided.
    let spec = match registry.resolve(&id) {
        Some(s) => {
            if let Some(ref fw) = framework {
                if s.framework != *fw {
                    // The matching spec belongs to a different framework; look in the right one.
                    match registry
                        .for_framework(fw)
                        .into_iter()
                        .find(|s| s.id == id || s.command == id)
                    {
                        Some(s) => s.clone(),
                        None => {
                            let duration_ms = start.elapsed().as_millis() as u64;
                            let available = registry
                                .for_framework(fw)
                                .iter()
                                .map(|s| s.id.as_str())
                                .collect::<Vec<_>>()
                                .join(", ");
                            let error = ErrorResponse::validation(&format!(
                                "Generator '{}' not found in framework '{}'. Available: {}",
                                id,
                                fw,
                                if available.is_empty() {
                                    "none"
                                } else {
                                    &available
                                }
                            ));
                            ResponseEnvelope::error("run", error, duration_ms).print();
                            return CommandResult::err("run", "Generator not found");
                        }
                    }
                } else {
                    s.clone()
                }
            } else {
                s.clone()
            }
        }
        None => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let known: Vec<String> = registry
                .all()
                .iter()
                .map(|s| format!("{} ({})", s.id, s.framework))
                .collect();
            let error = ErrorResponse::validation(&format!(
                "Unknown generator '{}'. Run `tsx run --list` to see all available generators.\nInstalled: {}",
                id,
                if known.is_empty() {
                    "none — install a framework package first".to_string()
                } else {
                    known.join(", ")
                }
            ));
            ResponseEnvelope::error("run", error, duration_ms).print();
            return CommandResult::err("run", "Unknown generator");
        }
    };

    // Parse JSON input (default: empty object so defaults can be applied).
    let raw = json_str.unwrap_or_else(|| "{}".to_string());
    let mut input: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::validation(&format!("Invalid --json: {}", e));
            ResponseEnvelope::error("run", error, duration_ms).print();
            return CommandResult::err("run", "Invalid JSON");
        }
    };

    // Load the stack profile (optional — silently absent if no .tsx/stack.json).
    let cwd = std::env::current_dir().unwrap_or_default();
    let stack = crate::stack::StackProfile::load(&cwd);

    // Inject style vars as __style_* so forge templates can use them.
    // These use a double-underscore prefix to avoid colliding with user input fields.
    if let Some(obj) = input.as_object_mut() {
        let style = stack.as_ref().map(|p| p.style.clone()).unwrap_or_default();
        obj.entry("__style_quotes")
            .or_insert_with(|| serde_json::json!(style.quotes));
        obj.entry("__style_indent")
            .or_insert_with(|| serde_json::json!(style.indent));
        obj.entry("__style_semicolons")
            .or_insert_with(|| serde_json::json!(style.semicolons));
    }

    // Inject slot content from peer packages into the input context.
    if let Some(ref profile) = stack {
        inject_slots(&mut input, &spec.framework, profile, &cwd);
    }

    // Apply schema defaults then validate.
    if let Some(schema) = &spec.schema {
        apply_defaults(&mut input, schema);
        let errors = validate_input(&input, schema);
        if !errors.is_empty() {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::validation(&format!(
                "Validation failed for '{}': {}",
                spec.id,
                errors.join("; ")
            ));
            ResponseEnvelope::error("run", error, duration_ms).print();
            return CommandResult::err("run", "Validation failed");
        }
    }

    // Resolve path config for output path overrides.
    let path_config = stack.as_ref().map(|p| &p.paths);

    // Dry-run: resolve output path templates and return without writing.
    if dry_run {
        let duration_ms = start.elapsed().as_millis() as u64;
        let dry_run_paths: Vec<String> = spec
            .output_paths
            .iter()
            .map(|p| expand_path_template(p, &input, path_config))
            .collect();
        let result = RunResult {
            id: spec.id.clone(),
            command: spec.command.clone(),
            framework: spec.framework.clone(),
            files_created: vec![],
            next_steps: spec.next_steps.clone(),
            dry_run_paths: Some(dry_run_paths),
            tokens_saved: spec.token_estimate,
        };
        let mut env = ResponseEnvelope::success("run", serde_json::to_value(result).unwrap(), duration_ms);
        if let Some(t) = spec.token_estimate {
            env = env.with_tokens_used(t);
        }
        env.print();
        return CommandResult::ok("run", vec![]);
    }

    // Dispatch through the batch execute_command machinery.
    use crate::commands::batch::execute_command_pub;
    match execute_command_pub(&spec.command, &input, overwrite, false) {
        Ok(files_created) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let result = RunResult {
                id: spec.id.clone(),
                command: spec.command.clone(),
                framework: spec.framework.clone(),
                files_created: files_created.clone(),
                next_steps: spec.next_steps.clone(),
                dry_run_paths: None,
                tokens_saved: spec.token_estimate,
            };
            let mut response = ResponseEnvelope::success(
                "run",
                serde_json::to_value(result).unwrap(),
                duration_ms,
            );
            if let Some(t) = spec.token_estimate {
                response = response.with_tokens_used(t);
            }
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
            CommandResult::ok("run", files_created)
        }
        Err((code, msg)) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(code, &msg);
            ResponseEnvelope::error("run", error, duration_ms).print();
            CommandResult::err("run", msg)
        }
    }
}
