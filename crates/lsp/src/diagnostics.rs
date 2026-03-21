//! Static diagnostics for `.forge` / `.jinja` template files.

use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Run diagnostics on a `.forge` or `.jinja` template source string.
/// Returns a list of LSP `Diagnostic` values.
pub fn check_template(src: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let mut if_depth: i32 = 0;
    let mut each_depth: i32 = 0;

    for (line_idx, line) in src.lines().enumerate() {
        let ln = line_idx as u32;

        // ── Jinja block tag balance ─────────────────────────────────────────
        // Count opening / closing Tera/Jinja2 block tags
        let mut pos = 0usize;
        while pos < line.len() {
            if line[pos..].starts_with("{%") {
                let tag_start = pos;
                if let Some(end) = line[pos..].find("%}") {
                    let inner = line[pos + 2..pos + end].trim();
                    if inner.starts_with("if ") || inner == "if" {
                        if_depth += 1;
                    } else if inner.starts_with("elif ") || inner.starts_with("else") {
                        // balance stays the same
                    } else if inner == "endif" {
                        if_depth -= 1;
                        if if_depth < 0 {
                            diags.push(diag(ln, tag_start as u32, (pos + end + 2) as u32,
                                DiagnosticSeverity::ERROR,
                                "Unexpected {% endif %} — no matching {% if %}"));
                            if_depth = 0;
                        }
                    } else if inner.starts_with("for ") {
                        each_depth += 1;
                    } else if inner == "endfor" {
                        each_depth -= 1;
                        if each_depth < 0 {
                            diags.push(diag(ln, tag_start as u32, (pos + end + 2) as u32,
                                DiagnosticSeverity::ERROR,
                                "Unexpected {% endfor %} — no matching {% for %}"));
                            each_depth = 0;
                        }
                    }
                    pos += end + 2;
                } else {
                    // Unclosed {%
                    diags.push(diag(ln, pos as u32, line.len() as u32,
                        DiagnosticSeverity::ERROR,
                        "Unclosed block tag — missing `%}`"));
                    break;
                }
            } else if line[pos..].starts_with("{{") {
                if let Some(end) = line[pos..].find("}}") {
                    pos += end + 2;
                } else {
                    diags.push(diag(ln, pos as u32, line.len() as u32,
                        DiagnosticSeverity::ERROR,
                        "Unclosed expression — missing `}}`"));
                    break;
                }
            } else {
                pos += 1;
            }
        }

        // ── @-directive balance ──────────────────────────────────────────────
        let trimmed = line.trim();
        if trimmed.starts_with("@if(") || trimmed.starts_with("@if (") {
            if_depth += 1;
        } else if trimmed.starts_with("@unless(") || trimmed.starts_with("@unless (") {
            if_depth += 1;
        } else if trimmed.starts_with("@each(") || trimmed.starts_with("@each (") {
            each_depth += 1;
        } else if trimmed == "@end" {
            // could close either @if or @each
            if if_depth > 0 {
                if_depth -= 1;
            } else if each_depth > 0 {
                each_depth -= 1;
            } else {
                diags.push(diag(ln, 0, line.len() as u32,
                    DiagnosticSeverity::ERROR,
                    "Unexpected @end — no matching @if / @each / @unless"));
            }
        }
    }

    // End-of-file: report unclosed blocks
    let eof_line = src.lines().count().saturating_sub(1) as u32;
    for _ in 0..if_depth {
        diags.push(diag(eof_line, 0, 0,
            DiagnosticSeverity::ERROR,
            "Unclosed {% if %} / @if block — missing {% endif %} or @end"));
    }
    for _ in 0..each_depth {
        diags.push(diag(eof_line, 0, 0,
            DiagnosticSeverity::ERROR,
            "Unclosed {% for %} / @each block — missing {% endfor %} or @end"));
    }

    diags
}

fn diag(line: u32, start_char: u32, end_char: u32, severity: DiagnosticSeverity, msg: &str) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position { line, character: start_char },
            end:   Position { line, character: end_char.max(start_char + 1) },
        },
        severity: Some(severity),
        source: Some("tsx-lsp".to_string()),
        message: msg.to_string(),
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_diags_for_valid_template() {
        let src = "{% if ctx.auth %}\nhello\n{% endif %}";
        assert!(check_template(src).is_empty());
    }

    #[test]
    fn detects_unclosed_expr() {
        let src = "{{ ctx.name }}\n{{ unclosed";
        let diags = check_template(src);
        assert!(!diags.is_empty());
        assert!(diags[0].message.contains("Unclosed expression"));
    }

    #[test]
    fn detects_unclosed_if_block() {
        let src = "{% if ctx.auth %}\nhello";
        let diags = check_template(src);
        assert!(!diags.is_empty());
        assert!(diags[0].message.contains("Unclosed"));
    }

    #[test]
    fn detects_unexpected_endif() {
        let src = "hello\n{% endif %}";
        let diags = check_template(src);
        assert!(!diags.is_empty());
        assert!(diags[0].message.contains("Unexpected"));
    }

    #[test]
    fn at_directive_balance() {
        let src = "@if(ctx.auth)\nhello\n@end";
        assert!(check_template(src).is_empty());
    }

    #[test]
    fn at_directive_unclosed() {
        let src = "@if(ctx.auth)\nhello";
        let diags = check_template(src);
        assert!(!diags.is_empty());
    }
}
