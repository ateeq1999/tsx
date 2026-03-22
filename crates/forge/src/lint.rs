//! Template linting for `.forge` files.
//!
//! [`lint_template`] performs static analysis on raw forge source and returns a
//! [`LintResult`] with categorised errors, warnings, and suggestions.
//!
//! # What is checked
//!
//! | Check | Severity | Code |
//! |-------|----------|------|
//! | Unclosed `@if` / `@for` / `@each` / `@slot` / `@macro` / `@feature` / `@variant` block | Error | F003 |
//! | Orphan `@end` with no matching open block | Error | F003 |
//! | `@extends` not on the first non-blank line | Warning | — |
//! | Unknown `@`-directive name | Warning | — |
//! | `@schema` present but has no `required` fields (all optional) | Suggestion | — |
//! | Raw `import` statement instead of `@import` | Suggestion | — |

use crate::error::ForgeError;

// ---------------------------------------------------------------------------
// Public result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintError {
    pub message: String,
    pub line: usize,
    pub code: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintWarning {
    pub message: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintSuggestion {
    pub message: String,
    pub line: usize,
}

#[derive(Debug, Default, Clone)]
pub struct LintResult {
    pub errors: Vec<LintError>,
    pub warnings: Vec<LintWarning>,
    pub suggestions: Vec<LintSuggestion>,
}

impl LintResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Convert all errors into a single [`ForgeError::LintError`].
    pub fn into_forge_error(self) -> ForgeError {
        let msgs: Vec<String> = self
            .errors
            .into_iter()
            .map(|e| format!("line {}: [{}] {}", e.line, e.code, e.message))
            .collect();
        ForgeError::LintError(msgs.join("\n"))
    }
}

// ---------------------------------------------------------------------------
// Known block-opening directives
// ---------------------------------------------------------------------------

const BLOCK_OPENERS: &[&str] = &[
    "if", "unless", "each", "for", "slot", "macro", "feature", "variant",
];

const KNOWN_DIRECTIVES: &[&str] = &[
    "if", "unless", "each", "for", "end",
    "import", "set", "slot", "inject", "include",
    "variant", "feature", "extends", "schema",
    "macro", "call", "hook", "block",
];

// ---------------------------------------------------------------------------
// Main linting function
// ---------------------------------------------------------------------------

