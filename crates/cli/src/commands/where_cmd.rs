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

pub fn where_cmd(thing: String, framework: Option<String>, verbose: bool) -> CommandResult {
    let start = Instant::now();
    let duration_ms = start.elapsed().as_millis() as u64;

    let mut loader = FrameworkLoader::default();
    let frameworks = loader.load_builtin_frameworks();

    if frameworks.is_empty() {
        let error = ErrorResponse::validation("No frameworks available");
        let response = ResponseEnvelope::error("where", error, duration_ms);
        response.print();
        return CommandResult::err("where", "No frameworks loaded");
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
        for (key, conv) in &reg.conventions.files {
            if key.to_lowercase().contains(&thing.to_lowercase())
                || thing.to_lowercase().contains(&key.to_lowercase())
            {
                let structure = &reg.structure;
                let path = conv.pattern.clone();

                let result = WhereResult {
                    thing: key.clone(),
                    framework: reg.framework.clone(),
                    path: apply_structure(&path, structure),
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
        }

        let available: Vec<String> = reg.conventions.files.keys().cloned().collect();
        let error = ErrorResponse::validation(&format!(
            "Unknown thing '{}'. Available: {}",
            thing,
            available.join(", ")
        ));
        let response = ResponseEnvelope::error("where", error, duration_ms);
        response.print();
        return CommandResult::err("where", "Thing not found");
    }

    let error = ErrorResponse::validation(&format!("Framework not found: {}", target_framework));
    let response = ResponseEnvelope::error("where", error, duration_ms);
    response.print();
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
