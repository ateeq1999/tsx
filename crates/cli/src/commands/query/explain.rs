use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

/// The public result type returned for each explain query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResult {
    pub topic: String,
    pub purpose: String,
    pub decisions: Vec<Decision>,
    pub tree: DecisionTree,
    pub learn_more: Vec<LearnMoreLink>,
    pub version: String,
    pub changelog: Vec<ChangelogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub title: String,
    pub rationale: String,
    pub alternative: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTree {
    pub root: String,
    pub branches: Vec<TreeBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeBranch {
    pub condition: String,
    pub outcome: String,
    pub children: Vec<TreeBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnMoreLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub change: String,
}

/// A single entry in the knowledge base JSON.
#[derive(Debug, Clone, Deserialize)]
struct KnowledgeEntry {
    key: String,
    version: String,
    purpose: String,
    decisions: Vec<Decision>,
    tree: DecisionTree,
    learn_more: Vec<LearnMoreLink>,
    changelog: Vec<ChangelogEntry>,
}

static EXPLAIN_JSON: &str = include_str!("../../../data/explain.json");

fn load_knowledge_base() -> Vec<KnowledgeEntry> {
    serde_json::from_str(EXPLAIN_JSON).unwrap_or_default()
}

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
    let h_words: std::collections::HashSet<&str> = h.split_whitespace().collect();
    let overlap = n_words.intersection(&h_words).count();
    if overlap > 0 {
        return 60 + (overlap.min(4) as u32 * 5);
    }
    0
}

/// Exposed for tests.
#[cfg(test)]
pub(crate) fn score_for_test(needle: &str, haystack: &str) -> u32 {
    fuzzy_score(needle, haystack)
}

pub fn explain(topic: String, verbose: bool) -> CommandResult {
    let start = Instant::now();
    let knowledge_base = load_knowledge_base();

    let best = knowledge_base
        .iter()
        .map(|e| (e, fuzzy_score(&topic, &e.key)))
        .filter(|(_, score)| *score > 0)
        .max_by_key(|(_, score)| *score);

    if let Some((entry, _)) = best {
        let duration_ms = start.elapsed().as_millis() as u64;

        let result = ExplainResult {
            topic: entry.key.clone(),
            version: entry.version.clone(),
            purpose: entry.purpose.clone(),
            decisions: entry.decisions.clone(),
            tree: entry.tree.clone(),
            learn_more: entry.learn_more.clone(),
            changelog: entry.changelog.clone(),
        };

        let response = ResponseEnvelope::success(
            "explain",
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

        return CommandResult::ok("explain", vec![]);
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let available: Vec<String> = knowledge_base.iter().map(|e| e.key.clone()).collect();
    let error = crate::json::error::ErrorResponse::validation(&format!(
        "Topic '{}' not found. Available: {}",
        topic,
        available.join(", ")
    ));
    ResponseEnvelope::error("explain", error, duration_ms).print();
    CommandResult::err("explain", "Topic not found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_exact_match_scores_100() {
        assert_eq!(score_for_test("atom", "atom"), 100);
    }

    #[test]
    fn fuzzy_substring_match_scores_80() {
        assert_eq!(score_for_test("auth", "auth"), 100);
        assert!(score_for_test("aut", "auth") >= 80);
    }

    #[test]
    fn fuzzy_no_match_scores_0() {
        assert_eq!(score_for_test("xyz123", "atom"), 0);
    }

    #[test]
    fn fuzzy_case_insensitive() {
        assert_eq!(score_for_test("ATOM", "atom"), 100);
        assert_eq!(score_for_test("Atom", "atom"), 100);
    }

    #[test]
    fn knowledge_base_loads_all_topics() {
        let kb = load_knowledge_base();
        assert!(!kb.is_empty(), "knowledge base should not be empty");
        let keys: Vec<&str> = kb.iter().map(|e| e.key.as_str()).collect();
        assert!(keys.contains(&"atom"), "should contain atom");
        assert!(keys.contains(&"feature"), "should contain feature");
        assert!(keys.contains(&"auth"), "should contain auth");
    }

    #[test]
    fn knowledge_base_entries_have_decisions_and_tree() {
        let kb = load_knowledge_base();
        for entry in &kb {
            assert!(!entry.decisions.is_empty(), "entry {} should have decisions", entry.key);
            assert!(!entry.tree.branches.is_empty(), "entry {} tree should have branches", entry.key);
        }
    }

    #[test]
    fn knowledge_base_entries_have_learn_more() {
        let kb = load_knowledge_base();
        for entry in &kb {
            assert!(!entry.learn_more.is_empty(), "entry {} should have learn_more links", entry.key);
        }
    }
}