/// Lint raw forge template source.
///
/// Returns a [`LintResult`] that may contain errors, warnings, and suggestions.
/// Does **not** render the template.
pub fn lint_template(src: &str) -> LintResult {
    let mut result = LintResult::default();

    // Block-stack tracking: (directive_name, line_number)
    let mut block_stack: Vec<(&'static str, usize)> = Vec::new();

    let mut seen_extends = false;
    let mut extends_line: Option<usize> = None;
    let mut first_content_line: Option<usize> = None;

    for (idx, line) in src.lines().enumerate() {
        let lineno = idx + 1;
        let trimmed = line.trim_start();

        // Track first non-blank, non-comment line
        if first_content_line.is_none()
            && !trimmed.is_empty()
            && !trimmed.starts_with("{#")
        {
            first_content_line = Some(lineno);
        }

        if let Some(rest) = trimmed.strip_prefix('@') {
            // Skip method-chain expressions like @ctx.name.pascal() — not directives
            if is_method_chain(rest) {
                continue;
            }
            let directive = directive_name(rest);

            match directive {
                // ── @end ──────────────────────────────────────────────────
                "end" => {
                    if block_stack.pop().is_none() {
                        result.errors.push(LintError {
                            message: "@end without a matching open block".to_string(),
                            line: lineno,
                            code: "F003",
                        });
                    }
                }

                // ── @extends ──────────────────────────────────────────────
                "extends" => {
                    seen_extends = true;
                    extends_line = Some(lineno);
                }

                // ── @schema ───────────────────────────────────────────────
                "schema" => {
                    check_schema_suggestions(trimmed, lineno, &mut result);
                }

                // ── block-opening directives ───────────────────────────────
                name if BLOCK_OPENERS.contains(&name) => {
                    // Leak the name as a static-lifetime &str via a known set
                    let static_name = BLOCK_OPENERS
                        .iter()
                        .find(|&&n| n == name)
                        .copied()
                        .unwrap_or("unknown");
                    block_stack.push((static_name, lineno));
                }

                // ── unknown directive ──────────────────────────────────────
                name if !KNOWN_DIRECTIVES.contains(&name) && !name.is_empty() => {
                    // Ignore method-chain expressions like @ctx.name.pascal()
                    if !name.contains('.') {
                        result.warnings.push(LintWarning {
                            message: format!("Unknown directive '@{name}'"),
                            line: lineno,
                        });
                    }
                }

                _ => {}
            }
        } else {
            // Check for raw ES import statements (suggest using @import)
            if trimmed.starts_with("import ") && trimmed.contains(" from ") {
                result.suggestions.push(LintSuggestion {
                    message: format!(
                        "Use '@import(...)' instead of raw import statement: {trimmed}"
                    ),
                    line: lineno,
                });
            }
        }
    }

    // Unclosed blocks after scanning the whole file
    for (name, open_line) in &block_stack {
        result.errors.push(LintError {
            message: format!("'@{name}' opened on line {open_line} was never closed with '@end'"),
            line: *open_line,
            code: "F003",
        });
    }

    // @extends not on first content line
    if seen_extends {
        if let (Some(ext_line), Some(first_line)) = (extends_line, first_content_line) {
            if ext_line != first_line {
                result.warnings.push(LintWarning {
                    message: format!(
                        "@extends should be the first directive in the template (found on line {ext_line}, first content on line {first_line})"
                    ),
                    line: ext_line,
                });
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Lint a template file by path (convenience wrapper)
// ---------------------------------------------------------------------------

/// Read `path` from disk and lint it.
pub fn lint_file(path: &std::path::Path) -> Result<LintResult, ForgeError> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| ForgeError::LoadError(format!("{}: {e}", path.display())))?;
    Ok(lint_template(&src))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return true if `rest` looks like a method-chain variable expression
/// (e.g. `ctx.name.pascal()`).  These are not directives and should not
/// be linted as unknown.
fn is_method_chain(rest: &str) -> bool {
    // A method chain starts with an identifier immediately followed by '.'
    let name_end = rest
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    name_end < rest.len() && rest.as_bytes().get(name_end) == Some(&b'.')
}

/// Extract the directive name from the rest of an `@…` line.
/// e.g. `"if(ctx.auth)"` → `"if"`, `"end"` → `"end"`, `"ctx.name.pascal()"` → `"ctx"`
fn directive_name(rest: &str) -> &str {
    let rest = rest.trim_end();
    // Take up to '(', '.', or whitespace
    let end = rest
        .find(|c: char| c == '(' || c == '.' || c.is_whitespace())
        .unwrap_or(rest.len());
    &rest[..end]
}

/// Check a `@schema(...)` line for common issues and emit suggestions.
fn check_schema_suggestions(line: &str, lineno: usize, result: &mut LintResult) {
    // Attempt to parse the JSON object inline
    if let Some(start) = line.find('{') {
        let inner = &line[start..];
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(inner.trim_end_matches(')')) {
            let has_required = map.values().any(|v| {
                v.get("required")
                    .and_then(|r| r.as_bool())
                    .unwrap_or(false)
            });
            if !has_required && !map.is_empty() {
                result.suggestions.push(LintSuggestion {
                    message: "@schema has no required fields — consider marking at least one field as required".to_string(),
                    line: lineno,
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_template_no_errors() {
        let src = "@if(ctx.auth)\nhello\n@end";
        let result = lint_template(src);
        assert!(result.is_ok(), "{:?}", result.errors);
    }

    #[test]
    fn detects_unclosed_if() {
        let src = "@if(ctx.auth)\nhello";
        let result = lint_template(src);
        assert!(!result.is_ok());
        assert!(result.errors[0].message.contains("@if"));
        assert_eq!(result.errors[0].code, "F003");
    }

    #[test]
    fn detects_orphan_end() {
        let src = "hello\n@end";
        let result = lint_template(src);
        assert!(!result.is_ok());
        assert!(result.errors[0].message.contains("without a matching"));
    }

    #[test]
    fn nested_blocks_ok() {
        let src = "@if(x)\n@each(items as item)\nhi\n@end\n@end";
        let result = lint_template(src);
        assert!(result.is_ok(), "{:?}", result.errors);
    }

    #[test]
    fn nested_blocks_unclosed_inner() {
        let src = "@if(x)\n@each(items as item)\nhi\n@end";
        // @if is unclosed
        let result = lint_template(src);
        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e| e.message.contains("@if")));
    }

    #[test]
    fn detects_unknown_directive() {
        let src = "@unknown_thing(foo)";
        let result = lint_template(src);
        assert!(result.warnings.iter().any(|w| w.message.contains("unknown_thing")));
    }

    #[test]
    fn suggests_import_over_raw() {
        let src = "import { z } from 'zod'";
        let result = lint_template(src);
        assert!(result.suggestions.iter().any(|s| s.message.contains("@import")));
    }

    #[test]
    fn extends_on_first_line_no_warning() {
        let src = "@extends(\"base.forge\")\n@slot(\"x\")\nhello\n@end";
        let result = lint_template(src);
        assert!(!result.warnings.iter().any(|w| w.message.contains("first directive")));
    }

    #[test]
    fn extends_not_on_first_line_warns() {
        let src = "some content\n@extends(\"base.forge\")";
        let result = lint_template(src);
        assert!(result.warnings.iter().any(|w| w.message.contains("first directive")));
    }

    #[test]
    fn schema_no_required_suggests() {
        let src = r#"@schema({ "name": { "type": "string" } })"#;
        let result = lint_template(src);
        assert!(result.suggestions.iter().any(|s| s.message.contains("required")));
    }

    #[test]
    fn schema_with_required_no_suggestion() {
        let src = r#"@schema({ "name": { "type": "string", "required": true } })"#;
        let result = lint_template(src);
        assert!(!result.suggestions.iter().any(|s| s.message.contains("required")));
    }

    #[test]
    fn method_chain_not_flagged_as_unknown() {
        let src = "@ctx.name.pascal()";
        let result = lint_template(src);
        // ctx is a method chain, not a directive — should not warn
        assert!(!result.warnings.iter().any(|w| w.message.contains("Unknown directive '@ctx'")));
    }

    #[test]
    fn feature_block_must_close() {
        let src = "@feature(\"auth\")\nhello";
        let result = lint_template(src);
        assert!(!result.is_ok());
        assert!(result.errors[0].message.contains("@feature"));
    }
}
