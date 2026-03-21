//! Documentation topic discovery — scans .tsx/knowledge/ and framework docs.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocTopic {
    pub title: String,
    pub path: PathBuf,
    pub category: String,
    /// Rough token estimate from YAML front-matter (optional)
    pub token_estimate: Option<u32>,
    /// Short description / first paragraph
    pub summary: String,
}

impl DocTopic {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let content = std::fs::read_to_string(&path).ok()?;
        let (front_matter, body) = split_front_matter(&content);

        let title = front_matter
            .as_ref()
            .and_then(|fm| extract_yaml_string(fm, "title"))
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .replace(['-', '_'], " ")
            });

        let category = front_matter
            .as_ref()
            .and_then(|fm| extract_yaml_string(fm, "category"))
            .unwrap_or_else(|| {
                path.parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("general")
                    .to_string()
            });

        let token_estimate = front_matter
            .as_ref()
            .and_then(|fm| extract_yaml_u32(fm, "token_estimate"));

        let summary = body
            .lines()
            .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .unwrap_or("")
            .trim_start_matches("# ")
            .trim()
            .chars()
            .take(120)
            .collect();

        Some(DocTopic { title, path, category, token_estimate, summary })
    }
}

/// Collect all documentation topics from the given root paths.
pub fn collect_topics(roots: &[PathBuf]) -> Vec<DocTopic> {
    let mut topics = Vec::new();
    for root in roots {
        if !root.exists() { continue; }
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") { continue; }
            if let Some(topic) = DocTopic::from_path(path.to_path_buf()) {
                topics.push(topic);
            }
        }
    }
    // Sort by category then title
    topics.sort_by(|a, b| a.category.cmp(&b.category).then(a.title.cmp(&b.title)));
    topics
}

/// Default knowledge roots: embedded + project-local
pub fn default_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    // Project-local: .tsx/knowledge/
    if let Ok(cwd) = std::env::current_dir() {
        let local = cwd.join(".tsx").join("knowledge");
        if local.exists() {
            roots.push(local);
        }
        // Also check for a frameworks directory
        let fw = cwd.join(".tsx").join("frameworks");
        if fw.exists() {
            roots.push(fw);
        }
    }

    roots
}

// ---------------------------------------------------------------------------
// Front-matter parsing (minimal YAML subset: key: value lines)
// ---------------------------------------------------------------------------

fn split_front_matter(content: &str) -> (Option<String>, &str) {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return (None, content);
    }
    let after_open = &content[3..];
    if let Some(close) = after_open.find("\n---") {
        let fm = &after_open[..close];
        let rest = &after_open[close + 4..];
        (Some(fm.to_string()), rest.trim_start())
    } else {
        (None, content)
    }
}

fn extract_yaml_string(fm: &str, key: &str) -> Option<String> {
    for line in fm.lines() {
        if let Some(rest) = line.trim().strip_prefix(&format!("{}:", key)) {
            return Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
}

fn extract_yaml_u32(fm: &str, key: &str) -> Option<u32> {
    extract_yaml_string(fm, key)?.parse().ok()
}
