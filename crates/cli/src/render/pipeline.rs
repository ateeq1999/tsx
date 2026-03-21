use anyhow::Result;
use std::path::PathBuf;

use crate::output::CommandResult;
use crate::render::engine::{build_engine_with_plugins, reset_import_collector};
use crate::utils::paths::{find_project_root, get_plugin_template_dirs, get_templates_dir};
use crate::utils::write::{write_file, WriteOutcome};

// ---------------------------------------------------------------------------
// C2: style context injection
// ---------------------------------------------------------------------------

/// Load the effective style from `.tsx/stack.json` + `user-stack.json` and
/// merge it into the incoming minijinja context as the `style` variable.
///
/// This lets templates branch on `style.forms`, `style.css`, etc. without
/// each generator having to load the stack manually.
fn inject_style(ctx: minijinja::Value, root: &PathBuf) -> minijinja::Value {
    use crate::stack::{StackProfile, UserStack};

    let base_style = StackProfile::load(root)
        .map(|s| s.style)
        .unwrap_or_default();

    let effective = UserStack::load(root)
        .map(|u| u.effective_style(&base_style))
        .unwrap_or_else(|| crate::stack::EffectiveStyle {
            quotes:     base_style.quotes.clone(),
            indent:     base_style.indent,
            semicolons: base_style.semicolons,
            css:        base_style.css.clone().unwrap_or_default(),
            components: base_style.components.clone().unwrap_or_default(),
            forms:      base_style.forms.clone().unwrap_or_default(),
            icons:      base_style.icons.clone().unwrap_or_default(),
            toast:      base_style.toast.clone().unwrap_or_default(),
        });

    // Merge via JSON: convert ctx → JSON object, add style, convert back
    let mut json: serde_json::Map<String, serde_json::Value> =
        serde_json::to_value(&ctx)
            .ok()
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();

    if let Ok(style_json) = serde_json::to_value(&effective) {
        json.insert("style".to_string(), style_json);
    }

    minijinja::Value::from_serialize(&json)
}

/// Shared render-and-write pipeline for all single-file generate commands.
///
/// 1. Finds the project root via `package.json` walk.
/// 2. Builds the MiniJinja engine and resets the import collector.
/// 3. Renders `template_name` with `ctx`.
/// 4. Formats the output with `format_fn` (falls back to unformatted on error).
/// 5. Writes (or skips) the file at `build_output_path(root)`.
/// 6. Returns a `CommandResult` with the created file path.
///
/// When `diff_only = true` the file is **not** written; instead a line-based diff
/// between the existing file and the generated content is placed in `result.warnings`
/// prefixed with `"diff:"`.  This implements `tsx generate <cmd> --diff`.
pub fn render_and_write<F>(
    command: &str,
    template_name: &str,
    ctx: minijinja::Value,
    build_output_path: F,
    format_fn: fn(&str) -> Result<String>,
    overwrite: bool,
    dry_run: bool,
    diff_only: bool,
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

    // C2: inject effective style so templates can branch on style.forms etc.
    let ctx = inject_style(ctx, &root);

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

    // ── diff_only mode ────────────────────────────────────────────────────────
    if diff_only {
        let existing = std::fs::read_to_string(&output_path).unwrap_or_default();
        let diff = compute_diff(output_path.to_string_lossy().as_ref(), &existing, &formatted);
        let mut result = CommandResult::ok(command, vec![output_path.to_string_lossy().to_string()]);
        result.warnings.push(format!("diff:{}", diff));
        if let Some(w) = format_warning {
            result.warnings.push(w);
        }
        return result;
    }

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

/// Produce a simple unified-style diff between `old` and `new` content.
pub fn compute_diff(path: &str, old: &str, new: &str) -> String {
    if old == new {
        return format!("--- {}\n(no changes)\n", path);
    }

    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let mut out = String::new();
    let label_a = if old.is_empty() { "/dev/null" } else { &format!("a/{}", path) };
    let label_b = format!("b/{}", path);
    out.push_str(&format!("--- {}\n", label_a));
    out.push_str(&format!("+++ {}\n", label_b));

    // Simple diff: emit removed/added lines without LCS (sufficient for reviewer context)
    let max = old_lines.len().max(new_lines.len());
    let mut chunk_start = None;
    let mut chunk: Vec<String> = Vec::new();

    for i in 0..max {
        let ol = old_lines.get(i).copied();
        let nl = new_lines.get(i).copied();
        if ol != nl {
            if chunk_start.is_none() {
                chunk_start = Some(i + 1);
            }
            if let Some(l) = ol { chunk.push(format!("-{}", l)); }
            if let Some(l) = nl { chunk.push(format!("+{}", l)); }
        } else if !chunk.is_empty() {
            let start = chunk_start.unwrap_or(1);
            out.push_str(&format!("@@ -{} +{} @@\n", start, start));
            for line in &chunk { out.push_str(line); out.push('\n'); }
            chunk.clear();
            chunk_start = None;
        }
    }
    if !chunk.is_empty() {
        let start = chunk_start.unwrap_or(1);
        out.push_str(&format!("@@ -{} +{} @@\n", start, start));
        for line in &chunk { out.push_str(line); out.push('\n'); }
    }

    // Trailing lines
    if new_lines.len() > old_lines.len() {
        let start = old_lines.len() + 1;
        out.push_str(&format!("@@ -{},{} +{},{} @@\n", start, 0, start, new_lines.len() - old_lines.len()));
        for l in &new_lines[old_lines.len()..] { out.push_str(&format!("+{}\n", l)); }
    } else if old_lines.len() > new_lines.len() {
        let start = new_lines.len() + 1;
        out.push_str(&format!("@@ -{},{} +{},{} @@\n", start, old_lines.len() - new_lines.len(), start, 0));
        for l in &old_lines[new_lines.len()..] { out.push_str(&format!("-{}\n", l)); }
    }

    out
}
