use anyhow::Result;
use std::path::PathBuf;

use crate::output::CommandResult;
use crate::render::engine::{build_engine_with_plugins, reset_import_collector};
use crate::utils::paths::{find_project_root, get_plugin_template_dirs, get_templates_dir};
use crate::utils::write::{write_file, WriteOutcome};

/// Shared render-and-write pipeline for all single-file generate commands.
///
/// 1. Finds the project root via `package.json` walk.
/// 2. Builds the MiniJinja engine and resets the import collector.
/// 3. Renders `template_name` with `ctx`.
/// 4. Formats the output with `format_fn` (falls back to unformatted on error).
/// 5. Writes (or skips) the file at `build_output_path(root)`.
/// 6. Returns a `CommandResult` with the created file path.
pub fn render_and_write<F>(
    command: &str,
    template_name: &str,
    ctx: minijinja::Value,
    build_output_path: F,
    format_fn: fn(&str) -> Result<String>,
    overwrite: bool,
    dry_run: bool,
) -> CommandResult
where
    F: FnOnce(&PathBuf) -> PathBuf,
{
    let root = match find_project_root() {
        Ok(r) => r,
        Err(e) => return CommandResult::err(command, e.to_string()),
    };

    let output_path = build_output_path(&root);
    let templates_dir = get_templates_dir(&root);
    let plugin_dirs = get_plugin_template_dirs(&root);
    let engine = build_engine_with_plugins(&templates_dir, &plugin_dirs);

    reset_import_collector();

    let template = match engine.get_template(template_name) {
        Ok(t) => t,
        Err(e) => {
            return CommandResult::err(
                command,
                format!(
                    "Template not found: {} — {}",
                    template_name, e
                ),
            )
        }
    };

    let rendered = match template.render(ctx) {
        Ok(r) => r,
        Err(e) => {
            return CommandResult::err(
                command,
                format!(
                    "Render failed for template {}: {}",
                    template_name, e
                ),
            )
        }
    };

    let (formatted, format_warning) = match format_fn(&rendered) {
        Ok(f) => (f, None),
        Err(e) => (
            rendered,
            Some(format!(
                "Formatter failed for {}: {} (writing unformatted output)",
                output_path.display(),
                e
            )),
        ),
    };

    let files_created = if dry_run {
        vec![output_path.to_string_lossy().to_string()]
    } else {
        match write_file(&output_path, &formatted, overwrite) {
            Ok(WriteOutcome::Created | WriteOutcome::Overwritten) => {
                vec![output_path.to_string_lossy().to_string()]
            }
            Ok(WriteOutcome::Skipped) => vec![],
            Err(e) => {
                return CommandResult::err(
                    command,
                    format!("Failed to write {}: {}", output_path.display(), e),
                )
            }
        }
    };

    let mut result = CommandResult::ok(command, files_created);
    if let Some(warning) = format_warning {
        result.warnings.push(warning);
    }
    result
}
