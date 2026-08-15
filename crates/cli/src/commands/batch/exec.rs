//! Single-command execution — the shared machinery `batch`, `run`, `generate`,
//! `repl`, and `replay` all use to dispatch one command by id.

use crate::framework::command_registry::CommandRegistry;
use crate::json::error::ErrorCode;

/// Public re-export of the batch command dispatcher for use by `generate.rs`,
/// `repl.rs`, `replay.rs`, and `run.rs`.
pub fn execute_command_pub(
    command: &str,
    options: &serde_json::Value,
    overwrite: bool,
    dry_run: bool,
) -> Result<Vec<String>, (ErrorCode, String)> {
    execute_command(command, options, overwrite, dry_run)
}

pub(super) fn execute_command(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_command_unknown_returns_unknown_command_error() {
        let result = execute_command("unknown:cmd", &serde_json::json!({}), false, false);
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, ErrorCode::UnknownCommand);
    }
}
