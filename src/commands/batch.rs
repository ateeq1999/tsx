use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::json::error::ErrorResponse;
use crate::json::payload::BatchPayload;
use crate::json::response::ResponseEnvelope;
use crate::output::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub results: Vec<BatchCommandResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCommandResult {
    pub index: u32,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BatchError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

pub fn batch(
    payload: BatchPayload,
    overwrite: bool,
    dry_run: bool,
    verbose: bool,
) -> CommandResult {
    let start = Instant::now();
    let duration_ms = start.elapsed().as_millis() as u64;

    let total = payload.commands.len() as u32;
    let mut succeeded: u32 = 0;
    let mut failed: u32 = 0;
    let mut results: Vec<BatchCommandResult> = Vec::new();

    for (index, cmd) in payload.commands.iter().enumerate() {
        let cmd_start = Instant::now();
        let cmd_duration_ms = cmd_start.elapsed().as_millis() as u64;

        let result = execute_command(&cmd.command, &cmd.options, overwrite, dry_run);

        match result {
            Ok(file_created) => {
                succeeded += 1;
                results.push(BatchCommandResult {
                    index: index as u32,
                    success: true,
                    result: Some(serde_json::json!({
                        "kind": cmd.command.clone(),
                        "path": file_created
                    })),
                    error: None,
                });
            }
            Err(e) => {
                failed += 1;
                results.push(BatchCommandResult {
                    index: index as u32,
                    success: false,
                    result: None,
                    error: Some(BatchError {
                        code: e.0,
                        message: e.1,
                        path: None,
                    }),
                });

                if payload.stop_on_failure {
                    break;
                }
            }
        }
    }

    let batch_result = BatchResult {
        total,
        succeeded,
        failed,
        results,
    };

    let response = ResponseEnvelope::success(
        "batch",
        serde_json::to_value(batch_result).unwrap(),
        duration_ms,
    );

    if verbose {
        let context = crate::json::response::Context {
            project_root: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            tsx_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let response = response.with_context(context);
        response.print();
    } else {
        response.print();
    }

    CommandResult::ok("batch", vec![])
}

fn execute_command(
    command: &str,
    options: &serde_json::Value,
    overwrite: bool,
    dry_run: bool,
) -> Result<String, (String, String)> {
    let options_str = serde_json::to_string(options)
        .map_err(|e| ("INVALID_PAYLOAD".to_string(), e.to_string()))?;

    match command {
        "add:schema" => {
            let args: crate::schemas::AddSchemaArgs = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = crate::commands::add_schema::add_schema(args, overwrite, dry_run);
            Ok(result.files_created.join(", "))
        }
        "add:server-fn" => {
            let args: crate::schemas::AddServerFnArgs = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = crate::commands::add_server_fn::add_server_fn(args, overwrite, dry_run);
            Ok(result.files_created.join(", "))
        }
        "add:query" => {
            let args: crate::schemas::AddQueryArgs = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = crate::commands::add_query::add_query(args, overwrite, dry_run);
            Ok(result.files_created.join(", "))
        }
        "add:form" => {
            let args: crate::schemas::AddFormArgs = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = crate::commands::add_form::add_form(args, overwrite, dry_run);
            Ok(result.files_created.join(", "))
        }
        "add:table" => {
            let args: crate::schemas::AddFormArgs = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = crate::commands::add_table::add_table(args, overwrite, dry_run);
            Ok(result.files_created.join(", "))
        }
        "add:page" => {
            let args: crate::schemas::AddPageArgs = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = crate::commands::add_page::add_page(args, overwrite, dry_run);
            Ok(result.files_created.join(", "))
        }
        "add:seed" => {
            let args: crate::schemas::AddSeedArgs = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = crate::commands::add_seed::add_seed(args, overwrite, dry_run);
            Ok(result.files_created.join(", "))
        }
        "add:feature" => {
            let args: crate::schemas::AddFeatureArgs = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = crate::commands::add_feature::add_feature(args, overwrite, dry_run);
            Ok(result.files_created.join(", "))
        }
        "add:auth" => {
            let args: crate::schemas::AddAuthArgs = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = crate::commands::add_auth::add_auth(args, overwrite, dry_run);
            Ok(result.files_created.join(", "))
        }
        "add:auth-guard" => {
            let args: crate::schemas::AddAuthGuardArgs = serde_json::from_str(&options_str)
                .map_err(|e| ("VALIDATION_ERROR".to_string(), e.to_string()))?;
            let result = crate::commands::add_auth_guard::add_auth_guard(args, overwrite, dry_run);
            Ok(result.files_created.join(", "))
        }
        _ => Err((
            "UNKNOWN_COMMAND".to_string(),
            format!("Unknown command: {}", command),
        )),
    }
}
