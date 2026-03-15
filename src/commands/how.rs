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

pub fn how(integration: String, framework: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();
    let duration_ms = start.elapsed().as_millis() as u64;

    let mut loader = FrameworkLoader::default();
    let frameworks = loader.load_builtin_frameworks();

    if frameworks.is_empty() {
        let error = ErrorResponse::validation("No frameworks available");
        let response = ResponseEnvelope::error("how", error, duration_ms);
        response.print();
        return CommandResult::err("how", "No frameworks loaded");
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
        let matched = reg.integrations.iter().find(|i| {
            i.package
                .to_lowercase()
                .contains(&integration.to_lowercase())
                || integration
                    .to_lowercase()
                    .contains(&i.package.to_lowercase())
        });

        if let Some(int) = matched {
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

        let available: Vec<String> = reg.integrations.iter().map(|i| i.package.clone()).collect();
        let error = ErrorResponse::validation(&format!(
            "Integration '{}' not found. Available: {}",
            integration,
            available.join(", ")
        ));
        let response = ResponseEnvelope::error("how", error, duration_ms);
        response.print();
        return CommandResult::err("how", "Integration not found");
    }

    let error = ErrorResponse::validation(&format!("Framework not found: {}", target_framework));
    let response = ResponseEnvelope::error("how", error, duration_ms);
    response.print();
    CommandResult::err("how", "Framework not found")
}
