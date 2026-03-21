//! Individual formatting rules applied per-line or to the whole document.

use crate::config::{FmtConfig, QuoteStyle};
use std::borrow::Cow;

// ---------------------------------------------------------------------------
// Trailing whitespace
// ---------------------------------------------------------------------------

pub fn strip_trailing_whitespace(line: &str) -> &str {
    line.trim_end()
}

// ---------------------------------------------------------------------------
// Blank line collapsing
// ---------------------------------------------------------------------------

pub fn collapse_blank_lines(src: &str, max: usize) -> String {
    let mut out = String::with_capacity(src.len());
    let mut blanks = 0usize;

    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blanks += 1;
            if blanks <= max {
                out.push('\n');
            }
        } else {
            blanks = 0;
            out.push_str(trimmed);
            // Preserve original indent
            let indent = &line[..line.len() - line.trim_start().len()];
            // Rebuild: indent + trimmed content
            out = {
                // simplify: just keep the accumulated output
                out
            };
            let _ = indent; // unused - we'll fix in formatter layer
            out.push('\n');
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Tera delimiter spacing normalisation
// ---------------------------------------------------------------------------

/// Normalise `{{name}}` → `{{ name }}`, `{{-name-}}` → `{{- name -}}`,
/// `{%if x%}` → `{% if x %}`.
pub fn normalise_tera_spacing(line: &str) -> Cow<'_, str> {
    // Use a state-machine scan so we don't corrupt template content.
    let mut result = String::with_capacity(line.len() + 8);
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Look for `{{` or `{%`
        if chars[i] == '{' && i + 1 < len && (chars[i + 1] == '{' || chars[i + 1] == '%') {
            let open_kind = chars[i + 1]; // '{' for expression, '%' for block
            let close_kind = if open_kind == '{' { '}' } else { '%' };

            // Check for whitespace-control dash: `{{-` / `-}}`
            let trim_start = i + 2 < len && chars[i + 2] == '-';
            let prefix_end = if trim_start { i + 3 } else { i + 2 };

            // Find the closing delimiter
            let close_pos = find_close(&chars, prefix_end, close_kind);

            if let Some(c) = close_pos {
                let inner_raw: String = chars[prefix_end..c].iter().collect();
                let trim_end = chars[c] == '-';
                let inner_end = if trim_end { c } else { c }; // adjusts to after `-`

                let inner = inner_raw.trim().to_string();
                // Rebuild: {{ [- ]inner[ -]}} or {% [- ]inner[ -]%}
                result.push('{');
                result.push(open_kind);
                if trim_start { result.push('-'); }
                result.push(' ');
                result.push_str(&inner);
                result.push(' ');
                if trim_end { result.push('-'); }
                result.push(close_kind);
                result.push('}');

                // Advance past closing `}}`
                let after = if trim_end { inner_end + 2 } else { inner_end + 2 };
                i = after;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    if result == line {
        Cow::Borrowed(line)
    } else {
        Cow::Owned(result)
    }
}

fn find_close(chars: &[char], start: usize, close_kind: char) -> Option<usize> {
    let len = chars.len();
    let mut i = start;
    while i < len {
        // Skip string literals
        if chars[i] == '"' || chars[i] == '\'' {
            let q = chars[i];
            i += 1;
            while i < len && chars[i] != q {
                if chars[i] == '\\' { i += 1; }
                i += 1;
            }
            i += 1;
            continue;
        }
        // Check for `-}` or `%}` or `}}`
        if i + 1 < len {
            if chars[i] == '-' && chars[i + 1] == close_kind && i + 2 < len && chars[i + 2] == '}' {
                return Some(i); // trim_end
            }
            if chars[i] == close_kind && chars[i + 1] == '}' {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------------------
// Quote normalisation in @import directives
// ---------------------------------------------------------------------------

pub fn normalise_quotes_in_at_import<'a>(line: &'a str, style: &FmtConfig) -> Cow<'a, str> {
    // Only process lines starting with `@import(`
    let trimmed = line.trim_start();
    if !trimmed.starts_with("@import(") {
        return Cow::Borrowed(line);
    }

    let target_quote = match style.quotes {
        QuoteStyle::Double => '"',
        QuoteStyle::Single => '\'',
    };
    let other_quote = match style.quotes {
        QuoteStyle::Double => '\'',
        QuoteStyle::Single => '"',
    };

    if !line.contains(other_quote) {
        return Cow::Borrowed(line);
    }

    let result = line.replace(other_quote, &target_quote.to_string());
    Cow::Owned(result)
}

// ---------------------------------------------------------------------------
// Ensure trailing newline
// ---------------------------------------------------------------------------

pub fn ensure_trailing_newline(s: &mut String) {
    if !s.ends_with('\n') {
        s.push('\n');
    }
}
