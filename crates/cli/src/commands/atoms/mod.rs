//! `tsx atoms` — queryable catalog of atoms and molecules for the active framework (H2).
//!
//! Subcommands:
//! - `tsx atoms list [--category <cat>]` — list available atoms/molecules
//! - `tsx atoms preview <id>` — show the raw template source for an atom/molecule

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::json::error::{ErrorCode, ErrorResponse};
use crate::json::response::ResponseEnvelope;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomEntry {
    /// Fully-qualified id: `drizzle/column`, `form/field_input`, etc.
    pub id: String,
    /// "atom" or "molecule"
    pub tier: String,
    /// Category (first path segment after `atoms/` or `molecules/`)
    pub category: String,
    /// Short human name derived from the filename
    pub name: String,
    /// Description extracted from the first template comment, or a default
    pub description: String,
}

// ---------------------------------------------------------------------------
// Public entrypoints
// ---------------------------------------------------------------------------

pub fn atoms_list(category: Option<String>, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let dirs = template_dirs();

    if dirs.is_empty() {
        return ResponseEnvelope::error(
            "atoms list",
            ErrorResponse::new(
                ErrorCode::ProjectNotFound,
                "No template directory found. Run `tsx init` first or check .tsx/templates/.",
            ),
            0,
        );
    }

    let mut entries: Vec<AtomEntry> = Vec::new();

    for dir in &dirs {
        collect_entries(dir, &mut entries);
    }

    // Filter by category if requested
    if let Some(cat) = &category {
        let cat_lower = cat.to_lowercase();
        entries.retain(|e| e.category.to_lowercase() == cat_lower);
    }

    entries.sort_by(|a, b| a.tier.cmp(&b.tier).then(a.id.cmp(&b.id)));

    let result = serde_json::json!({
        "count": entries.len(),
        "atoms": entries,
    });

    ResponseEnvelope::success("atoms list", result, start.elapsed().as_millis() as u64)
}

pub fn atoms_preview(id: String, _verbose: bool) -> ResponseEnvelope {
    let start = std::time::Instant::now();
    let dirs = template_dirs();

    // The id is like "drizzle/column" — try atoms/ and molecules/ prefixes
    let candidates: Vec<String> = vec![
        format!("atoms/{}.jinja", id),
        format!("atoms/{}.forge", id),
        format!("molecules/{}.jinja", id),
        format!("molecules/{}.forge", id),
    ];

    for dir in &dirs {
        for rel in &candidates {
            let path = dir.join(rel);
            if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let result = serde_json::json!({
                            "id": id,
                            "path": path.to_string_lossy(),
                            "source": content,
                        });
                        return ResponseEnvelope::success(
                            "atoms preview",
                            result,
                            start.elapsed().as_millis() as u64,
                        );
                    }
                    Err(e) => {
                        return ResponseEnvelope::error(
                            "atoms preview",
                            ErrorResponse::new(
                                ErrorCode::InternalError,
                                format!("Could not read {}: {}", path.display(), e),
                            ),
                            start.elapsed().as_millis() as u64,
                        )
                    }
                }
            }
        }
    }

    ResponseEnvelope::error(
        "atoms preview",
        ErrorResponse::new(
            ErrorCode::TemplateNotFound,
            format!(
                "Atom '{}' not found. Run `tsx atoms list` to see available atoms.",
                id
            ),
        ),
        start.elapsed().as_millis() as u64,
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the candidate template root directories, ordered by priority:
/// .tsx/templates/ (user overrides) → built-in templates/ → project templates/
fn template_dirs() -> Vec<PathBuf> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dirs = Vec::new();
    for candidate in [
        cwd.join(".tsx").join("templates"),
        cwd.join("templates"),
    ] {
        if candidate.exists() {
            dirs.push(candidate);
        }
    }
    dirs
}

/// Walk `root` and collect all `.jinja` / `.forge` files under `atoms/` and `molecules/`.
fn collect_entries(root: &PathBuf, entries: &mut Vec<AtomEntry>) {
    for tier_prefix in ["atoms", "molecules"] {
        let base = root.join(tier_prefix);
        if !base.exists() {
            continue;
        }
        for entry in WalkDir::new(&base)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "jinja" && ext != "forge" {
                continue;
            }

            // rel = "drizzle/column.jinja"
            let rel = path
                .strip_prefix(&base)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            // category = first path segment ("drizzle")
            let category = rel.split('/').next().unwrap_or("").to_string();

            // name = stem of filename ("column")
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            // id = "drizzle/column"
            let id = format!(
                "{}/{}",
                category,
                stem
            );

            let description = extract_description(path);

            entries.push(AtomEntry {
                id,
                tier: tier_prefix.trim_end_matches('s').to_string(), // "atom" or "molecule"
                category,
                name: stem,
                description,
            });
        }
    }
}

/// Extract a human-readable description from the first Jinja comment `{# ... #}`.
fn extract_description(path: &std::path::Path) -> String {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    // Look for {# ... #} on first non-empty line
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("{#") {
            return trimmed
                .trim_start_matches("{#")
                .trim_end_matches("#}")
                .trim()
                .to_string();
        }
    }
    // Fallback: derive from filename
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .replace('_', " ")
        .to_string()
}
