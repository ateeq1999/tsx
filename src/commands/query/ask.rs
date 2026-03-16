use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::framework::loader::FrameworkLoader;
use crate::json::error::ErrorResponse;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskResult {
    pub question: String,
    pub framework: String,
    pub answer: String,
    pub steps: Vec<AskStep>,
    pub files_affected: Vec<String>,
    pub dependencies: Vec<String>,
    pub learn_more: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskStep {
    pub action: String,
    pub code: Option<String>,
    pub description: Option<String>,
}

/// Returns a relevance score (0–100) for how well `needle` matches `haystack`.
/// Scoring: exact match → 100, substring → 80, word overlap → 60, char overlap → score.
fn fuzzy_score(needle: &str, haystack: &str) -> u32 {
    let n = needle.to_lowercase();
    let h = haystack.to_lowercase();

    if h == n {
        return 100;
    }
    if h.contains(&n) || n.contains(&h) {
        return 80;
    }

    // Word-level overlap score
    let n_words: std::collections::HashSet<&str> = n.split_whitespace().collect();
    let h_words: std::collections::HashSet<&str> = h.split_whitespace().collect();
    let overlap = n_words.intersection(&h_words).count();
    if overlap > 0 {
        return 60 + (overlap.min(4) as u32 * 5);
    }

    // Partial token match — any needle word is a substring of haystack
    let partial = n_words.iter().any(|w| h.contains(*w));
    if partial {
        return 40;
    }

    0
}

pub fn ask(question: String, framework: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();

    let mut loader = FrameworkLoader::default();
    let frameworks = loader.load_builtin_frameworks();

    if frameworks.is_empty() {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::validation("No frameworks available");
        ResponseEnvelope::error("ask", error, duration_ms).print();
        return CommandResult::err("ask", "No frameworks loaded");
    }

    let target_framework = framework.unwrap_or_else(|| {
        frameworks
            .first()
            .map(|f| f.slug.clone())
            .unwrap_or_default()
    });

    let registry = loader.get_registry(&target_framework);

    if let Some(reg) = registry {
        // Rank all questions by fuzzy score and pick the best match.
        let best = reg
            .questions
            .iter()
            .map(|q| (q, fuzzy_score(&question, &q.topic)))
            .filter(|(_, score)| *score > 0)
            .max_by_key(|(_, score)| *score);

        if let Some((q, _)) = best {
            let duration_ms = start.elapsed().as_millis() as u64;
            let result = AskResult {
                question: q.topic.clone(),
                framework: reg.framework.clone(),
                answer: q.answer.clone(),
                steps: q
                    .steps
                    .iter()
                    .map(|s| AskStep {
                        action: s.action.clone(),
                        code: s.code.clone(),
                        description: s.description.clone(),
                    })
                    .collect(),
                files_affected: q.files_affected.clone(),
                dependencies: q.dependencies.clone(),
                learn_more: q.learn_more.clone(),
            };

            let response = ResponseEnvelope::success(
                "ask",
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

            return CommandResult::ok("ask", vec![]);
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let similar: Vec<String> = reg.questions.iter().map(|q| q.topic.clone()).collect();
        let error = ErrorResponse::validation(&format!(
            "No matching question found for '{}'. Available topics: {}",
            question,
            similar.join(", ")
        ));
        ResponseEnvelope::error("ask", error, duration_ms).print();
        return CommandResult::err("ask", "Question not found");
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let error = ErrorResponse::validation(&format!("Framework not found: {}", target_framework));
    ResponseEnvelope::error("ask", error, duration_ms).print();
    CommandResult::err("ask", "Framework not found")
}
