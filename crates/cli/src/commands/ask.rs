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

pub fn ask(question: String, framework: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();
    let duration_ms = start.elapsed().as_millis() as u64;

    let mut loader = FrameworkLoader::default();
    let frameworks = loader.load_builtin_frameworks();

    if frameworks.is_empty() {
        let error = ErrorResponse::validation("No frameworks available");
        let response = ResponseEnvelope::error("ask", error, duration_ms);
        response.print();
        return CommandResult::err("ask", "No frameworks loaded");
    }

    let target_framework = if let Some(fw) = framework {
        fw
    } else {
        frameworks
            .first()
            .map(|f| f.slug.clone())
            .unwrap_or_default()
    };

    let registry = loader.get_registry(&target_framework);

    if let Some(reg) = registry {
        let matched_question = reg.questions.iter().find(|q| {
            q.topic.to_lowercase().contains(&question.to_lowercase())
                || question.to_lowercase().contains(&q.topic.to_lowercase())
        });

        if let Some(q) = matched_question {
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

        let similar: Vec<String> = reg.questions.iter().map(|q| q.topic.clone()).collect();
        let error = ErrorResponse::validation(&format!(
            "No matching question found. Similar topics: {}",
            similar.join(", ")
        ));
        let response = ResponseEnvelope::error("ask", error, duration_ms);
        response.print();
        return CommandResult::err("ask", "Question not found");
    }

    let error = ErrorResponse::validation(&format!("Framework not found: {}", target_framework));
    let response = ResponseEnvelope::error("ask", error, duration_ms);
    response.print();
    CommandResult::err("ask", "Framework not found")
}
