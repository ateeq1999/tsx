//! Pattern Pack runner — renders templates and writes output files.
//!
//! Entry point: [`run_pack`].
//!
//! ## Flow
//!
//! 1. Resolve which `PackCommand` to execute (named or default).
//! 2. Apply arg defaults; validate required args.
//! 3. Load all `.forge` templates from the pack directory into an `Engine`.
//! 4. For each output in the command: render template → write file (or print in dry-run).
//! 5. Apply marker injections into existing project files (idempotent).
//! 6. Run post-hooks.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::engine::Engine;
use crate::context::ForgeContext;
use crate::pack::PackManifest;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Options controlling how `run_pack` behaves.
#[derive(Debug, Default, Clone)]
pub struct RunOpts {
    /// Print what would happen without writing any files.
    pub dry_run: bool,
    /// Overwrite existing output files (default: skip them).
    pub overwrite: bool,
    /// Which pack command to run. `None` → use the default command.
    pub command: Option<String>,
    /// Compute a unified diff against existing files instead of writing.
    /// Implies dry-run (no files are written).
    pub diff: bool,
}

/// Result of a `run_pack` call.
#[derive(Debug, Default)]
pub struct RunResult {
    /// Files written (or would be written in dry-run).
    pub files_written: Vec<PathBuf>,
    /// Files skipped because they already exist and `overwrite` is false.
    pub files_skipped: Vec<PathBuf>,
    /// `(file, inserted_line)` pairs where a marker injection succeeded.
    pub markers_injected: Vec<(PathBuf, String)>,
    /// Shell commands that were executed.
    pub hooks_run: Vec<String>,
    /// Unified diffs computed in diff mode: `(file_path, diff_text)`.
    pub diffs: Vec<(PathBuf, String)>,
}

