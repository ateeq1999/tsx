use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::framework::command_registry::CommandRegistry;
use crate::json::error::ErrorCode;
use crate::json::payload::BatchPayload;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub results: Vec<BatchCommandResult>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rolled_back_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCommandResult {
    pub index: u32,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BatchError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

pub fn batch(
    payload: BatchPayload,
    overwrite: bool,
    dry_run: bool,
    verbose: bool,
    stream: bool,
) -> CommandResult {
    let start = Instant::now();

    let total = payload.commands.len() as u32;
    let mut succeeded: u32 = 0;
    let mut failed: u32 = 0;
    let mut results: Vec<BatchCommandResult> = Vec::new();
    // All file paths written so far, for rollback.
    let mut all_files_written: Vec<String> = Vec::new();

    for (index, cmd) in payload.commands.iter().enumerate() {
        let cmd_start = Instant::now();
        let result = execute_command(&cmd.command, &cmd.options, overwrite, dry_run);
        let cmd_duration_ms = cmd_start.elapsed().as_millis() as u64;

        let batch_cmd_result = match result {
            Ok(files_created) => {
                all_files_written.extend(files_created.clone());
                succeeded += 1;
                BatchCommandResult {
                    index: index as u32,
                    success: true,
                    result: Some(serde_json::json!({
                        "kind": cmd.command.clone(),
                        "files": files_created,
                        "duration_ms": cmd_duration_ms,
                    })),
                    error: None,
                }
            }
            Err((code, message)) => {
                failed += 1;
                BatchCommandResult {
                    index: index as u32,
                    success: false,
                    result: None,
                    error: Some(BatchError {
                        code,
                        message,
                        path: None,
                    }),
                }
            }
        };

        if stream {
            // Emit each result immediately as a newline-delimited JSON event.
            if let Ok(line) = serde_json::to_string(&batch_cmd_result) {
                println!("{}", line);
            }
        }

        let should_stop = batch_cmd_result.error.is_some() && payload.stop_on_failure;
        results.push(batch_cmd_result);

        if should_stop {
            break;
        }
    }

    // Rollback: delete all files written by earlier commands if requested.
    let mut rolled_back_files: Vec<String> = Vec::new();
    if failed > 0 && payload.rollback_on_failure && !dry_run {
        for path in &all_files_written {
            if std::fs::remove_file(path).is_ok() {
                rolled_back_files.push(path.clone());
            }
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    // Sum token estimates for all commands that were resolved via the registry.
    let registry = CommandRegistry::load_all();
    let total_tokens: u32 = payload
        .commands
        .iter()
        .filter_map(|cmd| registry.resolve(&cmd.command)?.token_estimate)
        .sum();

    let batch_result = BatchResult {
        total,
        succeeded,
        failed,
        results,
        rolled_back_files,
    };

    let response = ResponseEnvelope::success(
        "batch",
        serde_json::to_value(batch_result).unwrap(),
        duration_ms,
    )
    .with_tokens_used(total_tokens);

    if !stream {
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
    } else {
        // In stream mode, emit a final summary line.
        if let Ok(summary) = serde_json::to_string(&serde_json::json!({
            "event": "batch_complete",
            "total": total,
            "succeeded": succeeded,
            "failed": failed,
            "duration_ms": duration_ms,
        })) {
            println!("{}", summary);
        }
    }

    CommandResult::ok("batch", vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::payload::{BatchCommand, BatchPayload};

    #[test]
    fn batch_result_omits_empty_rolled_back() {
        let result = BatchResult {
            total: 1,
            succeeded: 1,
            failed: 0,
            results: vec![],
            rolled_back_files: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("rolled_back_files"), "empty rollback should be omitted");
    }

    #[test]
    fn batch_result_includes_rolled_back_when_present() {
        let result = BatchResult {
            total: 2,
            succeeded: 1,
            failed: 1,
            results: vec![],
            rolled_back_files: vec!["src/foo.ts".to_string()],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("rolled_back_files"));
        assert!(json.contains("src/foo.ts"));
    }

    #[test]
    fn batch_payload_rollback_defaults_false() {
        let json = r#"{"commands":[]}"#;
        let payload: BatchPayload = serde_json::from_str(json).unwrap();
        assert!(!payload.rollback_on_failure);
        assert!(!payload.stop_on_failure);
    }

    #[test]
    fn batch_payload_rollback_explicit_true() {
        let json = r#"{"commands":[],"stop_on_failure":true,"rollback_on_failure":true}"#;
        let payload: BatchPayload = serde_json::from_str(json).unwrap();
        assert!(payload.stop_on_failure);
        assert!(payload.rollback_on_failure);
    }

    #[test]
    fn batch_error_uses_error_code_enum() {
        let err = BatchError {
            code: ErrorCode::ValidationError,
            message: "bad input".to_string(),
            path: None,
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("VALIDATIONERROR") || json.contains("ValidationError") || json.contains("VALIDATION_ERROR"));
    }

    #[test]
    fn batch_command_unknown_returns_unknown_command_error() {
        let result = execute_command("unknown:cmd", &serde_json::json!({}), false, false);
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, ErrorCode::UnknownCommand);
    }
}

/// Plan a batch without executing it: resolve each command against the registry and
/// return what files would be created along with token estimates.
///
/// `tsx batch --json '<payload>' --plan`
pub fn batch_plan(payload: BatchPayload, _verbose: bool) -> ResponseEnvelope {
    let start = Instant::now();
    let registry = CommandRegistry::load_all();

    // Load path config from stack profile for output path expansion.
    let cwd = std::env::current_dir().unwrap_or_default();
    let stack = crate::stack::StackProfile::load(&cwd);
    let path_config = stack.as_ref().map(|p| &p.paths);

    let mut total_tokens: u32 = 0;
    let mut all_would_create: Vec<String> = vec![];

    let steps: Vec<serde_json::Value> = payload
        .commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            match registry.resolve(&cmd.command) {
                Some(spec) => {
                    let tokens = spec.token_estimate.unwrap_or(0);
                    total_tokens += tokens;

                    let would_create: Vec<String> = spec
                        .output_paths
                        .iter()
                        .map(|p| expand_plan_path(p, &cmd.options, path_config))
                        .collect();
                    all_would_create.extend(would_create.clone());

                    serde_json::json!({
                        "step": i + 1,
                        "command": cmd.command,
                        "package": spec.framework,
                        "token_estimate": tokens,
                        "would_create": would_create,
                        "next_steps": spec.next_steps,
                    })
                }
                None => serde_json::json!({
                    "step": i + 1,
                    "command": cmd.command,
                    "error": format!("Unknown command '{}' — run `tsx list` to see available commands", cmd.command),
                }),
            }
        })
        .collect();

    let duration_ms = start.elapsed().as_millis() as u64;
    ResponseEnvelope::success(
        "batch:plan",
        serde_json::json!({
            "steps": steps,
            "total_commands": payload.commands.len(),
            "total_token_estimate": total_tokens,
            "total_files": all_would_create.len(),
            "all_would_create": all_would_create,
        }),
        duration_ms,
    )
    .with_tokens_used(total_tokens)
}

/// Simple path template expander used for plan output (mirrors run.rs logic without
/// pulling in the full run machinery).
fn expand_plan_path(
    template: &str,
    options: &serde_json::Value,
    paths: Option<&crate::stack::PathConfig>,
) -> String {
    // Apply path prefix overrides first
    let expanded = if let Some(cfg) = paths {
        let overrides: &[(&str, Option<&str>)] = &[
            ("components/", cfg.components.as_deref()),
            ("routes/", cfg.routes.as_deref()),
            ("db/", cfg.db.as_deref()),
            ("server-functions/", cfg.server_fns.as_deref()),
            ("hooks/", cfg.hooks.as_deref()),
        ];
        let mut t = template.to_string();
        for (prefix, override_dir) in overrides {
            if let Some(dir) = override_dir {
                if t.starts_with(prefix) {
                    t = format!("{}/{}", dir.trim_end_matches('/'), &t[prefix.len()..]);
                    break;
                }
            }
        }
        t
    } else {
        template.to_string()
    };

    // Then expand {{field}} placeholders from options
    let Some(obj) = options.as_object() else {
        return expanded;
    };
    let mut result = expanded;
    for (key, value) in obj {
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

/// Public re-export of the batch command dispatcher for use by `generate.rs`.
pub fn execute_command_pub(
    command: &str,
    options: &serde_json::Value,
    overwrite: bool,
    dry_run: bool,
) -> Result<Vec<String>, (ErrorCode, String)> {
    execute_command(command, options, overwrite, dry_run)
}

fn execute_command(
    command: &str,
    options: &serde_json::Value,
    overwrite: bool,
    dry_run: bool,
) -> Result<Vec<String>, (ErrorCode, String)> {
    let options_str = serde_json::to_string(options)
        .map_err(|e| (ErrorCode::InvalidPayload, e.to_string()))?;

    macro_rules! dispatch {
        ($args_type:ty, $handler:expr) => {{
            let args: $args_type = serde_json::from_str(&options_str)
                .map_err(|e| (ErrorCode::ValidationError, e.to_string()))?;
            let result = $handler(args, overwrite, dry_run, false);
            if result.success {
                Ok(result.files_created)
            } else {
                Err((
                    ErrorCode::InternalError,
                    result.error.unwrap_or_else(|| "Command failed".to_string()),
                ))
            }
        }};
    }

    match command {
        "add:schema" => dispatch!(crate::schemas::AddSchemaArgs, crate::commands::add_schema::add_schema),
        "add:server-fn" => dispatch!(crate::schemas::AddServerFnArgs, crate::commands::add_server_fn::add_server_fn),
        "add:query" => dispatch!(crate::schemas::AddQueryArgs, crate::commands::add_query::add_query),
        "add:form" => dispatch!(crate::schemas::AddFormArgs, crate::commands::add_form::add_form),
        "add:table" => dispatch!(crate::schemas::AddTableArgs, crate::commands::add_table::add_table),
        "add:page" => dispatch!(crate::schemas::AddPageArgs, crate::commands::add_page::add_page),
        "add:seed" => dispatch!(crate::schemas::AddSeedArgs, crate::commands::add_seed::add_seed),
        "add:feature" => dispatch!(crate::schemas::AddFeatureArgs, crate::commands::add_feature::add_feature),
        "add:auth" => dispatch!(crate::schemas::AddAuthArgs, crate::commands::add_auth::add_auth),
        "add:auth-guard" => dispatch!(crate::schemas::AddAuthGuardArgs, crate::commands::add_auth_guard::add_auth_guard),
        // FPF fallback: resolve via registry and render forge templates
        _ => fpf_execute(command, options, overwrite, dry_run),
    }
}

/// Execute a command from any FPF package by rendering its forge templates.
///
/// Template convention: `frameworks/<pkg>/templates/<generator-id>/0.forge`,
/// `1.forge`, … where each file index corresponds to the matching entry in
/// `spec.output_paths`. The output path template is expanded with values from
/// `options` (e.g. `{{name}}` → the `name` field).
fn fpf_execute(
    command: &str,
    options: &serde_json::Value,
    overwrite: bool,
    dry_run: bool,
) -> Result<Vec<String>, (ErrorCode, String)> {
    let registry = CommandRegistry::load_all();
    let spec = registry
        .resolve(command)
        .ok_or_else(|| (ErrorCode::UnknownCommand, format!("Unknown command: {}", command)))?
        .clone();

    let cwd = std::env::current_dir().unwrap_or_default();
    let pkg_dir = find_fpf_package_dir(&spec.framework, &cwd)
        .ok_or_else(|| (ErrorCode::InternalError, format!("Package dir not found for '{}'", spec.framework)))?;

    let templates_dir = pkg_dir.join("templates").join(&spec.id);
    if !templates_dir.is_dir() {
        return Err((
            ErrorCode::InternalError,
            format!(
                "No templates found for '{}' — expected directory at {}",
                command,
                templates_dir.display()
            ),
        ));
    }

    // Collect N.forge files sorted numerically (0.forge, 1.forge, …).
    let mut template_files: Vec<(usize, std::path::PathBuf)> = std::fs::read_dir(&templates_dir)
        .map_err(|e| (ErrorCode::InternalError, e.to_string()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "forge"))
        .filter_map(|e| {
            let path = e.path();
            let stem = path.file_stem()?.to_str()?.to_string();
            let idx: usize = stem.parse().ok()?;
            Some((idx, path))
        })
        .collect();
    template_files.sort_by_key(|(idx, _)| *idx);

    if template_files.len() != spec.output_paths.len() {
        return Err((
            ErrorCode::InternalError,
            format!(
                "'{}' has {} output_paths but {} forge templates in {}",
                command,
                spec.output_paths.len(),
                template_files.len(),
                templates_dir.display()
            ),
        ));
    }

    // Build forge context from options.
    let mut ctx = forge::ForgeContext::new();
    if let Some(obj) = options.as_object() {
        for (k, v) in obj {
            match v {
                serde_json::Value::String(s) => ctx.insert_mut(k, s),
                serde_json::Value::Bool(b) => ctx.insert_mut(k, b),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        ctx.insert_mut(k, &i);
                    } else if let Some(f) = n.as_f64() {
                        ctx.insert_mut(k, &f);
                    }
                }
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                    ctx.insert_mut(k, v);
                }
                _ => {}
            }
        }
    }

    let mut engine = forge::Engine::new();
    // Load all templates for this generator.
    for (idx, path) in &template_files {
        let src = std::fs::read_to_string(path)
            .map_err(|e| (ErrorCode::InternalError, format!("Failed to read template {}: {}", idx, e)))?;
        let key = format!("{idx}.forge");
        engine
            .add_raw(&key, &src)
            .map_err(|e| (ErrorCode::InternalError, format!("Template parse error {}: {}", idx, e)))?;
    }

    let mut files_created: Vec<String> = Vec::new();

    for (idx, output_path_template) in spec.output_paths.iter().enumerate() {
        let key = format!("{idx}.forge");
        let rendered = engine
            .render(&key, &ctx)
            .map_err(|e| (ErrorCode::InternalError, format!("Render error for '{}' template {}: {}", command, idx, e)))?;

        // Expand {{field}} placeholders in the output path.
        let output_path = expand_output_path(output_path_template, options);

        if dry_run {
            files_created.push(output_path);
            continue;
        }

        // Write file.
        let dest = cwd.join(&output_path);
        if dest.exists() && !overwrite {
            // Skip existing files (matches legacy behaviour).
            files_created.push(output_path);
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| (ErrorCode::InternalError, format!("Could not create directory {}: {}", parent.display(), e)))?;
        }
        std::fs::write(&dest, rendered.trim_start())
            .map_err(|e| (ErrorCode::InternalError, format!("Could not write {}: {}", dest.display(), e)))?;
        files_created.push(output_path);
    }

    Ok(files_created)
}

/// Expand `{{field}}` placeholders in an output path template using input values.
fn expand_output_path(template: &str, options: &serde_json::Value) -> String {
    let Some(obj) = options.as_object() else {
        return template.to_string();
    };
    let mut result = template.to_string();
    for (key, value) in obj {
        if key.starts_with("__") {
            continue;
        }
        if let Some(s) = value.as_str() {
            result = result.replace(&format!("{{{{{}}}}}", key), s);
        }
    }
    result
}

/// Locate a package directory by slug across builtin, .tsx/packages, and .tsx/frameworks.
fn find_fpf_package_dir(slug: &str, cwd: &std::path::Path) -> Option<std::path::PathBuf> {
    use crate::utils::paths::get_frameworks_dir;
    let builtin = get_frameworks_dir().join(slug);
    if builtin.is_dir() { return Some(builtin); }
    let fpf = cwd.join(".tsx").join("packages").join(slug);
    if fpf.is_dir() { return Some(fpf); }
    let legacy = cwd.join(".tsx").join("frameworks").join(slug);
    if legacy.is_dir() { return Some(legacy); }
    None
}
