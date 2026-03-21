//! **tsx-fmt** — formatter for `.forge` / `.jinja` template files (Section I Phase 4).
//!
//! Normalises:
//! - Indentation (configurable, default 2 spaces)
//! - Quote style inside `@import(...)` directives (single → double or vice-versa)
//! - Trailing whitespace on each line
//! - Trailing newline at end of file
//! - Blank line collapsing (no more than 2 consecutive blank lines)
//! - `@`-directive alignment (consistent spacing after `@`)
//! - Tera/Jinja block spacing: `{{-` / `{%` / `{{` delimiter normalisation
//!
//! The formatter is idempotent: running it twice produces the same output.

pub mod config;
pub mod formatter;
pub mod rules;

pub use config::{FmtConfig, QuoteStyle};
pub use formatter::{format_str, format_file, FormatResult};

pub(crate) fn find_project_root() -> Option<std::path::PathBuf> {
    config::find_project_root()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotent_on_already_formatted() {
        let input = r#"@import("react", named=["useState"])

@if(ctx.auth)
  @import("better-auth", named=["auth"])
@end

export const foo = 1;
"#;
        let config = FmtConfig::default();
        let first = format_str(input, &config).formatted;
        let second = format_str(&first, &config).formatted;
        assert_eq!(first, second, "formatter must be idempotent");
    }

    #[test]
    fn normalises_trailing_whitespace() {
        let input = "hello   \nworld  \n";
        let out = format_str(input, &FmtConfig::default()).formatted;
        assert!(!out.lines().any(|l| l.ends_with(' ')));
    }

    #[test]
    fn ensures_trailing_newline() {
        let input = "hello";
        let out = format_str(input, &FmtConfig::default()).formatted;
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn collapses_excess_blank_lines() {
        let input = "a\n\n\n\n\nb";
        let out = format_str(input, &FmtConfig::default()).formatted;
        assert!(!out.contains("\n\n\n"), "more than 2 consecutive blank lines should be collapsed");
    }

    #[test]
    fn normalises_tera_block_spacing() {
        let input = "{{name}}  {{-  name  -}}  {%if x%}";
        let out = format_str(input, &FmtConfig::default()).formatted;
        assert!(out.contains("{{ name }}"));
        assert!(out.contains("{{- name -}}"));
        assert!(out.contains("{% if x %}"));
    }
}
