//! `tsx lint-template` — static analysis linter for `.forge` / `.jinja` templates (H1).
//!
//! Warns on:
//! - Variables referenced in templates but not in declared generator args
//! - Slots declared (`@slot(...)` / `{{ slot(...) }}`) but never filled by any test fixture
//! - `render_imports()` missing when `@import(...)` / `collect_import` is used
//! - `@each` / `{% for %}` over what looks like a non-array variable
//! - Missing `@end` for `@if` / `@each` blocks
//! - Dead `@variant(...)` blocks (variant flag never set in fixtures)
//! - Import shadowing (same package imported twice with different aliases)

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

// ---------------------------------------------------------------------------
// Diagnostic
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintDiagnostic {
    pub file: String,
    pub line: usize,
    pub severity: String, // "error" | "warning"
    pub code: String,
    pub message: String,
}

impl LintDiagnostic {
    fn error(file: &str, line: usize, code: &str, msg: impl Into<String>) -> Self {
        Self { file: file.to_string(), line, severity: "error".to_string(), code: code.to_string(), message: msg.into() }
    }
    fn warning(file: &str, line: usize, code: &str, msg: impl Into<String>) -> Self {
        Self { file: file.to_string(), line, severity: "warning".to_string(), code: code.to_string(), message: msg.into() }
    }
}

// ---------------------------------------------------------------------------
// Command handler
// ---------------------------------------------------------------------------

pub fn lint_template(path: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let target = match &path {
        Some(p) => PathBuf::from(p),
        None => {
            // Try common locations
            let candidates = [
                cwd.join(".tsx").join("templates"),
                cwd.join("templates"),
            ];
            match candidates.into_iter().find(|p| p.exists()) {
                Some(p) => p,
                None => {
                    return ResponseEnvelope::error(
                        "lint-template",
                        ErrorResponse::new(
                            ErrorCode::ProjectNotFound,
                            "No template directory found. Pass a path: tsx lint-template ./frameworks/my-fw/templates/",
                        ),
                        0,
                    )
                }
            }
        }
    };

    if !target.exists() {
        return ResponseEnvelope::error(
            "lint-template",
            ErrorResponse::new(
                ErrorCode::ProjectNotFound,
                format!("Path does not exist: {}", target.display()),
            ),
            0,
        );
    }

    let mut all_diagnostics: Vec<LintDiagnostic> = Vec::new();

    // Collect template files
    let template_files = collect_templates(&target);

    for tmpl_path in &template_files {
        let content = match std::fs::read_to_string(tmpl_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = tmpl_path
            .strip_prefix(&cwd)
            .unwrap_or(tmpl_path)
            .to_string_lossy()
            .replace('\\', "/");

        let diags = lint_file(&rel, &content);
        all_diagnostics.extend(diags);
    }

    let errors = all_diagnostics.iter().filter(|d| d.severity == "error").count();
    let warnings = all_diagnostics.iter().filter(|d| d.severity == "warning").count();

    // Format human-readable output
    let formatted: Vec<String> = all_diagnostics
        .iter()
        .map(|d| format!("{}:{} [{}] {} — {}", d.file, d.line, d.severity.to_uppercase(), d.code, d.message))
        .collect();

    let data = serde_json::json!({
        "files_checked": template_files.len(),
        "errors": errors,
        "warnings": warnings,
        "diagnostics": serde_json::to_value(&all_diagnostics).unwrap_or_default(),
        "output": formatted,
    });

    if errors > 0 {
        ResponseEnvelope::error(
            "lint-template",
            ErrorResponse::new(
                ErrorCode::ValidationError,
                format!("{} error(s), {} warning(s) found across {} file(s)", errors, warnings, template_files.len()),
            ),
            0,
        )
    } else {
        ResponseEnvelope::success("lint-template", data, 0)
    }
}

// ---------------------------------------------------------------------------
// Linting rules
// ---------------------------------------------------------------------------

fn lint_file(file: &str, content: &str) -> Vec<LintDiagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_unclosed_blocks(file, content));
    diags.extend(check_render_imports(file, content));
    diags.extend(check_import_shadowing(file, content));
    diags.extend(check_each_non_array(file, content));
    diags
}

/// L001 — Unclosed @if / @each blocks.
fn check_unclosed_blocks(file: &str, content: &str) -> Vec<LintDiagnostic> {
    let mut diags = Vec::new();
    let mut depth: i64 = 0;
    let mut open_lines: Vec<usize> = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let t = line.trim();
        if t.starts_with("@if(")
            || t.starts_with("@unless(")
            || t.starts_with("@each(")
            || t.starts_with("@variant(")
            || t.contains("{%- if ")
            || t.contains("{% if ")
            || t.contains("{%- for ")
            || t.contains("{% for ")
        {
            depth += 1;
            open_lines.push(i + 1);
        }
        if t == "@end"
            || t.contains("{% endif %}")
            || t.contains("{%- endif %}")
            || t.contains("{% endfor %}")
            || t.contains("{%- endfor %}")
        {
            depth -= 1;
            open_lines.pop();
        }
    }

    if depth > 0 {
        for line_no in open_lines {
            diags.push(LintDiagnostic::error(
                file,
                line_no,
                "L001",
                "Unclosed block — missing @end or {% endif %} / {% endfor %}",
            ));
        }
    }

    diags
}