/// Render and write a pattern pack to a project.
///
/// # Arguments
///
/// * `pack`         — the parsed `pack.json` manifest
/// * `pack_dir`     — directory that contains the pack's `.forge` files
/// * `args`         — user-supplied arg values (strings, bools, arrays)
/// * `project_root` — root of the target project (files are written relative to this)
/// * `opts`         — dry-run / overwrite / command selection
pub fn run_pack(
    pack: &PackManifest,
    pack_dir: &Path,
    args: HashMap<String, Value>,
    project_root: &Path,
    opts: &RunOpts,
) -> Result<RunResult, PackRunError> {
    // 1. Resolve command
    let cmd = pack
        .resolve_command(opts.command.as_deref())
        .ok_or_else(|| PackRunError::NoCommand(pack.id.clone()))?;

    // 2. Merge defaults + validate
    let args = pack.apply_defaults(args);
    let missing = pack.missing_required(&args);
    if !missing.is_empty() {
        return Err(PackRunError::MissingArgs(missing));
    }

    // 3. Build forge engine from pack directory
    let mut engine = Engine::new();
    engine
        .load_dir(pack_dir)
        .map_err(|e| PackRunError::Engine(e.to_string()))?;

    // 4. Build forge context
    let mut ctx = ForgeContext::new();
    for (k, v) in &args {
        ctx.insert_mut(k, v);
    }

    let mut result = RunResult::default();

    // 5. Render outputs
    for output in pack.command_outputs(cmd) {
        let rendered_path = interpolate_path(&output.path, &args)
            .map_err(|e| PackRunError::PathInterpolation(output.path.clone(), e))?;

        let target = project_root.join(&rendered_path);

        let effective_dry = opts.dry_run || opts.diff;

        if target.exists() && !opts.overwrite && !effective_dry {
            result.files_skipped.push(target);
            continue;
        }

        let content = engine
            .render(&output.template, &ctx)
            .map_err(|e| PackRunError::Render(output.template.clone(), e.to_string()))?;

        if opts.diff {
            let old = std::fs::read_to_string(&target).ok();
            let diff = simple_diff(
                &target.to_string_lossy(),
                old.as_deref(),
                &content,
            );
            result.diffs.push((target.clone(), diff));
            result.files_written.push(target);
        } else if opts.dry_run {
            result.files_written.push(target);
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| PackRunError::Io(target.clone(), e.to_string()))?;
            }
            std::fs::write(&target, &content)
                .map_err(|e| PackRunError::Io(target.clone(), e.to_string()))?;
            result.files_written.push(target);
        }
    }

    // 6. Apply markers
    for marker in &pack.markers {
        let insert_line = interpolate_path(&marker.insert, &args)
            .map_err(|e| PackRunError::PathInterpolation(marker.insert.clone(), e))?;

        let target = project_root.join(&marker.file);
        if !target.exists() {
            continue;
        }

        let injected = inject_marker(&target, &marker.marker, &insert_line, opts.dry_run)
            .map_err(|e| PackRunError::Io(target.clone(), e))?;

        if injected {
            result.markers_injected.push((target, insert_line));
        }
    }

    // 7. Post hooks
    if !opts.dry_run {
        let hook_keys: Vec<&str> = opts
            .command
            .as_deref()
            .map(|k| vec![k, "all"])
            .unwrap_or_else(|| vec!["all"]);

        let mut seen = std::collections::HashSet::new();
        for key in hook_keys {
            if let Some(hooks) = pack.post_hooks.get(key) {
                for hook in hooks {
                    if seen.insert(hook.clone()) {
                        run_hook(hook, project_root);
                        result.hooks_run.push(hook.clone());
                    }
                }
            }
        }
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Path interpolation
// ---------------------------------------------------------------------------

/// Render a path template (e.g. `"src/{{ name | snake_case }}.ts"`) using pack args.
///
/// Uses a fresh `Engine` with case filters registered — no forge `@` preprocessing,
/// since path templates use plain Tera `{{ }}` syntax.
pub fn interpolate_path(
    template: &str,
    args: &HashMap<String, Value>,
) -> Result<String, String> {
    if !template.contains("{{") && !template.contains("{%") {
        return Ok(template.to_string());
    }

    // Use a temporary Engine (already has all forge filters registered)
    let mut engine = Engine::new();
    engine
        .add_raw("__path__", template)
        .map_err(|e| e.to_string())?;

    let mut ctx = ForgeContext::new();
    for (k, v) in args {
        ctx.insert_mut(k, v);
    }

    engine.render("__path__", &ctx).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Marker injection
// ---------------------------------------------------------------------------

/// Find `marker` in `file` and insert `line` immediately after it, idempotently.
///
/// Returns `true` if the injection was performed (or would be in dry-run mode).
/// Returns `false` if the marker is absent or `line` is already present.
pub fn inject_marker(
    file: &Path,
    marker: &str,
    line: &str,
    dry_run: bool,
) -> Result<bool, String> {
    let content = std::fs::read_to_string(file).map_err(|e| e.to_string())?;

    // Already injected — idempotent
    if content.contains(line) {
        return Ok(false);
    }
    // Marker must be present
    if !content.contains(marker) {
        return Ok(false);
    }

    if dry_run {
        return Ok(true);
    }

    // Insert the line immediately after the marker line
    let new_content = content.replacen(marker, &format!("{marker}\n{line}"), 1);
    std::fs::write(file, new_content).map_err(|e| e.to_string())?;
    Ok(true)
}

// ---------------------------------------------------------------------------
// Diff helpers
// ---------------------------------------------------------------------------

/// Produce a simple unified-style diff string for display.
///
/// Not a minimal-edit LCS diff — shows all removed lines then all added lines
/// per changed hunk, which is clear enough for CLI output without a heavy dep.
fn simple_diff(path: &str, old: Option<&str>, new_content: &str) -> String {
    let new_lines: Vec<&str> = new_content.lines().collect();

    let Some(old_content) = old else {
        // New file: all lines are additions
        let mut out = format!("--- /dev/null\n+++ b/{path}\n@@ -0,0 +1,{} @@\n", new_lines.len());
        for line in &new_lines {
            out.push('+');
            out.push_str(line);
            out.push('\n');
        }
        return out;
    };

    let old_lines: Vec<&str> = old_content.lines().collect();
    if old_lines == new_lines {
        return format!("(no change: {path})\n");
    }

    let mut out = format!("--- a/{path}\n+++ b/{path}\n@@ modified @@\n");
    for line in &old_lines {
        out.push('-');
        out.push_str(line);
        out.push('\n');
    }
    for line in &new_lines {
        out.push('+');
        out.push_str(line);
        out.push('\n');
    }
    out
}

// ---------------------------------------------------------------------------
// Hook execution
// ---------------------------------------------------------------------------

fn run_hook(hook: &str, cwd: &Path) {
    let (shell, flag) = if cfg!(windows) {
        ("cmd", "/c")
    } else {
        ("sh", "-c")
    };
    let _ = std::process::Command::new(shell)
        .arg(flag)
        .arg(hook)
        .current_dir(cwd)
        .status();
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum PackRunError {
    NoCommand(String),
    MissingArgs(Vec<String>),
    Engine(String),
    Render(String, String),
    PathInterpolation(String, String),
    Io(PathBuf, String),
}

impl std::fmt::Display for PackRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackRunError::NoCommand(id) =>
                write!(f, "No command found for pack '{id}'"),
            PackRunError::MissingArgs(args) =>
                write!(f, "Missing required args: {}", args.join(", ")),
            PackRunError::Engine(e) =>
                write!(f, "Engine error: {e}"),
            PackRunError::Render(tmpl, e) =>
                write!(f, "Render error in '{tmpl}': {e}"),
            PackRunError::PathInterpolation(path, e) =>
                write!(f, "Path interpolation failed for '{path}': {e}"),
            PackRunError::Io(path, e) =>
                write!(f, "I/O error on '{}': {e}", path.display()),
        }
    }
}

