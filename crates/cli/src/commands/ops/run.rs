use serde::Serialize;
use std::path::{Path, PathBuf};
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

/// Locate the directory for a package slug by searching builtin, .tsx/frameworks, .tsx/packages.
fn find_package_dir(slug: &str, cwd: &Path) -> Option<PathBuf> {
    // Builtin
    let builtin = get_frameworks_dir().join(slug);
    if builtin.is_dir() {
        return Some(builtin);
    }
    // User-installed FPF
    let fpf = cwd.join(".tsx").join("packages").join(slug);
    if fpf.is_dir() {
        return Some(fpf);
    }
    // Legacy
    let legacy = cwd.join(".tsx").join("frameworks").join(slug);
    if legacy.is_dir() {
        return Some(legacy);
    }
    None
}

/// Scan all installed packages and inject slot content into the input JSON.
///
/// For each peer package listed in `stack.packages`:
///   - Load its `manifest.json`
///   - If `integrates_with[current_framework]` exists, get the slot name
///   - Load and render `slots/<slot>.forge` with tsx-forge using `input` as context
///   - Set `input["slot_<name>"]` to the rendered string
fn inject_slots(
    input: &mut serde_json::Value,
    current_framework: &str,
    stack: &crate::stack::StackProfile,
    cwd: &Path,
) {
    let package_names = stack.package_names();
    for pkg_name in &package_names {
        // Don't inject a package's slots into itself
        if *pkg_name == current_framework {
            continue;
        }
        let Some(pkg_dir) = find_package_dir(pkg_name, cwd) else {
            continue;
        };
        let manifest_path = pkg_dir.join("manifest.json");
        let Ok(manifest_str) = std::fs::read_to_string(&manifest_path) else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_str) else {
            continue;
        };
        let Some(integrates) = manifest
            .get("integrates_with")
            .and_then(|v| v.as_object())
        else {
            continue;
        };
        let Some(integration) = integrates.get(current_framework) else {
            continue;
        };
        let Some(slot_name) = integration.get("slot").and_then(|v| v.as_str()) else {
            continue;
        };

        // Load the .forge slot template
        let slot_path = pkg_dir.join("slots").join(format!("{slot_name}.forge"));
        let Ok(template_src) = std::fs::read_to_string(&slot_path) else {
            continue;
        };

        // Render with tsx-forge using the current generator input as context
        let rendered = {
            let mut engine = forge::Engine::new();
            let tpl_key = format!("slot_{slot_name}.forge");
            if engine.add_raw(&tpl_key, &template_src).is_err() {
                continue;
            }
            let mut ctx = forge::ForgeContext::new();
            if let Some(obj) = input.as_object() {
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        ctx.insert_mut(k, s);
                    } else if let Some(b) = v.as_bool() {
                        ctx.insert_mut(k, &b);
                    } else if let Some(n) = v.as_i64() {
                        ctx.insert_mut(k, &n);
                    }
                }
            }
            match engine.render(&tpl_key, &ctx) {
                Ok(s) => s,
                Err(_) => continue,
            }
        };

        // Inject as slot_<name> into the input object
        if let Some(obj) = input.as_object_mut() {
            let key = format!("slot_{slot_name}");
            obj.entry(key).or_insert_with(|| serde_json::json!(rendered));
        }
    }
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
        ("components/", Some(cfg.components.as_str())),
        ("routes/", Some(cfg.routes.as_str())),
        ("db/", Some(cfg.db.as_str())),
        ("server-functions/", Some(cfg.server_fns.as_str())),
        ("hooks/", Some(cfg.hooks.as_str())),
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
            components: "src/components".to_string(),
            ..Default::default()
        };
        let input = serde_json::json!({ "name": "Todo" });
        let result = expand_path_template("components/{{name}}Form.tsx", &input, Some(&cfg));
        assert_eq!(result, "src/components/TodoForm.tsx");
    }

    #[test]
    fn path_prefix_applies_default() {
        // PathConfig::default() has components = "app/components", so prefix is rewritten
        let cfg = PathConfig::default();
        let input = serde_json::json!({ "name": "Todo" });
        let result = expand_path_template("components/{{name}}Form.tsx", &input, Some(&cfg));
        assert_eq!(result, "app/components/TodoForm.tsx");
    }

    #[test]
    fn style_vars_not_expanded_in_paths() {
        let mut input = serde_json::json!({ "name": "todo" });
        input["__style_quotes"] = serde_json::json!("double");
        let result = expand_path_template("db/{{name}}.ts", &input, None);
        assert_eq!(result, "db/todo.ts");
    }
}
