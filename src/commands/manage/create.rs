use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::json::error::ErrorResponse;
use crate::json::payload::{BatchCommand, BatchPayload};
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;
use crate::utils::paths::get_frameworks_dir;

#[derive(Debug, Deserialize)]
struct StarterStep {
    cmd: String,
    args: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct StarterRecipe {
    id: String,
    name: String,
    description: String,
    #[allow(dead_code)]
    token_estimate: Option<u32>,
    steps: Vec<StarterStep>,
}

#[derive(Debug, Serialize)]
struct CreateResult {
    framework: String,
    starter: String,
    starter_name: String,
    description: String,
    steps_total: u32,
    steps_succeeded: u32,
    steps_failed: u32,
    files_created: Vec<String>,
}

pub fn create(
    from: String,
    starter: Option<String>,
    dry_run: bool,
    verbose: bool,
) -> CommandResult {
    let start = Instant::now();
    let starter_id = starter.unwrap_or_else(|| "basic".to_string());

    // Resolve starter JSON path
    let frameworks_dir = get_frameworks_dir();
    let starter_path = frameworks_dir
        .join(&from)
        .join("starters")
        .join(format!("{}.json", starter_id));

    if !starter_path.exists() {
        let duration_ms = start.elapsed().as_millis() as u64;
        let error = ErrorResponse::validation(&format!(
            "Starter '{}' not found for framework '{}'. Looked in: {}",
            starter_id,
            from,
            starter_path.display()
        ));
        ResponseEnvelope::error("create", error, duration_ms).print();
        return CommandResult::err("create", "Starter not found");
    }

    let content = match std::fs::read_to_string(&starter_path) {
        Ok(c) => c,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Failed to read starter: {}", e));
            ResponseEnvelope::error("create", error, duration_ms).print();
            return CommandResult::err("create", "Failed to read starter");
        }
    };

    let recipe: StarterRecipe = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            let error = ErrorResponse::new(crate::json::error::ErrorCode::InternalError, &format!("Invalid starter JSON: {}", e));
            ResponseEnvelope::error("create", error, duration_ms).print();
            return CommandResult::err("create", "Invalid starter JSON");
        }
    };

    let steps_total = recipe.steps.len() as u32;
    let mut steps_succeeded: u32 = 0;
    let mut steps_failed: u32 = 0;
    let mut all_files_created: Vec<String> = vec![];

    for step in &recipe.steps {
        match step.cmd.as_str() {
            "init" => {
                // Handle init specially — calls init::init()
                let name = step.args.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                let result = crate::commands::manage::init::init(name);
                if result.success {
                    steps_succeeded += 1;
                    all_files_created.extend(result.files_created);
                } else {
                    steps_failed += 1;
                    if verbose {
                        eprintln!("[create] init step failed: {}", result.error.as_deref().unwrap_or("unknown"));
                    }
                }
            }
            "add:migration" => {
                // Handle migration specially — no args
                let result = crate::commands::manage::add_migration::add_migration();
                if result.success {
                    steps_succeeded += 1;
                    all_files_created.extend(result.files_created);
                } else {
                    steps_failed += 1;
                    if verbose {
                        eprintln!("[create] add:migration step failed: {}", result.error.as_deref().unwrap_or("unknown"));
                    }
                }
            }
            other => {
                // All other commands go through batch machinery
                let batch_payload = BatchPayload {
                    commands: vec![BatchCommand {
                        command: other.to_string(),
                        options: step.args.clone(),
                    }],
                    stop_on_failure: false,
                    rollback_on_failure: false,
                };

                let result = crate::commands::batch::batch(
                    batch_payload,
                    false, // overwrite
                    dry_run,
                    false, // verbose (suppress inner output)
                    false, // stream
                );
                if result.success {
                    steps_succeeded += 1;
                    all_files_created.extend(result.files_created);
                } else {
                    steps_failed += 1;
                    if verbose {
                        eprintln!("[create] step '{}' failed: {}", other, result.error.as_deref().unwrap_or("unknown"));
                    }
                }
            }
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    let create_result = CreateResult {
        framework: from,
        starter: recipe.id,
        starter_name: recipe.name,
        description: recipe.description,
        steps_total,
        steps_succeeded,
        steps_failed,
        files_created: all_files_created.clone(),
    };

    let response = ResponseEnvelope::success(
        "create",
        serde_json::to_value(create_result).unwrap(),
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

    CommandResult::ok("create", all_files_created)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_recipe_deserializes() {
        let json = r#"{
            "id": "basic",
            "name": "Basic Starter",
            "description": "Minimal project",
            "token_estimate": 40,
            "steps": [
                { "cmd": "init", "args": {} },
                { "cmd": "add:schema", "args": { "name": "test" } }
            ]
        }"#;
        let recipe: StarterRecipe = serde_json::from_str(json).unwrap();
        assert_eq!(recipe.id, "basic");
        assert_eq!(recipe.steps.len(), 2);
        assert_eq!(recipe.steps[0].cmd, "init");
        assert_eq!(recipe.steps[1].cmd, "add:schema");
    }

    #[test]
    fn starter_recipe_without_token_estimate_deserializes() {
        let json = r#"{
            "id": "basic",
            "name": "Basic Starter",
            "description": "Minimal",
            "steps": []
        }"#;
        let recipe: StarterRecipe = serde_json::from_str(json).unwrap();
        assert!(recipe.token_estimate.is_none());
    }
}