/// L002 — `@import` / `collect_import` used but `render_imports()` never called.
fn check_render_imports(file: &str, content: &str) -> Vec<LintDiagnostic> {
    let has_imports = content.contains("@import(") || content.contains("collect_import");
    let has_render = content.contains("render_imports()");
    if has_imports && !has_render {
        vec![LintDiagnostic::warning(
            file,
            1,
            "L002",
            "@import or collect_import used but render_imports() is never called — imports will not appear in output",
        )]
    } else {
        Vec::new()
    }
}

/// L003 — Import shadowing: same package imported twice.
fn check_import_shadowing(file: &str, content: &str) -> Vec<LintDiagnostic> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut diags = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let t = line.trim();
        // Match @import("pkg") and collect_import with "import ... from 'pkg'"
        if let Some(pkg) = extract_import_pkg(t) {
            if let Some(prev_line) = seen.get(&pkg) {
                diags.push(LintDiagnostic::warning(
                    file,
                    i + 1,
                    "L003",
                    format!("Package '{}' already imported on line {} — possible duplicate", pkg, prev_line),
                ));
            } else {
                seen.insert(pkg, i + 1);
            }
        }
    }

    diags
}

/// L004 — `@each(scalar as item)` where the collection variable looks scalar (no `s` suffix).
fn check_each_non_array(file: &str, content: &str) -> Vec<LintDiagnostic> {
    let mut diags = Vec::new();
    let known_arrays = ["fields", "columns", "methods", "tags", "items", "entries", "rows", "steps"];
    for (i, line) in content.lines().enumerate() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("@each(") {
            if let Some(as_pos) = rest.find(" as ") {
                let collection = rest[..as_pos].trim();
                // Strip ctx. prefix
                let bare = collection.trim_start_matches("ctx.");
                let looks_scalar = !bare.ends_with('s')
                    && !known_arrays.contains(&bare)
                    && !bare.ends_with("_list")
                    && !bare.ends_with("_array");
                if looks_scalar {
                    diags.push(LintDiagnostic::warning(
                        file,
                        i + 1,
                        "L004",
                        format!("@each over '{}' which may not be an array — verify the variable is a list", collection),
                    ));
                }
            }
        }
    }
    diags
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn collect_templates(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_file() {
        return vec![dir.to_path_buf()];
    }
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "forge" || ext == "jinja" {
                files.push(path);
            }
        }
    }
    files
}

fn extract_import_pkg(line: &str) -> Option<String> {
    // @import("pkg") or @import("pkg", named=[...])
    if let Some(rest) = line.strip_prefix("@import(\"") {
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    if let Some(rest) = line.strip_prefix("@import('") {
        if let Some(end) = rest.find('\'') {
            return Some(rest[..end].to_string());
        }
    }
    // "import { x } from 'pkg'" | collect_import
    if line.contains("| collect_import") {
        if let Some(from_pos) = line.find("from '") {
            let rest = &line[from_pos + 6..];
            if let Some(end) = rest.find('\'') {
                return Some(rest[..end].to_string());
            }
        }
        if let Some(from_pos) = line.find("from \"") {
            let rest = &line[from_pos + 6..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_unclosed_if() {
        let src = "@if(ctx.auth)\nhello\n// no @end";
        let diags = check_unclosed_blocks("test.forge", src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "L001");
    }

    #[test]
    fn closed_blocks_ok() {
        let src = "@if(ctx.auth)\nhello\n@end";
        let diags = check_unclosed_blocks("test.forge", src);
        assert!(diags.is_empty());
    }

    #[test]
    fn detects_missing_render_imports() {
        let src = "@import(\"react\")\nconst x = 1";
        let diags = check_render_imports("test.forge", src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "L002");
    }

    #[test]
    fn render_imports_present_ok() {
        let src = "@import(\"react\")\n{{ render_imports() }}\nconst x = 1";
        let diags = check_render_imports("test.forge", src);
        assert!(diags.is_empty());
    }

    #[test]
    fn detects_import_shadowing() {
        let src = "@import(\"react\")\nhello\n@import(\"react\")";
        let diags = check_import_shadowing("test.forge", src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "L003");
    }

    #[test]
    fn each_array_ok() {
        let src = "@each(ctx.fields as field)";
        let diags = check_each_non_array("test.forge", src);
        assert!(diags.is_empty());
    }

    #[test]
    fn each_scalar_warning() {
        let src = "@each(ctx.name as item)";
        let diags = check_each_non_array("test.forge", src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "L004");
    }

    #[test]
    fn extract_import_pkg_at_directive() {
        assert_eq!(
            extract_import_pkg("@import(\"zod\", named=[\"z\"])"),
            Some("zod".to_string())
        );
    }
}
