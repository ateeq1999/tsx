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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_scores_100() {
        assert_eq!(fuzzy_score("how to create a component", "how to create a component"), 100);
    }

    #[test]
    fn substring_match_scores_80() {
        assert!(fuzzy_score("component", "how to create a component") >= 80);
    }

    #[test]
    fn word_overlap_scores_60_plus() {
        let score = fuzzy_score("create component", "how to create a component");
        assert!(score >= 60, "expected >= 60, got {}", score);
    }

    #[test]
    fn partial_token_scores_40() {
        let score = fuzzy_score("routing", "how to add routing");
        assert!(score >= 40, "expected >= 40, got {}", score);
    }

    #[test]
    fn no_match_scores_0() {
        assert_eq!(fuzzy_score("xyzzy", "how to create a component"), 0);
    }

    #[test]
    fn case_insensitive_matching() {
        assert_eq!(
            fuzzy_score("HOW TO CREATE A COMPONENT", "how to create a component"),
            100
        );
    }

    #[test]
    fn ask_result_serialises() {
        let result = AskResult {
            question: "how to create a component".to_string(),
            framework: "Vue".to_string(),
            answer: "Create a .vue file".to_string(),
            steps: vec![AskStep {
                action: "touch MyComponent.vue".to_string(),
                code: Some("touch src/components/MyComponent.vue".to_string()),
                description: None,
            }],
            files_affected: vec!["src/components/*.vue".to_string()],
            dependencies: vec![],
            learn_more: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("how to create a component"));
        assert!(json.contains("Vue"));
    }
}

pub fn ask(
    question: String,
    framework: Option<String>,
    depth: String,
    verbose: bool,
) -> CommandResult {
    use crate::framework::token_budget::Depth;

    let start = Instant::now();
    let depth = Depth::from_str(&depth);

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

            // Apply depth filtering.
            let steps = if depth.include_steps() {
                q.steps
                    .iter()
                    .map(|s| AskStep {
                        action: s.action.clone(),
                        code: s.code.clone(),
                        description: s.description.clone(),
                    })
                    .collect()
            } else {
                vec![]
            };
            let (files_affected, dependencies, learn_more) = if depth.include_extras() {
                (q.files_affected.clone(), q.dependencies.clone(), q.learn_more.clone())
            } else {
                (vec![], vec![], vec![])
            };

            let mut result = serde_json::json!({
                "question": q.topic,
                "framework": reg.framework,
                "answer": q.answer,
                "depth": depth.to_string(),
                "token_estimate": estimate_result_tokens(&q.answer, &steps),
            });
            if depth.include_steps() {
                result["steps"] = serde_json::to_value(&steps).unwrap();
            }
            if depth.include_extras() {
                result["files_affected"] = serde_json::to_value(&files_affected).unwrap();
                result["dependencies"] = serde_json::to_value(&dependencies).unwrap();
                result["learn_more"] = serde_json::to_value(&learn_more).unwrap();
            }

            let response = ResponseEnvelope::success("ask", result, duration_ms);
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

fn estimate_result_tokens(answer: &str, steps: &[AskStep]) -> u32 {
    let base = (answer.len() / 4) as u32;
    let steps_tokens: u32 = steps
        .iter()
        .map(|s| {
            let action_len = s.action.len() / 4;
            let code_len = s.code.as_deref().unwrap_or("").len() / 4;
            (action_len + code_len) as u32
        })
        .sum();
    base + steps_tokens
}
