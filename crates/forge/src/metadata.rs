//! Frontmatter parser for forge knowledge files.
//!
//! Parses the `---\nkey: value\n---\nbody` format used in framework knowledge/*.md files.

use serde::{Deserialize, Serialize};

/// Metadata extracted from a knowledge file's frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontMatter {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub question: Option<String>,
    /// Rough token count for the file body. Used for token-budget responses.
    #[serde(default)]
    pub token_estimate: Option<u32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub related: Vec<String>,
}

/// Parse `---\nkey: value\n---\nbody...` into `(FrontMatter, body)`.
/// If no frontmatter delimiters are found, returns default metadata and the full content as body.
pub fn parse(content: &str) -> (FrontMatter, String) {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return (FrontMatter::default(), content.to_string());
    }

    let after_open = &content[3..];
    // Find the closing ---
    let Some(end_pos) = after_open.find("\n---") else {
        return (FrontMatter::default(), content.to_string());
    };

    let yaml = after_open[..end_pos].trim();
    let body = after_open[end_pos + 4..]
        .trim_start_matches('\n')
        .to_string();

    let fm = parse_yaml(yaml);
    (fm, body)
}

fn parse_yaml(yaml: &str) -> FrontMatter {
    let mut fm = FrontMatter::default();
    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(colon) = line.find(':') else {
            continue;
        };
        let key = line[..colon].trim();
        let value = line[colon + 1..].trim();
        match key {
            "id" => fm.id = Some(value.to_string()),
            "question" => fm.question = Some(value.to_string()),
            "token_estimate" => fm.token_estimate = value.parse().ok(),
            "tags" => fm.tags = parse_list(value),
            "requires" => fm.requires = parse_list(value),
            "related" => fm.related = parse_list(value),
            _ => {}
        }
    }
    fm
}

fn parse_list(value: &str) -> Vec<String> {
    let value = value.trim();
    if value.starts_with('[') && value.ends_with(']') {
        let inner = &value[1..value.len() - 1];
        inner
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if !value.is_empty() {
        vec![value.to_string()]
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_frontmatter() {
        let src = "---\nid: add-auth\ntoken_estimate: 120\ntags: [auth, security]\nrelated: [add-migration]\n---\n\n## Body text";
        let (fm, body) = parse(src);
        assert_eq!(fm.id.as_deref(), Some("add-auth"));
        assert_eq!(fm.token_estimate, Some(120));
        assert_eq!(fm.tags, vec!["auth", "security"]);
        assert_eq!(fm.related, vec!["add-migration"]);
        assert!(body.contains("Body text"));
    }

    #[test]
    fn returns_defaults_when_no_frontmatter() {
        let src = "## Just a body";
        let (fm, body) = parse(src);
        assert!(fm.id.is_none());
        assert_eq!(body, "## Just a body");
    }

    #[test]
    fn handles_question_field() {
        let src = "---\nquestion: How do I add auth?\n---\nAnswer here.";
        let (fm, body) = parse(src);
        assert_eq!(fm.question.as_deref(), Some("How do I add auth?"));
        assert_eq!(body, "Answer here.");
    }
}