impl std::error::Error for PackRunError {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn interpolate_plain_path() {
        let args = HashMap::new();
        assert_eq!(
            interpolate_path("src/utils.ts", &args).unwrap(),
            "src/utils.ts"
        );
    }

    #[test]
    fn interpolate_snake_case() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), Value::String("TodoItem".to_string()));
        let result = interpolate_path("src/{{ name | snake_case }}.ts", &args).unwrap();
        assert_eq!(result, "src/todo_item.ts");
    }

    #[test]
    fn interpolate_pascal_case() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), Value::String("todo".to_string()));
        let result = interpolate_path("src/components/{{ name | pascal_case }}Form.tsx", &args).unwrap();
        assert_eq!(result, "src/components/TodoForm.tsx");
    }

    #[test]
    fn inject_marker_inserts_line() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("index.ts");
        std::fs::write(&file, "// [tsx:schemas]\nexport * from './users';\n").unwrap();

        let injected = inject_marker(
            &file,
            "// [tsx:schemas]",
            "export * from './todos';",
            false,
        ).unwrap();

        assert!(injected);
        let content = std::fs::read_to_string(&file).unwrap();
        assert!(content.contains("export * from './todos';"));
    }

    #[test]
    fn inject_marker_idempotent() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("index.ts");
        std::fs::write(&file, "// [tsx:schemas]\nexport * from './todos';\n").unwrap();

        let injected = inject_marker(
            &file,
            "// [tsx:schemas]",
            "export * from './todos';",
            false,
        ).unwrap();

        assert!(!injected); // already present
    }

    #[test]
    fn inject_marker_dry_run_does_not_write() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("index.ts");
        let original = "// [tsx:schemas]\nexport * from './users';\n";
        std::fs::write(&file, original).unwrap();

        inject_marker(&file, "// [tsx:schemas]", "export * from './todos';", true).unwrap();

        // File must be unchanged in dry-run
        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, original);
    }

    #[test]
    fn run_pack_writes_file() {
        let dir = TempDir::new().unwrap();

        // Create pack
        let pack_dir = dir.path().join(".tsx").join("patterns").join("my-pack");
        std::fs::create_dir_all(&pack_dir).unwrap();
        std::fs::write(
            pack_dir.join("main.forge"),
            "export const {{ name | pascal_case }} = true;\n",
        ).unwrap();

        let mut commands = HashMap::new();
        commands.insert("all".to_string(), crate::pack::PackCommand {
            description: "all".to_string(),
            outputs: vec!["main".to_string()],
            default: true,
        });

        let pack = PackManifest {
            id: "my-pack".to_string(),
            outputs: vec![crate::pack::PackOutput {
                id: "main".to_string(),
                template: "main.forge".to_string(),
                path: "src/{{ name | snake_case }}.ts".to_string(),
            }],
            commands,
            ..Default::default()
        };

        let mut args = HashMap::new();
        args.insert("name".to_string(), Value::String("todo".to_string()));

        let result = run_pack(
            &pack,
            &pack_dir,
            args,
            dir.path(),
            &RunOpts::default(),
        ).unwrap();

        assert_eq!(result.files_written.len(), 1);
        let content = std::fs::read_to_string(dir.path().join("src/todo.ts")).unwrap();
        assert!(content.contains("export const Todo = true;"));
    }
}
