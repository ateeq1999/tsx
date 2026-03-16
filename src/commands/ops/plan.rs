use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::framework::command_registry::CommandRegistry;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Deserialize)]
pub struct PlanGoal {
    pub goal: String,
}

#[derive(Serialize)]
struct PlanStep {
    goal: String,
    command: String,
    generator_id: String,
    framework: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_estimate: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    output_paths: Vec<String>,
}

#[derive(Serialize)]
struct PlanResult {
    steps: Vec<PlanStep>,
    unresolved: Vec<String>,
    total_token_estimate: u32,
}

/// Translate a list of natural-language goals into a concrete command sequence.
///
/// Matching works by tokenising both the goal string and the generator's id/command/description
/// and scoring overlap. The highest-scoring generator wins.
pub fn plan(goals: Vec<PlanGoal>, verbose: bool) -> CommandResult {
    let _ = verbose;
    let start = Instant::now();

    let registry = CommandRegistry::load_all();
    let specs = registry.all();

    let mut steps: Vec<PlanStep> = Vec::new();
    let mut unresolved: Vec<String> = Vec::new();

    for goal_item in &goals {
        let goal = &goal_item.goal;
        let goal_tokens = tokenise(goal);

        let best = specs.iter().max_by_key(|spec| {
            let candidate = format!(
                "{} {} {}",
                spec.id, spec.command, spec.description
            );
            score(&goal_tokens, &tokenise(&candidate))
        });

        if let Some(spec) = best {
            let candidate = format!("{} {} {}", spec.id, spec.command, spec.description);
            let s = score(&goal_tokens, &tokenise(&candidate));
            if s == 0 {
                unresolved.push(goal.clone());
                continue;
            }
            steps.push(PlanStep {
                goal: goal.clone(),
                command: spec.command.clone(),
                generator_id: spec.id.clone(),
                framework: spec.framework.clone(),
                description: spec.description.clone(),
                token_estimate: spec.token_estimate,
                output_paths: spec.output_paths.clone(),
            });
        } else {
            unresolved.push(goal.clone());
        }
    }

    let total_token_estimate = steps
        .iter()
        .filter_map(|s| s.token_estimate)
        .sum::<u32>();

    let result = PlanResult {
        steps,
        unresolved,
        total_token_estimate,
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    ResponseEnvelope::success("plan", serde_json::to_value(result).unwrap(), duration_ms).print();
    CommandResult::ok("plan", vec![])
}

/// Split a string into lowercase word tokens, stripping punctuation.
fn tokenise(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 1)
        .map(|t| t.to_lowercase())
        .collect()
}

/// Count how many tokens from `goal` appear in `candidate`.
fn score(goal: &[String], candidate: &[String]) -> usize {
    goal.iter()
        .filter(|t| candidate.contains(t))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenise_splits_on_punctuation() {
        let tokens = tokenise("add:schema for users");
        assert!(tokens.contains(&"add".to_string()));
        assert!(tokens.contains(&"schema".to_string()));
        assert!(tokens.contains(&"for".to_string()));
        assert!(tokens.contains(&"users".to_string()));
    }

    #[test]
    fn score_counts_overlap() {
        let goal = tokenise("add schema");
        let candidate = tokenise("add schema migration");
        assert_eq!(score(&goal, &candidate), 2);
    }

    #[test]
    fn score_zero_for_no_overlap() {
        let goal = tokenise("create dialog");
        let candidate = tokenise("add entity handler");
        assert_eq!(score(&goal, &candidate), 0);
    }
}
