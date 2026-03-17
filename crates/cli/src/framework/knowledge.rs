//! Framework knowledge file loader.
//!
//! Reads structured markdown files from a framework package's `knowledge/` directory.
//! Each file carries YAML frontmatter with `token_estimate` and `tags` so the CLI
//! can serve token-budgeted responses without loading full content.

use forge::metadata::FrontMatter;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A knowledge entry loaded from `knowledge/<section>.md`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub section: String,
    pub token_estimate: u32,
    pub content: String,
    pub frontmatter: FrontMatter,
}

/// Canonical section names, in display order.
const SECTIONS: &[&str] = &["overview", "concepts", "patterns", "faq", "decisions"];

/// Load all knowledge sections from `knowledge_dir`.
/// Sections that don't exist on disk are silently skipped.
pub fn load_knowledge(knowledge_dir: &Path) -> Vec<KnowledgeEntry> {
    let mut entries = Vec::new();
    if !knowledge_dir.exists() {
        return entries;
    }
    for section in SECTIONS {
        if let Some(entry) = load_section(knowledge_dir, section) {
            entries.push(entry);
        }
    }
    entries
}

/// Load a single section from `knowledge_dir/<section>.md`.
pub fn load_section(knowledge_dir: &Path, section: &str) -> Option<KnowledgeEntry> {
    let path = knowledge_dir.join(format!("{section}.md"));
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    let (fm, body) = forge::parse_frontmatter(&content);
    let token_estimate = fm.token_estimate.unwrap_or_else(|| estimate_tokens(&body));
    Some(KnowledgeEntry {
        section: section.to_string(),
        token_estimate,
        content: body,
        frontmatter: fm,
    })
}

/// Rough token estimate: ~4 characters per token.
fn estimate_tokens(text: &str) -> u32 {
    (text.len() as u32).saturating_div(4)
}
