use serde::Serialize;
use std::time::Instant;

use crate::framework::command_registry::{apply_defaults, validate_input, CommandRegistry};
use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::get_frameworks_dir;

#[derive(Serialize)]
struct RunResult {
    id: String,
    command: String,
    framework: String,
    files_created: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    next_steps: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dry_run_paths: Option<Vec<String>>,
    /// Approximate tokens an LLM would have spent writing this code manually.
    #[serde(skip_serializing_if = "Option::is_none")]
    tokens_saved: Option<u32>,
}

#[derive(Serialize)]
struct RunListEntry {
    id: String,
    command: String,
    framework: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_estimate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<serde_json::Value>,
}

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
        ResponseEnvelope::success("run", serde_json::to_value(result).unwrap(), duration_ms)
            .print();
        return CommandResult::ok("run", vec![]);
    }

    // Dispatch through the batch execute_command machinery.
    use crate::commands::ops::batch::execute_command_pub;
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
            let response = ResponseEnvelope::success(
                "run",
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

/// List all available generators, optionally filtered to a single framework.
pub fn run_list(framework: Option<String>, verbose: bool) -> CommandResult {
    let _ = verbose;
    let start = Instant::now();
    let registry = CommandRegistry::load_all();

    let specs = match &framework {
        Some(fw) => registry.for_framework(fw),
        None => registry.all(),
    };

    let entries: Vec<RunListEntry> = specs
        .iter()
        .map(|s| RunListEntry {
            id: s.id.clone(),
            command: s.command.clone(),
            framework: s.framework.clone(),
            description: s.description.clone(),
            token_estimate: s.token_estimate,
            schema: s.schema.clone(),
        })
        .collect();

    let count = entries.len();
    let duration_ms = start.elapsed().as_millis() as u64;
    let payload = serde_json::json!({
        "generators": entries,
        "total": count,
    });

    ResponseEnvelope::success("run:list", payload, duration_ms).print();
    CommandResult::ok("run:list", vec![])
}

/// Expand `{{field}}` placeholders in a path template using values from the JSON input.
/// If a `PathConfig` is provided, path prefix overrides from `.tsx/stack.json` are applied first.
fn expand_path_template(
    template: &str,
    input: &serde_json::Value,
    paths: Option<&crate::stack::PathConfig>,
) -> String {
    // Apply path prefix overrides from stack.json
    let template = if let Some(cfg) = paths {
        apply_path_prefix(template, cfg)
    } else {
        template.to_string()
    };

    let Some(obj) = input.as_object() else {
        return template;
    };

    let mut result = template;
    for (key, value) in obj {
        // Skip internal __style_* vars in path expansion
        if key.starts_with("__") {
            continue;
        }
        let placeholder = format!("{{{{{}}}}}", key);
        if let Some(s) = value.as_str() {
            result = result.replace(&placeholder, s);
        }
    }
    result
}

/// Replace well-known path prefixes with overrides from `.tsx/stack.json`.
/// E.g. if `paths.components = "src/components"`, then `"components/Foo.tsx"` →
/// `"src/components/Foo.tsx"`.
fn apply_path_prefix(template: &str, cfg: &crate::stack::PathConfig) -> String {
    let overrides: &[(&str, Option<&str>)] = &[
        ("components/", cfg.components.as_deref()),
        ("routes/", cfg.routes.as_deref()),
        ("db/", cfg.db.as_deref()),
        ("server-functions/", cfg.server_fns.as_deref()),
        ("hooks/", cfg.hooks.as_deref()),
    ];
    for (default_prefix, override_dir) in overrides {
        if let Some(dir) = override_dir {
            if template.starts_with(default_prefix) {
                return format!(
                    "{}/{}",
                    dir.trim_end_matches('/'),
                    &template[default_prefix.len()..]
                );
            }
        }
    }
    template.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::PathConfig;

    #[test]
    fn expand_path_template_replaces_placeholders() {
        let input = serde_json::json!({ "name": "users" });
        let result = expand_path_template("db/schema/{{name}}.ts", &input, None);
        assert_eq!(result, "db/schema/users.ts");
    }

    #[test]
    fn expand_path_template_no_match_unchanged() {
        let input = serde_json::json!({ "name": "users" });
        let result = expand_path_template("src/static.ts", &input, None);
        assert_eq!(result, "src/static.ts");
    }

    #[test]
    fn expand_path_template_handles_non_object_input() {
        let input = serde_json::json!("not-an-object");
        let result = expand_path_template("db/{{name}}.ts", &input, None);
        assert_eq!(result, "db/{{name}}.ts");
    }

    #[test]
    fn path_prefix_override_applied() {
        let cfg = PathConfig {
            components: Some("src/components".to_string()),
            ..Default::default()
        };
        let input = serde_json::json!({ "name": "Todo" });
        let result = expand_path_template("components/{{name}}Form.tsx", &input, Some(&cfg));
        assert_eq!(result, "src/components/TodoForm.tsx");
    }

    #[test]
    fn path_prefix_no_override_when_none() {
        let cfg = PathConfig::default();
        let input = serde_json::json!({ "name": "Todo" });
        let result = expand_path_template("components/{{name}}Form.tsx", &input, Some(&cfg));
        assert_eq!(result, "components/TodoForm.tsx");
    }

    #[test]
    fn style_vars_not_expanded_in_paths() {
        let mut input = serde_json::json!({ "name": "todo" });
        input["__style_quotes"] = serde_json::json!("double");
        let result = expand_path_template("db/{{name}}.ts", &input, None);
        assert_eq!(result, "db/todo.ts");
    }
}
