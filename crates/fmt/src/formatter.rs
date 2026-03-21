//! Main format entry points.

use std::path::Path;

use crate::config::FmtConfig;
use crate::rules;

// ---------------------------------------------------------------------------
// Result type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FormatResult {
    pub formatted: String,
    pub changed: bool,
    pub lines_changed: usize,
}

// ---------------------------------------------------------------------------
// Core formatting function
// ---------------------------------------------------------------------------

pub fn format_str(src: &str, config: &FmtConfig) -> FormatResult {
    let mut lines: Vec<String> = Vec::with_capacity(src.lines().count() + 1);

    for line in src.lines() {
        // 1. Strip trailing whitespace
        let line = rules::strip_trailing_whitespace(line);

        // 2. Normalise Tera delimiter spacing
        let line: std::borrow::Cow<'_, str> = if config.normalise_tera_spacing {
            rules::normalise_tera_spacing(line)
        } else {
            std::borrow::Cow::Borrowed(line)
        };

        // 3. Normalise quotes in @import directives
        let line = rules::normalise_quotes_in_at_import(&line, config);

        lines.push(line.into_owned());
    }

    // 4. Collapse excess blank lines
    let joined = lines.join("\n");
    let after_blanks = collapse_blanks(&joined, config.max_blank_lines);

    // 5. Ensure trailing newline
    let mut formatted = after_blanks;
    rules::ensure_trailing_newline(&mut formatted);

    let changed = formatted != src;
    let lines_changed = diff_lines(src, &formatted);

    FormatResult { formatted, changed, lines_changed }
}

/// Format a file in-place. Returns `Ok(FormatResult)`.
pub fn format_file(path: &Path, config: &FmtConfig, check_only: bool) -> std::io::Result<FormatResult> {
    let original = std::fs::read_to_string(path)?;
    let result = format_str(&original, config);

    if result.changed && !check_only {
        std::fs::write(path, &result.formatted)?;
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn collapse_blanks(src: &str, max: usize) -> String {
    let mut out = String::with_capacity(src.len());
    let mut consecutive_blanks = 0usize;

    for line in src.lines() {
        if line.trim().is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks <= max {
                out.push('\n');
            }
        } else {
            consecutive_blanks = 0;
            out.push_str(line);
            out.push('\n');
        }
    }

    // Remove trailing blank lines (keep only 1 trailing newline)
    while out.ends_with("\n\n") {
        out.pop();
    }
    out
}

fn diff_lines(a: &str, b: &str) -> usize {
    let a_lines: Vec<&str> = a.lines().collect();
    let b_lines: Vec<&str> = b.lines().collect();
    let max_len = a_lines.len().max(b_lines.len());
    (0..max_len)
        .filter(|&i| a_lines.get(i) != b_lines.get(i))
        .count()
}
