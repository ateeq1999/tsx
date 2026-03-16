use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::framework::loader::FrameworkLoader;
use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowResult {
    pub package: String,
    pub framework: String,
    pub install_command: String,
    pub setup_steps: Vec<HowSetupStep>,
    pub patterns: Vec<HowPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowSetupStep {
    pub file: String,
    pub template: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowPattern {
    pub name: String,
    pub pattern: String,
    pub description: Option<String>,
    pub example: Option<String>,
}

/// Returns a relevance score (0–100) for package name matching.
fn fuzzy_score(needle: &str, haystack: &str) -> u32 {
    let n = needle.to_lowercase();
    let h = haystack.to_lowercase();

    if h == n {
        return 100;
    }
    if h.contains(&n) || n.contains(&h) {
        return 80;
    }

    // Match on the package short name (after last '/')
    let h_short = h.split('/').last().unwrap_or(&h);
    let n_short = n.split('/').last().unwrap_or(&n);
    if h_short == n_short || h_short.contains(n_short) || n_short.contains(h_short) {
        return 70;
    }

    let n_words: std::collections::HashSet<&str> = n.split([' ', '-', '/', '@']).collect();
    let h_words: std::collections::HashSet<&str> = h.split([' ', '-', '/', '@']).collect();
    let overlap = n_words.intersection(&h_words).count();
    if overlap > 0 {
        return 50 + (overlap.min(4) as u32 * 5);
    }

    0
}

pub fn how(integration: String, framework: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let mut loader = FrameworkLoader::default();
    let frameworks = loader.load_builtin_frameworks();

    if frameworks.is_empty() {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::validation("No frameworks available");
        ResponseEnvelope::error("how", error, duration_ms).print();
        return CommandResult::err("how", "No frameworks loaded");
    }

    let target_framework = framework.unwrap_or_else(|| {
        frameworks
            .first()
            .map(|f| f.slug.clone())
            .unwrap_or_default()
    });

    let registry = loader.get_registry(&target_framework);

    if let Some(reg) = registry {
        // Rank integrations by fuzzy score.
        let best = reg
            .integrations
            .iter()
            .map(|i| (i, fuzzy_score(&integration, &i.package)))
            .filter(|(_, score)| *score > 0)
            .max_by_key(|(_, score)| *score);

        if let Some((int, _)) = best {
            let duration_ms = start.elapsed().as_millis() as u64;
            let result = HowResult {
                package: int.package.clone(),
                framework: reg.framework.clone(),
                install_command: int
                    .install
                    .clone()
                    .unwrap_or_else(|| format!("npm install {}", int.package)),
                setup_steps: int
                    .setup
                    .iter()
                    .map(|s| HowSetupStep {
                        file: s.file.clone(),
                        template: s.template.clone(),
                        description: s.description.clone(),
                    })
                    .collect(),
                patterns: int
                    .patterns
                    .iter()
                    .map(|p| HowPattern {
                        name: p.name.clone(),
                        pattern: p.pattern.clone(),
                        description: p.description.clone(),
                        example: p.example.clone(),
                    })
                    .collect(),
            };

            let response = ResponseEnvelope::success(
                "how",
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

            return CommandResult::ok("how", vec![]);
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let available: Vec<String> = reg.integrations.iter().map(|i| i.package.clone()).collect();
        let error = ErrorResponse::validation(&format!(
            "Integration '{}' not found. Available: {}",
            integration,
            available.join(", ")
        ));
        ResponseEnvelope::error("how", error, duration_ms).print();
        return CommandResult::err("how", "Integration not found");
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let error = ErrorResponse::validation(&format!("Framework not found: {}", target_framework));
    ResponseEnvelope::error("how", error, duration_ms).print();
    CommandResult::err("how", "Framework not found")
}
