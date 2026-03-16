use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::framework::loader::FrameworkLoader;
use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereResult {
    pub thing: String,
    pub framework: String,
    pub path: String,
    pub pattern: String,
    pub description: String,
    pub example: Option<String>,
}

/// Returns a relevance score (0–100) for fuzzy key matching.
fn fuzzy_score(needle: &str, haystack: &str) -> u32 {
    let n = needle.to_lowercase();
    let h = haystack.to_lowercase();

    if h == n {
        return 100;
    }
    if h.contains(&n) || n.contains(&h) {
        return 80;
    }
    let n_words: std::collections::HashSet<&str> = n.split_whitespace().collect();
    let h_words: std::collections::HashSet<&str> = h.split([' ', '-', '_']).collect();
    let overlap = n_words.intersection(&h_words).count();
    if overlap > 0 {
        return 60 + (overlap.min(4) as u32 * 5);
    }
    0
}

pub fn where_cmd(thing: String, framework: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let mut loader = FrameworkLoader::default();
    let frameworks = loader.load_builtin_frameworks();

    if frameworks.is_empty() {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::validation("No frameworks available");
        ResponseEnvelope::error("where", error, duration_ms).print();
        return CommandResult::err("where", "No frameworks loaded");
    }

    let target_framework = framework.unwrap_or_else(|| {
        frameworks
            .first()
            .map(|f| f.slug.clone())
            .unwrap_or_default()
    });

    let registry = loader.get_registry(&target_framework);

    if let Some(reg) = registry {
        // Rank all convention keys by fuzzy score and pick the best match.
        let best = reg
            .conventions
            .files
            .iter()
            .map(|(key, conv)| (key, conv, fuzzy_score(&thing, key)))
            .filter(|(_, _, score)| *score > 0)
            .max_by_key(|(_, _, score)| *score);

        if let Some((key, conv, _)) = best {
            let duration_ms = start.elapsed().as_millis() as u64;
            let structure = &reg.structure;
            let result = WhereResult {
                thing: key.clone(),
                framework: reg.framework.clone(),
                path: apply_structure(&conv.pattern, structure),
                pattern: conv.pattern.clone(),
                description: conv.description.clone().unwrap_or_default(),
                example: conv.example.clone(),
            };

            let response = ResponseEnvelope::success(
                "where",
                serde_json::to_value(result).unwrap(),
                duration_ms,
            );

            if verbose {
                let context = crate::json::response::Context {
                    project_root: std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    tsx_version: env!("CARGO_PKG_VERSION").to_string(),
                };
                response.with_context(context).print();
            } else {
                response.print();
            }

            return CommandResult::ok("where", vec![]);
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let available: Vec<String> = reg.conventions.files.keys().cloned().collect();
        let error = ErrorResponse::validation(&format!(
            "Unknown thing '{}'. Available: {}",
            thing,
            available.join(", ")
        ));
        ResponseEnvelope::error("where", error, duration_ms).print();
        return CommandResult::err("where", "Thing not found");
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let error = ErrorResponse::validation(&format!("Framework not found: {}", target_framework));
    ResponseEnvelope::error("where", error, duration_ms).print();
    CommandResult::err("where", "Framework not found")
}

fn apply_structure(
    pattern: &str,
    structure: &crate::framework::registry::ProjectStructure,
) -> String {
    let mut result = pattern.to_string();

    if let Some(ref src) = structure.src {
        result = result.replace("src/", &format!("{}/", src));
    }
    if let Some(ref routes) = structure.routes {
        result = result.replace("src/routes/", &format!("{}/", routes));
    }
    if let Some(ref components) = structure.components {
        result = result.replace("src/components/", &format!("{}/", components));
    }
    if let Some(ref lib) = structure.lib {
        result = result.replace("src/lib/", &format!("{}/", lib));
    }
    if let Some(ref db) = structure.db {
        result = result.replace("src/db/", &format!("{}/", db));
    }

    result
}
